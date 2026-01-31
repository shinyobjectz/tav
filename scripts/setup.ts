/**
 * Setup script - downloads required binaries and assets from Cloudflare R2
 * Run with: bun run scripts/setup.ts
 *
 * Downloads:
 *   - NitroGen ML model (ng.pt) -> src-tauri/binaries/
 *   - NitroGen sidecar executable -> src-tauri/binaries/
 *   - Character model files (.glb) -> packages/quaternius-character/
 */

import {
  S3Client,
  GetObjectCommand,
  HeadObjectCommand,
} from "@aws-sdk/client-s3";
import {
  readFileSync,
  existsSync,
  mkdirSync,
  writeFileSync,
  statSync,
} from "fs";
import { join } from "path";
import { Readable } from "stream";

// -------------------------------------------------------------------
// Config
// -------------------------------------------------------------------

const ROOT = join(__dirname, "..");

const envPath = join(ROOT, ".env.local");
if (existsSync(envPath)) {
  const env = readFileSync(envPath, "utf-8");
  env.split("\n").forEach((line) => {
    const [key, ...rest] = line.split("=");
    if (key && rest.length) process.env[key.trim()] = rest.join("=").trim();
  });
}

const ACCOUNT_ID = process.env.CLOUDFLARE_ACCOUNT_ID;
const ACCESS_KEY_ID = process.env.CLOUDFLARE_ACCESS_KEY_ID;
const SECRET_ACCESS_KEY = process.env.CLOUDFLARE_SECRET_ACCESS_KEY;
const BUCKET = process.env.CLOUDFLARE_BUCKET || "kobold";

if (!ACCOUNT_ID || !ACCESS_KEY_ID || !SECRET_ACCESS_KEY) {
  console.error(`
Missing R2 credentials in .env.local. Required:
  CLOUDFLARE_ACCOUNT_ID=<your-account-id>
  CLOUDFLARE_ACCESS_KEY_ID=<r2-access-key-id>
  CLOUDFLARE_SECRET_ACCESS_KEY=<r2-secret-access-key>
  CLOUDFLARE_BUCKET=kobold
`);
  process.exit(1);
}

const s3 = new S3Client({
  region: "auto",
  endpoint: `https://${ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: {
    accessKeyId: ACCESS_KEY_ID,
    secretAccessKey: SECRET_ACCESS_KEY,
  },
});

// -------------------------------------------------------------------
// Manifest of files to download
// -------------------------------------------------------------------

interface Asset {
  /** Key in the R2 bucket */
  remoteKey: string;
  /** Local path relative to project root */
  localPath: string;
  /** Human-readable description */
  description: string;
  /** If true, skip when file already exists locally */
  skipIfExists?: boolean;
}

const ASSETS: Asset[] = [
  {
    remoteKey: "binaries/ng.pt",
    localPath: "src-tauri/binaries/ng.pt",
    description: "NitroGen ML model (~1.9 GB)",
    skipIfExists: true,
  },
  {
    remoteKey: "binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    localPath:
      "src-tauri/binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    description: "NitroGen sidecar (Windows x64)",
    skipIfExists: true,
  },
  {
    remoteKey: "models/quaternius-character/character.glb",
    localPath: "packages/quaternius-character/character.glb",
    description: "Quaternius character model",
    skipIfExists: true,
  },
  {
    remoteKey: "models/quaternius-character/UAL1.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1.glb",
    description: "UAL1 character model",
    skipIfExists: true,
  },
  {
    remoteKey: "models/quaternius-character/UAL1_Standard.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1_Standard.glb",
    description: "UAL1 Standard character model",
    skipIfExists: true,
  },
  {
    remoteKey: "models/quaternius-character/Godot_Setup.png",
    localPath: "packages/quaternius-character/Godot_Setup.png",
    description: "Godot setup reference image",
    skipIfExists: true,
  },
];

// -------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

async function streamToBuffer(stream: Readable): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of stream) {
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks);
}

async function downloadAsset(asset: Asset): Promise<boolean> {
  const localFull = join(ROOT, asset.localPath);

  // Skip if already present
  if (asset.skipIfExists && existsSync(localFull)) {
    const size = statSync(localFull).size;
    console.log(`  SKIP  ${asset.localPath} (${formatSize(size)} exists)`);
    return true;
  }

  // Check remote exists
  let remoteSize = 0;
  try {
    const head = await s3.send(
      new HeadObjectCommand({ Bucket: BUCKET, Key: asset.remoteKey })
    );
    remoteSize = head.ContentLength ?? 0;
  } catch {
    console.log(`  MISS  ${asset.remoteKey} (not found in R2 — upload first)`);
    return false;
  }

  console.log(
    `  GET   ${asset.remoteKey} (${formatSize(remoteSize)}) -> ${asset.localPath}`
  );

  // Ensure directory exists
  const dir = join(localFull, "..");
  mkdirSync(dir, { recursive: true });

  // Download
  const resp = await s3.send(
    new GetObjectCommand({ Bucket: BUCKET, Key: asset.remoteKey })
  );
  const body = resp.Body as Readable;
  const buffer = await streamToBuffer(body);
  writeFileSync(localFull, buffer);

  console.log(`  OK    ${asset.localPath} (${formatSize(buffer.length)})`);
  return true;
}

// -------------------------------------------------------------------
// Main
// -------------------------------------------------------------------

async function main() {
  console.log("=== Tav Setup ===\n");

  // 1. Install npm dependencies
  console.log("[1/3] Checking dependencies...");
  if (!existsSync(join(ROOT, "node_modules"))) {
    console.log("  Installing npm packages...");
    const proc = Bun.spawnSync(["bun", "install"], { cwd: ROOT });
    if (proc.exitCode !== 0) {
      console.error("  Failed to install dependencies");
      process.exit(1);
    }
    console.log("  Done.\n");
  } else {
    console.log("  node_modules/ exists, skipping.\n");
  }

  // 2. Download binary assets from R2
  console.log("[2/3] Downloading assets from R2...");
  let downloaded = 0;
  let skipped = 0;
  let missing = 0;

  for (const asset of ASSETS) {
    const ok = await downloadAsset(asset);
    if (ok) {
      if (
        asset.skipIfExists &&
        existsSync(join(ROOT, asset.localPath))
      ) {
        // Could be skip or fresh download — count based on log
      }
      downloaded++;
    } else {
      missing++;
    }
  }

  console.log(
    `\n  ${downloaded} downloaded/present, ${missing} missing from R2.\n`
  );

  if (missing > 0) {
    console.log(
      "  To upload missing assets, run: bun run scripts/upload-to-r2.ts --binaries\n"
    );
  }

  // 3. Summary
  console.log("[3/3] Checking build prerequisites...");
  const checks = [
    {
      name: "Rust toolchain",
      cmd: "rustc --version",
      hint: "Install from https://rustup.rs",
    },
    {
      name: "Tauri CLI",
      cmd: "npx tauri --version",
      hint: "Run: bun add -D @tauri-apps/cli",
    },
  ];

  for (const check of checks) {
    try {
      const proc = Bun.spawnSync(check.cmd.split(" "), {
        cwd: ROOT,
        stdout: "pipe",
        stderr: "pipe",
      });
      if (proc.exitCode === 0) {
        const ver = proc.stdout.toString().trim();
        console.log(`  OK    ${check.name}: ${ver}`);
      } else {
        console.log(`  WARN  ${check.name} not found — ${check.hint}`);
      }
    } catch {
      console.log(`  WARN  ${check.name} not found — ${check.hint}`);
    }
  }

  console.log("\n=== Setup Complete ===");
  console.log("Run `bun run start` to launch the app.");
}

main().catch((err) => {
  console.error("Setup failed:", err);
  process.exit(1);
});
