extends CharacterBody2D
class_name Player
## 2D Top-Down Player - Uses Entity-Component Pattern
## HealthComponent attached as child handles damage/death

@export var speed: float = 200.0

@onready var health_comp: HealthComponent = $HealthComponent

func _ready() -> void:
	if health_comp:
		health_comp.died.connect(_on_died)
		health_comp.health_changed.connect(_on_health_changed)
	EventBus.player_spawned.emit(self)
	print("Top-down ready! Use WASD/Arrows to move, E to interact")

func _physics_process(_delta: float) -> void:
	var input_dir := Vector2(
		Input.get_axis("move_left", "move_right"),
		Input.get_axis("move_up", "move_down")
	)
	velocity = input_dir.normalized() * speed
	move_and_slide()

func take_damage(amount: int) -> void:
	if health_comp:
		health_comp.take_damage(amount)

func interact() -> void:
	# Override for interaction logic
	print("Interact pressed!")

func _on_health_changed(current: int, maximum: int) -> void:
	EventBus.health_changed.emit(current, maximum)

func _on_died() -> void:
	EventBus.player_died.emit()
	print("Player died!")
