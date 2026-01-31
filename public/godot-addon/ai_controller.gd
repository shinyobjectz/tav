extends Node
## AI Controller for Kobold - Enables AI game testing
## Supports both discrete inputs and analog gamepad values (for NitroGen)

var enabled: bool = false
var action_queue: Array[Dictionary] = []
var frame_count: int = 0
var player: Node = null
var game_events: Array[String] = []
var held_actions: Dictionary = {}
var project_dir: String = ""
var capture_interval: int = 6  # Capture every N frames (~10 FPS at 60 FPS game)

# Analog input state (for NitroGen gamepad output)
var analog_move: Vector2 = Vector2.ZERO  # Left stick
var analog_look: Vector2 = Vector2.ZERO  # Right stick

const CONTROLS := {
	"move": {"type": "direction", "values": ["left", "right", "up", "down"], "description": "Move player in direction"},
	"move_analog": {"type": "vector2", "description": "Analog movement (x, y from -1 to 1)"},
	"jump": {"type": "action", "description": "Jump (when grounded)"},
	"sprint": {"type": "hold", "description": "Sprint while moving"},
	"crouch": {"type": "hold", "description": "Crouch/sneak"},
	"attack": {"type": "action", "description": "Melee attack"},
	"interact": {"type": "action", "description": "Interact with nearby object"},
	"look": {"type": "vector2", "description": "Camera look direction (x, y degrees)"},
	"look_analog": {"type": "vector2", "description": "Analog camera (x, y from -1 to 1)"},
	"stop": {"type": "action", "description": "Stop all movement and release held inputs"}
}

func _ready() -> void:
	enabled = OS.get_environment("AGENT_ENABLED") == "true"
	if not enabled:
		return
	# Use project directory for IPC (matches Rust backend expectations)
	project_dir = ProjectSettings.globalize_path("res://")
	DirAccess.make_dir_absolute(project_dir + "user_screenshots")
	_find_player()
	_connect_events()
	_export_controls()
	_log("AIController initialized - IPC dir: " + project_dir)

func _connect_events() -> void:
	if not EventBus:
		return
	if EventBus.has_signal("player_damaged"):
		EventBus.player_damaged.connect(func(amt): _log("player_damaged: %d" % amt))
	if EventBus.has_signal("player_died"):
		EventBus.player_died.connect(func(): _log("player_died"))
	if EventBus.has_signal("coin_collected"):
		EventBus.coin_collected.connect(func(v): _log("coin_collected: %d" % v))
	if EventBus.has_signal("level_completed"):
		EventBus.level_completed.connect(func(): _log("level_completed"))

func _export_controls() -> void:
	var file = FileAccess.open(project_dir + "controls.json", FileAccess.WRITE)
	if file:
		file.store_string(JSON.stringify(CONTROLS, "\t"))
		file.close()
		_log("Controls exported")

func _find_player() -> void:
	for name in ["Player", "player", "Character", "character"]:
		player = get_tree().root.find_child(name, true, false)
		if player:
			_log("Found player: %s" % player.name)
			return

func _process(delta: float) -> void:
	if not enabled:
		return
	frame_count += 1
	_read_actions()
	if action_queue.size() > 0:
		_execute_action(action_queue.pop_front())
	# Apply analog movement continuously
	_apply_analog_input(delta)
	# Capture at higher frequency for NitroGen (every 6 frames = ~10 FPS)
	if frame_count % capture_interval == 0:
		_capture_screenshot()
		_save_state()

func _apply_analog_input(delta: float) -> void:
	# Apply analog movement via input simulation
	if analog_move.length() > 0.1:
		# Simulate joystick as directional keys with intensity
		if analog_move.x < -0.3:
			Input.action_press("move_left", abs(analog_move.x))
		else:
			Input.action_release("move_left")
		if analog_move.x > 0.3:
			Input.action_press("move_right", analog_move.x)
		else:
			Input.action_release("move_right")
		if analog_move.y < -0.3:
			Input.action_press("move_up", abs(analog_move.y))
		else:
			Input.action_release("move_up")
		if analog_move.y > 0.3:
			Input.action_press("move_down", analog_move.y)
		else:
			Input.action_release("move_down")
	
	# Apply analog look
	if analog_look.length() > 0.1:
		var event = InputEventMouseMotion.new()
		event.relative = analog_look * 15.0 * delta * 60.0  # Scale for ~60fps
		Input.parse_input_event(event)

func _read_actions() -> void:
	var path = project_dir + "agent_input.json"
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
		if EventBus:
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
		"move_analog": _do_move_analog(args)
		"jump": _do_jump()
		"sprint": _set_held("sprint", args[0] if args.size() > 0 else true)
		"crouch": _set_held("crouch", args[0] if args.size() > 0 else true)
		"look": _do_look(args)
		"look_analog": _do_look_analog(args)
		"stop": _stop_all()
		"attack":
			if player.has_method("attack"):
				player.attack()
			else:
				Input.action_press("attack")
				await get_tree().create_timer(0.1).timeout
				Input.action_release("attack")
		"interact":
			if player.has_method("interact"):
				player.interact()
			else:
				Input.action_press("interact")
				await get_tree().create_timer(0.1).timeout
				Input.action_release("interact")
		"pause": get_tree().paused = true
		"resume": get_tree().paused = false
		_: _log("Unknown action: %s" % func_name)

func _do_move_analog(args: Array) -> void:
	if args.size() < 2:
		return
	analog_move = Vector2(float(args[0]), float(args[1]))

func _do_look_analog(args: Array) -> void:
	if args.size() < 2:
		return
	analog_look = Vector2(float(args[0]), float(args[1]))

func _do_move(args: Array) -> void:
	if args.size() < 1:
		return
	# Release all movement first
	Input.action_release("move_left")
	Input.action_release("move_right")
	Input.action_release("move_up")
	Input.action_release("move_down")
	# Press the requested direction
	var dir = args[0] if args.size() > 0 else "right"
	match dir:
		"left": Input.action_press("move_left")
		"right": Input.action_press("move_right")
		"up": Input.action_press("move_up")
		"down": Input.action_press("move_down")
		"stop": pass  # already released above

func _do_jump() -> void:
	Input.action_press("jump")
	await get_tree().create_timer(0.1).timeout
	Input.action_release("jump")

func _set_held(action: String, pressed: bool) -> void:
	held_actions[action] = pressed
	if pressed:
		Input.action_press(action)
	else:
		Input.action_release(action)

func _do_look(args: Array) -> void:
	if args.size() < 2:
		return
	var x_deg = float(args[0])
	var y_deg = float(args[1])
	# Emit mouse motion event for camera control
	var event = InputEventMouseMotion.new()
	event.relative = Vector2(x_deg * 10, y_deg * 10)
	Input.parse_input_event(event)

func _stop_all() -> void:
	Input.action_release("move_left")
	Input.action_release("move_right")
	Input.action_release("move_up")
	Input.action_release("move_down")
	Input.action_release("sprint")
	Input.action_release("crouch")
	held_actions.clear()
	analog_move = Vector2.ZERO
	analog_look = Vector2.ZERO
	if player and "velocity" in player:
		player.velocity.x = 0
		player.velocity.z = 0

func _capture_screenshot() -> void:
	var vp = get_viewport()
	if not vp:
		_log("No viewport")
		return
	var tex = vp.get_texture()
	if not tex:
		_log("No viewport texture (headless?)")
		return
	var img = tex.get_image()
	if img and img.get_width() > 0:
		var path = project_dir + "user_screenshots/frame_%06d.png" % frame_count
		var err = img.save_png(path)
		if err == OK:
			if frame_count % 30 == 0:  # Log every 30th frame
				_log("Screenshot saved: " + path)
		else:
			_log("Failed to save screenshot: " + str(err))

func _save_state() -> void:
	var state = {
		"frame": frame_count,
		"scene": get_tree().current_scene.name if get_tree().current_scene else "unknown",
		"events": game_events.slice(-20),
	}
	if player:
		if "global_position" in player:
			var pos = player.global_position
			state["player_position"] = {"x": pos.x, "y": pos.y, "z": pos.z if pos is Vector3 else 0}
		if "velocity" in player:
			var vel = player.velocity
			state["player_velocity"] = {"x": vel.x, "y": vel.y, "z": vel.z if vel is Vector3 else 0}
		if "is_on_floor" in player and player.has_method("is_on_floor"):
			state["on_floor"] = player.is_on_floor()
	if GameState:
		state["score"] = GameState.score
		state["inventory"] = GameState.inventory
	var file = FileAccess.open(project_dir + "game_state.json", FileAccess.WRITE)
	if file:
		file.store_string(JSON.stringify(state))
		file.close()
	if EventBus:
		EventBus.agent_state_captured.emit(state)

func _log(msg: String) -> void:
	var entry = "[F%d] %s" % [frame_count, msg]
	game_events.append(entry)
	print("[AIController] %s" % entry)
