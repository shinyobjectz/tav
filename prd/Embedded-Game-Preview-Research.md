# Research Brief: Embedded Godot Game Preview in Desktop Application

## Project Context

We are building **Kobold**, a desktop IDE for Godot game development. The application is built with:
- **Tauri 2.0** (Rust backend + WebView frontend)
- **React/TypeScript** frontend
- **Godot 4.3** as the target game engine

The IDE needs to display a **live, interactive preview** of the user's Godot game directly inside the application window (not in a separate external window).

## Current Approach & Problems

### Current Implementation
We export the Godot project to HTML5/WebAssembly and display it in an iframe:

1. Run `godot --headless --export-debug "Web" ./output/index.html`
2. Serve the exported files via a local HTTP server (with CORS headers for WASM)
3. Display in an iframe inside our Tauri webview

### Problems with Current Approach
1. **Slow initial export** - Even for tiny projects (3-4 files), export takes 5-15 seconds due to Godot startup overhead
2. **Requires export templates** - User must have Godot Web export templates installed (~500MB download)
3. **Double webview overhead** - Running a webview inside a webview (Tauri's webview → iframe → Godot's web runtime)
4. **No hot reload** - Any change requires re-export

## What We Need

An approach that provides:
1. **Fast iteration** - Preview should update in <1 second after file changes
2. **Interactive** - User can click/type to interact with the game
3. **Embedded** - Must be inside our app window, not a separate window
4. **Cross-platform** - Must work on Windows, macOS, and Linux
5. **Minimal setup** - Ideally no extra downloads or configuration for users

## Research Questions

### 1. Godot Remote/Streaming Approaches
- Does Godot 4 have any built-in remote display or streaming capabilities?
- Can Godot render to a framebuffer that can be streamed over a socket?
- Is there a way to run Godot as a "server" that streams video to a client?
- What about `--render-driver` options - can Godot render offscreen and output frames?

### 2. Native Window Embedding
- Can a Godot window be reparented into another application's window?
  - Windows: `SetParent()` Win32 API
  - macOS: NSView embedding
  - Linux: X11 reparenting / Wayland?
- Has anyone successfully embedded Godot's window into Electron/Tauri/Qt applications?
- What are the cross-platform challenges?

### 3. Godot as a Library
- Can Godot 4 be compiled as a shared library (libgodot)?
- Is there a GDExtension or native interface that allows hosting Godot's renderer?
- Can we initialize Godot's rendering in our own window context?

### 4. Alternative Streaming Approaches
- **WebRTC**: Could Godot stream its display via WebRTC to our webview?
- **FFmpeg/video encoding**: Run Godot headless, capture frames, encode to video stream?
- **Shared memory**: Godot writes frames to shared memory, Rust reads and sends to frontend?
- **VNC/RDP-like**: Any lightweight remote desktop protocols suitable for localhost game streaming?

### 5. Godot's Web Export Optimization
- Can Godot's web export be made incremental (only re-export changed files)?
- Is there a way to keep the Godot export process "warm" to avoid startup overhead?
- Can we pre-compile/cache parts of the web export?

### 6. Editor Integration Approaches
- How do other Godot IDEs (official editor, Godot-Rust, etc.) handle live preview?
- Does Godot have a `--editor` mode that exposes preview functionality?
- Is there a way to connect to a running Godot instance and control it remotely?

### 7. Performance Benchmarks Needed
- What's the typical latency for:
  - WebRTC game streaming (localhost)?
  - Shared memory frame transfer?
  - Window embedding?
- What framerate is achievable for each approach?

## Technical Constraints

1. **Tauri WebView**: We use Tauri which embeds a system webview (WebView2 on Windows, WebKit on macOS/Linux). We can:
   - Render web content (HTML/JS/WASM)
   - Call Rust functions from JavaScript
   - Access native APIs from Rust
   
2. **Godot Version**: Targeting Godot 4.3+ (latest stable)

3. **Project Sizes**: Ranging from tiny (4-5 files) to large (hundreds of assets)

4. **User Environment**: Users have Godot installed locally; we know the path to their Godot executable

## Ideal Solution Characteristics

Ranked by priority:
1. **Speed** - Sub-second preview updates
2. **Cross-platform** - Single solution for all OSes
3. **Simplicity** - Minimal dependencies/setup
4. **Fidelity** - Full game rendering, not screenshots
5. **Interactivity** - Keyboard/mouse input works

## Existing Research/Projects to Investigate

- Godot's `--headless` and `--render-driver` command line options
- Godot's remote debugging protocol
- GodotSteam and other embedding projects
- Electron-based Godot tools (how do they handle preview?)
- Unity's approach to embedded preview in external tools
- Unreal's approach to remote session rendering

## Deliverables Requested

1. **Comparison matrix** of approaches (speed, complexity, cross-platform support)
2. **Recommended approach** with justification
3. **Proof-of-concept guidance** for top 2-3 approaches
4. **Links to relevant documentation**, GitHub issues, or community discussions
5. **Known limitations or blockers** for each approach

---

## Contact

For clarification on requirements or technical constraints, please ask. We can provide:
- Sample Godot project for testing
- Access to test the current HTML5 export approach
- Specific Tauri/Rust implementation details if needed
