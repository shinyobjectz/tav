# Packages

Downloadable asset bundles hosted on Cloudflare R2.

## Available Packages

### quaternius-character (Recommended)
CC0-licensed animated humanoid from Quaternius Universal Animation Library.

**Contents:**
- Single `character.glb` with 100+ animations
- Universal humanoid rig
- Works out of the box with 3D templates

**R2 URL:** `https://pub-b3ceaf5076804d56bc32fe9d83e9a3a9.r2.dev/quaternius-character.zip`

**Auto-downloaded:** Yes, for all 3D templates

---

### psx-mannequin (Legacy)
PSX-style low-poly Mixamo-compatible characters.

**Contents:**
- 6 character models (male/female variants)
- Texture packs (skin tones, clothing)
- Placeholder animations (requires Mixamo)

**R2 URL:** `https://pub-b3ceaf5076804d56bc32fe9d83e9a3a9.r2.dev/psx-mannequin.zip`

## Adding New Packages

1. Create folder: `packages/<package-name>/`
2. Add assets and README.md
3. Create zip: `Compress-Archive -Path <files> -DestinationPath <package-name>.zip`
4. Upload to R2: `bunx wrangler r2 object put kobold/<package-name>.zip --file=<package-name>.zip`
5. Update Rust code to reference the new package

## Package vs Template

- **Packages**: Downloadable assets (models, textures, animations) from R2
- **Templates**: Project scaffolding code that creates new projects
