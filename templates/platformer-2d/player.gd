extends CharacterBody2D
class_name Player
## 2D Platformer Player - Uses Entity-Component Pattern
## HealthComponent attached as child handles damage/death

@export var speed: float = 300.0
@export var jump_force: float = -400.0

var gravity: float = ProjectSettings.get_setting("physics/2d/default_gravity")
@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	# Connect to component signals
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Player ready! Use WASD/Arrows + Space to jump")

func _physics_process(delta: float) -> void:
	if not is_on_floor():
		velocity.y += gravity * delta
	
	if Input.is_action_just_pressed("jump") and is_on_floor():
		velocity.y = jump_force
	
	var direction := Input.get_axis("move_left", "move_right")
	velocity.x = direction * speed if direction else move_toward(velocity.x, 0, speed)
	
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
	print("Player died!")
