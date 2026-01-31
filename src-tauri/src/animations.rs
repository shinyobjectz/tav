use serde::{Deserialize, Serialize};

/// Animation pack metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationPack {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub animations: Vec<AnimationInfo>,
    pub source: AnimationSource,
    pub license: String,
    pub rig_type: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationInfo {
    pub name: String,
    pub file: String,
    pub loop_mode: String,
    pub duration: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AnimationSource {
    #[serde(rename = "url")]
    Url { url: String },
    #[serde(rename = "github")]
    GitHub { repo: String, path: String },
    #[serde(rename = "itch")]
    Itch { page: String, file: String },
    #[serde(rename = "bundled")]
    Bundled { asset_name: String },
}

/// Built-in animation catalog - prioritizing Quaternius CC0 packs
pub fn get_animation_catalog() -> Vec<AnimationPack> {
    vec![
        // Quaternius Universal Animation Library - THE recommended option
        AnimationPack {
            id: "quaternius-ual".to_string(),
            name: "Quaternius Universal Animation Library".to_string(),
            description: "120+ CC0 humanoid animations on a universal rig. Locomotion, combat, emotes, and more. Godot-ready.".to_string(),
            category: "complete".to_string(),
            license: "CC0".to_string(),
            rig_type: "humanoid-universal".to_string(),
            download_url: Some("https://quaternius.itch.io/universal-animation-library".to_string()),
            source: AnimationSource::Itch {
                page: "quaternius/universal-animation-library".to_string(),
                file: "Universal_Animation_Library.zip".to_string(),
            },
            animations: vec![
                // Locomotion
                AnimationInfo { name: "Idle".to_string(), file: "Idle.glb".to_string(), loop_mode: "loop".to_string(), duration: 2.0, tags: vec!["idle".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Walk_F".to_string(), file: "Walk_F.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.0, tags: vec!["walk".to_string(), "forward".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Walk_B".to_string(), file: "Walk_B.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.0, tags: vec!["walk".to_string(), "backward".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Walk_L".to_string(), file: "Walk_L.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.0, tags: vec!["walk".to_string(), "strafe".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Walk_R".to_string(), file: "Walk_R.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.0, tags: vec!["walk".to_string(), "strafe".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Jog_F".to_string(), file: "Jog_F.glb".to_string(), loop_mode: "loop".to_string(), duration: 0.7, tags: vec!["jog".to_string(), "run".to_string(), "locomotion".to_string()] },
                AnimationInfo { name: "Sprint_F".to_string(), file: "Sprint_F.glb".to_string(), loop_mode: "loop".to_string(), duration: 0.5, tags: vec!["sprint".to_string(), "run".to_string(), "locomotion".to_string()] },
                // Jumping
                AnimationInfo { name: "Jump".to_string(), file: "Jump.glb".to_string(), loop_mode: "once".to_string(), duration: 0.5, tags: vec!["jump".to_string(), "air".to_string()] },
                AnimationInfo { name: "Jump_Idle".to_string(), file: "Jump_Idle.glb".to_string(), loop_mode: "loop".to_string(), duration: 0.5, tags: vec!["fall".to_string(), "air".to_string()] },
                AnimationInfo { name: "Jump_Land".to_string(), file: "Jump_Land.glb".to_string(), loop_mode: "once".to_string(), duration: 0.3, tags: vec!["land".to_string()] },
                // Crouch
                AnimationInfo { name: "Crouch_Idle".to_string(), file: "Crouch_Idle.glb".to_string(), loop_mode: "loop".to_string(), duration: 2.0, tags: vec!["crouch".to_string(), "idle".to_string()] },
                AnimationInfo { name: "Crouch_Walk_F".to_string(), file: "Crouch_Walk_F.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.2, tags: vec!["crouch".to_string(), "walk".to_string()] },
            ],
        },
        
        // Quaternius Universal Animation Library 2 - Extended set
        AnimationPack {
            id: "quaternius-ual2".to_string(),
            name: "Quaternius Universal Animation Library 2".to_string(),
            description: "130+ additional CC0 animations: parkour, melee combos, farming, fishing, zombie locomotion.".to_string(),
            category: "extended".to_string(),
            license: "CC0".to_string(),
            rig_type: "humanoid-universal".to_string(),
            download_url: Some("https://quaternius.itch.io/universal-animation-library-2".to_string()),
            source: AnimationSource::Itch {
                page: "quaternius/universal-animation-library-2".to_string(),
                file: "Universal_Animation_Library_2.zip".to_string(),
            },
            animations: vec![
                // Parkour
                AnimationInfo { name: "Vault".to_string(), file: "Vault.glb".to_string(), loop_mode: "once".to_string(), duration: 0.8, tags: vec!["parkour".to_string(), "vault".to_string()] },
                AnimationInfo { name: "Climb".to_string(), file: "Climb.glb".to_string(), loop_mode: "once".to_string(), duration: 1.2, tags: vec!["parkour".to_string(), "climb".to_string()] },
                AnimationInfo { name: "Roll".to_string(), file: "Roll.glb".to_string(), loop_mode: "once".to_string(), duration: 0.6, tags: vec!["parkour".to_string(), "roll".to_string(), "dodge".to_string()] },
                // Combat
                AnimationInfo { name: "Sword_Slash_1".to_string(), file: "Sword_Slash_1.glb".to_string(), loop_mode: "once".to_string(), duration: 0.5, tags: vec!["combat".to_string(), "melee".to_string(), "sword".to_string()] },
                AnimationInfo { name: "Sword_Slash_2".to_string(), file: "Sword_Slash_2.glb".to_string(), loop_mode: "once".to_string(), duration: 0.5, tags: vec!["combat".to_string(), "melee".to_string(), "sword".to_string()] },
                AnimationInfo { name: "Punch".to_string(), file: "Punch.glb".to_string(), loop_mode: "once".to_string(), duration: 0.4, tags: vec!["combat".to_string(), "melee".to_string(), "unarmed".to_string()] },
                AnimationInfo { name: "Block".to_string(), file: "Block.glb".to_string(), loop_mode: "once".to_string(), duration: 0.3, tags: vec!["combat".to_string(), "defense".to_string()] },
                // Interactions
                AnimationInfo { name: "Pick_Up".to_string(), file: "Pick_Up.glb".to_string(), loop_mode: "once".to_string(), duration: 0.8, tags: vec!["interact".to_string(), "pickup".to_string()] },
                AnimationInfo { name: "Use".to_string(), file: "Use.glb".to_string(), loop_mode: "once".to_string(), duration: 0.5, tags: vec!["interact".to_string(), "use".to_string()] },
            ],
        },
        
        // Quaternius Animated Mannequin (simpler starter option)
        AnimationPack {
            id: "quaternius-mannequin".to_string(),
            name: "Quaternius Animated Mannequin".to_string(),
            description: "Simple animated mannequin character with basic locomotion. Great for prototyping.".to_string(),
            category: "starter".to_string(),
            license: "CC0".to_string(),
            rig_type: "humanoid".to_string(),
            download_url: Some("https://quaternius.com/packs/ultimateanimatedcharacter.html".to_string()),
            source: AnimationSource::Url {
                url: "https://quaternius.com/packs/ultimateanimatedcharacter.html".to_string(),
            },
            animations: vec![
                AnimationInfo { name: "Idle".to_string(), file: "Idle.glb".to_string(), loop_mode: "loop".to_string(), duration: 2.0, tags: vec!["idle".to_string()] },
                AnimationInfo { name: "Walk".to_string(), file: "Walk.glb".to_string(), loop_mode: "loop".to_string(), duration: 1.0, tags: vec!["walk".to_string()] },
                AnimationInfo { name: "Run".to_string(), file: "Run.glb".to_string(), loop_mode: "loop".to_string(), duration: 0.6, tags: vec!["run".to_string()] },
                AnimationInfo { name: "Jump".to_string(), file: "Jump.glb".to_string(), loop_mode: "once".to_string(), duration: 1.0, tags: vec!["jump".to_string()] },
            ],
        },
    ]
}

/// GDScript template for Quaternius animation setup
pub const ANIMATION_LIBRARY_SETUP_GD: &str = r#"extends Node
class_name AnimationLibrarySetup
## Loads animations from GLB files into an AnimationLibrary
## Works with Quaternius Universal Animation Library

@export var animation_player: AnimationPlayer
@export var animations_folder: String = "res://assets/animations/"

var _library: AnimationLibrary

func _ready() -> void:
	if animation_player:
		setup_animation_library()

func setup_animation_library() -> void:
	_library = AnimationLibrary.new()
	
	var dir = DirAccess.open(animations_folder)
	if not dir:
		push_warning("Animations folder not found: " + animations_folder)
		return
	
	var count := 0
	dir.list_dir_begin()
	var file_name = dir.get_next()
	while file_name != "":
		if file_name.ends_with(".glb") or file_name.ends_with(".gltf"):
			var scene_path = animations_folder.path_join(file_name)
			var packed_scene = load(scene_path) as PackedScene
			if packed_scene:
				var instance = packed_scene.instantiate()
				var anim_player = instance.get_node_or_null("AnimationPlayer") as AnimationPlayer
				if anim_player:
					for anim_name in anim_player.get_animation_list():
						if anim_name != "RESET":
							var anim = anim_player.get_animation(anim_name)
							var clean_name = file_name.get_basename()
							_library.add_animation(clean_name, anim.duplicate())
							count += 1
				instance.queue_free()
		file_name = dir.get_next()
	
	animation_player.add_animation_library("", _library)
	print("[AnimationLibrary] Loaded ", count, " animations from ", animations_folder)
"#;

/// GDScript template for Quaternius-style locomotion with 8-direction support
pub const LOCOMOTION_BLEND_TREE_GD: &str = r#"extends Node
class_name QuaterniusLocomotion
## Full locomotion controller for Quaternius Universal Animation Library
## Supports 8-direction movement, crouch, sprint, and airborne states

@export var character: CharacterBody3D
@export var animation_tree: AnimationTree
@export var model: Node3D

@export_group("Movement Speeds")
@export var walk_speed: float = 2.5
@export var jog_speed: float = 5.0
@export var sprint_speed: float = 8.0

@export_group("Blending")
@export var blend_speed: float = 10.0
@export var rotation_speed: float = 12.0

# State
var current_blend: Vector2 = Vector2.ZERO
var is_crouching: bool = false
var is_sprinting: bool = false
var is_grounded: bool = true

func _ready() -> void:
	if animation_tree:
		animation_tree.active = true

func _physics_process(delta: float) -> void:
	if not character or not animation_tree:
		return
	
	is_grounded = character.is_on_floor()
	var velocity := character.velocity
	var h_velocity := Vector2(velocity.x, velocity.z)
	var speed := h_velocity.length()
	
	# Get movement direction relative to character facing
	var move_dir := Vector3.ZERO
	if speed > 0.1 and model:
		move_dir = Vector3(velocity.x, 0, velocity.z).normalized()
		# Convert to local space
		var local_dir := model.global_transform.basis.inverse() * move_dir
		
		# Rotate model to face movement (smooth)
		var target_rotation := atan2(-move_dir.x, -move_dir.z)
		model.rotation.y = lerp_angle(model.rotation.y, target_rotation, rotation_speed * delta)
	
	# Calculate blend position
	# X: strafe (-1 left, +1 right), Y: forward/back speed (0 idle, 0.5 walk, 1.0 run)
	var target_blend := Vector2.ZERO
	
	if speed < 0.1:
		target_blend = Vector2.ZERO  # Idle
	else:
		# Y axis: speed blend (0=idle, 0.5=walk/jog, 1.0=sprint)
		var max_speed := sprint_speed if is_sprinting else jog_speed
		target_blend.y = clampf(speed / max_speed, 0.0, 1.0)
	
	# Smooth blending
	current_blend = current_blend.lerp(target_blend, blend_speed * delta)
	
	# Update animation tree
	var playback := animation_tree.get("parameters/playback") as AnimationNodeStateMachinePlayback
	if not playback:
		return
	
	if not is_grounded:
		if velocity.y > 0:
			playback.travel("Jump")
		else:
			playback.travel("Jump_Idle")  # Falling
	elif is_crouching:
		if speed < 0.1:
			playback.travel("Crouch_Idle")
		else:
			playback.travel("Crouch_Walk_F")
	else:
		if speed < 0.1:
			playback.travel("Idle")
		else:
			# Choose locomotion animation based on speed
			if is_sprinting and speed > jog_speed:
				playback.travel("Sprint_F")
			elif speed > walk_speed:
				playback.travel("Jog_F")
			else:
				playback.travel("Walk_F")
	
	# Set blend space position if using BlendSpace2D
	animation_tree.set("parameters/Locomotion/blend_position", current_blend)

func set_crouch(crouch: bool) -> void:
	is_crouching = crouch

func set_sprint(sprint: bool) -> void:
	is_sprinting = sprint

func play_action(action_name: String) -> void:
	## Play a one-shot action animation (attack, interact, etc.)
	var playback := animation_tree.get("parameters/playback") as AnimationNodeStateMachinePlayback
	if playback:
		playback.travel(action_name)
"#;

/// Generate AnimationTree scene resource
pub fn generate_animation_tree_tscn(animations: &[String]) -> String {
    let mut tscn = String::from(r#"[gd_scene load_steps=2 format=3]

[ext_resource type="Script" path="res://scripts/locomotion_blend_tree.gd" id="1"]

[sub_resource type="AnimationNodeStateMachine" id="AnimationNodeStateMachine_1"]

[sub_resource type="AnimationNodeBlendTree" id="AnimationNodeBlendTree_1"]
graph_offset = Vector2(-200, 0)
"#);

    // Add animation node references
    for (i, anim) in animations.iter().enumerate() {
        tscn.push_str(&format!(
            r#"
[sub_resource type="AnimationNodeAnimation" id="anim_{i}"]
animation = &"{anim}"
"#,
            i = i,
            anim = anim
        ));
    }

    tscn.push_str(r#"
[node name="AnimationTree" type="AnimationTree"]
script = ExtResource("1")
tree_root = SubResource("AnimationNodeStateMachine_1")
anim_player = NodePath("../AnimationPlayer")
"#);

    tscn
}
