# AI Game-Playing Implementation: Code Examples & Architecture

**Companion to**: AI-Game-Playing-Research.md  
**Purpose**: Ready-to-use code snippets and architectural patterns for your Godot Agentic IDE

---

## Part 1: Godot Side (GDScript)

### 1.1 AIController Node (Simple Version)

**File: addons/agent_controller/ai_controller.gd**

```gdscript
extends Node
class_name AIController

# Configuration
var actions_enabled: bool = false
var action_queue: PackedStringArray = []
var game_state_path: String = "user://game_state.json"
var logs_path: String = "user://game.log"

# Game state tracking
var frame_count: int = 0
var game_events: PackedStringArray = []

func _ready():
	# Create output directories
	DirAccess.make_absolute("user://screenshots/")
	DirAccess.make_absolute("user://logs/")
	
	# Enable agent input if environment variable set
	if OS.get_environment("AGENT_ENABLED") == "true":
		actions_enabled = true
		log_event("Agent controller initialized")

func _process(delta: float):
	frame_count += 1
	
	# Process queued actions
	if action_queue.size() > 0:
		var action = action_queue.pop_front()
		execute_action(action)
	
	# Periodic screenshot capture (every 30 frames = 1 second at 30fps)
	if frame_count % 30 == 0:
		capture_screenshot()
		save_game_state()

func execute_action(action: String) -> void:
	"""Execute a game action (move, jump, attack, etc)"""
	log_event("Executing action: " + action)
	
	# Find the player node
	var player = get_tree().root.find_child("Player", true, false)
	if not player:
		log_event("ERROR: Player node not found")
		return
	
	# Route action to appropriate handler
	match action:
		"move_left":
			player.velocity.x = -player.speed
		"move_right":
			player.velocity.x = player.speed
		"move_up":
			if "velocity" in player:
				player.velocity.y = -player.speed
		"move_down":
			if "velocity" in player:
				player.velocity.y = player.speed
		"jump":
			if player.has_method("jump"):
				player.jump()
		"attack":
			if player.has_method("attack"):
				player.attack()
		"interact":
			if player.has_method("interact"):
				player.interact()
		"pause":
			get_tree().paused = true
		"resume":
			get_tree().paused = false
		_:
			log_event("WARNING: Unknown action: " + action)

func queue_action(action: String) -> void:
	"""Add action to queue"""
	action_queue.append(action)

func capture_screenshot() -> void:
	"""Capture current frame as PNG"""
	var viewport = get_viewport()
	var image = viewport.get_texture().get_image()
	
	var filename = "user://screenshots/frame_%06d.png" % frame_count
	image.save_png(filename)
	
	log_event("Screenshot: " + filename)

func save_game_state() -> void:
	"""Save game state as JSON for agent analysis"""
	var player = get_tree().root.find_child("Player", true, false)
	
	var state = {
		"frame": frame_count,
		"scene": get_tree().current_scene.name,
		"player_position": {} if not player else {
			"x": player.global_position.x,
			"y": player.global_position.y
		},
		"game_events": game_events
	}
	
	# Add any custom game state (override in subclasses)
	if has_method("get_custom_state"):
		state.merge(get_custom_state())
	
	var json = JSON.stringify(state)
	var file = FileAccess.open(game_state_path, FileAccess.WRITE)
	if file:
		file.store_string(json)
		log_event("State saved: " + game_state_path)

func log_event(message: String) -> void:
	"""Log events for debugging"""
	var timestamp = Time.get_ticks_msec()
	var log_line = "[%d] [F:%d] %s" % [timestamp, frame_count, message]
	
	game_events.append(log_line)
	
	# Append to log file
	var file = FileAccess.open(logs_path, FileAccess.WRITE_READ)
	if file:
		file.seek_end()
		file.store_line(log_line)
	
	print(log_line)
```

### 1.2 Custom Game Integration

**File: scenes/level_01/level_01.gd**

```gdscript
extends Node2D

@onready var player = $Player
@onready var ai_controller = $AIController

# Game-specific state
var coins_collected: int = 0
var total_coins: int = 5
var level_complete: bool = false

func _ready():
	# Connect signals
	player.coin_collected.connect(_on_coin_collected)
	
	# Load custom state function for AIController
	if ai_controller:
		ai_controller.get_custom_state = func():
			return {
				"coins_collected": coins_collected,
				"total_coins": total_coins,
				"level_complete": level_complete
			}

func _on_coin_collected():
	coins_collected += 1
	ai_controller.log_event("Coin collected! %d/%d" % [coins_collected, total_coins])
	
	if coins_collected >= total_coins:
		level_complete = true
		ai_controller.log_event("LEVEL COMPLETE!")
		ai_controller.queue_action("pause")

# Agent can call this for level reset/testing
func reset_level():
	coins_collected = 0
	level_complete = false
	player.position = Vector2(100, 300)
```

### 1.3 Advanced: Vision-Aware State

**File: addons/agent_controller/vision_aware_controller.gd** (for visual debugging)

```gdscript
extends AIController

# Vision analysis state (filled by external VLM)
var vision_analysis: Dictionary = {}
var object_detection_results: Array = []

func update_vision_analysis(analysis: Dictionary) -> void:
	"""Update vision analysis from Claude API"""
	vision_analysis = analysis
	log_event("Vision analysis: " + JSON.stringify(analysis))

func get_object_positions() -> Array:
	"""Detect important game objects (enemies, items, goals)"""
	# This would be called by the agent to understand what it sees
	var objects = []
	
	# Find all important nodes
	for node in get_tree().get_nodes_in_group("game_objects"):
		objects.append({
			"name": node.name,
			"type": node.get_class(),
			"position": {
				"x": node.global_position.x,
				"y": node.global_position.y
			}
		})
	
	return objects

func create_annotated_screenshot() -> void:
	"""Create screenshot with visual annotations for debugging"""
	# Draw boxes around detected objects
	var viewport = get_viewport()
	var image = viewport.get_texture().get_image()
	
	# Save both raw and annotated versions
	image.save_png("user://screenshots/frame_%06d_raw.png" % frame_count)
	
	# In production, use Canvas/Viewport to overlay annotations
	# For now, we'll pass raw to Claude and let it analyze
	
	log_event("Created annotated screenshot")
```

---

## Part 2: Tauri Rust Backend

### 2.1 Game Controller Module

**File: src-tauri/src/commands/game_controller.rs**

```rust
use std::process::{Command, Child, Stdio};
use std::path::Path;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct GameFrame {
    pub screenshot: String, // base64 PNG
    pub game_state: serde_json::Value, // JSON game state
    pub logs: Vec<String>,
    pub frame_count: u32,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize)]
pub struct GameState {
    pub session_id: String,
    pub running: bool,
    pub frame_count: u32,
}

pub struct GameSession {
    pub id: String,
    pub process: Option<Child>,
    pub project_path: String,
    pub scene_path: String,
    pub logs_path: String,
    pub screenshots_dir: String,
    pub frame_count: u32,
}

// Global session storage (use proper state management in production)
lazy_static::lazy_static! {
    static ref GAME_SESSIONS: std::sync::Mutex<std::collections::HashMap<String, GameSession>> 
        = std::sync::Mutex::new(std::collections::HashMap::new());
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

    let scene_full_path = format!("{}{}", project_path, scene_path);
    if !Path::new(&scene_full_path).exists() {
        return Err(format!("Scene path does not exist: {}", scene_full_path));
    }

    // Create output directories
    let logs_dir = format!("{}user://logs/", project_path);
    let screenshots_dir = format!("{}user://screenshots/", project_path);
    
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;

    // Launch Godot headless
    let mut child = Command::new("godot")
        .args(&[
            "--headless",
            "--verbose",
            &scene_full_path,
        ])
        .env("AGENT_ENABLED", "true")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Godot: {}", e))?;

    let session_id = uuid::Uuid::new_v4().to_string();
    
    let session = GameSession {
        id: session_id.clone(),
        process: Some(child),
        project_path: project_path.clone(),
        scene_path,
        logs_path: logs_dir,
        screenshots_dir,
        frame_count: 0,
    };

    // Store session
    GAME_SESSIONS
        .lock()
        .unwrap()
        .insert(session_id.clone(), session);

    // Give Godot time to start
    sleep(Duration::from_secs(2)).await;

    Ok(session_id)
}

#[tauri::command]
pub async fn get_game_frame(session_id: String) -> Result<GameFrame, String> {
    let sessions = GAME_SESSIONS.lock().unwrap();
    let session = sessions.get(&session_id)
        .ok_or("Session not found")?;

    // Find latest screenshot
    let screenshots_dir = &session.screenshots_dir;
    let mut latest_screenshot = String::new();
    let mut latest_count = 0;

    if let Ok(entries) = fs::read_dir(screenshots_dir) {
        for entry in entries.flatten() {
            if let Ok(filename) = entry.file_name().into_string() {
                if filename.ends_with(".png") && filename.contains("frame_") {
                    if let Ok(num_str) = filename
                        .strip_prefix("frame_")
                        .unwrap_or("")
                        .strip_suffix(".png")
                        .unwrap_or("")
                        .parse::<u32>() {
                        if num_str > latest_count {
                            latest_count = num_str;
                            latest_screenshot = entry.path().to_string_lossy().to_string();
                        }
                    }
                }
            }
        }
    }

    // Read screenshot
    let screenshot_b64 = if !latest_screenshot.is_empty() {
        let image_data = fs::read(&latest_screenshot)
            .map_err(|e| format!("Failed to read screenshot: {}", e))?;
        base64::encode(&image_data)
    } else {
        String::new()
    };

    // Read game state JSON
    let state_path = format!("{}user://game_state.json", session.project_path);
    let game_state = if Path::new(&state_path).exists() {
        let content = fs::read_to_string(&state_path)
            .unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&content).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    // Read logs
    let logs_content = fs::read_to_string(&session.logs_path)
        .unwrap_or_default();
    let logs: Vec<String> = logs_content
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(GameFrame {
        screenshot: screenshot_b64,
        game_state,
        logs: logs.iter().rev().take(20).map(|s| s.to_string()).collect(),
        frame_count: latest_count,
        timestamp: chrono::Local::now().timestamp(),
    })
}

#[tauri::command]
pub async fn send_game_action(
    session_id: String,
    action: String,
) -> Result<GameFrame, String> {
    // Write action to input file that GDScript reads
    let sessions = GAME_SESSIONS.lock().unwrap();
    let session = sessions.get(&session_id)
        .ok_or("Session not found")?;

    let action_file = format!("{}user://agent_input.json", session.project_path);
    let action_json = serde_json::json!({
        "action": action,
        "timestamp": chrono::Local::now().timestamp()
    });

    fs::write(&action_file, action_json.to_string())
        .map_err(|e| format!("Failed to write action: {}", e))?;

    // Wait for game to process
    sleep(Duration::from_millis(200)).await;

    drop(sessions); // Release lock before calling get_game_frame
    
    // Return new frame
    get_game_frame(session_id).await
}

#[tauri::command]
pub async fn stop_game_session(session_id: String) -> Result<(), String> {
    let mut sessions = GAME_SESSIONS.lock().unwrap();
    
    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(mut process) = session.process {
            process.kill().map_err(|e| e.to_string())?;
            process.wait().map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn export_gameplay_video(
    session_id: String,
    output_path: String,
    fps: Option<u32>,
) -> Result<String, String> {
    let sessions = GAME_SESSIONS.lock().unwrap();
    let session = sessions.get(&session_id)
        .ok_or("Session not found")?;

    let fps = fps.unwrap_or(30);
    let screenshots_dir = &session.screenshots_dir;

    // Use ffmpeg to create video from screenshots
    let command = format!(
        "ffmpeg -framerate {} -i {}frame_%06d.png -c:v libx264 -pix_fmt yuv420p {}",
        fps, screenshots_dir, output_path
    );

    let output = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
        .map_err(|e| format!("FFmpeg failed: {}", e))?;

    if !output.status.success() {
        return Err(format!("FFmpeg error: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(format!("Video exported to: {}", output_path))
}
```

### 2.2 VLM Integration (Claude API)

**File: src-tauri/src/commands/vl_controller.rs**

```rust
use anthropic_sdk::client::Client as AnthropicClient;
use anthropic_sdk::messages::{MessageParam, UserMessageContent, ContentBlock};
use base64::encode;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct GameAnalysis {
    pub observation: String,
    pub action: String,
    pub memory: String,
    pub confidence: f32,
}

#[tauri::command]
pub async fn analyze_game_frame(
    screenshot_base64: String,
    objective: String,
    memory: String,
    available_actions: Vec<String>,
) -> Result<GameAnalysis, String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "Missing ANTHROPIC_API_KEY environment variable".to_string())?;

    let client = AnthropicClient::new(api_key);

    let action_list = available_actions.join(", ");

    let prompt = format!(
        r#"You are an AI agent playing a video game.

OBJECTIVE: {}

AVAILABLE ACTIONS: {}

PREVIOUS OBSERVATIONS AND MEMORY:
{}

Analyze the current screenshot and decide your next action.

Respond EXACTLY in this format (no extra text):
OBSERVATION: <what you see in detail>
ACTION: <one action from the available list>
MEMORY: <important state to remember>
CONFIDENCE: <0.0 to 1.0 how confident in this action>"#,
        objective, action_list, memory
    );

    // Make API call with vision
    let response = client
        .messages()
        .create(anthropic_sdk::messages::MessageCreateParams {
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 300,
            messages: vec![
                MessageParam::User {
                    content: UserMessageContent::MultiBlock(vec![
                        anthropic_sdk::messages::ContentBlockParam::Text {
                            text: prompt,
                        },
                        anthropic_sdk::messages::ContentBlockParam::Image {
                            source: anthropic_sdk::messages::ImageSource::Base64 {
                                media_type: "image/png".to_string(),
                                data: screenshot_base64,
                            },
                        },
                    ]),
                    cache_control: None,
                },
            ],
            ..Default::default()
        })
        .await
        .map_err(|e| format!("Claude API error: {}", e))?;

    // Parse response
    let response_text = response
        .content
        .iter()
        .find_map(|block| {
            if let ContentBlock::Text { text } = block {
                Some(text.as_str())
            } else {
                None
            }
        })
        .ok_or("No text response from Claude")?;

    parse_game_analysis(response_text)
}

fn parse_game_analysis(response: &str) -> Result<GameAnalysis, String> {
    let mut observation = String::new();
    let mut action = String::new();
    let mut memory = String::new();
    let mut confidence = 0.5;

    for line in response.lines() {
        if let Some(content) = line.strip_prefix("OBSERVATION:") {
            observation = content.trim().to_string();
        } else if let Some(content) = line.strip_prefix("ACTION:") {
            action = content.trim().to_string();
        } else if let Some(content) = line.strip_prefix("MEMORY:") {
            memory = content.trim().to_string();
        } else if let Some(content) = line.strip_prefix("CONFIDENCE:") {
            confidence = content.trim().parse().unwrap_or(0.5);
        }
    }

    if action.is_empty() {
        return Err("No action found in Claude response".to_string());
    }

    Ok(GameAnalysis {
        observation,
        action,
        memory,
        confidence,
    })
}

#[tauri::command]
pub async fn analyze_with_local_vlm(
    screenshot_path: String,
    prompt: String,
) -> Result<String, String> {
    // Call local Ollama instance
    let client = reqwest::Client::new();
    
    let request = serde_json::json!({
        "model": "llava",
        "prompt": prompt,
        "stream": false,
        "images": [screenshot_path]
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    let body = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())?;

    Ok(body["response"]
        .as_str()
        .unwrap_or("No response")
        .to_string())
}
```

### 2.3 Tauri Command Registration

**File: src-tauri/src/lib.rs**

```rust
mod commands {
    pub mod game_controller;
    pub mod vl_controller;
}

use tauri::Manager;

pub fn init() -> impl Fn(tauri::Invoke) -> bool {
    tauri::generate_handler![
        commands::game_controller::start_game_session,
        commands::game_controller::get_game_frame,
        commands::game_controller::send_game_action,
        commands::game_controller::stop_game_session,
        commands::game_controller::export_gameplay_video,
        commands::vl_controller::analyze_game_frame,
        commands::vl_controller::analyze_with_local_vlm,
    ]
}
```

---

## Part 3: Frontend React Components

### 3.1 Game Playing Widget

**File: src/components/GamePlayground.tsx**

```typescript
import React, { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface GameFrame {
  screenshot: string;
  game_state: Record<string, any>;
  logs: string[];
  frame_count: number;
  timestamp: number;
}

interface GameAnalysis {
  observation: string;
  action: string;
  memory: string;
  confidence: number;
}

export const GamePlayground: React.FC = () => {
  const [sessionId, setSessionId] = useState<string>("");
  const [gameRunning, setGameRunning] = useState(false);
  const [currentFrame, setCurrentFrame] = useState<GameFrame | null>(null);
  const [frameHistory, setFrameHistory] = useState<GameFrame[]>([]);
  const [agentMemory, setAgentMemory] = useState("");
  const [objective, setObjective] = useState("Reach the goal");
  const [availableActions, setAvailableActions] = useState([
    "move_left",
    "move_right",
    "jump",
    "attack",
  ]);
  const [isThinking, setIsThinking] = useState(false);
  const [autoPlay, setAutoPlay] = useState(false);

  const startGame = async () => {
    try {
      const newSessionId = await invoke<string>("start_game_session", {
        projectPath: "/path/to/godot/project/",
        scenePath: "res://scenes/test_level.tscn",
      });

      setSessionId(newSessionId);
      setGameRunning(true);
      setFrameHistory([]);
      setAgentMemory("");

      // Get initial frame
      const frame = await invoke<GameFrame>("get_game_frame", {
        sessionId: newSessionId,
      });
      setCurrentFrame(frame);
      setFrameHistory([frame]);
    } catch (error) {
      console.error("Failed to start game:", error);
    }
  };

  const gameStep = async () => {
    if (!sessionId || !currentFrame) return;

    setIsThinking(true);

    try {
      // Get AI decision
      const analysis = await invoke<GameAnalysis>("analyze_game_frame", {
        screenshotBase64: currentFrame.screenshot,
        objective,
        memory: agentMemory,
        availableActions,
      });

      // Execute action
      const newFrame = await invoke<GameFrame>("send_game_action", {
        sessionId,
        action: analysis.action,
      });

      // Update state
      setCurrentFrame(newFrame);
      setFrameHistory([...frameHistory, newFrame]);
      setAgentMemory(analysis.memory);

      // Log
      console.log(`Action: ${analysis.action}`);
      console.log(`Confidence: ${analysis.confidence}`);
    } catch (error) {
      console.error("Game step failed:", error);
    } finally {
      setIsThinking(false);
    }
  };

  const exportVideo = async () => {
    if (!sessionId) return;

    try {
      await invoke("export_gameplay_video", {
        sessionId,
        outputPath: "/tmp/gameplay.mp4",
        fps: 30,
      });
      alert("Video exported to /tmp/gameplay.mp4");
    } catch (error) {
      console.error("Export failed:", error);
    }
  };

  const stopGame = async () => {
    if (!sessionId) return;

    try {
      await invoke("stop_game_session", { sessionId });
      setGameRunning(false);
      setSessionId("");
    } catch (error) {
      console.error("Failed to stop game:", error);
    }
  };

  // Auto-play loop
  useEffect(() => {
    if (autoPlay && !isThinking) {
      const timer = setTimeout(gameStep, 1000);
      return () => clearTimeout(timer);
    }
  }, [autoPlay, isThinking, sessionId, currentFrame]);

  return (
    <div className="game-playground">
      <div className="controls">
        {!gameRunning ? (
          <button onClick={startGame}>Start Game</button>
        ) : (
          <>
            <button onClick={gameStep} disabled={isThinking}>
              {isThinking ? "AI Thinking..." : "Step"}
            </button>
            <button
              onClick={() => setAutoPlay(!autoPlay)}
              style={{
                backgroundColor: autoPlay ? "#4CAF50" : "#ccc",
              }}
            >
              {autoPlay ? "Auto-Play (On)" : "Auto-Play (Off)"}
            </button>
            <button onClick={exportVideo}>Export Video</button>
            <button onClick={stopGame}>Stop Game</button>
          </>
        )}
      </div>

      <div className="game-area">
        {currentFrame && (
          <div className="viewport">
            <img
              src={`data:image/png;base64,${currentFrame.screenshot}`}
              alt="Game Screenshot"
            />
            <div className="info-overlay">
              <div>Frame: {currentFrame.frame_count}</div>
              <div>
                Status: {isThinking ? "Thinking..." : "Ready"}
              </div>
            </div>
          </div>
        )}

        <div className="sidebar">
          <div className="section">
            <h3>Objective</h3>
            <input
              type="text"
              value={objective}
              onChange={(e) => setObjective(e.target.value)}
              placeholder="Game objective"
            />
          </div>

          <div className="section">
            <h3>Agent Memory</h3>
            <textarea
              value={agentMemory}
              onChange={(e) => setAgentMemory(e.target.value)}
              rows={4}
              placeholder="Agent observations..."
            />
          </div>

          <div className="section">
            <h3>Game State</h3>
            <pre>
              {currentFrame ? JSON.stringify(currentFrame.game_state, null, 2) : "{}"}
            </pre>
          </div>
        </div>
      </div>

      <div className="logs">
        <h3>Logs (Last 10)</h3>
        <div className="log-content">
          {currentFrame?.logs.map((log, i) => (
            <div key={i} className="log-line">
              {log}
            </div>
          ))}
        </div>
      </div>

      <div className="timeline">
        <h3>Frame Timeline</h3>
        <div className="frame-strip">
          {frameHistory.map((frame, i) => (
            <div
              key={i}
              className={`frame-thumb ${i === frameHistory.length - 1 ? "active" : ""}`}
              onClick={() => {
                setCurrentFrame(frame);
              }}
              title={`Frame ${frame.frame_count}`}
            >
              {i}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};
```

### 3.2 Styling

**File: src/styles/GamePlayground.css**

```css
.game-playground {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #1a1a1a;
  color: #fff;
  font-family: monospace;
}

.controls {
  padding: 10px;
  background: #2a2a2a;
  border-bottom: 1px solid #444;
  display: flex;
  gap: 10px;
}

.controls button {
  padding: 8px 16px;
  background: #0066cc;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-weight: bold;
}

.controls button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.controls button:hover:not(:disabled) {
  background: #0052a3;
}

.game-area {
  display: flex;
  flex: 1;
  gap: 10px;
  padding: 10px;
  overflow: auto;
}

.viewport {
  flex: 2;
  position: relative;
  background: #111;
  border: 2px solid #444;
  border-radius: 4px;
  overflow: hidden;
}

.viewport img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.info-overlay {
  position: absolute;
  bottom: 10px;
  left: 10px;
  background: rgba(0, 0, 0, 0.7);
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
}

.sidebar {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 10px;
  overflow: auto;
}

.section {
  background: #2a2a2a;
  padding: 10px;
  border-radius: 4px;
  border: 1px solid #444;
}

.section h3 {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #0066cc;
}

.section input,
.section textarea {
  width: 100%;
  background: #1a1a1a;
  color: #fff;
  border: 1px solid #444;
  padding: 6px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 11px;
}

.section pre {
  margin: 0;
  font-size: 10px;
  max-height: 200px;
  overflow: auto;
}

.logs {
  background: #2a2a2a;
  border-top: 1px solid #444;
  padding: 10px;
  max-height: 150px;
  overflow: auto;
}

.logs h3 {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #0066cc;
}

.log-content {
  font-size: 10px;
  max-height: 120px;
  overflow: auto;
}

.log-line {
  padding: 2px 0;
  border-bottom: 1px solid #333;
  color: #aaa;
}

.timeline {
  background: #2a2a2a;
  border-top: 1px solid #444;
  padding: 10px;
  max-height: 80px;
  overflow: hidden;
}

.timeline h3 {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #0066cc;
}

.frame-strip {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  height: 40px;
}

.frame-thumb {
  min-width: 40px;
  height: 40px;
  background: #1a1a1a;
  border: 1px solid #444;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 10px;
  border-radius: 4px;
  transition: all 0.2s;
}

.frame-thumb:hover {
  background: #333;
  border-color: #0066cc;
}

.frame-thumb.active {
  background: #0066cc;
  border-color: #0066cc;
}
```

---

## Part 4: Agent Integration (agent.md)

**File: agent.md** (in your Godot project)

```markdown
# Godot Game Testing with AI Agent

## Overview
This project has integrated AI game-playing capabilities. Your AI agent can:
1. Start and play Godot games headlessly
2. Capture screenshots and analyze them with Claude Vision
3. Make intelligent gameplay decisions
4. Record gameplay for debugging and analysis

## Game Playing MCP Tools

Available tools for game interaction:

### game_start
Start playing a Godot scene in headless mode with AI control.

```json
{
  "scene_path": "res://scenes/level_01.tscn",
  "objective": "Collect all coins and reach the goal"
}
```

Returns: `session_id` (use for subsequent commands)

### game_send_action
Send an input action to the running game.

```json
{
  "session_id": "...",
  "action": "move_right"
}
```

Available actions:
- `move_left`, `move_right`, `move_up`, `move_down`
- `jump`, `attack`, `interact`
- `pause`, `resume`

### game_get_frame
Get the current game screenshot and state.

```json
{
  "session_id": "..."
}
```

Returns frame with screenshot (base64), game state (JSON), and logs.

### game_export_video
Export recorded gameplay to MP4.

```json
{
  "session_id": "...",
  "output_path": "/tmp/gameplay.mp4",
  "fps": 30
}
```

## Typical Game Testing Workflow

When user asks to "test the jumping mechanic":

1. **Start game:**
   ```
   game_start(scene="res://test/jump_test.tscn", objective="test jump heights")
   ```

2. **Observe initial state:**
   ```
   game_get_frame()
   → Analyze: "Player is on ground, goal above"
   ```

3. **Execute test action:**
   ```
   game_send_action("jump")
   game_get_frame()
   → Analyze: "Player jumped X pixels"
   ```

4. **Verify result:**
   - If jump height is wrong, note the values
   - Suggest code fix
   - Test again to verify

5. **Export evidence:**
   ```
   game_export_video(output_path="jump_test.mp4")
   ```

6. **Report findings:**
   ```
   "Jump test complete. Player jumps 200px (should be 150px).
    Issue: jump_force is set to 500 instead of 300.
    Suggested fix: [code snippet]"
   ```

## Example: Testing Player Movement

```
User: "Can you test the player movement? Make sure moving right works."

Your Plan:
1. Start game in test scene
2. Press 'move_right' for 3 frames
3. Analyze if player position increased
4. Compare with expected speed
5. Report findings

Your Execution:
1. game_start("res://test/movement.tscn")
2. game_send_action("move_right")
3. game_send_action("move_right")
4. game_send_action("move_right")
5. game_get_frame()
6. Analyze screenshot and game_state
7. Response: "Player moved 300 pixels right at expected speed of 100 px/frame"
```

## Debugging Failed Tests

If something goes wrong:

1. **Capture the moment:**
   ```
   game_export_video() → save to analyze later
   ```

2. **Check logs:**
   ```
   game_get_frame() → examine the logs array
   ```

3. **Inspect game state:**
   ```
   game_get_frame() → check game_state JSON for unexpected values
   ```

4. **Use vision analysis:**
   ```
   Describe what you see in the screenshot
   Compare with expected visual state
   ```

## Notes

- Game runs headless (no GUI) for speed
- Screenshots are captured every 30 frames (~1 second at 30fps)
- Large gameplay sessions are recorded for later analysis
- You can pause and resume the game
- All gameplay is logged for debugging
```

---

## Part 5: Integration with Godot MCP Server

**Add to your godot-mcp server (Node.js)**

```typescript
// Add to tools array in your MCP server

const gamePlayingTools = [
  {
    name: "play_game",
    description: "Play a Godot game and capture gameplay footage",
    inputSchema: {
      type: "object" as const,
      properties: {
        action: {
          type: "string",
          enum: ["start", "step", "capture", "stop", "export"],
          description: "Action to perform"
        },
        scene_path: {
          type: "string",
          description: "Path to Godot scene (res://...)"
        },
        game_action: {
          type: "string",
          description: "Input action (move_left, jump, etc.)"
        },
        output_path: {
          type: "string",
          description: "Where to save video"
        }
      },
      required: ["action"]
    }
  }
];

// Handler
async function handlePlayGame(params: any) {
  switch (params.action) {
    case "start":
      return await invoke("start_game_session", {
        projectPath: params.project_path,
        scenePath: params.scene_path
      });
    
    case "step":
      return await invoke("send_game_action", {
        sessionId: params.session_id,
        action: params.game_action
      });
    
    case "capture":
      return await invoke("get_game_frame", {
        sessionId: params.session_id
      });
    
    case "export":
      return await invoke("export_gameplay_video", {
        sessionId: params.session_id,
        outputPath: params.output_path,
        fps: params.fps || 30
      });
    
    case "stop":
      return await invoke("stop_game_session", {
        sessionId: params.session_id
      });
  }
}
```

---

## Quick Start Checklist

- [ ] Copy GDScript files to `addons/agent_controller/`
- [ ] Add Tauri commands to Rust backend
- [ ] Add React components to frontend
- [ ] Set `ANTHROPIC_API_KEY` environment variable
- [ ] Update `agent.md` with game-specific actions
- [ ] Test with a simple game scene
- [ ] Enable agent in Godot: Set `AGENT_ENABLED=true`
- [ ] Run first gameplay test

---

**Status**: Implementation ready | **Last Updated**: January 24, 2026
