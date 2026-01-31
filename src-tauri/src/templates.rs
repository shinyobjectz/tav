// ============================================================================
// GDScript Templates - Godot Development Rules & Code Templates
// ============================================================================

pub const GODOT_RULES: &str = r#"# Kobold - Godot Development Rules

You are an AI assistant specialized in Godot 4.x game development. Follow these guidelines:

## Architecture: Signal Bus Pattern (CRITICAL)
ALL systems communicate through EventBus signals, never direct references.
```gdscript
# GOOD: Emit signal, let listeners react
EventBus.player_damaged.emit(damage)

# BAD: Direct coupling
health_ui.update_display(health)
audio.play_damage_sound()
```

## Architecture: Entity-Component Pattern
Compose entities from reusable components. Avoid god scripts.
```gdscript
# Player (entity) has child components:
# - HealthComponent.gd
# - MovementComponent.gd
# Components emit signals, parent orchestrates
```

## Project Structure (Feature-Based)
```
assets/
  entities/player/     # Player scene + sprites + sounds together
  entities/enemies/    # Enemy scenes organized by type
  ui/                  # UI scenes and components
  worlds/              # Levels, tilemaps
src/
  core/                # EventBus, GameState, ConfigManager
  systems/             # Health, Inventory, Audio systems
  components/          # Reusable HealthComponent, etc.
scenes/                # Main entry scenes
autoload/              # Singletons (EventBus, GameState, AIController)
```

## GDScript Best Practices
- Use static typing: `var speed: float = 100.0`
- Use `@onready` for node references: `@onready var sprite: Sprite2D = $Sprite2D`
- Use `@export` for inspector variables: `@export var max_health: int = 100`
- Use EventBus for cross-system communication
- Use `class_name` to register custom classes

## Component Pattern Example
```gdscript
# src/components/health_component.gd
class_name HealthComponent extends Node

signal health_changed(current: int, maximum: int)
signal died

@export var max_health: int = 100
var current_health: int

func _ready() -> void:
    current_health = max_health

func take_damage(amount: int) -> void:
    current_health = max(0, current_health - amount)
    health_changed.emit(current_health, max_health)
    EventBus.entity_damaged.emit(get_parent(), amount)
    if current_health <= 0:
        died.emit()
        EventBus.entity_died.emit(get_parent())

func heal(amount: int) -> void:
    current_health = min(max_health, current_health + amount)
    health_changed.emit(current_health, max_health)
```

## EventBus Usage
```gdscript
# Emitting (from any system)
EventBus.player_damaged.emit(damage_amount)
EventBus.coin_collected.emit(coin_value)
EventBus.level_completed.emit()

# Listening (in UI, audio, etc.)
func _ready() -> void:
    EventBus.player_damaged.connect(_on_player_damaged)

func _on_player_damaged(amount: int) -> void:
    # React to damage (play sound, shake screen, etc.)
    pass
```

## Signal Naming
- Use past tense: `health_changed`, `player_died`, `level_completed`
- Include relevant data: `damage_taken(amount)`, `item_collected(item_id)`

## File Naming
- snake_case for files: `player_controller.gd`, `main_menu.tscn`
- Components: `health_component.gd`, `movement_component.gd`
"#;

// ============================================================================
// Core Autoloads - Signal Bus Pattern
// ============================================================================

pub const EVENT_BUS_GD: &str = r#"extends Node
## EventBus - Central signal hub for decoupled communication
## All game systems emit/listen here instead of direct references

# Player Events
signal player_spawned(player: Node)
signal player_damaged(amount: int)
signal player_healed(amount: int)
signal player_died
signal player_state_changed(state_name: String)

# Entity Events (generic for enemies, NPCs)
signal entity_damaged(entity: Node, amount: int)
signal entity_died(entity: Node)
signal entity_spawned(entity: Node)

# Game Flow Events
signal game_started
signal game_paused
signal game_resumed
signal level_completed
signal level_failed

# Collectibles & Inventory
signal coin_collected(value: int)
signal item_collected(item_id: String)
signal item_used(item_id: String)

# UI Events
signal score_changed(new_score: int)
signal health_changed(current: int, maximum: int)
signal dialog_started(dialog_id: String)
signal dialog_ended

# AI/Agent Events
signal agent_action_received(action: String, args: Array)
signal agent_state_captured(state: Dictionary)
"#;

pub const GAME_STATE_GD: &str = r#"extends Node
## GameState - Persistent cross-scene data singleton
## Survives scene transitions, tracks player progress

# Player Progress
var player_level: int = 1
var player_experience: int = 0
var score: int = 0:
	set(value):
		score = value
		EventBus.score_changed.emit(score)

# Game Flags
var discovered_areas: Array[String] = []
var completed_levels: Array[String] = []
var unlocked_abilities: Array[String] = []

# Inventory (simple dictionary - extend for complex systems)
var inventory: Dictionary = {}

# Settings (runtime, not saved)
var current_difficulty: int = 1
var is_paused: bool = false

func _ready() -> void:
	process_mode = Node.PROCESS_MODE_ALWAYS

func add_score(points: int) -> void:
	score += points

func add_item(item_id: String, quantity: int = 1) -> void:
	if inventory.has(item_id):
		inventory[item_id] += quantity
	else:
		inventory[item_id] = quantity
	EventBus.item_collected.emit(item_id)

func remove_item(item_id: String, quantity: int = 1) -> bool:
	if not inventory.has(item_id) or inventory[item_id] < quantity:
		return false
	inventory[item_id] -= quantity
	if inventory[item_id] <= 0:
		inventory.erase(item_id)
	EventBus.item_used.emit(item_id)
	return true

func has_item(item_id: String, quantity: int = 1) -> bool:
	return inventory.has(item_id) and inventory[item_id] >= quantity

func mark_area_discovered(area_id: String) -> void:
	if area_id not in discovered_areas:
		discovered_areas.append(area_id)

func mark_level_completed(level_id: String) -> void:
	if level_id not in completed_levels:
		completed_levels.append(level_id)
		EventBus.level_completed.emit()

func reset() -> void:
	player_level = 1
	player_experience = 0
	score = 0
	discovered_areas.clear()
	completed_levels.clear()
	unlocked_abilities.clear()
	inventory.clear()
"#;

// ============================================================================
// Reusable Components
// ============================================================================

pub const HEALTH_COMPONENT_GD: &str = r#"extends Node
class_name HealthComponent
## Reusable health management - attach to any entity

signal health_changed(current: int, maximum: int)
signal died

@export var max_health: int = 100
@export var invincibility_time: float = 0.0

var current_health: int
var _invincible: bool = false

func _ready() -> void:
	current_health = max_health

func take_damage(amount: int) -> void:
	if _invincible or amount <= 0:
		return
	current_health = max(0, current_health - amount)
	health_changed.emit(current_health, max_health)
	EventBus.entity_damaged.emit(get_parent(), amount)
	if invincibility_time > 0:
		_start_invincibility()
	if current_health <= 0:
		died.emit()
		EventBus.entity_died.emit(get_parent())

func heal(amount: int) -> void:
	if amount <= 0:
		return
	var old_health = current_health
	current_health = min(max_health, current_health + amount)
	if current_health != old_health:
		health_changed.emit(current_health, max_health)

func set_max_health(value: int, heal_to_max: bool = false) -> void:
	max_health = value
	if heal_to_max:
		current_health = max_health
	else:
		current_health = min(current_health, max_health)
	health_changed.emit(current_health, max_health)

func _start_invincibility() -> void:
	_invincible = true
	await get_tree().create_timer(invincibility_time).timeout
	_invincible = false

func get_health_percent() -> float:
	return float(current_health) / float(max_health) if max_health > 0 else 0.0
"#;

pub const MOVEMENT_COMPONENT_2D_GD: &str = r#"extends Node
class_name MovementComponent2D
## Reusable 2D movement - attach to CharacterBody2D parent

signal velocity_changed(velocity: Vector2)

@export var speed: float = 200.0
@export var acceleration: float = 1000.0
@export var friction: float = 800.0
@export var jump_force: float = -400.0
@export var apply_gravity: bool = true

var gravity: float = ProjectSettings.get_setting("physics/2d/default_gravity")
var _body: CharacterBody2D
var _enabled: bool = true

func _ready() -> void:
	_body = get_parent() as CharacterBody2D
	assert(_body != null, "MovementComponent2D requires CharacterBody2D parent")

func _physics_process(delta: float) -> void:
	if not _enabled or not _body:
		return
	if apply_gravity and not _body.is_on_floor():
		_body.velocity.y += gravity * delta

func move_horizontal(direction: float, delta: float) -> void:
	if not _body:
		return
	if direction != 0:
		_body.velocity.x = move_toward(_body.velocity.x, direction * speed, acceleration * delta)
	else:
		_body.velocity.x = move_toward(_body.velocity.x, 0, friction * delta)
	velocity_changed.emit(_body.velocity)

func move_direction(direction: Vector2, delta: float) -> void:
	if not _body:
		return
	direction = direction.normalized()
	if direction.length() > 0:
		_body.velocity = _body.velocity.move_toward(direction * speed, acceleration * delta)
	else:
		_body.velocity = _body.velocity.move_toward(Vector2.ZERO, friction * delta)
	velocity_changed.emit(_body.velocity)

func jump() -> bool:
	if not _body or not _body.is_on_floor():
		return false
	_body.velocity.y = jump_force
	velocity_changed.emit(_body.velocity)
	return true

func apply_movement() -> void:
	if _body:
		_body.move_and_slide()

func set_enabled(enabled: bool) -> void:
	_enabled = enabled
	if not enabled and _body:
		_body.velocity = Vector2.ZERO
"#;

// ============================================================================
// FSM System - State Machine Pattern
// ============================================================================

pub const STATE_MACHINE_GD: &str = r#"extends Node
class_name StateMachine
## Generic Finite State Machine - attach states as children
## Emits state changes through EventBus for decoupled reactions

signal state_changed(from_state: String, to_state: String)

@export var initial_state: State
@export var debug_mode: bool = false

var current_state: State
var states: Dictionary = {}

func _ready() -> void:
	# Register all child states
	for child in get_children():
		if child is State:
			states[child.name.to_lower()] = child
			child.state_machine = self
			child.process_mode = Node.PROCESS_MODE_DISABLED
	
	# Start initial state
	if initial_state:
		_transition_to(initial_state, {})
	elif states.size() > 0:
		_transition_to(states.values()[0], {})

func _process(delta: float) -> void:
	if current_state:
		current_state.update(delta)

func _physics_process(delta: float) -> void:
	if current_state:
		current_state.physics_update(delta)

func _unhandled_input(event: InputEvent) -> void:
	if current_state:
		current_state.handle_input(event)

func transition_to(state_name: String, data: Dictionary = {}) -> void:
	var target = states.get(state_name.to_lower())
	if target and target != current_state:
		_transition_to(target, data)

func _transition_to(target_state: State, data: Dictionary) -> void:
	var from_name = current_state.name if current_state else "None"
	
	if current_state:
		current_state.exit()
		current_state.process_mode = Node.PROCESS_MODE_DISABLED
	
	current_state = target_state
	current_state.process_mode = Node.PROCESS_MODE_INHERIT
	current_state.enter(data)
	
	if debug_mode:
		print("[StateMachine] %s -> %s" % [from_name, current_state.name])
	
	state_changed.emit(from_name, current_state.name)
	EventBus.player_state_changed.emit(current_state.name)

func get_current_state_name() -> String:
	return current_state.name if current_state else ""
"#;

pub const STATE_GD: &str = r#"extends Node
class_name State
## Base class for all states - override enter/exit/update methods

var state_machine: StateMachine

## Called when entering this state
func enter(_data: Dictionary = {}) -> void:
	pass

## Called when exiting this state
func exit() -> void:
	pass

## Called every frame (use for non-physics logic)
func update(_delta: float) -> void:
	pass

## Called every physics frame (use for movement)
func physics_update(_delta: float) -> void:
	pass

## Called for unhandled input events
func handle_input(_event: InputEvent) -> void:
	pass
"#;

pub const IDLE_STATE_GD: &str = r#"extends State
class_name IdleState
## Player idle state - transitions to Move on input, Air when falling

@export var player_path: NodePath = "../.."
@export var friction: float = 10.0

@onready var player: CharacterBody3D = get_node(player_path)

func enter(_data: Dictionary = {}) -> void:
	pass

func physics_update(delta: float) -> void:
	var input_dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	
	if input_dir.length() > 0.1:
		state_machine.transition_to("Move")
		return
	
	if not player.is_on_floor():
		state_machine.transition_to("Air", {"jumped": false})
		return
	
	if Input.is_action_just_pressed("jump"):
		state_machine.transition_to("Air", {"jumped": true})
		return
	
	player.velocity.x = move_toward(player.velocity.x, 0, friction * delta)
	player.velocity.z = move_toward(player.velocity.z, 0, friction * delta)
	player.move_and_slide()
"#;

pub const MOVE_STATE_GD: &str = r#"extends State
class_name MoveState
## Player movement state - handles walking/running on ground

@export var player_path: NodePath = "../.."
@export var walk_speed: float = 4.0
@export var run_speed: float = 8.0
@export var acceleration: float = 10.0

@onready var player: CharacterBody3D = get_node(player_path)

var input_dir: Vector2
var direction: Vector3

func physics_update(delta: float) -> void:
	input_dir = Input.get_vector("move_left", "move_right", "move_up", "move_down")
	
	if input_dir.length() < 0.1:
		state_machine.transition_to("Idle")
		return
	
	if not player.is_on_floor():
		state_machine.transition_to("Air", {"jumped": false})
		return
	
	if Input.is_action_just_pressed("jump"):
		state_machine.transition_to("Air", {"jumped": true})
		return
	
	# Camera-relative movement direction
	var camera := get_viewport().get_camera_3d()
	if camera:
		var cam_basis := camera.global_transform.basis
		direction = (cam_basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
		direction.y = 0
		direction = direction.normalized()
	else:
		direction = Vector3(input_dir.x, 0, input_dir.y).normalized()
	
	# Speed based on input magnitude (analog support)
	var speed: float = lerpf(walk_speed, run_speed, input_dir.length())
	var target_velocity: Vector3 = direction * speed
	
	player.velocity.x = move_toward(player.velocity.x, target_velocity.x, acceleration * delta * speed)
	player.velocity.z = move_toward(player.velocity.z, target_velocity.z, acceleration * delta * speed)
	
	# Model rotation is handled by LocomotionController
	player.move_and_slide()
"#;

pub const AIR_STATE_GD: &str = r#"extends State
class_name AirState
## Player air state - handles jumping and falling

@export var player_path: NodePath = "../.."
@export var jump_velocity: float = 6.0
@export var air_control: float = 3.0
@export var gravity_multiplier: float = 1.0
@export var coyote_time: float = 0.15
@export var jump_buffer_time: float = 0.1

@onready var player: CharacterBody3D = get_node(player_path)

var gravity: float
var jumped: bool = false
var coyote_timer: float = 0.0
var jump_buffer_timer: float = 0.0

func _ready() -> void:
	gravity = ProjectSettings.get_setting("physics/3d/default_gravity")

func enter(data: Dictionary = {}) -> void:
	jumped = data.get("jumped", false)
	
	if jumped:
		player.velocity.y = jump_velocity
		coyote_timer = 0.0
	else:
		coyote_timer = coyote_time

func physics_update(delta: float) -> void:
	coyote_timer -= delta
	jump_buffer_timer -= delta
	
	if Input.is_action_just_pressed("jump"):
		jump_buffer_timer = jump_buffer_time
	
	if jump_buffer_timer > 0 and coyote_timer > 0 and not jumped:
		player.velocity.y = jump_velocity
		jumped = true
		coyote_timer = 0.0
		jump_buffer_timer = 0.0
	
	player.velocity.y -= gravity * gravity_multiplier * delta
	
	if jumped and Input.is_action_just_released("jump") and player.velocity.y > 0:
		player.velocity.y *= 0.5
	
	var input_dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	var camera := get_viewport().get_camera_3d()
	var direction := Vector3.ZERO
	
	if camera and input_dir.length() > 0.1:
		var cam_basis := camera.global_transform.basis
		direction = (cam_basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
		direction.y = 0
		direction = direction.normalized()
		
		player.velocity.x = move_toward(player.velocity.x, direction.x * 5.0, air_control * delta)
		player.velocity.z = move_toward(player.velocity.z, direction.z * 5.0, air_control * delta)
	
	player.move_and_slide()
	
	if player.is_on_floor():
		if input_dir.length() > 0.1:
			state_machine.transition_to("Move")
		else:
			state_machine.transition_to("Idle")
"#;

// ============================================================================
// Locomotion System - Mixamo-Compatible
// ============================================================================

pub const LOCOMOTION_CONTROLLER_GD: &str = r#"extends Node
class_name LocomotionController
## Drives AnimationTree based on character velocity and state

@export var character_path: NodePath
@export var animation_tree_path: NodePath
@export var model_path: NodePath = "../Model"
@export var blend_speed: float = 10.0
@export var rotation_speed: float = 12.0

@onready var character: CharacterBody3D = get_node_or_null(character_path)
@onready var anim_tree: AnimationTree = get_node_or_null(animation_tree_path)
@onready var model: Node3D = get_node_or_null(model_path)

var current_blend: float = 0.0
var target_rotation: float = 0.0
var is_grounded: bool = true
var is_jumping: bool = false

func _ready() -> void:
	if anim_tree:
		anim_tree.active = true

func _physics_process(delta: float) -> void:
	if not character or not anim_tree:
		return
	
	var h_velocity: Vector2 = Vector2(character.velocity.x, character.velocity.z)
	var speed: float = h_velocity.length()
	
	# Blend position: 0 = idle, 0.5 = walk (~3 m/s), 1.0 = run (~7 m/s)
	var walk_speed: float = 3.0
	var run_speed: float = 7.0
	var target_blend: float = 0.0
	
	if speed < 0.1:
		target_blend = 0.0  # Idle
	elif speed < walk_speed:
		target_blend = (speed / walk_speed) * 0.5  # Blend to walk
	else:
		target_blend = 0.5 + ((speed - walk_speed) / (run_speed - walk_speed)) * 0.5
	
	target_blend = clampf(target_blend, 0.0, 1.0)
	current_blend = lerpf(current_blend, target_blend, blend_speed * delta)
	
	# Set blend position (works with BlendSpace1D)
	anim_tree.set("parameters/locomotion/blend_position", current_blend)
	
	# Rotate model to face movement direction
	if model and speed > 0.5:
		var move_dir: Vector3 = Vector3(character.velocity.x, 0, character.velocity.z).normalized()
		target_rotation = atan2(-move_dir.x, -move_dir.z)
		model.rotation.y = lerp_angle(model.rotation.y, target_rotation, rotation_speed * delta)
	
	# Get the state machine playback
	var playback = anim_tree.get("parameters/playback") as AnimationNodeStateMachinePlayback
	if playback:
		is_grounded = character.is_on_floor()
		
		if is_grounded:
			if is_jumping:
				is_jumping = false
			if speed < 0.1:
				playback.travel("idle")
			else:
				playback.travel("locomotion")
		else:
			if character.velocity.y > 0:
				playback.travel("jump")
				is_jumping = true
			else:
				playback.travel("fall")
	
	# Set blend position for locomotion blend space
	anim_tree.set("parameters/StateMachine/locomotion/blend_position", Vector2(0, current_blend))

func set_animation_state(state_name: String) -> void:
	if not anim_tree:
		return
	var playback = anim_tree.get("parameters/StateMachine/playback") as AnimationNodeStateMachinePlayback
	if playback:
		playback.travel(state_name)
"#;

pub const MIXAMO_RETARGETER_GD: &str = r#"extends Node
class_name MixamoRetargeter
## Utility for retargeting Mixamo animations to Godot skeleton

const MIXAMO_TO_GODOT := {
	"mixamorig:Hips": "Hips",
	"mixamorig:Spine": "Spine",
	"mixamorig:Spine1": "Spine1",
	"mixamorig:Spine2": "Spine2",
	"mixamorig:Neck": "Neck",
	"mixamorig:Head": "Head",
	"mixamorig:LeftShoulder": "LeftShoulder",
	"mixamorig:LeftArm": "LeftUpperArm",
	"mixamorig:LeftForeArm": "LeftLowerArm",
	"mixamorig:LeftHand": "LeftHand",
	"mixamorig:RightShoulder": "RightShoulder",
	"mixamorig:RightArm": "RightUpperArm",
	"mixamorig:RightForeArm": "RightLowerArm",
	"mixamorig:RightHand": "RightHand",
	"mixamorig:LeftUpLeg": "LeftUpperLeg",
	"mixamorig:LeftLeg": "LeftLowerLeg",
	"mixamorig:LeftFoot": "LeftFoot",
	"mixamorig:LeftToeBase": "LeftToes",
	"mixamorig:RightUpLeg": "RightUpperLeg",
	"mixamorig:RightLeg": "RightLowerLeg",
	"mixamorig:RightFoot": "RightFoot",
	"mixamorig:RightToeBase": "RightToes",
}

@export var auto_retarget: bool = true

func _ready() -> void:
	if auto_retarget:
		retarget_skeleton()

func retarget_skeleton() -> void:
	var skeleton := get_parent() as Skeleton3D
	if not skeleton:
		push_warning("MixamoRetargeter must be child of Skeleton3D")
		return
	
	for i in skeleton.get_bone_count():
		var bone_name := skeleton.get_bone_name(i)
		if bone_name in MIXAMO_TO_GODOT:
			skeleton.set_bone_name(i, MIXAMO_TO_GODOT[bone_name])
	
	print("[MixamoRetargeter] Retargeted skeleton bones")
"#;

pub const CAMERA_RIG_3D_GD: &str = r#"extends Node3D
class_name CameraRig3D
## Third-person camera rig with smooth follow, orbit, and collision avoidance

@export var target_path: NodePath
@export var follow_speed: float = 10.0
@export var rotation_speed: float = 3.0
@export var min_pitch: float = -40.0
@export var max_pitch: float = 60.0
@export var distance: float = 5.0
@export var collision_margin: float = 0.2
@export var mouse_sensitivity: float = 0.003

@onready var target: Node3D = get_node_or_null(target_path)
@onready var spring_arm: SpringArm3D = $SpringArm3D
@onready var camera: Camera3D = $SpringArm3D/Camera3D

var _pitch: float = 0.0
var _yaw: float = 0.0
var _mouse_captured: bool = false

func _ready() -> void:
	if spring_arm:
		spring_arm.spring_length = distance
		spring_arm.margin = collision_margin
	print("Click to capture mouse, ESC to release")

func _input(event: InputEvent) -> void:
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
			_mouse_captured = true
	
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		_yaw -= event.relative.x * mouse_sensitivity
		_pitch -= event.relative.y * mouse_sensitivity
		_pitch = clamp(_pitch, deg_to_rad(min_pitch), deg_to_rad(max_pitch))
	
	if event.is_action_pressed("ui_cancel"):
		if Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_VISIBLE
			_mouse_captured = false
		else:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
			_mouse_captured = true

func _physics_process(delta: float) -> void:
	if not target:
		return
	
	global_position = global_position.lerp(target.global_position, follow_speed * delta)
	
	rotation.y = _yaw
	if spring_arm:
		spring_arm.rotation.x = _pitch

func set_target(new_target: Node3D) -> void:
	target = new_target
	if target:
		global_position = target.global_position
"#;

// ============================================================================
// AI Controller
// ============================================================================

pub const AI_CONTROLLER_GD: &str = r#"extends Node
## AI Controller for Kobold - Enables AI game testing

var enabled: bool = false
var action_queue: Array[Dictionary] = []
var frame_count: int = 0
var player: Node = null
var game_events: Array[String] = []

func _ready() -> void:
	enabled = OS.get_environment("AGENT_ENABLED") == "true"
	if not enabled:
		return
	DirAccess.make_dir_absolute(OS.get_user_data_dir() + "/user_screenshots")
	_find_player()
	_connect_events()
	_log("AIController initialized")

func _connect_events() -> void:
	if EventBus:
		EventBus.player_damaged.connect(func(amt): _log("player_damaged: %d" % amt))
		EventBus.player_died.connect(func(): _log("player_died"))
		EventBus.coin_collected.connect(func(v): _log("coin_collected: %d" % v))
		EventBus.level_completed.connect(func(): _log("level_completed"))
		EventBus.entity_died.connect(func(e): _log("entity_died: %s" % e.name if e else "unknown"))

func _find_player() -> void:
	for name in ["Player", "player", "Character", "character"]:
		player = get_tree().root.find_child(name, true, false)
		if player:
			_log("Found player: %s" % player.name)
			return

func _process(_delta: float) -> void:
	if not enabled:
		return
	frame_count += 1
	_read_actions()
	if action_queue.size() > 0:
		_execute_action(action_queue.pop_front())
	if frame_count % 10 == 0:
		_capture_screenshot()
		_save_state()

func _read_actions() -> void:
	var path = OS.get_user_data_dir() + "/agent_input.json"
	if not FileAccess.file_exists(path):
		return
	var file = FileAccess.open(path, FileAccess.READ)
	if not file:
		return
	var content = file.get_as_text()
	file.close()
	if content.is_empty() or content == "{}":
		return
	var json = JSON.new()
	if json.parse(content) != OK:
		return
	var action = json.data
	if action.has("function"):
		action_queue.append(action)
		EventBus.agent_action_received.emit(action.get("function"), action.get("args", []))
		var clear = FileAccess.open(path, FileAccess.WRITE)
		if clear:
			clear.store_string("{}")
			clear.close()

func _execute_action(action: Dictionary) -> void:
	if not player:
		_find_player()
	if not player:
		_log("ERROR: No player found")
		return
	var func_name = action.get("function", "")
	var args = action.get("args", [])
	_log("Executing: %s %s" % [func_name, args])
	match func_name:
		"move": _do_move(args)
		"jump": _do_jump()
		"attack":
			if player.has_method("attack"):
				player.attack()
		"interact":
			if player.has_method("interact"):
				player.interact()
		"pause": get_tree().paused = true
		"resume": get_tree().paused = false
		_: _log("Unknown action: %s" % func_name)

func _do_move(args: Array) -> void:
	if args.size() < 1:
		return
	var dir = args[0] if args.size() > 0 else "right"
	var vel = Vector2.ZERO
	match dir:
		"left": vel = Vector2.LEFT
		"right": vel = Vector2.RIGHT
		"up": vel = Vector2.UP
		"down": vel = Vector2.DOWN
	var speed = player.get("speed") if player.get("speed") else 200.0
	if "velocity" in player:
		player.velocity = vel * speed
	elif "position" in player:
		player.position += vel * 50

func _do_jump() -> void:
	if player.has_method("jump"):
		player.jump()
	elif "velocity" in player:
		var jf = player.get("jump_force") if player.get("jump_force") else -400.0
		player.velocity.y = jf

func _capture_screenshot() -> void:
	var vp = get_viewport()
	if not vp:
		return
	var img = vp.get_texture().get_image()
	if img:
		img.save_png(OS.get_user_data_dir() + "/user_screenshots/frame_%06d.png" % frame_count)

func _save_state() -> void:
	var state = {
		"frame": frame_count,
		"scene": get_tree().current_scene.name if get_tree().current_scene else "unknown",
		"events": game_events.slice(-20),
	}
	if player:
		if "global_position" in player:
			state["player_position"] = {"x": player.global_position.x, "y": player.global_position.y}
		if "velocity" in player:
			state["player_velocity"] = {"x": player.velocity.x, "y": player.velocity.y}
	if GameState:
		state["score"] = GameState.score
		state["inventory"] = GameState.inventory
	var file = FileAccess.open(OS.get_user_data_dir() + "/game_state.json", FileAccess.WRITE)
	if file:
		file.store_string(JSON.stringify(state))
		file.close()
	EventBus.agent_state_captured.emit(state)

func _log(msg: String) -> void:
	var entry = "[F%d] %s" % [frame_count, msg]
	game_events.append(entry)
	print("[AIController] %s" % entry)
"#;

// ============================================================================
// Documentation
// ============================================================================

pub const CLAUDE_MD: &str = r#"# Kobold Project - Claude Code Instructions

This is a Godot 4.x game project with professional architecture patterns.

## Architecture Patterns (CRITICAL)

### Signal Bus Pattern
ALL cross-system communication goes through EventBus autoload:
```gdscript
# GOOD: Emit signal, let listeners react
EventBus.player_damaged.emit(damage)

# BAD: Direct coupling between systems
health_ui.update(health)
audio.play_sound()
```

### Entity-Component Pattern
Entities are composed of reusable components:
```gdscript
# Player has HealthComponent, MovementComponent as children
# Components emit signals, parent orchestrates
@onready var health_comp: HealthComponent = $HealthComponent
health_comp.died.connect(_on_died)
```

## Project Structure (Feature-Based)
```
project.godot           # Config with EventBus, GameState, AIController autoloads
autoload/
  event_bus.gd          # Signal hub - ALL cross-system communication
  game_state.gd         # Persistent data, score, inventory
  ai_controller.gd      # AI game testing (auto-enabled)
assets/
  entities/player/      # Player scene + scripts + assets together
  entities/enemies/     # Enemy types organized together
  ui/                   # UI scenes and components
  worlds/               # Levels, tilemaps
src/
  components/           # Reusable: HealthComponent, MovementComponent2D
  systems/              # Game systems (inventory, dialogue)
  utilities/            # Helper functions
scenes/                 # Main entry scenes
```

## Available Components
- `HealthComponent` - Damage, healing, death signals
- `MovementComponent2D` - 2D movement with acceleration/friction
- `StateMachine` - Generic FSM for any entity
- `State` - Base class for states (extend for custom states)
- `CameraRig3D` - Third-person camera with orbit and collision

## Locomotion States (3D)
- `IdleState` - Standing still, waits for input
- `MoveState` - Walking/running, camera-relative movement
- `AirState` - Jumping/falling with coyote time & jump buffering

## EventBus Signals Available
```gdscript
# Player
EventBus.player_spawned.emit(player)
EventBus.player_damaged.emit(amount)
EventBus.player_died.emit()

# Entities
EventBus.entity_damaged.emit(entity, amount)
EventBus.entity_died.emit(entity)

# Game Flow
EventBus.level_completed.emit()
EventBus.score_changed.emit(new_score)

# Items
EventBus.coin_collected.emit(value)
EventBus.item_collected.emit(item_id)
```

## GameState Usage
```gdscript
GameState.add_score(100)
GameState.add_item("health_potion", 3)
GameState.has_item("key")
GameState.mark_level_completed("level_1")
```
"#;

pub const ANIMATION_SETUP_GUIDE: &str = r#"## Character Animation Guide

### Default Character (Quaternius)
The project includes a pre-animated character from Quaternius Universal Animation Library.
- License: CC0 (Public Domain)
- Location: `assets/characters/character.glb`
- 100+ built-in animations including locomotion, combat, and interactions

### Included Animations
- Idle, Idle_Combat
- Walk_F, Walk_B, Walk_L, Walk_R
- Jog_F, Jog_B, Jog_L, Jog_R  
- Sprint_F, Sprint_B, Sprint_L, Sprint_R
- Crouch_Idle, Crouch_Walk_F/B/L/R
- Jump, Jump_Idle, Jump_Land
- And many more...

### How It Works
The player script automatically:
1. Loads the character GLB from `assets/characters/`
2. Finds the AnimationPlayer
3. Creates an AnimationTree with BlendSpace1D for locomotion
4. Connects to the LocomotionController

### Using Your Own Character
1. Export your character as GLB with embedded animations
2. Name it `character.glb`
3. Place in `assets/characters/`
4. The player script will auto-detect it

### File Structure
assets/
  characters/
    character.glb      <- Animated character model

### More Animations
Download additional packs from:
- https://quaternius.itch.io/universal-animation-library
- https://quaternius.itch.io/universal-animation-library-2
"#;

// ============================================================================
// Web Export Preset
// ============================================================================

pub const WEB_EXPORT_PRESET: &str = r#"[preset.0]

name="Web"
platform="Web"
runnable=true
dedicated_server=false
custom_features=""
export_filter="all_resources"
include_filter=""
exclude_filter=""
export_path="export/web/index.html"
encryption_include_filters=""
encryption_exclude_filters=""
encrypt_pck=false
encrypt_directory=false

[preset.0.options]

custom_template/debug=""
custom_template/release=""
variant/extensions_support=false
vram_texture_compression/for_desktop=true
vram_texture_compression/for_mobile=false
html/export_icon=true
html/custom_html_shell=""
html/head_include=""
html/canvas_resize_policy=2
html/focus_canvas_on_start=true
html/experimental_virtual_keyboard=false
progressive_web_app/enabled=false
progressive_web_app/offline_page=""
progressive_web_app/display=1
progressive_web_app/orientation=0
progressive_web_app/icon_144x144=""
progressive_web_app/icon_180x180=""
progressive_web_app/icon_512x512=""
progressive_web_app/background_color=Color(0, 0, 0, 1)
"#;

// ============================================================================
// Kobold Bridge - Native API for Play Mode Testing
// ============================================================================

pub const KOBOLD_BRIDGE_GD: &str = r#"extends Node
## Kobold Bridge - Provides native API for Kobold play mode testing
## Auto-injected during web export for preview

signal state_updated(state: Dictionary)

var _player: Node = null
var _camera: Node = null  # Can be Camera3D or Camera2D
var _last_state: Dictionary = {}

func _ready() -> void:
	# Register JavaScript callback for web builds
	if OS.has_feature("web"):
		_setup_js_bridge()
	
	# Find player and camera after scene loads
	await get_tree().process_frame
	_find_game_nodes()
	print("[KoboldBridge] Ready")

func _setup_js_bridge() -> void:
	# Expose functions to JavaScript
	var js_code = """
	window.KoboldBridge = {
		getState: function() { return window._kobold_get_state(); },
		sendInput: function(action, pressed) { return window._kobold_send_input(action, pressed); },
		getSceneTree: function() { return window._kobold_get_scene_tree(); },
		captureFrame: function() { return window._kobold_capture_frame(); },
		captureNode: function(nodeId, options) { return window._kobold_capture_node(nodeId, JSON.stringify(options || {})); },
		findNode: function(name) { return window._kobold_find_node(name); }
	};
	window.dispatchEvent(new CustomEvent('kobold-bridge-ready'));
	"""
	JavaScriptBridge.eval(js_code)
	
	# Create callbacks
	var get_state_cb = JavaScriptBridge.create_callback(_js_get_state)
	var send_input_cb = JavaScriptBridge.create_callback(_js_send_input)
	var get_tree_cb = JavaScriptBridge.create_callback(_js_get_scene_tree)
	var capture_cb = JavaScriptBridge.create_callback(_js_capture_frame)
	var capture_node_cb = JavaScriptBridge.create_callback(_js_capture_node)
	var find_node_cb = JavaScriptBridge.create_callback(_js_find_node)
	
	JavaScriptBridge.get_interface("window").set("_kobold_get_state", get_state_cb)
	JavaScriptBridge.get_interface("window").set("_kobold_send_input", send_input_cb)
	JavaScriptBridge.get_interface("window").set("_kobold_get_scene_tree", get_tree_cb)
	JavaScriptBridge.get_interface("window").set("_kobold_capture_frame", capture_cb)
	JavaScriptBridge.get_interface("window").set("_kobold_capture_node", capture_node_cb)
	JavaScriptBridge.get_interface("window").set("_kobold_find_node", find_node_cb)

func _find_game_nodes() -> void:
	# Find player (look for common patterns)
	for node in get_tree().get_nodes_in_group("player"):
		_player = node
		break
	if not _player:
		_player = _find_node_by_name(get_tree().root, ["Player", "player", "Character"])
	
	# Find camera (3D or 2D)
	_camera = get_viewport().get_camera_3d()

func _find_node_by_name(root: Node, names: Array) -> Node:
	for child in root.get_children():
		if child.name in names or child.name.to_lower() in names:
			return child
		var found = _find_node_by_name(child, names)
		if found:
			return found
	return null

func _physics_process(_delta: float) -> void:
	_last_state = get_game_state()

# ============================================================================
# Public API
# ============================================================================

func get_game_state() -> Dictionary:
	var state: Dictionary = {
		"timestamp": Time.get_ticks_msec(),
		"scene": get_tree().current_scene.scene_file_path if get_tree().current_scene else "",
	}
	
	# Player state
	if _player:
		state["player"] = {
			"position": _node_position(_player),
			"velocity": _node_velocity(_player),
			"on_floor": _player.is_on_floor() if _player.has_method("is_on_floor") else null,
			"animation": _get_animation(_player),
		}
	
	# Camera state
	if _camera:
		var cam_state: Dictionary = {"position": _node_position(_camera)}
		if _camera is Node3D:
			cam_state["rotation"] = _v3_to_dict(_camera.rotation)
		state["camera"] = cam_state
	
	# Input state
	state["input"] = _get_input_state()
	
	return state

func send_input(action: String, pressed: bool = true) -> bool:
	if not InputMap.has_action(action):
		push_warning("[KoboldBridge] Unknown action: " + action)
		return false
	
	var event = InputEventAction.new()
	event.action = action
	event.pressed = pressed
	Input.parse_input_event(event)
	return true

func send_input_sequence(actions: Array, duration_ms: int = 500) -> void:
	# Press all actions
	for action in actions:
		send_input(action, true)
	
	# Wait
	await get_tree().create_timer(duration_ms / 1000.0).timeout
	
	# Release all actions
	for action in actions:
		send_input(action, false)

func get_scene_tree_info() -> Dictionary:
	return _serialize_node(get_tree().root)

# ============================================================================
# JavaScript Callbacks
# ============================================================================

func _js_get_state(_args: Array) -> String:
	return JSON.stringify(get_game_state())

func _js_send_input(args: Array) -> String:
	if args.size() < 1:
		return JSON.stringify({"error": "Missing action"})
	var action = str(args[0])
	var pressed = args[1] if args.size() > 1 else true
	var result = send_input(action, pressed)
	return JSON.stringify({"success": result, "action": action})

func _js_get_scene_tree(_args: Array) -> String:
	return JSON.stringify(get_scene_tree_info())

func _js_capture_frame(_args: Array) -> String:
	# Trigger capture via the existing canvas method
	return JSON.stringify({"method": "use_canvas_capture"})

func _js_find_node(args: Array) -> String:
	if args.size() < 1:
		return JSON.stringify({"error": "Missing node name"})
	var node = find_node(str(args[0]))
	if node:
		var info: Dictionary = {
			"found": true,
			"name": node.name,
			"path": str(node.get_path()),
			"class": node.get_class()
		}
		if node is Node3D:
			info["position"] = _v3_to_dict(node.global_position)
			info["bounds"] = get_node_bounds(node)
		return JSON.stringify(info)
	return JSON.stringify({"found": false})

# ============================================================================
# Helpers
# ============================================================================

func _node_position(node: Node) -> Dictionary:
	if node is Node3D:
		return _v3_to_dict(node.global_position)
	elif node is Node2D:
		return {"x": node.global_position.x, "y": node.global_position.y}
	return {}

func _node_velocity(node: Node) -> Dictionary:
	if node is CharacterBody3D:
		return _v3_to_dict(node.velocity)
	elif node is CharacterBody2D:
		return {"x": node.velocity.x, "y": node.velocity.y}
	return {}

func _v3_to_dict(v: Vector3) -> Dictionary:
	return {"x": v.x, "y": v.y, "z": v.z}

func _get_animation(node: Node) -> String:
	# Check for AnimationPlayer
	var anim_player = node.find_child("AnimationPlayer", true, false)
	if anim_player and anim_player is AnimationPlayer:
		return anim_player.current_animation
	
	# Check for AnimationTree
	var anim_tree = node.find_child("AnimationTree", true, false)
	if anim_tree and anim_tree is AnimationTree:
		var playback = anim_tree.get("parameters/StateMachine/playback")
		if playback:
			return playback.get_current_node()
	
	return ""

func _get_input_state() -> Dictionary:
	var input_state: Dictionary = {}
	for action in InputMap.get_actions():
		if not action.begins_with("ui_"):
			input_state[action] = Input.is_action_pressed(action)
	return input_state

func _serialize_node(node: Node, depth: int = 0) -> Dictionary:
	if depth > 5:
		return {"name": node.name, "truncated": true}
	
	var data: Dictionary = {
		"name": node.name,
		"class": node.get_class(),
	}
	
	if node is Node3D:
		data["position"] = _v3_to_dict(node.position)
	elif node is Node2D:
		data["position"] = {"x": node.position.x, "y": node.position.y}
	
	if node.get_child_count() > 0:
		data["children"] = []
		for child in node.get_children():
			data["children"].append(_serialize_node(child, depth + 1))
	
	return data

# ============================================================================
# Object Capture System - Multi-angle screenshots for AI analysis
# ============================================================================

var _capture_camera: Camera3D = null
var _capture_viewport: SubViewport = null
var _capture_pending: bool = false
var _capture_results: Array = []
var _capture_callback: Callable

func _ensure_capture_system() -> void:
	if _capture_viewport:
		return
	
	# Create a SubViewport for isolated rendering
	_capture_viewport = SubViewport.new()
	_capture_viewport.size = Vector2i(512, 512)
	_capture_viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
	_capture_viewport.transparent_bg = true
	add_child(_capture_viewport)
	
	# Create capture camera
	_capture_camera = Camera3D.new()
	_capture_camera.current = false
	_capture_viewport.add_child(_capture_camera)

func find_node(identifier: String) -> Node:
	# Try as path first
	if identifier.begins_with("/") or identifier.begins_with("@"):
		var node = get_tree().root.get_node_or_null(NodePath(identifier.trim_prefix("@")))
		if node:
			return node
	
	# Search by name
	return _find_node_recursive(get_tree().root, identifier)

func _find_node_recursive(root: Node, name: String) -> Node:
	if root.name.to_lower() == name.to_lower():
		return root
	for child in root.get_children():
		var found = _find_node_recursive(child, name)
		if found:
			return found
	return null

func get_node_bounds(node: Node3D) -> Dictionary:
	# Calculate AABB for the node and its children
	var aabb = AABB()
	var found_mesh = false
	
	# Check if node itself has a mesh
	if node is MeshInstance3D:
		aabb = node.get_aabb()
		found_mesh = true
	
	# Check children for meshes
	for child in node.get_children():
		if child is MeshInstance3D:
			var child_aabb = child.get_aabb()
			child_aabb.position += child.position
			if found_mesh:
				aabb = aabb.merge(child_aabb)
			else:
				aabb = child_aabb
				found_mesh = true
		# Recursively check grandchildren
		for grandchild in child.get_children():
			if grandchild is MeshInstance3D:
				var gc_aabb = grandchild.get_aabb()
				gc_aabb.position += child.position + grandchild.position
				if found_mesh:
					aabb = aabb.merge(gc_aabb)
				else:
					aabb = gc_aabb
					found_mesh = true
	
	if not found_mesh:
		# Estimate bounds from node
		aabb = AABB(Vector3(-0.5, 0, -0.5), Vector3(1, 2, 1))
	
	return {
		"center": _v3_to_dict(node.global_position + aabb.get_center()),
		"size": _v3_to_dict(aabb.size),
		"found_mesh": found_mesh
	}

func capture_node_multi_angle(node_identifier: String, options: Dictionary = {}) -> Dictionary:
	var node = find_node(node_identifier)
	if not node:
		return {"error": "Node not found: " + node_identifier}
	
	if not node is Node3D:
		return {"error": "Node is not a 3D node: " + node_identifier}
	
	_ensure_capture_system()
	
	var distance: float = options.get("distance", 3.0)
	var height_offset: float = options.get("height", 1.0)
	var angles: Array = options.get("angles", ["front", "back", "left", "right"])
	var include_top: bool = options.get("top", false)
	var custom_angle: Dictionary = options.get("custom", {})
	
	var bounds = get_node_bounds(node)
	var center = Vector3(bounds.center.x, bounds.center.y, bounds.center.z)
	var size = Vector3(bounds.size.x, bounds.size.y, bounds.size.z)
	
	# Auto-adjust distance based on object size
	var max_dim = max(size.x, max(size.y, size.z))
	if distance <= 0:
		distance = max_dim * 2.5
	
	var results: Dictionary = {
		"node": node_identifier,
		"bounds": bounds,
		"captures": {}
	}
	
	# Standard angles (yaw in degrees)
	var angle_map = {
		"front": 0.0,
		"right": 90.0,
		"back": 180.0,
		"left": 270.0,
		"front_right": 45.0,
		"back_right": 135.0,
		"back_left": 225.0,
		"front_left": 315.0
	}
	
	for angle_name in angles:
		if angle_map.has(angle_name):
			var yaw = deg_to_rad(angle_map[angle_name])
			var cam_pos = center + Vector3(
				sin(yaw) * distance,
				height_offset,
				cos(yaw) * distance
			)
			var capture = await _capture_from_position(cam_pos, center)
			results.captures[angle_name] = capture
	
	# Top-down view
	if include_top:
		var top_pos = center + Vector3(0, distance * 1.5, 0.01)
		var capture = await _capture_from_position(top_pos, center)
		results.captures["top"] = capture
	
	# Custom angle
	if custom_angle.size() > 0:
		var custom_yaw = deg_to_rad(custom_angle.get("yaw", 0.0))
		var custom_pitch = deg_to_rad(custom_angle.get("pitch", 0.0))
		var custom_dist = custom_angle.get("distance", distance)
		var cam_pos = center + Vector3(
			sin(custom_yaw) * cos(custom_pitch) * custom_dist,
			sin(custom_pitch) * custom_dist + height_offset,
			cos(custom_yaw) * cos(custom_pitch) * custom_dist
		)
		var capture = await _capture_from_position(cam_pos, center)
		results.captures["custom"] = capture
	
	return results

func _capture_from_position(cam_pos: Vector3, look_at_pos: Vector3) -> String:
	_capture_camera.global_position = cam_pos
	_capture_camera.look_at(look_at_pos)
	
	# Render one frame
	_capture_viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
	await RenderingServer.frame_post_draw
	
	# Get image and convert to base64
	var img = _capture_viewport.get_texture().get_image()
	var png_data = img.save_png_to_buffer()
	return Marshalls.raw_to_base64(png_data)

func _js_capture_node(args: Array) -> String:
	if args.size() < 1:
		return JSON.stringify({"error": "Missing node identifier"})
	
	var node_id = str(args[0])
	var options: Dictionary = {}
	if args.size() > 1 and args[1] is Dictionary:
		options = args[1]
	elif args.size() > 1:
		# Parse JSON string
		var parsed = JSON.parse_string(str(args[1]))
		if parsed is Dictionary:
			options = parsed
	
	# This needs to be async, so we return a promise ID
	var promise_id = "capture_" + str(Time.get_ticks_msec())
	_start_async_capture(promise_id, node_id, options)
	return JSON.stringify({"promise_id": promise_id, "status": "pending"})

func _start_async_capture(promise_id: String, node_id: String, options: Dictionary) -> void:
	var result = await capture_node_multi_angle(node_id, options)
	# Emit result to JavaScript
	if OS.has_feature("web"):
		var js_result = JSON.stringify(result)
		var js_code = "window.dispatchEvent(new CustomEvent('kobold-capture-complete', { detail: { id: '%s', result: %s } }));" % [promise_id, js_result]
		JavaScriptBridge.eval(js_code)
"#;
