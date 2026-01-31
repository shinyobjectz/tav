/**
 * Upload assets to Cloudflare R2
 *
 * Usage:
 *   bun run upload              # Upload packages & templates only
 *   bun run upload:all          # Upload everything including binaries
 *
 * Uses wrangler CLI (requires CLOUDFLARE_API_TOKEN in .env.local)
 *
 * Note: wrangler has a 300 MB file size limit. For files larger than
 * that (e.g. ng.pt), create R2 API tokens and use the S3 multipart
 * upload, or upload via the Cloudflare dashboard.
 */

import {
  readFileSync,
  statSync,
  existsSync,
  createWriteStream,
  readdirSync,
  unlinkSync,
} from "fs";
import { join } from "path";
import archiver from "archiver";

const ROOT = join(__dirname, "..");
const BUCKET = "kobold";

// Load env
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
// Helpers
// -------------------------------------------------------------------

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

async function zipFolder(sourceDir: string, outPath: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const output = createWriteStream(outPath);
    const archive = archiver("zip", { zlib: { level: 9 } });

    output.on("close", () => resolve());
    archive.on("error", (err) => reject(err));

    archive.pipe(output);
    archive.directory(sourceDir, false);
    archive.finalize();
  });
}

async function uploadFile(
  localPath: string,
  remoteKey: string
): Promise<boolean> {
  const stats = statSync(localPath);
  const size = stats.size;

  if (size > 300 * 1024 * 1024) {
    console.log(
      `  SKIP  ${remoteKey} (${formatSize(size)} exceeds wrangler 300 MB limit — upload via dashboard)`
    );
    return false;
  }

  console.log(`  PUT   ${remoteKey} (${formatSize(size)})...`);

  const proc = Bun.spawnSync(
    [
      "npx",
      "wrangler",
      "r2",
      "object",
      "put",
      `${BUCKET}/${remoteKey}`,
      `--file=${localPath}`,
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
    console.log(`  OK    ${remoteKey}`);
    return true;
  } else {
    const stderr = proc.stderr.toString().trim().split("\n").pop();
    console.log(`  FAIL  ${remoteKey}: ${stderr}`);
    return false;
  }
}

// -------------------------------------------------------------------
// Upload targets
// -------------------------------------------------------------------

async function uploadPackagesAndTemplates() {
  const packagesDir = join(ROOT, "packages");
  const templatesDir = join(ROOT, "templates");

  // Zip and upload packages
  console.log("\n--- Packages ---");
  const packages = readdirSync(packagesDir, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name);

  for (const pkg of packages) {
    const pkgPath = join(packagesDir, pkg);
    const zipPath = join(packagesDir, `${pkg}.zip`);

    console.log(`  Zipping ${pkg}...`);
    await zipFolder(pkgPath, zipPath);
    await uploadFile(zipPath, `${pkg}.zip`);
    unlinkSync(zipPath); // clean up temp zip
  }

  // Zip and upload templates
  console.log("\n--- Templates ---");
  const templates = readdirSync(templatesDir, { withFileTypes: true })
    .filter((d) => d.isDirectory())
    .map((d) => d.name);

  for (const template of templates) {
    const templatePath = join(templatesDir, template);
    const zipPath = join(templatesDir, `${template}.zip`);

    console.log(`  Zipping ${template}...`);
    await zipFolder(templatePath, zipPath);
    await uploadFile(zipPath, `templates/${template}.zip`);
    unlinkSync(zipPath); // clean up temp zip
  }
}

async function uploadBinaries() {
  console.log("\n--- Sidecar binaries ---");

  // ng.pt is downloaded from HuggingFace, not R2
  const binFiles = [
    {
      local: join(
        ROOT,
        "src-tauri/binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe"
      ),
      remote: "binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    },
  ];

  for (const f of binFiles) {
    if (existsSync(f.local)) {
      await uploadFile(f.local, f.remote);
    } else {
      console.log(`  SKIP  ${f.remote} (not found locally)`);
    }
  }

  console.log("\n--- Model assets ---");

  const modelFiles = [
    {
      local: join(ROOT, "packages/quaternius-character/character.glb"),
      remote: "models/quaternius-character/character.glb",
    },
    {
      local: join(ROOT, "packages/quaternius-character/Unreal-Godot/UAL1.glb"),
      remote: "models/quaternius-character/UAL1.glb",
    },
    {
      local: join(
        ROOT,
        "packages/quaternius-character/Unreal-Godot/UAL1_Standard.glb"
      ),
      remote: "models/quaternius-character/UAL1_Standard.glb",
    },
    {
      local: join(ROOT, "packages/quaternius-character/Godot_Setup.png"),
      remote: "models/quaternius-character/Godot_Setup.png",
    },
  ];

  for (const f of modelFiles) {
    if (existsSync(f.local)) {
      await uploadFile(f.local, f.remote);
    } else {
      console.log(`  SKIP  ${f.remote} (not found locally)`);
    }
  }
}

// -------------------------------------------------------------------
// Main
// -------------------------------------------------------------------

async function main() {
  console.log("=== Cloudflare R2 Asset Upload ===");

  await uploadPackagesAndTemplates();

  if (process.argv.includes("--binaries") || process.argv.includes("--all")) {
    await uploadBinaries();
  }

  console.log("\n=== Upload Complete ===");
}

main().catch(console.error);
