# R10-03: Two-handed bat swing that looks natural (GitHub #3) — PHASE A

Read `tasks/R10-common.md` first, then `docs/RIG_CONTRACT.md`. See the screenshot: `gh issue view 3`.

Godot reference: `../zombi servival/scripts/Characters/HumanoidAnimTree.cs`, `HumanoidVisual.cs`, `scripts/Player/PlayerAnimator.cs`, `scripts/Combat/WeaponMounts.cs`, `scripts/Combat/MeleeHandler.cs`. Port how Godot poses the swing (which joints rotate, by how much, over what time).

## You own
- `humanoid/src/**` (animator, actions/poses, weapon grip)
- `humanoid/tests/**`
- `data/weapons.json` — only to add a grip flag per weapon if needed (see below)

## Problems to fix
1. Only one hand holds the bat; the other arm points straight up. → Two-handed weapons need **both hands on the handle**: the left hand follows a second grip point on the weapon (simple 2-bone IK for the left arm: upper arm + forearm, elbow bending down/out).
2. The whole body bends backward like a bow. → The swing should **twist at the waist** (spine yaw), with only a small forward lean on the hit. No backward arch beyond what Godot does.
3. The head tips sideways. → Keep the head upright and facing the target: counter-rotate the head/neck against the spine twist.

## Data
- Which weapons are two-handed? Check Godot (`WeaponDef.cs` / `weapons.json` in the Godot repo) for an existing flag (e.g. `twoHanded`, `grip`, `offHand`). If Godot has one, port its key and values into our `data/weapons.json`. If Godot has none, add `"twoHanded": true` to bat, axe, katana and the long guns' **aim** pose only if Godot holds them with two hands, and write that choice in your notes.
- The left-hand grip point: an offset along the weapon's handle. Use the weapon's existing grip data if there is any; otherwise a per-weapon `"offHandGrip": [x,y,z]` in weapons.json (numbers in your notes with how you picked them).

## Tests (`humanoid/tests/`)
- At the swing's start, middle and hit time: left-hand joint world position is within 6 cm of the bat's off-hand grip point.
- Spine pitch never leans backward past Godot's value (quote it); spine yaw reaches Godot's twist at the hit time.
- Head roll stays within ±5° through the whole swing.
- One-handed weapons (knife, pistol one-handed if Godot does so) are unchanged: the left arm pose equals today's.

## Wiring for Claude
Say whether `rotland/src/player_visual.rs` needs to pass anything new (e.g. the weapon's off-hand grip) into the animator, and the exact call.

## Worker notes
