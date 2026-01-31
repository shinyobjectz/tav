# Project Templates

Pre-configured project scaffolding for different game types.

## Available Templates

| Template | Dimension | Description |
|----------|-----------|-------------|
| `third-person-3d` | 3D | Third-person character with FSM, camera rig, locomotion |
| `first-person-3d` | 3D | First-person controller with mouse look |
| `platformer-2d` | 2D | Side-scrolling platformer with gravity |
| `top-down-2d` | 2D | Top-down movement (RPG/adventure style) |
| `puzzle` | 2D/3D | Minimal template for puzzle games |
| `empty` | 2D/3D | Blank project with architecture ready |

## Template Structure

Each template folder contains:
- `template.json` - Metadata and configuration
- `scene.tscn` - Main scene content
- `player.gd` - Player script
- Optional package dependencies

## How Templates Work

1. User selects template in Viewfinder
2. Rust reads template definition
3. Creates folder structure
4. Writes core autoloads (EventBus, GameState, AIController)
5. Writes components (HealthComponent, StateMachine, etc.)
6. Writes template-specific scene and player script
7. Optionally downloads packages (mannequin)

## Adding New Templates

1. Create folder: `templates/<template-name>/`
2. Add `template.json` with metadata
3. Add scene and script files
4. Update Rust code to register the template
