/**
 * Setup script - downloads required binaries and assets from Cloudflare R2
 * Run with: bun run scripts/setup.ts
 *
 * Uses wrangler CLI (requires CLOUDFLARE_API_TOKEN in .env.local)
 *
 * Downloads:
 *   - NitroGen ML model (ng.pt) -> src-tauri/binaries/
 *   - NitroGen sidecar executable -> src-tauri/binaries/
 *   - Character model files (.glb) -> packages/quaternius-character/
 */

import { existsSync, readFileSync, mkdirSync, statSync } from "fs";
import { join, dirname } from "path";

const ROOT = join(__dirname, "..");
const BUCKET = "kobold";

// Load .env.local
const envPath = join(ROOT, ".env.local");
if (existsSync(envPath)) {
  const env = readFileSync(envPath, "utf-8");
  env.split("\n").forEach((line) => {
    const [key, ...rest] = line.split("=");
    if (key && rest.length) process.env[key.trim()] = rest.join("=").trim();
  });
}

if (!process.env.CLOUDFLARE_API_TOKEN) {
  console.error(`
Missing CLOUDFLARE_API_TOKEN in .env.local. Required:
  CLOUDFLARE_API_TOKEN=<your-api-token>
`);
  process.exit(1);
}

// -------------------------------------------------------------------
// Asset manifest
// -------------------------------------------------------------------

interface Asset {
  remoteKey: string;
  localPath: string;
  description: string;
}

const ASSETS: Asset[] = [
  {
    remoteKey: "binaries/ng.pt",
    localPath: "src-tauri/binaries/ng.pt",
    description: "NitroGen ML model (~1.9 GB)",
  },
  {
    remoteKey: "binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    localPath:
      "src-tauri/binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    description: "NitroGen sidecar (Windows x64)",
  },
  {
    remoteKey: "models/quaternius-character/character.glb",
    localPath: "packages/quaternius-character/character.glb",
    description: "Quaternius character model",
  },
  {
    remoteKey: "models/quaternius-character/UAL1.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1.glb",
    description: "UAL1 character model",
  },
  {
    remoteKey: "models/quaternius-character/UAL1_Standard.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1_Standard.glb",
    description: "UAL1 Standard character model",
  },
  {
    remoteKey: "models/quaternius-character/Godot_Setup.png",
    localPath: "packages/quaternius-character/Godot_Setup.png",
    description: "Godot setup reference image",
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

async function downloadAsset(asset: Asset): Promise<boolean> {
  const localFull = join(ROOT, asset.localPath);

  if (existsSync(localFull)) {
    const size = statSync(localFull).size;
    console.log(`  SKIP  ${asset.localPath} (${formatSize(size)} exists)`);
    return true;
  }

  console.log(`  GET   ${asset.remoteKey} -> ${asset.localPath}`);

  // Ensure directory exists
  mkdirSync(dirname(localFull), { recursive: true });

  const proc = Bun.spawnSync(
    [
      "npx",
      "wrangler",
      "r2",
      "object",
      "get",
      `${BUCKET}/${asset.remoteKey}`,
      `--file=${localFull}`,
      "--remote",
    ],
    {
      cwd: ROOT,
      env: { ...process.env },
      stdout: "pipe",
      stderr: "pipe",
    }
  );

  if (proc.exitCode === 0) {
    const size = existsSync(localFull) ? statSync(localFull).size : 0;
    console.log(`  OK    ${asset.localPath} (${formatSize(size)})`);
    return true;
  } else {
    const stderr = proc.stderr.toString();
    if (stderr.includes("The specified key does not exist")) {
      console.log(
        `  MISS  ${asset.remoteKey} (not found in R2 — upload first)`
      );
    } else {
      console.log(`  FAIL  ${asset.remoteKey}: ${stderr.trim().split("\n").pop()}`);
    }
    return false;
  }
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
  let ok = 0;
  let fail = 0;

  for (const asset of ASSETS) {
    const success = await downloadAsset(asset);
    if (success) ok++;
    else fail++;
  }

  console.log(`\n  ${ok} downloaded/present, ${fail} missing.\n`);

  if (fail > 0) {
    console.log(
      "  To upload missing assets, run: bun run upload:all\n"
    );
  }

  // 3. Check build prerequisites
  console.log("[3/3] Checking build prerequisites...");
  const checks = [
    {
      name: "Rust toolchain",
      cmd: ["rustc", "--version"],
      hint: "Install from https://rustup.rs",
    },
    {
      name: "Tauri CLI",
      cmd: ["npx", "tauri", "--version"],
      hint: "Run: bun add -D @tauri-apps/cli",
    },
  ];

  for (const check of checks) {
    try {
      const proc = Bun.spawnSync(check.cmd, {
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
