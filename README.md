# Tav

A desktop IDE for AI-assisted Godot game development. Tav wraps agentic AI code assistants (Claude Code, OpenCode) with native Godot engine integration, letting AI agents write game code, create scenes, and preview results in a unified workspace.

## Tech Stack

- **Frontend:** React + TypeScript (Vite, Tailwind CSS, Radix UI)
- **Backend:** Rust via Tauri 2.0
- **Game Engine:** Godot 4.x (headless integration)
- **AI Agents:** Claude Code / OpenCode via MCP (Model Context Protocol)
- **Asset Storage:** Cloudflare R2

## Prerequisites

- [Bun](https://bun.sh/) (package manager)
- [Rust](https://rustup.rs/) 1.86+
- [Godot 4.x](https://godotengine.org/) (for game preview/testing)

## Setup

```bash
# Clone the repo
git clone https://github.com/shinyobjectz/tav.git
cd tav

# Install dependencies
bun install

# Download binary assets from R2 (ML models, sidecar, 3D models)
bun run setup
```

The setup script downloads required binaries that are too large for git:
- `ng.pt` — NitroGen ML model (~1.9 GB)
- `nitrogen-sidecar` — Compiled Python sidecar for AI game playing
- Character model files (`.glb`) for the Quaternius character package

### Environment Variables

Create a `.env.local` file with your credentials:

```
CLOUDFLARE_ACCOUNT_ID=<your-account-id>
CLOUDFLARE_ACCESS_KEY_ID=<r2-access-key-id>
CLOUDFLARE_SECRET_ACCESS_KEY=<r2-secret-access-key>
CLOUDFLARE_BUCKET=kobold
```

## Development

```bash
# Start the Tauri dev server (frontend + backend with HMR)
bun run start

# Frontend only (Vite dev server on :1420)
bun run dev

# Build for production
bun run build
```

## Project Structure

```
tav/
├── src/                    # React frontend
│   ├── components/         # UI components (Chat, Console, Editor, etc.)
│   ├── hooks/              # React hooks (Zustand store)
│   └── lib/                # Utilities
├── src-tauri/              # Rust backend
│   ├── src/                # Tauri commands, controls, animations
│   ├── binaries/           # Sidecar scripts & executables (gitignored)
│   └── capabilities/       # Tauri security capabilities
├── packages/               # Godot addon packages
│   ├── amsg/               # Advanced Movement System (GDScript)
│   ├── quaternius-character/ # Character models (downloaded via setup)
│   └── psx-mannequin/      # PSX-style character models
├── templates/              # Godot project templates
│   ├── first-person-3d/
│   ├── third-person-3d/
│   ├── platformer-2d/
│   └── top-down-2d/
├── scripts/                # Build & utility scripts
│   ├── setup.ts            # Download binaries from R2
│   └── upload-to-r2.ts     # Upload assets to R2
└── prd/                    # Design docs & research
```

## Scripts

| Command | Description |
|---------|-------------|
| `bun run start` | Launch Tauri dev mode |
| `bun run dev` | Frontend dev server only |
| `bun run build` | Production build |
| `bun run setup` | Download binary assets from R2 |
| `bun run upload` | Upload packages & templates to R2 |
| `bun run upload:all` | Upload everything including binaries |

## Architecture

```
┌──────────────────────────────────────────────┐
│              Tauri Desktop App               │
│  ┌────────────────────────────────────────┐  │
│  │   React Frontend                       │  │
│  │   Chat · Editor · Preview · Console    │  │
│  └──────────────┬─────────────────────────┘  │
│                 │ Tauri IPC                   │
│  ┌──────────────┴─────────────────────────┐  │
│  │   Rust Backend                         │  │
│  │   Commands · Process Manager · FS      │  │
│  └──────────────┬─────────────────────────┘  │
│                 │ Spawns                      │
│  ┌──────────────┴─────────────────────────┐  │
│  │   External Processes                   │  │
│  │   AI Agent (Claude/OpenCode)           │  │
│  │   Godot Engine (Headless)              │  │
│  │   Godot MCP Server                     │  │
│  └────────────────────────────────────────┘  │
└──────────────────────────────────────────────┘
```

## License

Private
