/**
 * Organize template folder into standardized structure
 * Run with: bun run scripts/organize-template.ts
 */

import { existsSync, mkdirSync, readdirSync, statSync, copyFileSync, rmSync, writeFileSync } from "fs";
import { join, extname, basename, dirname } from "path";

const TEMPLATE_DIR = join(__dirname, "../templates/psx-mannequin");
const SOURCE_DIR = join(__dirname, "../templates/psx-manniquin"); // Note: misspelled source

// Standard structure
const STRUCTURE = {
  root: TEMPLATE_DIR,
  assets: {
    models: join(TEMPLATE_DIR, "assets/models"),
    textures: {
      male: join(TEMPLATE_DIR, "assets/textures/male"),
      female: join(TEMPLATE_DIR, "assets/textures/female"),
      bonus: join(TEMPLATE_DIR, "assets/textures/bonus"),
    },
  },
  godot: {
    import: join(TEMPLATE_DIR, "godot/import"),
  },
};

// File type mappings
const FILE_DESTINATIONS: Record<string, (filename: string) => string | null> = {
  ".fbx": (f) => join(STRUCTURE.assets.models, basename(f)),
  ".glb": (f) => join(STRUCTURE.assets.models, basename(f)),
  ".gltf": (f) => join(STRUCTURE.assets.models, basename(f)),
  ".png": (f) => {
    const name = basename(f).toLowerCase();
    if (name.includes("male") && !name.includes("female")) {
      return join(STRUCTURE.assets.textures.male, basename(f));
    } else if (name.includes("female") || name.includes("f_")) {
      return join(STRUCTURE.assets.textures.female, basename(f));
    } else {
      return join(STRUCTURE.assets.textures.bonus, basename(f));
    }
  },
  ".jpg": (f) => {
    const name = basename(f).toLowerCase();
    if (name.includes("male")) return join(STRUCTURE.assets.textures.male, basename(f));
    if (name.includes("female")) return join(STRUCTURE.assets.textures.female, basename(f));
    return join(STRUCTURE.assets.textures.bonus, basename(f));
  },
  ".import": (f) => join(STRUCTURE.godot.import, basename(f)),
  // Skip these
  ".identifier": () => null,
  ".zone": () => null,
};

function getAllFiles(dir: string, files: string[] = []): string[] {
  if (!existsSync(dir)) return files;
  
  for (const file of readdirSync(dir)) {
    const fullPath = join(dir, file);
    if (statSync(fullPath).isDirectory()) {
      getAllFiles(fullPath, files);
    } else {
      files.push(fullPath);
    }
  }
  return files;
}

function createDirectories() {
  console.log("Creating directory structure...");
  
  const dirs = [
    STRUCTURE.root,
    STRUCTURE.assets.models,
    STRUCTURE.assets.textures.male,
    STRUCTURE.assets.textures.female,
    STRUCTURE.assets.textures.bonus,
    STRUCTURE.godot.import,
  ];
  
  for (const dir of dirs) {
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
      console.log(`  Created: ${dir}`);
    }
  }
}

function organizeFiles() {
  console.log("\nOrganizing files...");
  
  const files = getAllFiles(SOURCE_DIR);
  let copied = 0;
  let skipped = 0;
  
  for (const file of files) {
    const ext = extname(file).toLowerCase();
    const name = basename(file).toLowerCase();
    
    // Skip Zone.Identifier files
    if (name.includes("zone.identifier") || name.endsWith(".identifier")) {
      skipped++;
      continue;
    }
    
    const getDestination = FILE_DESTINATIONS[ext];
    if (getDestination) {
      const dest = getDestination(file);
      if (dest) {
        const destDir = dirname(dest);
        if (!existsSync(destDir)) {
          mkdirSync(destDir, { recursive: true });
        }
        copyFileSync(file, dest);
        console.log(`  ${basename(file)} -> ${dest.replace(TEMPLATE_DIR, "")}`);
        copied++;
      } else {
        skipped++;
      }
    } else {
      console.log(`  [SKIP] Unknown type: ${basename(file)}`);
      skipped++;
    }
  }
  
  console.log(`\nCopied: ${copied}, Skipped: ${skipped}`);
}

function createReadme() {
  const readme = `# PSX Mannequin - Mixamo-Compatible Character Pack

## Contents

### Models (\`assets/models/\`)
- \`male_pants_base.fbx\` - Male character with pants
- \`male_shorts_base.fbx\` - Male character with shorts  
- \`female_pants_base.fbx\` - Female character with pants
- \`female_skirt_base.fbx\` - Female character with skirt

### Textures (\`assets/textures/\`)
- \`male/\` - Male skin tones and clothing textures
- \`female/\` - Female skin tones and clothing textures
- \`bonus/\` - Additional NPC outfit textures

### Godot Import Settings (\`godot/import/\`)
Pre-configured .import files with Mixamo bone retargeting.

## Usage in Godot

1. Copy \`assets/\` folder to your Godot project
2. Import the FBX files
3. Apply Mixamo BoneMap for animation retargeting:
   \`\`\`
   Skeleton3D > Import Settings > Retarget > Bone Map = Mixamo BoneMap
   \`\`\`

## Mixamo Animations

Download animations from [mixamo.com](https://www.mixamo.com):
1. Upload any character (or use Mixamo's)
2. Choose animations (Idle, Walk, Run, Jump)
3. Download as FBX (Without Skin, 30fps)
4. Import to Godot and apply to these models

## License

PSX-style low-poly characters for game development.
`;
  
  writeFileSync(join(TEMPLATE_DIR, "README.md"), readme);
  console.log("\nCreated README.md");
}

async function main() {
  console.log("=== Template Organizer ===\n");
  console.log(`Source: ${SOURCE_DIR}`);
  console.log(`Destination: ${TEMPLATE_DIR}\n`);
  
  if (!existsSync(SOURCE_DIR)) {
    console.error("Source directory not found!");
    process.exit(1);
  }
  
  createDirectories();
  organizeFiles();
  createReadme();
  
  console.log("\n=== Done ===");
  console.log(`Template organized at: ${TEMPLATE_DIR}`);
  console.log("\nNext: Run upload script to push to R2");
}

main().catch(console.error);
