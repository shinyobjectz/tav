extends CharacterBody3D
class_name Player
## Third Person Player - Standalone controller with character loading

@export var speed: float = 5.0
@export var sprint_speed: float = 8.0
@export var jump_velocity: float = 5.0
@export var mouse_sensitivity: float = 0.003

var gravity: float = ProjectSettings.get_setting("physics/3d/default_gravity")
var health_comp: Node

@onready var model: Node3D = $Model
@onready var camera_pivot: Node3D = $CameraPivot
@onready var camera: Camera3D = $CameraPivot/SpringArm3D/Camera3D

var _pitch: float = 0.0

func _ready() -> void:
	health_comp = get_node_or_null("HealthComponent")
	if health_comp and health_comp.has_signal("died"):
		health_comp.died.connect(_on_died)
	
	_load_character()
	
	if EventBus:
		EventBus.player_spawned.emit(self)
	
	print("Click to capture mouse, WASD to move, Shift sprint, Space jump")

func _load_character() -> void:
	var char_path := "res://assets/characters/character.glb"
	if ResourceLoader.exists(char_path):
		var scene = load(char_path) as PackedScene
		if scene:
			var instance = scene.instantiate()
			instance.name = "Character"
			model.add_child(instance)
			print("[Player] Character loaded: ", char_path)
	else:
		# Create placeholder capsule
		var capsule := CapsuleMesh.new()
		capsule.radius = 0.35
		capsule.height = 1.8
		var mat := StandardMaterial3D.new()
		mat.albedo_color = Color(0.4, 0.6, 0.9)
		capsule.material = mat
		var mesh := MeshInstance3D.new()
		mesh.mesh = capsule
		mesh.position.y = 0.9
		model.add_child(mesh)
		print("[Player] Using placeholder - character.glb not found")

func _input(event: InputEvent) -> void:
	# Click to capture mouse (required for web)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	
	# Mouse look when captured
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		camera_pivot.rotate_y(-event.relative.x * mouse_sensitivity)
		_pitch -= event.relative.y * mouse_sensitivity
		_pitch = clamp(_pitch, deg_to_rad(-80), deg_to_rad(80))
		$CameraPivot/SpringArm3D.rotation.x = _pitch
	
	# ESC to toggle mouse capture
	if event.is_action_pressed("ui_cancel"):
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE if Input.mouse_mode == Input.MOUSE_MODE_CAPTURED else Input.MOUSE_MODE_CAPTURED

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y -= gravity * delta
	
	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = jump_velocity
	
	var input_dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	var direction := (camera_pivot.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	
	var current_speed := sprint_speed if Input.is_action_pressed("sprint") else speed
	
	if direction:
		velocity.x = direction.x * current_speed
		velocity.z = direction.z * current_speed
		if model:
			model.rotation.y = lerp_angle(model.rotation.y, atan2(direction.x, direction.z), 10.0 * delta)
	else:
		velocity.x = move_toward(velocity.x, 0, current_speed)
		velocity.z = move_toward(velocity.z, 0, current_speed)
	
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp and health_comp.has_method("take_damage"):
		health_comp.take_damage(amount)

func _on_died() -> void:
	if EventBus:
		EventBus.player_died.emit()
