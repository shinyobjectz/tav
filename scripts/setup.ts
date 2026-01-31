/**
 * Setup script - downloads required binaries and assets
 * Run with: bun run scripts/setup.ts
 *
 * Downloads from:
 *   - HuggingFace: NitroGen ML model (ng.pt)
 *   - Cloudflare R2: sidecar executable, character models
 *
 * R2 downloads require CLOUDFLARE_API_TOKEN in .env.local
 */

import { existsSync, readFileSync, mkdirSync, statSync, createWriteStream } from "fs";
import { join, dirname } from "path";
import { Writable } from "stream";

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

// -------------------------------------------------------------------
// Asset manifest
// -------------------------------------------------------------------

interface Asset {
  /** Local path relative to project root */
  localPath: string;
  /** Human-readable description */
  description: string;
  /** Download source: "r2" uses wrangler, "url" fetches directly */
  source: "r2" | "url";
  /** R2 key (when source is "r2") */
  remoteKey?: string;
  /** Direct URL (when source is "url") */
  url?: string;
}

const ASSETS: Asset[] = [
  {
    source: "url",
    url: "https://huggingface.co/nvidia/NitroGen/resolve/main/ng.pt?download=true",
    localPath: "src-tauri/binaries/ng.pt",
    description: "NitroGen ML model (~1.9 GB)",
  },
  {
    source: "r2",
    remoteKey: "binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    localPath:
      "src-tauri/binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    description: "NitroGen sidecar (Windows x64)",
  },
  {
    source: "r2",
    remoteKey: "models/quaternius-character/character.glb",
    localPath: "packages/quaternius-character/character.glb",
    description: "Quaternius character model",
  },
  {
    source: "r2",
    remoteKey: "models/quaternius-character/UAL1.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1.glb",
    description: "UAL1 character model",
  },
  {
    source: "r2",
    remoteKey: "models/quaternius-character/UAL1_Standard.glb",
    localPath: "packages/quaternius-character/Unreal-Godot/UAL1_Standard.glb",
    description: "UAL1 Standard character model",
  },
  {
    source: "r2",
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

async function downloadFromUrl(url: string, destPath: string): Promise<boolean> {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok || !res.body) {
    console.log(`  FAIL  HTTP ${res.status} ${res.statusText}`);
    return false;
  }

  const totalBytes = Number(res.headers.get("content-length") || 0);
  const totalStr = totalBytes ? formatSize(totalBytes) : "unknown size";
  let downloaded = 0;

  const file = Bun.file(destPath);
  const writer = file.writer();

  for await (const chunk of res.body) {
    writer.write(chunk);
    downloaded += chunk.byteLength;
    if (totalBytes) {
      const pct = ((downloaded / totalBytes) * 100).toFixed(1);
      process.stdout.write(
        `\r  ...   ${formatSize(downloaded)} / ${totalStr} (${pct}%)`
      );
    }
  }
  await writer.end();

  if (totalBytes) process.stdout.write("\n");
  return true;
}

async function downloadFromR2(remoteKey: string, destPath: string): Promise<boolean> {
  if (!process.env.CLOUDFLARE_API_TOKEN) {
    console.log(`  SKIP  ${remoteKey} (no CLOUDFLARE_API_TOKEN set)`);
    return false;
  }

  const proc = Bun.spawnSync(
    [
      "npx",
      "wrangler",
      "r2",
      "object",
      "get",
      `${BUCKET}/${remoteKey}`,
      `--file=${destPath}`,
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
    return true;
  } else {
    const stderr = proc.stderr.toString();
    if (stderr.includes("The specified key does not exist")) {
      console.log(
        `  MISS  ${remoteKey} (not found in R2 — upload first)`
      );
    } else {
      console.log(`  FAIL  ${remoteKey}: ${stderr.trim().split("\n").pop()}`);
    }
    return false;
  }
}

async function downloadAsset(asset: Asset): Promise<boolean> {
  const localFull = join(ROOT, asset.localPath);

  if (existsSync(localFull)) {
    const size = statSync(localFull).size;
    console.log(`  SKIP  ${asset.localPath} (${formatSize(size)} exists)`);
    return true;
  }

  const label = asset.source === "url" ? asset.url! : asset.remoteKey!;
  console.log(`  GET   ${label}`);
  console.log(`        -> ${asset.localPath}`);

  // Ensure directory exists
  mkdirSync(dirname(localFull), { recursive: true });

  let success: boolean;
  if (asset.source === "url") {
    success = await downloadFromUrl(asset.url!, localFull);
  } else {
    success = await downloadFromR2(asset.remoteKey!, localFull);
  }

  if (success && existsSync(localFull)) {
    const size = statSync(localFull).size;
    console.log(`  OK    ${asset.localPath} (${formatSize(size)})`);
  }

  return success;
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

  // 2. Download binary assets
  console.log("[2/3] Downloading assets...");
  let ok = 0;
  let fail = 0;

  for (const asset of ASSETS) {
    const success = await downloadAsset(asset);
    if (success) ok++;
    else fail++;
  }

  console.log(`\n  ${ok} downloaded/present, ${fail} missing.\n`);

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
