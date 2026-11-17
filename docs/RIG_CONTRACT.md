# Rig contract: humanoid characters (player, zombies, NPCs)

Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

Art (Blender scripts) and code (Godot C#) both build against this file. **Don't change names without updating this contract and its self-test.**

## Files
| File | What |
|---|---|
| `tools/blender/characters/humanoid.py` | Builds the mesh, armature and clips from parameters (variant = palette cells + proportions) |
| `tools/blender/characters/poses.py` | **Pose tables**: per clip, keyframes as `{bone: (rx, ry, rz)}` in degrees. A bad clip is a data fix |
| `assets/models/characters/human_player.glb` | Player |
| `assets/models/characters/zombie_01.glb` … `zombie_03.glb` | Zombie variants (same rig, zombie palette, slightly different proportions) |
| `assets/models/characters/npc_trader.glb`, `npc_guard.glb`, `npc_survivor.glb` | Settlement NPCs (V1-15): same rig and player clips |
| `assets/models/characters/npc_villager.glb` | Villager NPC look (V11-03): narrower, long hair panel; same rig and player clips |
| `tools/blender/characters/poses_camp.py` | Phase-3 camp clips (V11-03), player and NPCs only |
| `assets/models/gear/backpack_small.glb`, `backpack_large.glb` | Backpacks worn on the chest (V11-03), see "Gear attach" below |
| `assets/models/_previews/<name>_<clip>.png` | **Contact sheet** per clip: 8 frames in a row, 3/4 view |

## Mesh rules
- **One mesh** skinned to one armature. **Rigid weights:** every vertex is 100% on exactly one bone (part-based body: no weight painting).
- Low-poly, flat-shaded, colors from the shared palette texture via UVs (same system as the props). Budget **≤ 1500 triangles** per character.
- Height **1.80 m** (top of head), feet at origin (z = 0 in Blender, y = 0 in Godot). Shoulder width about 0.46 m, head about 0.24 m.
- **Facing:** the character faces **+Y in Blender**, which is **−Z in Godot** (Godot forward). The character's right hand is at **+X**.
- Parts (minimum): head (with hair cap and nose/brow to read the face direction), neck, torso (shirt), hips (pants), upper/lower arms, hands, upper/lower legs, feet (shoes). Zombies: torn-shirt color patches, darker/greener skin, one variant with a missing-sleeve look.

## Armature (exact bone names, underscores only)
```
root                      (at origin, on the ground; never animated except in die)
└ hips
  ├ spine
  │ └ chest
  │   ├ neck ─ head
  │   ├ shoulder_L ─ upper_arm_L ─ forearm_L ─ hand_L
  │   └ shoulder_R ─ upper_arm_R ─ forearm_R ─ hand_R
  ├ thigh_L ─ shin_L ─ foot_L ─ toe_L
  └ thigh_R ─ shin_R ─ foot_R ─ toe_R
```
- Rest pose = **A-pose** (arms about 30° down from horizontal), standing straight.
- `hand_R` is the **weapon socket**. In Godot, code adds a `BoneAttachment3D` on `hand_R`.

## Weapon grip convention (weapon .glb files)
- The origin is at the **grip center** (where the right hand holds it).
- The weapon points along **+Y in Blender** (−Z in Godot), with up = +Z in Blender.
- Two-handed weapons also have an empty named `grip_L` at the support-hand position (for future IK; optional in v1).
- A weapon can have a separate mesh node named `mag` (V12-04, AK-47 only): the reload's mag swap hides it through `WeaponVisuals.SetMagHidden`. Built with `Model.add_part("mag", bm)`.
- Player-only clips (V12-04: `rifle_auto_fire`, `rifle_reload_mag`, `gun_bash`) are in `poses_combat.PLAYER_ONLY_CLIPS`. Only `human_player.glb` has them; NPC and zombie glbs do not.

## Animation clips (exact names; in place, **no root motion**)
| Clip | Loop | Length | Phase | Notes |
|---|---|---|---|---|
| `idle` | yes | 2.0 s | 1 | breathing, slight sway |
| `walk` | yes | 1.0 s | 1 | about 1.6 m/s natural; code time-scales |
| `run` | yes | 0.7 s | 1 | about 4–7 m/s; code time-scales for sprint |
| `die` | no | 1.2 s | 1 | falls backward to the ground, root may rotate |
| `jump` | yes | 0.5 s | 2 | single pose (knees up), used while airborne |
| `crouch` | yes | 1.0 s | 2 | crouched idle pose; code plays it slower or faster for crouch-walk |
| `melee_1h` | no | 0.5 s | 2 | right-hand swing (knife, machete) |
| `melee_2h` | no | 0.7 s | 2 | two-hand swing (bat, axe) |
| `pistol_aim` | yes | 1.0 s | 2 | arm extended forward |
| `pistol_fire` | no | 0.15 s | 2 | small kick |
| `rifle_aim` | yes | 1.0 s | 2 | two-hand shouldered (rifle, shotgun) |
| `rifle_fire` | no | 0.2 s | 2 | kick |
| `rifle_auto_fire` | yes | 0.1 s | 2 (player only, V12-04) | AK-47 auto fire: a small shoulder jolt per round, loops while holding fire |
| `rifle_reload_mag` | no | 2.2 s | 2 (player only, V12-04) | AK-47 mag change: drops the empty mag, takes one from the belt, slaps it in, pulls the charging handle. The code hides the `mag` node from 0.7 s to 1.35 s |
| `gun_bash` | no | 0.6 s | 2 (player only, V12-04) | gun used as a club: pulled back, then butt-stroked forward. Hit at 0.30 s |
| `reload` | no | 1.4 s | 2 | generic |
| `hit` | no | 0.3 s | 2 | flinch |
| `zombie_shamble` | yes | 1.4 s | 2 | zombies only: arms forward, dragging walk |
| `zombie_attack` | no | 0.6 s | 2 | zombies only: lunge/bite |
| `sit` | yes | 2.0 s | 3 | player and NPCs. Chair sit (V11-06 fix): thighs horizontal, knees bent 90°, shins vertical, feet flat on the floor; hips about 0.5 m, slight breathing |
| `sit_ground` | yes | 2.0 s | 3 | player and NPCs. Sitting on the ground by a fire: hips about 0.04 m, knees up, one arm back to lean |
| `talk` | yes | 2.4 s | 3 | player and NPCs. Standing, right hand gestures, head nods |
| `listen` | yes | 2.4 s | 3 | player and NPCs. Standing, arms folded, head tilts |
| `wave` | no | 1.2 s | 3 | player and NPCs. One-shot: right arm up and out, forearm waves |
| `carry` | yes | 1.0 s | 3 | player and NPCs. Walk legs, both forearms forward at waist height |
| `chop` | yes | 1.2 s | 3 | player and NPCs. Two hands on an axe: raised overhead, chopped down |
| `zombie_stagger` | no | 0.8 s | Phase 3 (zombies) | zombies only. One-shot: knocked back, arms flail, one leg steps back, regains balance |
| `zombie_lurch` | yes | 1.6 s | Phase 3 (zombies) | zombies only. Uneven limping shamble (one stiff hitch leg, the other drags), a different gait from `zombie_shamble` |
| `zombie_crawl` | yes | 1.8 s | Phase 3 (zombies) | zombies only. On the ground on the hands, legs limp behind. Hips about 0.25 m (for zombies with destroyed legs) |
| `zombie_bang` | yes | 1.0 s | Phase 3 (zombies) | zombies only. Standing, fists pounding forward at chest height, taking turns (doors) |
| `zombie_eat` | yes | 2.0 s | Phase 3 (zombies) | zombies only. Kneeling, hunched forward, hands up to the mouth |
| `zombie_scream` | no | 1.4 s | Phase 3 (zombies) | zombies only. One-shot: head back, arms flung out wide (alerts a horde) |
| `zombie_runner_sprint` | yes | 0.6 s | Phase 3 (zombies) | zombies only. Fast wild run with both arms forward (runners) |

Phase 1 = the minimum to prove the pipeline (idle, walk, run, die). Phase 2 = everything else. Phase 3 = camp life (V11-03): the player and settlement NPCs get it, zombies do not. The Phase-3 zombie clips (V11-06) are for zombie_01..03 only.

Arm angles in the pose tables are measured from the torso: upper arm and forearm Z/X values add to the spine/chest lean, so a hunched zombie needs a bigger arm value to point the arm forward.

Arm Z is relative to the arms-down offset (`poses_shared.ARM_DOWN_OFFSET`). For the right arm a negative Z moves it out to the side; for the left arm a positive Z moves it out.

## Gear attach (backpacks, V11-03)
- `assets/models/gear/backpack_small.glb` (48 tris, budget 200) and `backpack_large.glb` (88 tris, budget 300). Static meshes, no skeleton, no colliders.
- **Origin = the attach point** on the upper back: the top front edge of the bag, which touches the back of the chest. The bag hangs below and behind the origin (Blender -Z and -Y). Its visible top is at the origin (the origin is at the top, not the ground).
- **Attach to the `chest` bone** with a `BoneAttachment3D`. At rest, the attach point is **0.14 m above the chest bone's head and 0.10 m behind it**: Godot offset `(0, 0.14, 0.10)` from the chest bone's head, in the rest pose's axes. The chest bone's head is at about 1.22 m (Blender) on the centre line.
- Previews: `assets/models/_previews/backpack_*_worn.png` show the rest-pose player with the bag on.

## Godot side (code lane)
- Character scene: `scenes/Characters/Humanoid.tscn` instances the glb. An `AnimationTree` with a locomotion BlendSpace1D (idle → walk → run by speed) plus a one-shot upper-body layer for actions (filter = spine and up).
- The player's `Body` gets the humanoid. In first person the body casts **shadows only**; there are no first-person arms in v1 (only the weapon view model).
- Zombies use the zombie glbs with `zombie_shamble` / `zombie_attack` / `die`, and the Phase-3 zombie clips (`zombie_lurch`, `zombie_crawl`, `zombie_bang`, `zombie_eat`, `zombie_scream`, `zombie_stagger`, `zombie_runner_sprint`) where the code picks them.

## Self-test (must stay green)
`RigContract` test: for every glb in `assets/models/characters/`:
- it has a Skeleton3D with **all** bone names above,
- it has an AnimationPlayer with at least the Phase-1 clips (the player and zombies get the Phase-2 clips once delivered; the player and NPCs also get the Phase-3 camp clips),
- the AABB height is 1.70–1.90 m,
- the `hand_R` global X is > 0 in rest pose and `toe_L` is in front (z < 0) of `foot_L` (facing check),
- it's within the triangle budget.
