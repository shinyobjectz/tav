# Quaternius Character Package (Pro)

CC0-licensed animated humanoid with **120+ animations**. [Quaternius UAL](https://quaternius.itch.io/universal-animation-library).

## Animation List (Godot prefix: `UAL1_Source/`)

### Locomotion
- `Idle_Loop`, `Idle_LookAround_Loop`, `Idle_Tired_Loop`
- `Walk_Loop`, `Walk_Formal_Loop`
- `Jog_Fwd_Loop`, `Jog_Bwd_Loop`, `Jog_Left_Loop`, `Jog_Right_Loop`
- `Jog_Fwd_L_Loop`, `Jog_Fwd_R_Loop`, `Jog_Bwd_L_Loop`, `Jog_Bwd_R_Loop`
- `Jog_Fwd_LeanL_Loop`, `Jog_Fwd_LeanR_Loop`
- `Sprint_Loop`, `Sprint_Enter`, `Sprint_Exit`
- `Turn90_L`, `Turn90_R`

### Crouch
- `Crouch_Idle_Loop`, `Crouch_Enter`, `Crouch_Exit`
- `Crouch_Fwd_Loop`, `Crouch_Bwd_Loop`, `Crouch_Left_Loop`, `Crouch_Right_Loop`
- `Crouch_Fwd_L_Loop`, `Crouch_Fwd_R_Loop`, `Crouch_Bwd_L_Loop`, `Crouch_Bwd_R_Loop`

### Crawl
- `Crawl_Idle_Loop`, `Crawl_Enter`, `Crawl_Exit`
- `Crawl_Fwd_Loop`, `Crawl_Bwd_Loop`, `Crawl_Left_Loop`, `Crawl_Right_Loop`

### Jump & Dodge
- `Jump_Start`, `Jump_Loop`, `Jump_Land`
- `BackFlip`
- `Dodge_Left`, `Dodge_Left_RM` (root motion)
- `Dodge_Right`, `Dodge_Right_RM` (root motion)
- `Roll`, `Roll_RM` (root motion)

### Climbing
- `Climb_Enter`, `Climb_Exit`, `Climb_Idle_Loop`
- `Climb_Up_Loop`, `Climb_Down_Loop`
- `Climb_Left_Loop`, `Climb_Left_RM_Loop` (root motion)
- `Climb_Right_Loop`, `Climb_Right_RM_Loop` (root motion)
- `ClimbLedge`, `ClimbLedge_RM` (root motion)

### Combat - Melee
- `Punch_Jab`, `Punch_Cross`, `Kick`
- `PunchKick_Enter`, `PunchKick_Exit`
- `Sword_Idle`, `Sword_Enter`, `Sword_Exit`
- `Sword_Attack`, `Sword_Attack_RM`, `Sword_Attack_Standing`

### Combat - Ranged
- `Pistol_Idle_Loop`, `Pistol_Aim_Neutral`, `Pistol_Aim_Up`, `Pistol_Aim_Down`
- `Pistol_Shoot`, `Pistol_Reload`

### Magic
- `Spell_Simple_Enter`, `Spell_Simple_Idle_Loop`, `Spell_Simple_Exit`, `Spell_Simple_Shoot`
- `Spell_Double_Enter`, `Spell_Double_Idle_Loop`, `Spell_Double_Exit`, `Spell_Double_Shoot_Loop`

### Damage
- `Hit_Chest`, `Hit_Head`, `Hit_Stomach`, `Hit_Shoulder_L`, `Hit_Shoulder_R`
- `Death01`, `Death02`

### Sitting
- `Sitting_Enter`, `Sitting_Exit`
- `Sitting_Idle_Loop`, `Sitting_Idle02_Loop`, `Sitting_Idle03_Loop`
- `Sitting_Talking_Loop`, `Sitting_Nodding_Loop`
- `GroundSit_Enter`, `GroundSit_Exit`, `GroundSit_Idle_Loop`

### Actions
- `Interact`, `PickUp_Table`, `PickUp_Kneeling`
- `Push_Enter`, `Push_Loop`, `Push_Exit`
- `Drink`, `Fixing_Kneeling`
- `Counter_Enter`, `Counter_Idle_Loop`, `Counter_Exit`, `Counter_Give`, `Counter_Show`, `Counter_Angry`

### Swimming
- `Swim_Idle_Loop`, `Swim_Fwd_Loop`

### Other
- `Idle_Talking_Loop`, `Idle_Torch_Loop`
- `Idle_Paper`, `Idle_Rock`, `Idle_Scissors` (rock-paper-scissors!)
- `Dance_Loop`, `Celebration`, `Crying`
- `Driving_Loop`
- `A_TPose`

## Root Motion Animations (`_RM` suffix)
These have movement baked in:
- `Dodge_Left_RM`, `Dodge_Right_RM`
- `Roll_RM`
- `Climb_Left_RM_Loop`, `Climb_Right_RM_Loop`
- `ClimbLedge_RM`
- `Sword_Attack_RM`

## Usage
Auto-downloaded to `assets/characters/character.glb` for 3D projects.

## License
**CC0 (Public Domain)** - [Quaternius](https://quaternius.com)

## R2
`https://pub-b3ceaf5076804d56bc32fe9d83e9a3a9.r2.dev/quaternius-character.zip`
