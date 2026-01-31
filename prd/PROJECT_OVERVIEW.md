# Godot Agentic IDE - Project Overview

**Project**: Desktop IDE wrapper for agentic AI code assistants (OpenCode/Claude Code) with native Godot game development integration  
**Tech Stack**: Tauri (desktop), OpenCode/Claude Code (AI agents), Godot 4.x (headless game engine), MCP (Model Context Protocol)  
**Goal**: Create a unified workspace for game development where AI agents can write code, preview games, test interactions, and manage resources natively.

---

## 1. Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                    TAURI DESKTOP APP                            │
│  (Main Process - Rust Backend + TypeScript/React Frontend)      │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         FRONTEND (React/TypeScript)                      │  │
│  │  - Chat Interface (AI Conversation)                      │  │
│  │  - Code Editor (File Viewer/Editor)                      │  │
│  │  - Godot Preview Panel (Game Runtime)                    │  │
│  │  - Shortcut Tools (Quick Actions)                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│           ↕ (Tauri IPC - invoke commands)                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         BACKEND (Rust)                                   │  │
│  │  - Tauri Commands Handler                                │  │
│  │  - Godot MCP Server Integration                          │  │
│  │  - AI Agent Process Manager                              │  │
│  │  - File System Operations                                │  │
│  │  - Godot Headless Executor                               │  │
│  └──────────────────────────────────────────────────────────┘  │
│           ↓ (spawns child processes)                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         EXTERNAL PROCESSES (Sidecars/CLI)                │  │
│  │  ┌─────────────────────┬──────────────────────────────┐  │  │
│  │  │  OpenCode/Claude    │  Godot Engine               │  │  │
│  │  │  Code Agent         │  (Headless Mode)            │  │  │
│  │  │  - Chat Modes       │  - Script Execution         │  │  │
│  │  │  - Build/Plan/Docs  │  - Scene Preview/Export     │  │  │
│  │  │  - File Editing     │  - Template Downloads       │  │  │
│  │  │  - Terminal Access  │  - Asset Management         │  │  │
│  │  └─────────────────────┴──────────────────────────────┘  │  │
│  └──────────────────────────────────────────────────────────┘  │
│           ↓ (MCP Protocol Communication)                       │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         GODOT MCP SERVER                                 │  │
│  │  - Scene Management (create, edit, delete)               │  │
│  │  - Node Operations (add, remove, modify)                 │  │
│  │  - Resource Access (templates, assets)                   │  │
│  │  - Project Management (files, settings)                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│           ↕ (Direct Godot Integration)                        │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │         GODOT PROJECT DIRECTORY                          │  │
│  │  - Scenes (.tscn)                                        │  │
│  │  - Scripts (.gd)                                         │  │
│  │  - Assets (sprites, audio, models)                       │  │
│  │  - project.godot (project configuration)                 │  │
│  │  - Templates/Libraries (downloaded via MCP)              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow: Chat → AI Agent → Code Changes → Godot Preview

```
User Input (Chat)
  ↓
OpenCode/Claude Code Agent (Agentic Loop)
  ├─ Perceive: Read codebase, understand project structure
  ├─ Plan: Determine implementation approach
  ├─ Action: Write scripts, create scenes
  └─ Observe: Validate results via Godot MCP
  ↓
Godot MCP Server
  ├─ Create/modify scenes
  ├─ Manage nodes and assets
  ├─ Download templates
  └─ Update project settings
  ↓
Godot Headless Executor
  ├─ Run game (headless mode)
  ├─ Test interactions
  ├─ Capture output/errors
  └─ Generate preview
  ↓
Frontend Preview Panel
  ├─ Display game output
  ├─ Show console logs
  ├─ Render error messages
  └─ Update code editor
```

---

## 2. Core Technologies Explained

### 2.1 Tauri v2 (Desktop Framework)

**What it does**: Wraps your web frontend (React/TypeScript) into a native desktop app with Rust backend.

**Key Features for this project**:
- **IPC (Inter-Process Communication)**: Frontend ↔ Backend communication via `invoke()` calls
- **File System Access**: Direct filesystem operations through `tauri_plugin_fs`
- **Shell/Process Management**: Spawn and manage CLI processes (OpenCode, Godot) via `tauri_plugin_shell`
- **Asset Protocol**: Serve local files (game previews, assets) to frontend
- **Capabilities System**: Fine-grained permissions for security

**Why Tauri?**
- Lightweight compared to Electron
- Native OS rendering engine (WebKit on macOS, WebView2 on Windows)
- Rust backend for system-level access
- Single codebase across Windows/macOS/Linux

**Documentation snippet**:
```rust
// Tauri v2 command handler pattern
#[tauri::command]
async fn run_godot_preview(
    project_path: String,
    scene_path: String,
) -> Result<PreviewOutput, String> {
    // Spawn godot headless, capture output
    // Return preview data to frontend
}

// In main.rs
tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_shell::init())
    .invoke_handler(tauri::generate_handler![
        run_godot_preview,
        save_file,
        read_file
    ])
    .run(tauri::generate_context!())
```

### 2.2 OpenCode/Claude Code (Agentic AI)

**What it does**: Autonomous AI assistant that can read files, run commands, and write code.

**Agent Loop** (the "agentic" part):
```
1. PERCEPTION: Read codebase, understand task
   └─ Can browse files, understand project structure
   
2. PLANNING: Break down problem into steps
   └─ Uses reasoning to plan approach
   
3. ACTION: Execute plan
   └─ Writes files, runs commands, modifies code
   
4. OBSERVATION: Check results
   └─ Validates output, runs tests
   
5. REFLECTION: Iterate or confirm success
   └─ Self-corrects if needed
```

**Why use both OpenCode and Claude Code?**
- **OpenCode**: Fully open-source, runs locally, free models available
- **Claude Code**: Advanced reasoning, better at complex problems, via Anthropic API
- **Switching dynamically**: Let AI agents handle different tasks with best-suited model

**In your context**:
```bash
# OpenCode (free, local)
opencode
> Create a player character scene with sprite animation

# Claude Code (premium, API-based)
claude --allow-tool "file-edit,bash,godot-mcp"
> Implement a state machine for player movement physics
```

**Key tool access** (what agents need):
- File operations (read/write/create)
- Terminal/bash access (run commands)
- MCP tools (Godot MCP server access)
- Project context (codebase understanding)

### 2.3 Godot MCP Server (Model Context Protocol)

**What it does**: Standardized interface letting AI agents talk to Godot engine.

**MCP = Model Context Protocol**: Open standard for AI assistants to access tools/resources.

**Godot MCP capabilities**:

| Tool | Purpose | Example |
|------|---------|---------|
| `create_scene` | Create new .tscn file | `create_scene(type: "CharacterBody2D", path: "Player.tscn")` |
| `add_node` | Add node to scene | `add_node(scene: "Player.tscn", type: "Sprite2D", name: "sprite")` |
| `edit_node` | Modify node properties | `edit_node(node: "Player/sprite", properties: {position: {x: 100, y: 50}})` |
| `remove_node` | Delete node | `remove_node(node: "Player/CollisionShape2D")` |
| `load_sprite` | Load texture into Sprite2D | `load_sprite(node: "sprite", texture: "res://assets/player.png")` |
| `run_project` | Launch game (headless) | `run_project(scene: "scenes/main.tscn")` |
| `get_file_uid` | Get resource UID (Godot 4.4+) | `get_file_uid(path: "res://Player.tscn")` |
| `export_meshlib` | Create MeshLibrary from scene | `export_meshlib(scene: "3d_assets.tscn")` |

**Installation**:
```bash
npm install @satelliteoflove/godot-mcp
# or from source
git clone https://github.com/bradypp/godot-mcp
cd godot-mcp
npm install && npm run build
```

**Configuration** (in Claude/OpenCode config):
```json
{
  "godotMcpServer": {
    "projectPath": "/path/to/godot/project",
    "executable": "/path/to/Godot",
    "readOnlyMode": false,
    "autoApprove": ["run_project", "get_scene_content"]
  }
}
```

**Documentation snippet** from Godot MCP:
```javascript
// Tool: create_scene
{
  "type": "function",
  "function": {
    "name": "create_scene",
    "description": "Create a new Godot scene file",
    "parameters": {
      "properties": {
        "projectPath": { "type": "string", "description": "Path to Godot project" },
        "scenePath": { "type": "string", "description": "Path for new scene (e.g., 'scenes/Player.tscn')" },
        "rootNodeType": { "type": "string", "description": "Root node type (e.g., 'CharacterBody2D')" }
      },
      "required": ["projectPath", "scenePath", "rootNodeType"]
    }
  }
}
```

### 2.4 Godot Headless Mode

**What it does**: Run Godot without GUI for CLI execution, testing, preview.

**Why headless?**
- AI agents can programmatically test game code
- Generate previews without opening Godot editor
- Automate builds/exports
- Capture console output for debugging

**Headless execution**:
```bash
# Run scene headless (GUI disabled)
godot --headless scene.tscn

# Run and exit when script finishes
godot --headless --quit-after 30 scene.tscn

# Redirect output
godot --headless scene.tscn > output.log 2>&1

# Custom arguments to script
godot --headless -a arg1 -a arg2 scene.tscn
```

**Project settings for headless**:
```gdscript
# In project.godot
[display]
display_server = "headless"

[audio]
driver = "Dummy"

[application]
run/print_header = false
```

**Example: Run preview and capture output**
```gdscript
# In your Godot script (main.gd)
extends Node

func _ready():
    print("Game started!")
    # Simulate game
    await get_tree().create_timer(3.0).timeout
    get_tree().quit()
```

**Tauri Rust handler**:
```rust
#[tauri::command]
async fn run_godot_preview(project_path: String) -> Result<String, String> {
    let output = std::process::Command::new("godot")
        .args(&["--headless", &project_path])
        .output()
        .map_err(|e| e.to_string())?;
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
```

---

## 3. Feature Set & Implementation

### 3.1 Chat Interface

**Purpose**: Conversation with AI agents for code generation/modification.

**Features**:
- Real-time chat with model selection (OpenCode variants or Claude via API)
- Mode switching: Build, Plan, Docs (inherited from OpenCode/Claude Code)
- Context awareness (codebase loaded, project understanding)
- Token counter display (how much context used)

**Architecture**:
```
Frontend Chat Panel
  ↓ (user types request)
Tauri Backend receives message
  ↓
Launch AI agent (OpenCode/Claude Code) subprocess
  ├─ Pass codebase context
  ├─ Set MCP server location
  └─ Enable Godot tools
  ↓
Agent processes request & returns result
  ↓
Stream response back to frontend
  ↓
Display in chat, update code editor, trigger preview
```

**Example flow**:
```
User: "Create a player sprite with animation"
  ↓
OpenCode Agent (Build mode):
  1. Reads project structure
  2. Creates Player.tscn scene
  3. Adds Sprite2D node
  4. Loads animation file
  5. Returns success message
  ↓
Godot MCP validates scene created
  ↓
Frontend shows: "Player scene created at res://scenes/Player.tscn"
```

### 3.2 File Viewer & Code Editor

**Purpose**: Browse and edit project files (scripts, scenes, assets).

**Features**:
- File tree view (left sidebar)
- Code editor with syntax highlighting (GDScript, JSON, etc.)
- Search/replace functionality
- Quick file preview
- Integration with AI: AI can suggest edits

**Architecture**:
```
Frontend File Tree
  ↓
Tauri fs::read_dir command
  ↓
Backend lists project directory
  ↓
Frontend displays hierarchy
  ↓
Click file → Read contents
  ↓
Display in editor with syntax highlighting
```

**Tauri Rust handlers**:
```rust
#[tauri::command]
async fn list_project_files(project_path: String) -> Result<Vec<FileEntry>, String> {
    // Use tauri_plugin_fs to list directory
}

#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    // Read file contents
}

#[tauri::command]
async fn write_file(path: String, content: String) -> Result<(), String> {
    // Write to file (triggered by user edit or AI)
}
```

### 3.3 Godot Preview Panel

**Purpose**: Live preview of Godot game running headless in real-time.

**Features**:
- Renders game output (captured via headless execution)
- Console log display (stdout/stderr from Godot)
- Error messages with line references
- Refresh button to re-run scene
- Performance stats (FPS, memory)

**How it works**:
1. User clicks "Preview" for a scene
2. Tauri spawns Godot in headless mode
3. Godot renders frame → PNG/WebP → saved to temp
4. Frontend requests image via asset protocol
5. Display in preview panel
6. Repeat at 30fps or on-demand

**Alternative: WebSocket streaming**:
```rust
// Optional: Godot outputs to WebSocket
// More complex but enables live updates
spawn_godot_with_websocket(project_path)
  ↓
Godot sends frame data every N ms
  ↓
Tauri WebSocket receives frames
  ↓
Frontend displays real-time
```

**Simple implementation** (frame capture):
```rust
#[tauri::command]
async fn preview_scene(project_path: String, scene_path: String) -> Result<String, String> {
    // Run godot headless
    let output = Command::new("godot")
        .args(&["--headless", &scene_path])
        .output()?;
    
    // Capture screenshot (if godot supports it)
    // Save to temp directory
    // Return path for frontend to load
}
```

### 3.4 Shortcut Tools for Godot

**Purpose**: Quick actions for common Godot tasks.

**Examples**:
- **New Scene**: Create blank scene with root node selection
- **Add Node**: Insert node at selected position
- **Create Script**: Generate .gd file template
- **Download Template**: Access Godot asset library
- **Export Project**: Build for platform
- **Run Tests**: Execute test scenes

**Architecture**:
```
Shortcut Button Clicked
  ↓
Tauri Command Invoked
  ↓
MCP Tool Called (if Godot operation)
  ↓
Result displayed in sidebar
  ↓
Refresh file tree/preview if needed
```

**Example: Create New Scene**
```typescript
// Frontend: shortcuts.tsx
async function createNewScene(rootNodeType: string) {
  const result = await invoke('create_scene', {
    projectPath: currentProject,
    scenePath: `scenes/NewScene.tscn`,
    rootNodeType: rootNodeType
  });
  
  // Refresh file tree
  refreshFileTree();
  // Show in editor
  openFile(`scenes/NewScene.tscn`);
}
```

```rust
// Backend: commands.rs
#[tauri::command]
async fn create_scene(
    project_path: String,
    scene_path: String,
    root_node_type: String,
) -> Result<String, String> {
    // Call Godot MCP server
    // Returns success message
}
```

---

## 4. Integration Architecture: How It All Works Together

### 4.1 Message Flow Example: "Add enemy patrol behavior"

```
┌─────────────────────────────────────────────────────────────────┐
│ User types in Chat: "Add enemy patrol AI with waypoints"        │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Tauri backend receives message                                  │
│ - Loads current project context                                 │
│ - Gathers file list, script snippets                            │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Spawn OpenCode/Claude Code process                              │
│ - Pass project path                                             │
│ - Register Godot MCP server address                             │
│ - Set agent mode (Build)                                        │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ AI Agent Starts Loop:                                           │
│                                                                 │
│ PERCEPTION:                                                     │
│  - Read project structure                                       │
│  - Find existing Enemy.gd script                                │
│  - Check scene hierarchy                                        │
│                                                                 │
│ PLANNING:                                                       │
│  - Design waypoint system                                       │
│  - Plan patrol script modifications                             │
│                                                                 │
│ ACTION:                                                         │
│  - Create Waypoint.gd script                                    │
│  - Add waypoint nodes to Enemy scene                            │
│  - Update Enemy.gd with patrol logic                            │
│                                                                 │
│ OBSERVATION (via Godot MCP):                                    │
│  - Validate scene structure                                     │
│  - Check script syntax                                          │
│  - Test enemy movement                                          │
│                                                                 │
│ REFLECTION:                                                     │
│  - Confirm implementation matches request                       │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Agent returns response with file changes                        │
│ - Created: Waypoint.gd                                          │
│ - Modified: Enemy.tscn                                          │
│ - Modified: Enemy.gd                                            │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Tauri backend processes changes                                 │
│ - Apply file modifications                                      │
│ - Refresh file tree                                             │
│ - Trigger preview                                               │
└─────────────────────────────────────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────────┐
│ Frontend updates:                                               │
│ - Show chat response                                            │
│ - Refresh file tree (show new Waypoint.gd)                      │
│ - Run preview (headless Godot shows enemy patrolling)           │
│ - Display console output & errors                               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Zed-like Model Switching

**Concept**: Like Zed editor switches between Claude Code and Gemini, switch between OpenCode variants and Claude Code.

**Implementation**:
```typescript
// Frontend: agent selector
const models = [
  { name: "OpenCode (Low)", variant: "low", local: true },
  { name: "OpenCode (Medium)", variant: "medium", local: true },
  { name: "OpenCode (High)", variant: "high", local: true },
  { name: "Claude Code", provider: "anthropic", local: false },
];

// User can switch mid-conversation
// Each agent has access to same project context via MCP
```

**Backend decision logic**:
```rust
enum AgentBackend {
    OpenCodeLow,
    OpenCodeMedium,
    OpenCodeHigh,
    ClaudeCode,
}

#[tauri::command]
async fn send_chat_message(
    message: String,
    agent_backend: AgentBackend,
    project_path: String,
) -> Result<String, String> {
    match agent_backend {
        AgentBackend::OpenCodeLow => {
            spawn_opencode_process(project_path, "low")
        }
        AgentBackend::ClaudeCode => {
            spawn_claude_code_process(project_path)
        }
        // ...
    }
}
```

### 4.3 Custom Rules & Agentic Configuration

**Purpose**: Define project-specific rules for AI agents.

**Implementation**: Create `agent.md` file in project root (OpenCode standard)

**agent.md template**:
```markdown
# Godot Project Guidelines

## Project Structure
- Scenes: `res://scenes/`
- Scripts: `res://scripts/`
- Assets: `res://assets/`

## Coding Standards
- Use GDScript 2.0 (Godot 4.x)
- Follow snake_case for functions
- Use type hints for all functions
- Prefer nodes over raw transforms

## Architecture Patterns
- Use Scenes for composition
- Scripts extend Nodes
- Signals for event handling
- Use State machines for AI

## Godot MCP Tools Available
- create_scene
- add_node
- edit_node
- run_project
- load_sprite

## Restrictions
- Do not modify godot.engine.exe
- Do not delete scenes without confirmation
- Keep frame rate stable (60 FPS target)

## Common Tasks
### Create new player character
1. Create CharacterBody2D scene
2. Add Sprite2D child
3. Add CollisionShape2D child
4. Write PlayerController.gd script

### Add enemy patrol
1. Create Enemy scene from template
2. Use Waypoint pattern
3. Use Signal for death event
```

**Usage in agent context**:
```bash
# OpenCode automatically reads agent.md
opencode
# Now has project context and rules

# Claude Code with MCP
claude --config agent.md
# Same rules apply
```

### 4.4 Godot Template/Asset Library Access

**Purpose**: AI agents can download and use Godot asset templates.

**Implementation options**:

**Option 1: Local Template Library**
```
godot-project/
├─ templates/
│  ├─ 2d-platformer/
│  │  ├─ Player.tscn
│  │  ├─ Enemy.tscn
│  │  └─ scripts/
│  ├─ top-down-rpg/
│  └─ 3d-fps/
```

**Option 2: Godot Assetlib Integration**
```bash
# MCP tool to download from official Godot asset library
download_asset(
  asset_id: "12345",
  destination: "res://addons/my_asset/"
)
```

**Option 3: GitHub Templates**
```rust
#[tauri::command]
async fn download_template(template_url: String, dest_path: String) -> Result<(), String> {
    // Clone GitHub repo or download ZIP
    // Extract to project
}
```

**Agent usage**:
```
User: "Use the 2D platformer template for this game"
  ↓
Agent:
  1. Reads templates/ directory via MCP
  2. Copies 2d-platformer/ files to project
  3. Adapts to project structure
  4. Modifies scripts as needed
```

---

## 5. Technical Stack Breakdown

### Frontend (Tauri Window)
```
React + TypeScript
├─ Chat Interface (input, message display)
├─ File Explorer (tree view)
├─ Code Editor (Monaco or similar)
├─ Preview Panel (game output)
├─ Shortcuts Panel (quick actions)
└─ Agent Selector (model/variant switching)
```

### Backend (Rust)
```
Tauri Runtime (v2)
├─ IPC Handler (chat, file ops, shortcuts)
├─ Process Manager (spawn OpenCode/Godot/MCP)
├─ File System Ops (read/write via plugin)
├─ Shell Commands (run executables)
└─ Asset Protocol Server (serve local files)
```

### External Processes
```
OpenCode Agent (CLI)
├─ Runs in terminal mode
├─ Connects to MCP server
├─ Accesses Godot tools
└─ Returns file changes

Claude Code Agent (CLI)
├─ Similar to OpenCode
├─ Via Anthropic API
├─ Premium reasoning
└─ Tool access via ACP/MCP

Godot Engine (Headless)
├─ Runs scenes
├─ Outputs logs/errors
├─ No GUI
└─ Exits automatically
```

### MCP Server (Node.js)
```
Godot MCP Server
├─ Listens on localhost:5000
├─ Handles scene operations
├─ Manages resources
└─ Validates changes
```

---

## 6. Development Roadmap

### Phase 1: MVP (Weeks 1-2)
- [ ] Tauri app skeleton (React frontend + Rust backend)
- [ ] Basic chat interface (stream OpenCode output)
- [ ] File tree view (list project files)
- [ ] File editor (read/display files)
- [ ] Godot MCP server installation & testing

### Phase 2: Core Integration (Weeks 3-4)
- [ ] Tauri IPC commands for all file operations
- [ ] Godot headless preview runner
- [ ] Preview panel with live output
- [ ] Connect OpenCode agent to MCP server
- [ ] Simple shortcut: "Create New Scene"

### Phase 3: Advanced Features (Weeks 5-6)
- [ ] Claude Code integration (API key handling)
- [ ] Model switching UI
- [ ] Agent.md configuration system
- [ ] Template library system
- [ ] Code editor with syntax highlighting
- [ ] Error display with line references

### Phase 4: Polish & Optimization (Week 7+)
- [ ] WebSocket streaming for live preview
- [ ] Performance optimization
- [ ] Error handling & edge cases
- [ ] Settings/preferences panel
- [ ] Multi-project support
- [ ] Custom keyboard shortcuts

---

## 7. Key Implementation Notes

### Handling Process Lifetimes
```rust
// Spawn agent process, don't wait for completion
// Stream output back in real-time
let mut child = Command::new("opencode")
    .args(&[project_path])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

// Read output in background, emit events
tokio::spawn(async move {
    while let Some(line) = read_line(&mut stdout).await {
        window.emit("agent_output", &line).ok();
    }
});
```

### MCP Server Registration
```rust
// Before spawning agent, ensure MCP server is running
// Register with environment variable
std::env::set_var("MCP_SERVER", "http://localhost:5000");

// Or pass as argument
let agent_process = Command::new("opencode")
    .env("GODOT_MCP_URL", "http://localhost:5000")
    .spawn()?;
```

### File Sync During Agent Operations
```
User starts chat
  ↓
Agent creates/modifies files
  ↓
Agent returns "files modified: [list]"
  ↓
Tauri backend detects changes
  ↓
Frontend refreshes file tree
  ↓
Show diffs for user review (optional)
```

### Security Considerations
- **Sandboxing**: Tauri capabilities control file access
- **Process isolation**: Agents run as separate processes
- **Read-only mode**: Option for Godot MCP (analysis without changes)
- **User approval**: Confirm major operations (scene creation, asset download)

---

## 8. Reference Documentation

### Tauri v2 Docs
- Main: https://v2.tauri.app/
- Commands: https://v2.tauri.app/reference/
- Plugins: https://v2.tauri.app/features/system/
- File System Plugin: https://docs.rs/tauri-plugin-fs/

### Godot MCP
- GitHub: https://github.com/bradypp/godot-mcp
- NPM: https://www.npmjs.com/package/@satelliteoflove/godot-mcp
- Tools: Listed in section 2.3 above

### OpenCode
- Docs: https://opencode.ai/
- GitHub: https://github.com/opencodesai/opencode
- Agent system: https://opencode.ai/docs/agents/

### Claude Code / Claude Agent SDK
- Docs: https://anthropic.com/agents
- Best practices: https://anthropic.com/engineering/claude-code-best-practices
- ACP Standard: https://zed.dev/acp/agent/claude-code

### Godot Headless
- CLI docs: https://docs.godotengine.org/stable/
- Headless setup: See section 2.4
- Server mode: https://docs.godotengine.org/stable/tutorials/

---

## 9. Example Commands Reference

### Tauri IPC Calls (Frontend → Backend)
```typescript
// Chat message
const response = await invoke('send_chat_message', {
  message: "Create a player scene",
  agentBackend: "opencode-medium",
  projectPath: "/path/to/project"
});

// File operations
const content = await invoke('read_file', { path: 'res://scripts/Player.gd' });
await invoke('write_file', { path: 'res://scripts/Player.gd', content: newContent });

// Godot preview
const preview = await invoke('run_godot_preview', {
  projectPath: '/path/to/project',
  scenePath: 'res://scenes/main.tscn'
});

// Shortcuts
await invoke('create_scene', {
  rootNodeType: 'CharacterBody2D',
  sceneName: 'Player'
});
```

### Agent Backend Commands (Via Subprocess)
```bash
# OpenCode with project context
opencode --project /path/to/godot/project

# Claude Code with MCP
claude --mcp-server http://localhost:5000

# Godot headless with script
godot --headless --script res://scripts/test_enemy.gd

# MCP direct access
curl http://localhost:5000/tools/create_scene \
  -d '{"type":"CharacterBody2D","path":"res://Player.tscn"}'
```

---

## 10. Next Steps

1. **Set up Tauri project**: `cargo tauri init`
2. **Install Godot MCP**: `npm install @satelliteoflove/godot-mcp`
3. **Create React frontend**: Set up file tree, chat, preview panels
4. **Implement core Tauri commands**: File I/O, process spawning
5. **Test OpenCode integration**: Verify agent can access project
6. **Add preview runner**: Godot headless execution & display
7. **Implement shortcuts**: Quick scene/node creation
8. **Add model switching**: OpenCode variant + Claude Code selection

---

**Status**: Architecture defined, ready for implementation  
**Primary Challenge**: Seamless integration between four systems (Tauri ↔ Agent ↔ MCP ↔ Godot)  
**Key Success Factor**: Robust message passing and error handling across process boundaries
