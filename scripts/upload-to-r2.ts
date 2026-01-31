/**
 * Upload assets to Cloudflare R2
 * Run with: bun run scripts/upload-to-r2.ts
 */

import { S3Client, PutObjectCommand, HeadBucketCommand } from "@aws-sdk/client-s3";
import { readFileSync, statSync, existsSync, createWriteStream, readdirSync } from "fs";
import { join } from "path";
import archiver from "archiver";

// Load env
const envPath = join(__dirname, "../.env.local");
if (existsSync(envPath)) {
  const env = readFileSync(envPath, "utf-8");
  env.split("\n").forEach((line) => {
    const [key, value] = line.split("=");
    if (key && value) process.env[key.trim()] = value.trim();
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

async function zipFolder(sourceDir: string, outPath: string): Promise<void> {
  return new Promise((resolve, reject) => {
    const output = createWriteStream(outPath);
    const archive = archiver('zip', { zlib: { level: 9 } });

    output.on('close', () => resolve());
    archive.on('error', (err) => reject(err));

    archive.pipe(output);
    archive.directory(sourceDir, false);
    archive.finalize();
  });
}

async function uploadFile(localPath: string, remotePath: string) {
  const fileContent = readFileSync(localPath);
  const stats = statSync(localPath);
  
  console.log(`Uploading ${localPath} (${(stats.size / 1024 / 1024).toFixed(2)} MB)...`);
  
  await s3.send(new PutObjectCommand({
    Bucket: BUCKET,
    Key: remotePath,
    Body: fileContent,
    ContentType: getContentType(remotePath),
  }));
  
  console.log(`✓ Uploaded to ${remotePath}`);
}

function getContentType(path: string): string {
  if (path.endsWith(".zip")) return "application/zip";
  if (path.endsWith(".fbx")) return "application/octet-stream";
  if (path.endsWith(".glb")) return "model/gltf-binary";
  if (path.endsWith(".png")) return "image/png";
  if (path.endsWith(".jpg") || path.endsWith(".jpeg")) return "image/jpeg";
  if (path.endsWith(".exe")) return "application/octet-stream";
  if (path.endsWith(".pt")) return "application/octet-stream";
  return "application/octet-stream";
}

async function uploadBinaries() {
  console.log("\n--- Uploading sidecar binaries ---");

  const binDir = join(__dirname, "../src-tauri/binaries");
  const binFiles = [
    { local: "ng.pt", remote: "binaries/ng.pt" },
    {
      local: "nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
      remote: "binaries/nitrogen-sidecar-x86_64-pc-windows-msvc.exe",
    },
  ];

  for (const f of binFiles) {
    const localPath = join(binDir, f.local);
    if (existsSync(localPath)) {
      await uploadFile(localPath, f.remote);
    } else {
      console.log(`  SKIP  ${f.local} (not found locally)`);
    }
  }

  console.log("\n--- Uploading model assets ---");

  const modelFiles = [
    {
      local: join(__dirname, "../packages/quaternius-character/character.glb"),
      remote: "models/quaternius-character/character.glb",
    },
    {
      local: join(
        __dirname,
        "../packages/quaternius-character/Unreal-Godot/UAL1.glb"
      ),
      remote: "models/quaternius-character/UAL1.glb",
    },
    {
      local: join(
        __dirname,
        "../packages/quaternius-character/Unreal-Godot/UAL1_Standard.glb"
      ),
      remote: "models/quaternius-character/UAL1_Standard.glb",
    },
    {
      local: join(
        __dirname,
        "../packages/quaternius-character/Godot_Setup.png"
      ),
      remote: "models/quaternius-character/Godot_Setup.png",
    },
  ];

  for (const f of modelFiles) {
    if (existsSync(f.local)) {
      await uploadFile(f.local, f.remote);
    } else {
      console.log(`  SKIP  ${f.local} (not found locally)`);
    }
  }
}

async function main() {
  console.log("=== Cloudflare R2 Asset Sync ===");
  
  try {
    await s3.send(new HeadBucketCommand({ Bucket: BUCKET }));
    console.log("✓ Bucket accessible");
  } catch (e: any) {
    console.error("Bucket access error:", e.message);
    process.exit(1);
  }

  const packagesDir = join(__dirname, "../packages");
  const templatesDir = join(__dirname, "../templates");

  // Zip and upload packages
  const packages = readdirSync(packagesDir, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name);

  for (const pkg of packages) {
    const pkgPath = join(packagesDir, pkg);
    const zipPath = join(packagesDir, `${pkg}.zip`);
    
    console.log(`Processing package: ${pkg}`);
    await zipFolder(pkgPath, zipPath);
    await uploadFile(zipPath, `${pkg}.zip`);
  }

  // Zip and upload templates
  const templates = readdirSync(templatesDir, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => dirent.name);

  for (const template of templates) {
    const templatePath = join(templatesDir, template);
    const zipPath = join(templatesDir, `${template}.zip`);
    
    console.log(`Processing template: ${template}`);
    await zipFolder(templatePath, zipPath);
    await uploadFile(zipPath, `templates/${template}.zip`);
  }

  // Upload binaries if --binaries flag is passed
  if (process.argv.includes("--binaries") || process.argv.includes("--all")) {
    await uploadBinaries();
  }

  console.log("\n=== Sync Complete ===");
}

main().catch(console.error);
