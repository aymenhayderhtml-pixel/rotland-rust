# Animal contract: the pet dog (V11-08) and the wild animals (V12-01)

Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

Art (Blender) and code (Godot C#) both build against this file. **Don't change names without updating this contract and `AnimalRigContract`.**

## Species at a glance
| Model (`assets/models/animals/`) | Shoulder | Notes | Clips | Budget |
|---|---|---|---|---|
| `dog_tan.glb`, `dog_dark.glb` | about 0.55 m | Pet dog, two coats | 8 (below) | 1200 |
| `deer.glb` | 1.0 m | Doe, no antlers | `idle`, `graze`, `walk`, `run`, `alert`, `die` | 1200 |
| `deer_buck.glb` | 1.0 m | Same rig and clips as the doe, plus antlers (about 1.67 m to the antler tips) | as the doe | 1200 |
| `rabbit.glb` | about 0.20 m | Ears up to about 0.41 m | `idle`, `graze`, `walk`, `run`, `alert`, `die` | 400 |
| `boar.glb` | 0.80 m | Tusks, bristle ridge on the back | `idle`, `graze`, `walk`, `run`, `charge`, `alert`, `attack`, `die` | 1000 |

The wild animals (V12-01) have the same bone names as the dog, so one code path can play any species. Only sizes and clip lists differ. Their code (hunting, AI) comes later.

## Files
| File | What |
|---|---|
| `tools/blender/animals/dog_rig.py`, `dog_mesh.py`, `dog_poses.py` | The dog: armature, body parts, clip tables |
| `tools/blender/animals/build_dog.py` | Builds both dog coats. **It rewrites the dog glbs**, so only run it when the dog changes. `blender -b --python tools/blender/animals/build_dog.py` |
| `tools/blender/animals/wild_species.py` | Wild data: bones, legs, body parts and budget for each species (one dict per species) |
| `tools/blender/animals/wild_rig.py`, `wild_mesh.py`, `wild_poses.py` | The wild rig, rigid box mesh and clips (per species style: deer, rabbit, boar) |
| `tools/blender/animals/build_wild.py` | Builds all four wild models. `blender -b --python tools/blender/animals/build_wild.py`, or `-- rabbit boar` for some. Prints a ground check per clip |
| `assets/models/animals/*.glb` | The models above (the glbs embed their palette textures) |
| `assets/models/_previews/<name>.png` | Rest pose |
| `assets/models/_previews/<name>_<clip>.png` | Contact sheet: 8 frames across each clip, left to right |
| `tools/blender/icons/item_meshes.py` | `hide` and `antler` icon meshes for future drops (no item uses them yet, so no icon is rendered) |

## Size and budget
- Dog: shoulder about 0.55 m, length about 0.95 m (nose to tail tip), top of the head about 0.76 m. Budget **1200 triangles** (actual: 300).
- Wild animals: the sizes in the table above; the test ranges are in `AnimalRigContract`. Budgets: deer 1200 (actual 300, buck 372), rabbit 400 (actual 276), boar 1000 (actual 300).
- Standing on the ground: the lowest point is **y = 0** (the paws).

## Axes
- **Forward is −Z in Godot** (+Y in Blender). Right is +X, so left-side bones use `_L` and sit at −X. Up is +Y in Godot.
- The root is at the origin on the ground, under the middle of the body.

## Armature (exact bone names, underscores only)
```
root
├ hips
│ ├ spine ─ chest ─ neck ─ head ─┬ jaw
│ │       │                      ├ ear_L
│ │       ├ upper_FL ─ lower_FL ─ paw_FL     (front legs hang from the chest)
│ │       └ upper_FR ─ lower_FR ─ paw_FR      with the ears on the head
│ ├ upper_BL ─ lower_BL ─ paw_BL              (hind legs hang from the hips)
│ ├ upper_BR ─ lower_BR ─ paw_BR
│ ├ tail_1 ─ tail_2
```
Bones: `root`, `hips`, `spine`, `chest`, `neck`, `head`, `jaw`, `ear_L`, `ear_R`, `tail_1`, `tail_2`, and for each of `FL`, `FR`, `BL`, `BR`: `upper_<leg>`, `lower_<leg>`, `paw_<leg>`. That is 23 bones in all. Every wild species has all 23 (no extra bones were needed).

- Rest pose: standing, legs straight down. Each `paw_*` bone's head is at the ankle height (dog 0.06 m, deer 0.06 m, boar 0.05 m, rabbit 0.02 m), so the paws touch the ground.
- Bone rolls: legs and ears use roll (0,1,0) and the body bones use roll (0,0,1), so a positive X rotation swings a leg forward or lifts the nose. Ears turn the other way: to keep ears upright while the head pitches down, a rabbit's ears use a negative X (see `wild_poses.py`).

## Mesh rules
- One mesh, skinned to the armature with **rigid weights** (100% on exactly one bone per vertex). Blocky low-poly parts, flat-shaded, colours from the shared palette sheet B.
- Dog: `dog_tan` (tan coat, dark ears, nose and paws) and `dog_dark` (dark coat, black paws and nose). Both have a `dog_cream` chest bib.
- Wild: deer `deer_fur` with `deer_pale` rump, belly and tail, `bark_dark` nose and hooves. Rabbit `rabbit_fur` with `rabbit_pale` belly, tail and hind feet. Boar `boar_hide` with a `boar_bristle` back ridge and snout, `boar_tusk` tusks. Antlers are `antler`. All of these colours are on palette sheet B except `bark_dark` (sheet A).
- Wild parts are boxes: a box runs along its bone (head to tail), so a neck or leg follows the pose.

## Clips (exact names; in place, **no root motion**)
Dog:
| Clip | Loop | Length | Notes |
|---|---|---|---|
| `idle` | yes | 2.0 s | Breathing, two tail wags per loop |
| `walk` | yes | 0.8 s | Diagonal pairs (FL with BR, FR with BL) |
| `run` | yes | 0.45 s | Gallop: hind pair leads, front pair follows |
| `sit` | yes | 2.0 s | Haunches lowered, front legs straight |
| `lie` | yes | 2.0 s | Down on the belly, head low, paws on the ground |
| `bark` | no | 0.6 s | Jaw opens and closes twice, head jerks up |
| `bite` | no | 0.5 s | Lunge forward, jaw snaps shut |
| `die` | no | 1.2 s | Falls onto its side and stays there (holds the last pose) |

Wild animals (length in seconds):
| Clip | Deer (doe, buck) | Rabbit | Boar | Notes |
|---|---|---|---|---|
| `idle` | 2.0 loop | 2.0 loop | 2.0 loop | Breathing, a tail flick (rabbit: nose twitch) |
| `graze` | 3.0 loop | 2.0 loop | 3.0 loop | Head down (boar: snout to the ground, a side shove) |
| `walk` | 1.0 loop | 0.8 loop | 1.0 loop | Diagonal pairs (rabbit: a slow hop) |
| `run` | 0.6 loop | 0.4 loop | 0.6 loop | Gallop (rabbit: a hop cycle) |
| `charge` | | | 0.7 loop | Head low, a faster gallop (boar only) |
| `alert` | 1.5 once | 1.2 once | 1.5 once | Head up, ears up, then back to rest |
| `attack` | | | 0.6 once | Head swings side to side, tusks forward (boar only) |
| `die` | 1.2 once | 1.0 once | 1.2 once | Rolls onto its side and stays there (holds the last pose) |

- Keyed at 60 fps (`dog_poses.FPS`, `wild_poses.FPS`). Loop clips return to their first pose at the end (the first and last keys match), so there is no jump.
- **Floor rule:** a frame whose legs or head would dip below y = 0 is lifted by the body (the hips) only as far as needed, so no clip has a limb through the floor. `build_wild.py` prints the lowest point of each clip (min should be 0.000).
- Movement speed is the code's job: `walk` and `run` are time-scaled to the real speed, like the humanoid clips.

## Facing and checks
- The head is in front of the hips (head z < hips z in Godot). Shoulder (chest bone) heights: dog 0.45–0.62 m, deer 0.90–1.10 m, rabbit 0.16–0.26 m, boar 0.72–0.88 m. Paw bones at the ankle height above.
- No static collider (animals move under code, like characters). A `CollisionShape3D` is not expected.

## Self-test
`AnimalRigContract` (`scripts/Debug/Tests/AnimalRigContract.cs`) holds one table per species: size ranges, shoulder and paw heights, and the clip list with lengths. Every glb in `assets/models/animals` must be listed there (a missing or unknown file fails). Each model has a Skeleton3D with every bone above and an AnimationPlayer with the clips for its species; each clip has the length above; loop clips are seamless; the head faces −Z; the shoulder and paws sit at the heights above; the size is in range. The triangle budget is checked by `ModelsTriangleBudget` (`ModelTestUtil.Budgets`).
