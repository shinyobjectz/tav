# AI Game-Playing for Agentic Loops: SOTA Research & Implementation Guide

**Purpose**: Research document for integrating game-playing capabilities into the Godot Agentic IDE, enabling Claude Code / OpenCode agents to play Godot games, capture footage, and debug visually.

---

## 1. Executive Summary: State-of-the-Art Approaches (2024-2025)

### Current Landscape
- **Vision-Language Models (VLMs)** are the dominant approach for game-playing AI (not RL)
- **Best performers**: Gemini 2.5 Pro, Claude 3.5 Sonnet, GPT-4o with vision
- **SOTA success rates**: Even frontier models achieve only 0.48-1.6% completion on real video games
- **Key bottleneck**: Inference latency (1-2+ seconds per action decision)
- **Frontier approach**: Vision-Language-Action models (VLAs) + specialized task modules

### Why VLMs Over RL?
- **No game API needed**: Works with raw pixels only
- **Generalizes better**: Single model plays multiple game genres
- **Human-interpretable**: Can explain reasoning via text
- **Faster iteration**: No 1000+ hour training loops like RL
- **Better integration with agentic loops**: Natural language reasoning + action planning

---

## 2. SOTA Frameworks & Architectures

### 2.1 **PORTAL Framework** (Latest 2025, Thousands of 3D Games)

**Paper**: "Agents Play Thousands of 3D Video Games via Language-Guided Policy Generation"  
**Key Innovation**: Converts gameplay into language modeling task using Domain-Specific Language (DSL)

**Architecture**:
```
LLM → Generate Behavior Trees (DSL) → Parse to Actions → Execute → Vision-Language Feedback Loop
```

**How it works**:
1. LLM generates high-level behavior tree in domain-specific language
2. Hybrid policy: Rule-based nodes + neural network components for precision control
3. Dual-feedback mechanism:
   - Quantitative game metrics (score, health, position)
   - Vision-language model analysis of screenshots for tactical improvements
4. Policies are instantly deployable and human-interpretable

**Advantages**:
- Handles FPS games, strategy games, 3D environments
- No reinforcement learning training overhead
- Behavior trees are debuggable and modifiable
- Generalizes across thousands of games

**Relevance to Your Project**: 
- Use LLM to generate Godot-specific behavior scripts
- Parse into GDScript or MCP tool calls
- Perfect for your agent → Godot workflow

---

### 2.2 **GameSense Framework** (March 2025, Fluent Gameplay)

**Paper**: "Cultivating Game Sense for Yourself: Making VLMs Gaming Experts"  
**Key Innovation**: VLM develops task-specific execution modules instead of direct control

**Architecture**:
```
VLM (High-level reasoner)
  ↓
Observes → Plans → Delegates to specialized modules
  ↓
Shooting Module | Combat Module | Navigation Module
  ↓
Real-time execution with neural network decision-making
```

**How it works**:
1. VLM identifies task type (shooting, combat, puzzle-solving, etc.)
2. For each task, VLM develops a specialized execution module
3. Modules can be:
   - Rule-based (e.g., "if enemy in range, press shoot")
   - Neural network (trained via observation of gameplay)
4. VLM trains these modules by observing task execution and refining action-feedback logic
5. Modules handle real-time interactions while VLM focuses on high-level strategy

**Performance**:
- First framework to achieve fluent gameplay across ACT (action), FPS, and casual (Flappy Bird)
- Handles complex, dynamic scenarios that pure VLM reasoning can't manage alone

**Relevance to Your Project**: 
- Agents can develop GDScript modules for specific gameplay tasks
- Store modules as reusable scripts in project
- Iteratively improve through observation-based learning

---

### 2.3 **AVA Framework** (March 2025, StarCraft II)

**Paper**: "Attentive VLM Agent for Mastering StarCraft II"  
**Focus**: Complex RTS game with multiple units and strategic reasoning

**Architecture**:
```
Vision-Language Model + Self-Attention Mechanisms
  ├─ Strategic unit targeting
  ├─ Battlefield assessment
  └─ Retrieval-Augmented Generation (RAG)
```

**Components**:
1. **Enhanced VLM**: Standard VLM + specialized attention layers
2. **Unit tracking**: Identify, track, classify game entities
3. **RAG system**: Retrieve past strategies and apply to current state
4. **Multimodal grounding**: Link visual percepts (unit positions) to semantic concepts (tactics)

**Key Insight**: RAG helps VLM remember effective strategies from past observations

**Relevance to Your Project**: 
- Use RAG to store successful Godot gameplay patterns
- Agents can query past gameplay strategies to solve similar challenges
- Visual entity tracking for debugging ("where was the player when the bug occurred?")

---

### 2.4 **CombatVLA** (March 2025, 3D Action RPGs)

**Paper**: "Efficient Vision-Language-Action Model for Combat Tasks in ARPGs"

**Key Innovation**: Specialized VLA (Vision-Language-Action) model for action-heavy games

**Architecture**:
```
Input: Screenshot + Previous Actions (video-action pairs)
  ↓
VLA Model (3B parameters)
  ├─ Visual encoding (what's on screen)
  ├─ Language understanding (task description)
  └─ Action generation (next input to game)
  ↓
Output: Direct game action (keyboard/mouse input)
```

**Training data**:
- Video-action pair sequences
- Action-of-thought (AoT) format: screenshots → textual reasoning → actions
- Collected from gameplay footage with input logging

**Performance**:
- Optimized for real-time combat (fast inference)
- Works without pausing game
- Better than VGM alone for action-heavy tasks

**Relevance to Your Project**: 
- Fine-tune VLAs on Godot game recordings
- Store gameplay video + input logs for training
- Use for faster, more reactive agents

---

## 3. Vision-Language Models (VLMs) for Game-Playing

### 3.1 Which VLMs Work Best?

**Frontier Models (2025 Rankings)**:

| Model | Provider | Strengths | Limitations | Cost |
|-------|----------|-----------|------------|------|
| **Gemini 2.5 Pro** | Google | Fastest inference, best real-time performance | Still 0.48% game completion | $20/1M tokens |
| **Claude 3.5 Sonnet** | Anthropic | Best reasoning, multi-step planning | Slower inference | $3/$15 per 1M tokens |
| **GPT-4o** | OpenAI | Good visual understanding, multimodal | Expensive, slower | $5/$15 per 1M tokens |
| **Qwen-VL** | Alibaba | Open-source, multilingual | Less game-specific training | Free (open) |
| **LLaVA** | Open-source | Free, runnable locally | Lower accuracy than proprietary | Free (open) |

### 3.2 Best Practices for VLM Game-Playing

**Input Strategy**:
```
Vision Input
├─ Single screenshot (0.5s delay recommended)
├─ Or last 5 frames (for turn-based interpretation)
├─ Or video clip (for temporal understanding)
└─ Low resolution ok (240p sufficient to reduce tokens)

Text Input
├─ High-level objective ("reach the goal", "defeat enemy")
├─ Control instructions ("arrow keys move, space jumps")
├─ Current game state summary (optional but helps)
└─ Memory/scratchpad (what agent learned so far)
```

**Memory Mechanism**:
```
After each step:
1. VLM outputs: [THINKING] internal reasoning about game state
2. VLM outputs: [MEMORY] important info to track (enemy positions, inventory, etc.)
3. Store MEMORY in text scratchpad
4. Pass scratchpad to next VLM call
→ Enables temporal reasoning and learning from experience
```

**Action Output Format**:
```
VLM generates natural language: "press up_arrow for 2 seconds", "click mouse at 50%,60%"
→ Parse with regex/template matching
→ Execute via emulator/game control interface
```

**Latency Optimization**:
```
Real-time (direct):
└─ VLM inference time: 1-3 seconds
└─ Game moves during VLM thinking
└─ Agent is out of sync (Gemini 2.5 Pro: 0.48% success)

Turn-based / Lite mode (paused):
└─ Game pauses while VLM thinks
└─ Infinite reasoning time per action
└─ Much better performance (Gemini 2.5 Pro: 1.6% success)
└─ Recommended for complex strategy games
```

---

## 4. Game Playing Architecture for Your Godot IDE

### 4.1 Overall System Design

```
┌─────────────────────────────────────────────────────┐
│         Claude Code / OpenCode Agent                │
│         (Running in subprocess)                      │
└─────────────┬───────────────────────────────────────┘
              │ (MCP requests + game commands)
              ↓
┌─────────────────────────────────────────────────────┐
│         Tauri Backend                               │
│  - Game Controller (capture, control, state)        │
│  - VLM Interface (Claude API, etc.)                 │
│  - Logging & Screenshot Management                  │
└─────────────┬───────────────────────────────────────┘
              │ (IPC commands)
              ↓
┌──────────────────────┬──────────────────────────────┐
│  Godot Headless      │    Screenshot Capture        │
│  (Game Runtime)      │    Input Injection           │
│  - Running scene     │    Frame Buffer Access       │
│  - Executing scripts │                              │
└──────────────────────┴──────────────────────────────┘
```

### 4.2 Tauri Backend: Game Control Module

**New Tauri Command for Game Playing**:

```rust
// src/commands/game_controller.rs

#[tauri::command]
async fn start_game_session(
    project_path: String,
    scene_path: String,
    agent_backend: String, // "claude" or "opencode"
) -> Result<GameSessionId, String> {
    // 1. Launch Godot headless
    // 2. Start game in specified scene
    // 3. Initialize screenshot capture
    // 4. Return session ID
}

#[tauri::command]
async fn get_game_screenshot(session_id: String) -> Result<String, String> {
    // 1. Capture current frame from running Godot
    // 2. Encode as base64 PNG
    // 3. Return image data
    // Size: 640x480 or lower (balance quality/tokens)
}

#[tauri::command]
async fn send_game_input(
    session_id: String,
    action: String, // "key_press:up", "mouse_click:50,50", etc.
) -> Result<GameState, String> {
    // 1. Parse action string
    // 2. Inject into Godot via emulation/IPC
    // 3. Advance game by 1 frame (or N frames)
    // 4. Return new game state/screenshot
}

#[tauri::command]
async fn get_game_state(session_id: String) -> Result<GameState, String> {
    // Return structured game state:
    // - Current scene
    // - Player position
    // - Health/score
    // - Any custom game variables exposed
}

#[tauri::command]
async fn stop_game_session(session_id: String) -> Result<(), String> {
    // Cleanup: stop Godot process, close logs, etc.
}
```

**Game State Structure**:
```rust
#[derive(Serialize)]
struct GameState {
    scene: String,
    screenshot: String, // base64 PNG
    logs: Vec<String>, // console output
    frame_number: u32,
    custom_data: serde_json::Value, // Game-specific data exposed to agent
}
```

### 4.3 Screenshot Capture Implementation

**Option A: Headless Godot Screenshot API**
```gdscript
# In your Godot main.gd scene (headless mode)
extends Node

var screenshot_dir = "user://screenshots/"
var frame_count = 0

func _ready():
    DirAccess.make_absolute("user://screenshots/")

func _process(_delta):
    # Capture screenshot every frame
    var viewport = get_viewport()
    var image = viewport.get_texture().get_image()
    var filename = screenshot_dir.path_join("frame_%06d.png" % frame_count)
    image.save_png(filename)
    frame_count += 1
    
    # Also print game state for logs
    print("GAME_STATE: frame=%d" % frame_count)
```

**Option B: Godot GDExtension / C++**
- Direct frame buffer access
- Faster than disk I/O
- Return frames via IPC to Tauri

**Option C: Godot --output flag (headless)**
```bash
godot --headless --script=record.gd --output=frames.mkv
# Record entire session to video, then extract frames
```

### 4.4 Input Injection to Godot

**Method 1: GDScript Input Simulation (Built-in)**
```gdscript
# In your Godot scene - agent-controlled input
extends CharacterBody2D

func _input(event):
    if event is InputEventKey:
        if event.keycode == KEY_UP:
            velocity.y -= 100

# Override _process to handle simulated input
func _process(delta):
    # Check for input from agent
    if Input.is_action_pressed("ui_up"):
        velocity.y = -speed
```

**Method 2: Direct Node Control (Cleaner)**
```gdscript
# Create AgentController node that directly calls game functions
extends Node
class_name AgentController

var player: CharacterBody2D

func _ready():
    player = get_tree().root.find_child("Player", true, false)

func execute_action(action: String):
    match action:
        "move_left":
            player.velocity.x = -player.speed
        "move_right":
            player.velocity.x = player.speed
        "jump":
            if player.is_on_floor():
                player.velocity.y = -player.jump_force
        "attack":
            player.attack()
```

**Method 3: Godot Script Execution via MCP**
```
Agent → MCP tool: execute_godot_command(node_path, method, args)
MCP → GDScript: player.execute_action("jump")
```

### 4.5 VLM Integration: Claude API Wrapper

**File: src/commands/vl_controller.rs**

```rust
use anthropic::client::Client;
use base64::encode;
use std::fs;

#[tauri::command]
async fn analyze_game_state(
    screenshot_base64: String,
    objective: String,
    memory: String, // Scratchpad of previous observations
    available_actions: Vec<String>, // Possible actions for this game
) -> Result<AgentDecision, String> {
    let client = Client::new(
        std::env::var("ANTHROPIC_API_KEY")
            .map_err(|_| "Missing ANTHROPIC_API_KEY".to_string())?
    );

    // Build prompt with game context
    let prompt = format!(
        r#"You are an AI agent playing a video game. 

Objective: {}

Available Actions: {}

Previous Observations and Memory:
{}

Current screenshot analysis:
1. What do you see in the game?
2. What is your current goal?
3. What action should you take next?
4. Why is this the best action?

Respond in this format:
OBSERVATION: <what you see>
MEMORY: <important state to remember>
ACTION: <specific action like "move_left" or "attack">
REASONING: <explanation>"#,
        objective,
        available_actions.join(", "),
        memory
    );

    let message = client
        .messages()
        .create(anthropic::messages::MessageCreateParams {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 500,
            messages: vec![
                anthropic::messages::MessageParam::User(
                    anthropic::messages::UserMessageParam {
                        content: anthropic::messages::UserMessageContent::MultiBlock(
                            vec![
                                anthropic::messages::ContentBlockParam::Text(
                                    anthropic::messages::TextBlockParam {
                                        text: prompt,
                                    },
                                ),
                                anthropic::messages::ContentBlockParam::Image(
                                    anthropic::messages::ImageBlockParam {
                                        source: anthropic::messages::ImageBlockParamSource::Base64 {
                                            media_type: anthropic::messages::ImageMediaType::ImagePng,
                                            data: screenshot_base64.clone(),
                                        },
                                    },
                                ),
                            ],
                        ),
                    },
                ),
            ],
            ..Default::default()
        })
        .await
        .map_err(|e| e.to_string())?;

    // Parse response
    let response_text = message
        .content
        .iter()
        .find_map(|block| {
            if let anthropic::messages::ContentBlock::Text(text_block) = block {
                Some(text_block.text.as_str())
            } else {
                None
            }
        })
        .ok_or("No text response from Claude")?;

    parse_agent_decision(response_text)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct AgentDecision {
    observation: String,
    memory: String,
    action: String,
    reasoning: String,
}

fn parse_agent_decision(response: &str) -> Result<AgentDecision, String> {
    // Parse OBSERVATION, MEMORY, ACTION, REASONING from response
    // Return structured decision
    todo!()
}
```

### 4.6 Godot Agent MCP Tool

**New MCP Tool: play_game**

```json
{
  "name": "play_game",
  "description": "Play a Godot game and capture gameplay footage for debugging",
  "input_schema": {
    "type": "object",
    "properties": {
      "action": {
        "type": "string",
        "enum": ["start_session", "take_screenshot", "send_input", "get_state", "stop_session", "get_logs"],
        "description": "What to do with the game"
      },
      "scene_path": {
        "type": "string",
        "description": "Path to Godot scene to play (e.g., 'res://scenes/main.tscn')"
      },
      "game_input": {
        "type": "string",
        "description": "Input action (e.g., 'move_left', 'jump', 'attack')"
      },
      "objective": {
        "type": "string",
        "description": "High-level goal for the AI agent (e.g., 'reach the goal at the top of the level')"
      }
    },
    "required": ["action"]
  }
}
```

**Example Agent Flow**:
```
Agent: "I'll play the game to test the player movement code"
  ↓
Agent uses MCP: play_game(action="start_session", scene_path="res://scenes/level1.tscn", objective="reach the goal")
  ↓
Tauri spawns Godot, returns screenshot
  ↓
Agent: "I see a player on the left, goal on the right. I'll move right."
  ↓
Agent uses MCP: play_game(action="send_input", game_input="move_right")
  ↓
Tauri injects input, captures new screenshot
  ↓
Agent: "Player moved! Let me continue..."
  ↓
Agent uses MCP: play_game(action="take_screenshot") [loop until goal reached or issue found]
  ↓
Agent uses MCP: play_game(action="get_logs") [retrieve console output for debugging]
  ↓
Agent: "Found an issue: player jumps too high. Here's the fix..."
```

---

## 5. Open-Source Tools & Libraries

### 5.1 Game Automation Frameworks

| Tool | Purpose | Language | License | Use Case |
|------|---------|----------|---------|----------|
| **Playwright** | Browser/Game automation via screenshot | Python/JS/Rust | MIT | Capture frames, inject inputs (via custom wrapper) |
| **OpenAI Gym** | RL environment standard | Python | MIT | Wrap Godot as OpenAI Gym environment |
| **PettingZoo** | Multi-agent RL environments | Python | MIT | Multi-agent gameplay scenarios |
| **ALE (Atari)** | Classic game emulation | C++/Python | GPL | Reference for pixel-based agents |
| **PyGame** | Game framework (can integrate Godot) | Python | LGPL | Lightweight game simulation |
| **GDRLAgents** | Godot RL Agents library | GDScript | MIT | Native Godot RL integration (see below) |

### 5.2 Godot-Specific RL/Agent Libraries

**godot_rl_agents** (GitHub: edbeeching/godot_rl_agents)

```
Purpose: Integrate RL agents into Godot
- AIController3D / AIController2D nodes
- Python gym environment for Godot scenes
- Headless gameplay with observation/action spaces
- Perfect for training agents on Godot games

Installation:
git clone https://github.com/edbeeching/godot_rl_agents
pip install -r requirements.txt

Usage in Godot:
1. Create scene with AIController node
2. Define observation space (what agent sees)
3. Define action space (what agent can do)
4. Train with PPO / SAC via Python
```

### 5.3 Vision-Language Model Libraries

| Library | Purpose | Installation |
|---------|---------|--------------|
| **anthropic** | Claude API (Python/JS) | `pip install anthropic` |
| **openai** | GPT-4o vision API | `pip install openai` |
| **google-generativeai** | Gemini API | `pip install google-generativeai` |
| **ollama** | Local LLM inference | Download from ollama.ai |
| **llama-cpp-python** | Run LLaVA locally | `pip install llama-cpp-python` |
| **transformers** | HuggingFace VLM inference | `pip install transformers torch` |

### 5.4 Video/Screenshot Processing

| Library | Purpose | Use in Your Project |
|---------|---------|-------------------|
| **opencv-python** | Image processing | Frame analysis, optical flow for motion detection |
| **pillow** | Image I/O | Screenshot capture, conversion |
| **moviepy** | Video editing | Compile screenshots into gameplay video |
| **ffmpeg** | Video encoding | Encode gameplay videos for storage |
| **perceptual-hash** | Image matching | Track progress using hashed frames (like VideoGameBench) |

---

## 6. Implementation: Step-by-Step for Your Project

### 6.1 Phase 1: MVP - Basic Game Capture & Control

**Goal**: Agent can play simple Godot game and capture footage

**Implementation**:
1. **Godot Side** (GDScript):
   ```gdscript
   # addons/agent_controller/agent_controller.gd
   extends Node
   
   var agent_enabled = false
   var action_queue = []
   
   func queue_action(action: String):
       action_queue.push_back(action)
   
   func _process(_delta):
       if action_queue.size() > 0:
           var action = action_queue.pop_front()
           execute_action(action)
   
   func execute_action(action: String):
       # Connect to player script
       var player = get_tree().root.find_child("Player", true, false)
       match action:
           "move_left": player.velocity.x = -player.speed
           "move_right": player.velocity.x = player.speed
           "jump": player.jump()
           _: print("Unknown action: " + action)
   ```

2. **Tauri Backend** (Rust):
   ```rust
   #[tauri::command]
   async fn capture_game_frame(project_path: String) -> Result<String, String> {
       // Spawn Godot headless, capture frame, return as base64
       let output = Command::new("godot")
           .args(&["--headless", &format!("{}res://main.tscn", project_path)])
           .output()?;
       // Extract screenshot (implement)
   }
   
   #[tauri::command]
   async fn inject_game_input(action: String) -> Result<(), String> {
       // Send action to running Godot process
   }
   ```

3. **Frontend** (React):
   ```typescript
   const [screenshot, setScreenshot] = useState("");
   
   const captureFrame = async () => {
       const frame = await invoke("capture_game_frame", { projectPath });
       setScreenshot(frame);
   };
   
   const playGame = async (actions: string[]) => {
       for (const action of actions) {
           await invoke("inject_game_input", { action });
           const newFrame = await captureFrame();
           setScreenshot(newFrame);
           await new Promise(r => setTimeout(r, 500)); // 0.5s per action
       }
   };
   ```

### 6.2 Phase 2: Claude Integration

**Goal**: Claude analyzes game state and decides next action

**Implementation**:
```rust
#[tauri::command]
async fn ai_game_step(
    screenshot: String, // base64
    objective: String,
    memory: String,
) -> Result<AgentDecision, String> {
    let client = anthropic::Client::new(
        std::env::var("ANTHROPIC_API_KEY")?
    );
    
    let message = client.messages().create(
        // Build request with screenshot + context (see section 4.5)
    ).await?;
    
    // Parse action from response
    Ok(parse_decision(&message.content[0]))
}
```

**Agent Loop** (in OpenCode/Claude Code):
```bash
# Agent runs this:
loop {
    1. screenshot = capture_game_frame()
    2. decision = ai_game_step(screenshot, objective, memory)
    3. inject_game_input(decision.action)
    4. memory.append(decision.memory)
    5. if decision.action == "debug_pause":
        break
}
```

### 6.3 Phase 3: Visual Debugging Dashboard

**Goal**: Real-time visualization of agent gameplay with logs

**Frontend Component**:
```typescript
// GameplayDebugger.tsx
export const GameplayDebugger = () => {
    const [frames, setFrames] = useState<FrameCapture[]>([]);
    const [currentFrame, setCurrentFrame] = useState(0);
    const [logs, setLogs] = useState<string[]>([]);
    
    return (
        <div className="gameplay-debugger">
            <div className="viewport">
                {frames[currentFrame] && (
                    <img src={frames[currentFrame].screenshot} />
                )}
            </div>
            
            <div className="timeline">
                {frames.map((f, i) => (
                    <button
                        key={i}
                        onClick={() => setCurrentFrame(i)}
                        className={currentFrame === i ? "active" : ""}
                    >
                        Frame {i}: {f.action}
                    </button>
                ))}
            </div>
            
            <div className="logs">
                {logs.map((log, i) => (
                    <div key={i}>{log}</div>
                ))}
            </div>
        </div>
    );
};
```

### 6.4 Phase 4: Advanced Features

**A. Gameplay Video Export**:
```python
# Export frames to video
import cv2
from pathlib import Path

frames_dir = Path("user://screenshots/")
frames = sorted(frames_dir.glob("frame_*.png"))

video = cv2.VideoWriter(
    "gameplay.mp4",
    cv2.VideoWriter_fourcc(*"mp4v"),
    30, # 30 fps
    (640, 480)
)

for frame_path in frames:
    img = cv2.imread(str(frame_path))
    video.write(img)

video.release()
```

**B. Progress Tracking (like VideoGameBench)**:
```python
# Use perceptual hashing to track progress
from PIL import Image
import imagehash

checkpoint_frames = load_walkthrough_checkpoints()
gameplay_frames = load_captured_frames()

completion_score = 0
for checkpoint in checkpoint_frames:
    checkpoint_hash = imagehash.phash(checkpoint)
    for frame in gameplay_frames:
        frame_hash = imagehash.phash(frame)
        distance = checkpoint_hash - frame_hash  # Hamming distance
        if distance < 5:  # Threshold
            completion_score += 1
            break

completion_percent = (completion_score / len(checkpoint_frames)) * 100
```

**C. State Machine Visualization**:
```
During gameplay, track and visualize:
- Current scene
- Player state (position, health, animation)
- Enemy positions
- Game events triggered
- AI decision timeline
```

---

## 7. Integration with Your Architecture

### 7.1 Modified Data Flow

```
User Chat: "Debug why player movement is broken"
  ↓
OpenCode Agent (Planning mode)
  1. Reads player.gd script
  2. Understands issue: "Jump too high"
  3. Plans: Play game to observe, verify issue
  ↓
Agent uses MCP: play_game (start_session, scene="player_test.tscn")
  ↓
Tauri Backend
  - Spawns Godot headless
  - Injects AIController
  - Captures 10 frames of gameplay
  ↓
Agent receives screenshots + logs
  ↓
Claude API (Vision) analyzes:
  "I see player jump is 500 pixels, should be 300. Let me fix..."
  ↓
Agent modifies player.gd
  ↓
Agent uses MCP: play_game (send_input, "jump")
  ↓
New screenshots show corrected behavior
  ↓
Agent: "Fixed! Player now jumps correct height."
```

### 7.2 New MCP Tools to Add

**To your godot-mcp server** (or create new VLM-aware MCP):

```json
{
  "tools": [
    {
      "name": "game_start",
      "description": "Start playing a Godot scene headless with agent control",
      "input": {
        "scene_path": "string (res://...)",
        "duration_seconds": "number (how long to play)"
      }
    },
    {
      "name": "game_send_action",
      "description": "Send input action to running game",
      "input": {
        "action": "string (move_left, jump, attack, etc.)"
      }
    },
    {
      "name": "game_get_state",
      "description": "Get current game state with screenshot",
      "input": {}
    },
    {
      "name": "game_get_logs",
      "description": "Retrieve console output from gameplay",
      "input": {}
    },
    {
      "name": "game_export_video",
      "description": "Export captured gameplay to MP4",
      "input": {
        "output_path": "string",
        "fps": "number"
      }
    }
  ]
}
```

### 7.3 Agent Configuration (agent.md)

```markdown
# Agent Instructions for Godot Game Testing

## Game Playing Capabilities

You have access to game-playing tools:
- `game_start`: Launch game in headless mode
- `game_send_action`: Input gameplay action
- `game_get_state`: Capture screenshot + game state
- `game_export_video`: Create MP4 of gameplay

## How to Test Games

1. When user asks to "test X behavior", use game_start
2. Observe the behavior through screenshots
3. If issue observed, use game_export_video to capture it
4. Analyze what went wrong
5. Modify the relevant GDScript
6. Test again to verify fix

## Available Game Actions

For this Godot project, these actions are available:
- move_left, move_right, move_up, move_down
- jump, attack, interact
- pause, resume

## Example Workflow

User: "Debug why player can't jump"
Your Plan:
  1. game_start(scene="res://test/player.tscn")
  2. game_send_action("jump") - observe if player jumps
  3. game_get_state() - capture screenshot
  4. If jump doesn't work:
     - game_export_video() - record for analysis
     - Edit player.gd to fix jump logic
     - Test again
```

---

## 8. SOTA LLM/API Options

### 8.1 Vision-Language Model APIs (Recommended)

| API | Best For | Pricing | Recommendation |
|-----|----------|---------|-----------------|
| **Claude 3.5 Sonnet** (Anthropic) | Complex reasoning, planning | $3/M input tokens | ⭐⭐⭐ Best reasoning |
| **GPT-4o** (OpenAI) | Multimodal understanding | $5/M input tokens | ⭐⭐⭐ Balanced |
| **Gemini 2.5 Pro** (Google) | Fast inference, real-time | $20/M input tokens | ⭐⭐⭐ Fastest |
| **Claude Opus (3)** (Anthropic) | Maximum capability | $15/M input tokens | ⭐⭐⭐⭐ Overkill but best |

**Recommendation for Your Project**:
1. **Primary**: Claude 3.5 Sonnet (best reasoning for game understanding)
2. **Secondary**: Gemini 2.5 Pro (fastest for real-time gameplay)
3. **Local fallback**: Ollama + Mistral (free, privacy-respecting)

### 8.2 Local VLM Options (Self-Hosted)

**Ollama + Llama 2-Vision**:
```bash
# Install
brew install ollama
ollama pull llava

# Run
ollama run llava "Analyze this game screenshot..."

# Python integration
from ollama import OllamaAPI
response = OllamaAPI.generate("llava", prompt=...)
```

**LLaVA (Open-source Vision-Language)**:
```bash
git clone https://github.com/haotian-liu/LLaVA
cd LLaVA
pip install -e .

# Usage
python llava/mm_utils.py --image game.png --prompt "What's happening?"
```

**Qwen-VL (Alibaba Open-Source)**:
```bash
pip install transformers torch
from transformers import AutoProcessor, AutoModelForVision2Seq

processor = AutoProcessor.from_pretrained("Qwen/Qwen-VL-Chat")
model = AutoModelForVision2Seq.from_pretrained("Qwen/Qwen-VL-Chat")

# Inference...
```

### 8.3 Hybrid Strategy

```rust
// In your Tauri backend, support multiple VLM backends

enum VLMProvider {
    ClaudeSonnet,
    GPT4o,
    Gemini25Pro,
    LocalOllama,
    LocalLLaVA,
}

async fn analyze_game_state(
    screenshot: String,
    provider: VLMProvider,
    objective: String,
) -> Result<AgentDecision, String> {
    match provider {
        VLMProvider::ClaudeSonnet => {
            // Use Anthropic API
            call_claude_api(screenshot, objective).await
        }
        VLMProvider::LocalOllama => {
            // Call local Ollama instance (fast, free)
            call_ollama_api(screenshot, objective).await
        }
        _ => { /* ... */ }
    }
}
```

**Benefits**:
- **Fast iteration**: Use local LLaVA during development (instant)
- **Production**: Switch to Claude for better reasoning
- **Fallback**: If API down, use local model
- **Cost optimization**: Use cheaper models for simple tasks

---

## 9. Real-World Examples & Benchmarks

### 9.1 VideoGameBench Results (Reference)

**Benchmark**: Play 10 classic 1990s games completely

| Model | Real-Time | Lite (Paused) | Notes |
|-------|-----------|---------------|-------|
| Gemini 2.5 Pro | 0.48% | 1.6% | Best overall, still poor |
| Claude 3.5 Sonnet | 0.29% | 1.2% | Slower inference |
| GPT-4o | 0.09% | 0.8% | Conservative decisions |
| GPT-4 Turbo | 0% | 0% | Older model, struggles |

**Key Insight**: Even frontier models barely play simple games. However, with structured environments (like your Godot games with clear objectives), performance should be much better.

### 9.2 Expected Performance for Your Project

**Simple Godot Game** (collect items, avoid obstacles):
- Expected success rate: 30-70% with Claude + structured actions
- With specialized modules (GameSense): 70-90%

**Medium Complexity** (platformer, some puzzle):
- Expected: 20-50%
- Requires better memory + planning

**Complex** (strategy, resource management):
- Expected: 5-20%
- Need RAG or specialized training

---

## 10. Recommended Implementation Roadmap

### Timeline: 6-8 Weeks

**Week 1-2: Foundation**
- [ ] Implement Tauri screenshot capture
- [ ] Create GDScript AIController addon for Godot
- [ ] Basic input injection (keyboard/mouse)
- [ ] Test with simple game (move + collect)

**Week 3-4: VLM Integration**
- [ ] Integrate Claude API (analyze screenshots)
- [ ] Implement decision parsing (action extraction)
- [ ] Memory/scratchpad system
- [ ] Test on 2-3 simple games

**Week 5-6: Visual Debugging**
- [ ] Gameplay recording (PNG sequences)
- [ ] Frame timeline UI
- [ ] Log capture & display
- [ ] Video export (MP4)

**Week 7-8: Polish & Optimization**
- [ ] Local VLM fallback (Ollama)
- [ ] Error recovery (handle stuck states)
- [ ] Performance optimization
- [ ] Documentation + examples

---

## 11. Code Examples & Snippets

### 11.1 Complete Game Controller Tauri Command

**File: src-tauri/src/commands/game.rs**

```rust
use std::process::{Command, Stdio};
use std::path::Path;
use std::fs;
use base64::encode;

#[derive(serde::Serialize)]
pub struct GameFrame {
    pub screenshot: String, // base64 PNG
    pub logs: Vec<String>,
    pub frame_count: u32,
}

pub struct GameSession {
    process: std::process::Child,
    project_path: String,
    scene_path: String,
    logs_path: String,
}

#[tauri::command]
pub async fn start_game_session(
    project_path: String,
    scene_path: String,
) -> Result<String, String> {
    // Validate paths
    if !Path::new(&project_path).exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    let logs_dir = format!("{}user://logs/", project_path);
    fs::create_dir_all(&logs_dir).ok();

    // Launch Godot headless
    let child = Command::new("godot")
        .args(&[
            "--headless",
            "--verbose",
            &scene_path,
        ])
        .env("AGENT_ENABLED", "true")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Godot: {}", e))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Store session info (implement with proper state management)
    // STATE.sessions.insert(session_id.clone(), GameSession { ... });

    Ok(session_id)
}

#[tauri::command]
pub async fn get_game_frame(session_id: String) -> Result<GameFrame, String> {
    // Get latest screenshot from user://screenshots/
    let screenshot_path = format!("user://screenshots/latest.png");
    
    let image_data = fs::read(&screenshot_path)
        .map_err(|e| format!("Failed to read screenshot: {}", e))?;
    
    let screenshot_b64 = encode(&image_data);

    // Read logs
    let logs_path = format!("user://logs/game.log");
    let logs_content = fs::read_to_string(&logs_path).unwrap_or_default();
    let logs: Vec<String> = logs_content.lines().map(|s| s.to_string()).collect();

    Ok(GameFrame {
        screenshot: screenshot_b64,
        logs,
        frame_count: 0, // TODO: track frame count
    })
}

#[tauri::command]
pub async fn send_game_action(
    session_id: String,
    action: String,
) -> Result<GameFrame, String> {
    // Write action to named pipe / file that Godot reads
    let action_file = format!("user://agent_input.txt");
    fs::write(&action_file, &action)
        .map_err(|e| format!("Failed to write action: {}", e))?;

    // Wait for game to process (implement proper sync)
    std::thread::sleep(std::time::Duration::from_millis(100));

    // Return new frame
    get_game_frame(session_id).await
}

#[tauri::command]
pub async fn stop_game_session(session_id: String) -> Result<(), String> {
    // Kill Godot process
    // Clean up temp files
    Ok(())
}
```

### 11.2 Claude Integration

**File: src-tauri/src/commands/vl.rs**

```rust
use anthropic::client::Client;
use anthropic::messages::MessageCreateParams;

#[tauri::command]
pub async fn analyze_game_screenshot(
    screenshot_base64: String,
    objective: String,
    memory: String,
) -> Result<GameAction, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "Missing ANTHROPIC_API_KEY".to_string())?;

    let client = Client::new(Some(api_key.clone()));

    let prompt = format!(
        r#"You are an AI agent controlling a video game character.

Objective: {}

Previous observations and memory:
{}

Looking at the current screenshot, respond with:
1. What you observe on screen
2. What action to take next
3. Updated memory for future steps

Format your response as:
OBSERVATION: [what you see]
ACTION: [specific action to take]
MEMORY: [important state to track]"#,
        objective, memory
    );

    let response = client
        .messages()
        .create(MessageCreateParams {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 256,
            messages: vec![
                anthropic::messages::MessageParam::User(
                    anthropic::messages::UserMessageParam {
                        content: anthropic::messages::UserMessageContent::MultiBlock(
                            vec![
                                anthropic::messages::ContentBlockParam::Text(
                                    anthropic::messages::TextBlockParam {
                                        text: prompt,
                                    },
                                ),
                                anthropic::messages::ContentBlockParam::Image(
                                    anthropic::messages::ImageBlockParam {
                                        source: anthropic::messages::ImageBlockParamSource::Base64 {
                                            media_type: anthropic::messages::ImageMediaType::ImagePng,
                                            data: screenshot_base64,
                                        },
                                    },
                                ),
                            ],
                        ),
                    },
                ),
            ],
            ..Default::default()
        })
        .await
        .map_err(|e| format!("Claude API error: {}", e))?;

    // Parse response
    if let Some(content) = response.content.first() {
        if let anthropic::messages::ContentBlock::Text(text_block) = content {
            return parse_game_action(&text_block.text);
        }
    }

    Err("No response from Claude".to_string())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GameAction {
    pub observation: String,
    pub action: String,
    pub memory: String,
}

fn parse_game_action(response: &str) -> Result<GameAction, String> {
    let mut observation = String::new();
    let mut action = String::new();
    let mut memory = String::new();

    for line in response.lines() {
        if line.starts_with("OBSERVATION:") {
            observation = line.strip_prefix("OBSERVATION:").unwrap_or("").trim().to_string();
        } else if line.starts_with("ACTION:") {
            action = line.strip_prefix("ACTION:").unwrap_or("").trim().to_string();
        } else if line.starts_with("MEMORY:") {
            memory = line.strip_prefix("MEMORY:").unwrap_or("").trim().to_string();
        }
    }

    if action.is_empty() {
        return Err("No action found in response".to_string());
    }

    Ok(GameAction {
        observation,
        action,
        memory,
    })
}
```

---

## 12. Challenges & Mitigations

### Challenge 1: Inference Latency

**Problem**: Claude takes 1-2+ seconds per decision, game moves in real-time

**Solutions**:
1. **Pause game during thinking** (VideoGameBench Lite approach)
   - Simplest: pause game, wait for Claude, resume
   - Trade-off: Loses real-time challenge but vastly improves success

2. **Specialized fast models**
   - Use Gemini 2.5 Pro for real-time (faster but less accurate)
   - Use Claude for complex decisions (slower but better reasoning)

3. **Batch actions**
   - Decide 3-5 actions in sequence from single screenshot
   - Execute without reanalysis

### Challenge 2: Poor Game Understanding

**Problem**: VLMs struggle with spatial reasoning, tracking objectives

**Solutions**:
1. **Structured game state**
   - Expose game variables to agent (via MCP)
   - Example: `{player_pos: (100,200), health: 75, goal: collect_3_coins}`
   - Agent uses both vision + structured data

2. **Game-specific prompting**
   - Include controls description in every prompt
   - Show example correct actions
   - Use few-shot prompting with recorded gameplay

3. **Memory + RAG**
   - Store successful strategies
   - Retrieve on similar game states
   - Learn from past gameplay

### Challenge 3: Error Recovery

**Problem**: Agent gets stuck, repeats wrong actions

**Solutions**:
1. **Timeout detection**
   - If same action repeated 3x with no progress, reset
   - Use progress metrics (checkpoint-based like VideoGameBench)

2. **Branching/backtracking**
   - Save game state at decision points
   - If stuck, load and try different action
   - Build decision tree of explored states

3. **Human intervention**
   - UI button: "Agent stuck, help it!"
   - Human suggests action, agent learns pattern

---

## 13. Resources & Further Reading

### Papers & Benchmarks
- **VideoGameBench** (2025): https://arxiv.org/abs/2505.18134 - Comprehensive VLM game eval
- **PORTAL** (2025): Game playing via LLM policy generation
- **GameSense** (2025): VLM developing specialized execution modules
- **AVA** (2025): Vision-Language agents for StarCraft II
- **CombatVLA** (2025): Vision-Language-Action for ARPGs
- **BALROG** (2024): Benchmarking agentic LLMs on games

### Open-Source Projects
- **Browser-Use**: https://github.com/browser-use/browser-use (UI automation with VLM)
- **godot_rl_agents**: https://github.com/edbeeching/godot_rl_agents (Godot RL integration)
- **Ollama**: https://ollama.ai (Local LLM inference)
- **LLaVA**: https://github.com/haotian-liu/LLaVA (Open-source VLM)
- **Anthropic SDK**: https://github.com/anthropics/anthropic-sdk-python

### Documentation
- Godot Headless: https://docs.godotengine.org/stable/
- Claude API: https://docs.anthropic.com/
- Playwright: https://playwright.dev/
- Tauri: https://v2.tauri.app/

---

## 14. Conclusion

**Key Takeaway**: Vision-Language Models are the modern approach for game-playing AI. They don't require game APIs, generalize across genres, and integrate naturally with agentic loops.

**For Your Godot IDE**:
1. Integrate Claude API for game understanding
2. Implement basic screenshot capture + input injection
3. Build visual debugging dashboard for agent gameplay
4. Add support for gameplay recording and analysis
5. Use GameSense-style modules for complex tasks

**Expected Outcomes**:
- Agents can debug simple Godot games visually
- Capture issues as video for analysis
- Automate testing through gameplay
- Iterate faster with visual feedback loop

This approach positions your IDE as a cutting-edge development environment where AI agents not only write code but also test it in real-time with visual understanding.

---

**Document Version**: 1.0 | Last Updated: January 24, 2026  
**Status**: Ready for implementation