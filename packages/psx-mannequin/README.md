# PSX Mannequin Package

Mixamo-compatible low-poly characters for 3D games.

## Contents

```
assets/
├── models/           # FBX character models
│   ├── male_pants_base.fbx
│   ├── male_shorts_base.fbx
│   ├── male_nude_base.fbx
│   ├── female_pants_base.fbx
│   ├── female_skirt_base.fbx
│   └── female_nude_base.fbx
├── textures/         # Character textures
│   ├── male/
│   ├── female/
│   └── bonus/
└── animations/       # Placeholder animation resources
    ├── idle.tres
    ├── walk.tres
    ├── run.tres
    ├── jump.tres
    ├── fall.tres
    └── land.tres
godot/
└── import/           # Godot import configs
```

## Usage

This package is automatically downloaded when creating a 3D project with "Include PSX Mannequin" checked.

**Destination:** `<project>/assets/characters/`

## Adding Mixamo Animations

1. Go to [mixamo.com](https://www.mixamo.com)
2. Upload `male_pants_base.fbx`
3. Download animations (FBX, Without Skin, 30fps)
4. Place in `assets/characters/animations/`

## R2 Location

`https://pub-b3ceaf5076804d56bc32fe9d83e9a3a9.r2.dev/psx-mannequin.zip`
