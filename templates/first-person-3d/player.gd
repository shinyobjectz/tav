extends CharacterBody3D
class_name Player
## First Person Player - Uses Entity-Component Pattern

@export var speed: float = 5.0
@export var mouse_sensitivity: float = 0.002

var gravity: float = ProjectSettings.get_setting("physics/3d/default_gravity")
@onready var camera: Camera3D = $Camera3D
@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	# Don't capture mouse in _ready - wait for click (required for web)
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Click to capture mouse, WASD to move, ESC to release")

func _input(event: InputEvent) -> void:
	# Capture mouse on click (web-compatible)
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if Input.mouse_mode != Input.MOUSE_MODE_CAPTURED:
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
	
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		rotate_y(-event.relative.x * mouse_sensitivity)
		camera.rotate_x(-event.relative.y * mouse_sensitivity)
		camera.rotation.x = clamp(camera.rotation.x, -PI/2, PI/2)
	
	if event.is_action_pressed("ui_cancel"):
		Input.mouse_mode = Input.MOUSE_MODE_VISIBLE

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y -= gravity * delta
	
	var input_dir := Input.get_vector("move_left", "move_right", "move_up", "move_down")
	var direction := (transform.basis * Vector3(input_dir.x, 0, input_dir.y)).normalized()
	
	velocity.x = direction.x * speed if direction else move_toward(velocity.x, 0, speed)
	velocity.z = direction.z * speed if direction else move_toward(velocity.z, 0, speed)
	
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
