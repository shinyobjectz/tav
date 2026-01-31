//! Control mapping system for NitroGen gamepad-to-game input translation
//! 
//! Maps NitroGen's gamepad outputs (joysticks + 17 buttons) to game-specific
//! input actions defined in the project's control_mappings.json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// NitroGen gamepad output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamepadState {
    /// Left joystick (x, y) normalized -1.0 to 1.0
    pub j_left: (f32, f32),
    /// Right joystick (x, y) normalized -1.0 to 1.0  
    pub j_right: (f32, f32),
    /// 17 button states matching Xbox layout
    pub buttons: GamepadButtons,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GamepadButtons {
    pub west: bool,      // X
    pub south: bool,     // A
    pub east: bool,      // B
    pub north: bool,     // Y
    pub back: bool,
    pub start: bool,
    pub guide: bool,
    pub left_shoulder: bool,
    pub right_shoulder: bool,
    pub left_thumb: bool,
    pub right_thumb: bool,
    pub dpad_up: bool,
    pub dpad_down: bool,
    pub dpad_left: bool,
    pub dpad_right: bool,
    pub left_trigger: f32,  // 0.0 to 1.0
    pub right_trigger: f32, // 0.0 to 1.0
}

/// Control mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMappings {
    /// Map left joystick directions to game actions
    pub joystick_left: JoystickMapping,
    /// Map right joystick to camera/look
    pub joystick_right: JoystickMapping,
    /// Map buttons to game actions
    pub buttons: HashMap<String, String>,
    /// Joystick deadzone (0.0 to 1.0)
    #[serde(default = "default_deadzone")]
    pub deadzone: f32,
    /// Look sensitivity multiplier
    #[serde(default = "default_sensitivity")]
    pub sensitivity: f32,
}

fn default_deadzone() -> f32 { 0.2 }
fn default_sensitivity() -> f32 { 1.0 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoystickMapping {
    pub up: Option<String>,
    pub down: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    /// For analog output (camera look)
    pub x: Option<String>,
    pub y: Option<String>,
}

impl Default for JoystickMapping {
    fn default() -> Self {
        Self {
            up: None, down: None, left: None, right: None,
            x: None, y: None,
        }
    }
}

/// Game action to send to Godot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameAction {
    pub function: String,
    pub args: Vec<serde_json::Value>,
}

impl Default for ControlMappings {
    fn default() -> Self {
        let mut buttons = HashMap::new();
        buttons.insert("SOUTH".to_string(), "jump".to_string());
        buttons.insert("WEST".to_string(), "attack".to_string());
        buttons.insert("EAST".to_string(), "interact".to_string());
        buttons.insert("RIGHT_SHOULDER".to_string(), "sprint".to_string());
        
        Self {
            joystick_left: JoystickMapping {
                up: Some("move_up".to_string()),
                down: Some("move_down".to_string()),
                left: Some("move_left".to_string()),
                right: Some("move_right".to_string()),
                x: None, y: None,
            },
            joystick_right: JoystickMapping {
                up: None, down: None, left: None, right: None,
                x: Some("look_x".to_string()),
                y: Some("look_y".to_string()),
            },
            buttons,
            deadzone: 0.2,
            sensitivity: 1.0,
        }
    }
}

/// Control mapper that translates gamepad state to game actions
pub struct ControlMapper {
    pub mappings: ControlMappings,
    prev_state: Option<GamepadState>,
}

impl ControlMapper {
    pub fn new(mappings: ControlMappings) -> Self {
        Self { mappings, prev_state: None }
    }

    /// Load mappings from project directory
    pub fn load_from_project(project_path: &Path) -> Self {
        let mappings_path = project_path.join(".tav/control_mappings.json");
        let mappings = if mappings_path.exists() {
            fs::read_to_string(&mappings_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            ControlMappings::default()
        };
        Self::new(mappings)
    }

    /// Save current mappings to project directory
    pub fn save_to_project(&self, project_path: &Path) -> Result<(), String> {
        let kobold_dir = project_path.join(".tav");
        fs::create_dir_all(&kobold_dir).map_err(|e| e.to_string())?;
        let mappings_path = kobold_dir.join("control_mappings.json");
        let json = serde_json::to_string_pretty(&self.mappings).map_err(|e| e.to_string())?;
        fs::write(&mappings_path, json).map_err(|e| e.to_string())
    }

    /// Generate default mappings from template controls
    pub fn from_template_controls(controls: &serde_json::Value) -> ControlMappings {
        let mut mappings = ControlMappings::default();
        
        if let Some(ctrl) = controls.as_object() {
            // Map common control patterns from template
            for (name, _def) in ctrl {
                let name_lower = name.to_lowercase();
                if name_lower.contains("jump") {
                    mappings.buttons.insert("SOUTH".to_string(), name.clone());
                } else if name_lower.contains("attack") || name_lower.contains("fire") {
                    mappings.buttons.insert("WEST".to_string(), name.clone());
                } else if name_lower.contains("interact") || name_lower.contains("use") {
                    mappings.buttons.insert("EAST".to_string(), name.clone());
                } else if name_lower.contains("sprint") || name_lower.contains("run") {
                    mappings.buttons.insert("RIGHT_SHOULDER".to_string(), name.clone());
                } else if name_lower.contains("crouch") || name_lower.contains("duck") {
                    mappings.buttons.insert("LEFT_SHOULDER".to_string(), name.clone());
                }
            }
        }
        
        mappings
    }

    /// Convert gamepad state to game actions
    pub fn map_to_actions(&mut self, state: &GamepadState) -> Vec<GameAction> {
        let mut actions = Vec::new();
        let dz = self.mappings.deadzone;

        // Left joystick -> movement
        let (lx, ly) = state.j_left;
        if lx.abs() > dz || ly.abs() > dz {
            // Determine primary direction
            if ly < -dz {
                if let Some(_) = &self.mappings.joystick_left.up {
                    actions.push(GameAction {
                        function: "move".to_string(),
                        args: vec![serde_json::json!("up")],
                    });
                }
            } else if ly > dz {
                if let Some(_) = &self.mappings.joystick_left.down {
                    actions.push(GameAction {
                        function: "move".to_string(),
                        args: vec![serde_json::json!("down")],
                    });
                }
            }
            
            if lx < -dz {
                if let Some(_) = &self.mappings.joystick_left.left {
                    actions.push(GameAction {
                        function: "move".to_string(),
                        args: vec![serde_json::json!("left")],
                    });
                }
            } else if lx > dz {
                if let Some(_) = &self.mappings.joystick_left.right {
                    actions.push(GameAction {
                        function: "move".to_string(),
                        args: vec![serde_json::json!("right")],
                    });
                }
            }
        } else {
            // No joystick input - stop movement
            actions.push(GameAction {
                function: "move".to_string(),
                args: vec![serde_json::json!("stop")],
            });
        }

        // Right joystick -> camera look (analog)
        let (rx, ry) = state.j_right;
        if rx.abs() > dz || ry.abs() > dz {
            let sens = self.mappings.sensitivity;
            actions.push(GameAction {
                function: "look".to_string(),
                args: vec![
                    serde_json::json!(rx * 30.0 * sens),  // degrees
                    serde_json::json!(ry * 30.0 * sens),
                ],
            });
        }

        // Buttons
        if state.buttons.south {
            actions.push(GameAction {
                function: "jump".to_string(),
                args: vec![],
            });
        }

        if state.buttons.west {
            actions.push(GameAction {
                function: "attack".to_string(),
                args: vec![],
            });
        }

        if state.buttons.east {
            actions.push(GameAction {
                function: "interact".to_string(),
                args: vec![],
            });
        }

        // Sprint via shoulder button
        if state.buttons.right_shoulder {
            actions.push(GameAction {
                function: "sprint".to_string(),
                args: vec![serde_json::json!(true)],
            });
        }

        self.prev_state = Some(state.clone());
        actions
    }

    /// Convert raw NitroGen output to GamepadState
    pub fn parse_nitrogen_output(
        j_left: &[f32],
        j_right: &[f32],
        buttons: &[f32],
    ) -> GamepadState {
        let button_threshold = 0.5;
        
        GamepadState {
            j_left: (
                j_left.get(0).copied().unwrap_or(0.0),
                j_left.get(1).copied().unwrap_or(0.0),
            ),
            j_right: (
                j_right.get(0).copied().unwrap_or(0.0),
                j_right.get(1).copied().unwrap_or(0.0),
            ),
            buttons: GamepadButtons {
                west: buttons.get(0).map(|&v| v > button_threshold).unwrap_or(false),
                south: buttons.get(1).map(|&v| v > button_threshold).unwrap_or(false),
                back: buttons.get(2).map(|&v| v > button_threshold).unwrap_or(false),
                dpad_down: buttons.get(3).map(|&v| v > button_threshold).unwrap_or(false),
                dpad_left: buttons.get(4).map(|&v| v > button_threshold).unwrap_or(false),
                dpad_right: buttons.get(5).map(|&v| v > button_threshold).unwrap_or(false),
                dpad_up: buttons.get(6).map(|&v| v > button_threshold).unwrap_or(false),
                guide: buttons.get(7).map(|&v| v > button_threshold).unwrap_or(false),
                left_shoulder: buttons.get(10).map(|&v| v > button_threshold).unwrap_or(false),
                left_trigger: buttons.get(11).copied().unwrap_or(0.0),
                right_shoulder: buttons.get(16).map(|&v| v > button_threshold).unwrap_or(false),
                right_trigger: buttons.get(17).copied().unwrap_or(0.0),
                start: buttons.get(18).map(|&v| v > button_threshold).unwrap_or(false),
                east: buttons.get(19).map(|&v| v > button_threshold).unwrap_or(false),
                north: buttons.get(20).map(|&v| v > button_threshold).unwrap_or(false),
                left_thumb: buttons.get(14).map(|&v| v > button_threshold).unwrap_or(false),
                right_thumb: buttons.get(15).map(|&v| v > button_threshold).unwrap_or(false),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_mappings() {
        let mappings = ControlMappings::default();
        assert!(mappings.buttons.contains_key("SOUTH"));
        assert_eq!(mappings.deadzone, 0.2);
    }

    #[test]
    fn test_joystick_to_movement() {
        let mappings = ControlMappings::default();
        let mut mapper = ControlMapper::new(mappings);
        
        let state = GamepadState {
            j_left: (0.0, -0.8), // Up
            j_right: (0.0, 0.0),
            buttons: GamepadButtons::default(),
        };
        
        let actions = mapper.map_to_actions(&state);
        assert!(actions.iter().any(|a| a.function == "move"));
    }
}
