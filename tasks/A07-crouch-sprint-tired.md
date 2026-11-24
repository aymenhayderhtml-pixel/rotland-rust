# A07 (DeepSeek): crouch walking, sprinting and a tired body

Files: humanoid/src/posture.rs, humanoid/src/posture_tests.rs, data/posture.json, tools/blender/characters/poses_posture.py, tasks/A07-crouch-sprint-tired.md, humanoid/src/animator.rs, humanoid/src/lib.rs, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, rotland/src/app/state.rs, rotland/src/app/render_frame.rs, rotland/src/player_visual.rs
Read: humanoid/src/animator.rs, humanoid/src/airborne.rs, humanoid/src/layers.rs, humanoid/src/stance.rs, game/src/player.rs

This is job 7 of 20 of the character animation system (plan: docs/ANIMATION_PLAN.md). Done so far: layers, weapon stances, aiming, directional legs, feet on the ground, jump/fall/land (see `airborne.rs` as the model to copy for structure and config loading).

**The problem:** crouching is one still `crouch` pose that slides along the ground. Sprinting looks like a normal run. Running out of stamina changes nothing in the body.

**After this job:**
- Crouch has a real crouch walk: legs step low, body low, blended by speed from the crouch idle, and it follows the side/back stepping of job 4 if that is easy (otherwise forward only, note it).
- Sprint looks different from a run: stronger forward lean, longer stride, arms pumping when unarmed; a held weapon is lowered/tucked to the chest while sprinting (no aiming pose).
- Tired: when stamina is low, the body hunches a little and the chest/shoulders heave (heavy breathing). After a sprint ends with low stamina, the idle becomes a short "catch breath" (hands toward knees) before normal idle. The effect fades in and out smoothly.
- **Animation only.** Do not change movement speed, stamina rules, controls or gameplay.

## New clips: `tools/blender/characters/poses_posture.py`
Same keyframe format as the other pose files; export `CLIPS`; merge into `PLAYER_ONLY_CLIPS` in poses.py.
- `crouch_walk` (loop, ~0.9 s): low stepping cycle.
- `sprint` (loop, ~0.55 s): long strides, strong lean, arm pump.
- `sprint_arms_weapon` (loop, upper body): weapon held low across the chest while sprinting.
- `tired_breath` (loop, ~1.4 s, upper body/spine/shoulders only): small hunch and chest heave, meant to be added or blended on top.
- `catch_breath` (no loop or loop, ~2.0 s): hands toward knees, bent over, breathing.

## `data/posture.json`
All numbers live here (thresholds for sprint speed, tired stamina fraction, blend rates, crouch walk speed range, catch-breath time, breathing weight). Pick sensible values and note them in Worker notes.

## Logic: `humanoid/src/posture.rs`
A small state + config (like `airborne.rs`): input = speed, crouching, sprinting flag, stamina fraction (0..1); output = clip choices and layer weights for crouch walk, sprint, tired breathing and catch breath. Keep `animator.rs` edits to a few lines (it is already 748 lines); put logic in posture.rs. Use existing layers (Crouch, Locomotion, Additive, Stance/Aim) — do not add a new layer kind unless truly needed.

**If the new input setter is never called, the animator must behave exactly as before** (job 1 golden test must still pass unchanged).

## Wiring
- `PlayerPose` (rotland/src/app/state.rs) gets `sprinting: bool` and `stamina_frac: f32`; find where it is built and fill them from the real player state (stamina is in game/src/player.rs).
- In render_frame.rs next to `pv.animator.set_move_velocity(...)`, call the new animator setter.
- Skip in first person if the body is not shown.

## Tests: `humanoid/src/posture_tests.rs`
Data parses; all 5 clips are defined in poses_posture.py (read as text); never-called changes nothing; crouch walk weight follows speed; sprint only above threshold; tired fades in below the stamina threshold and out above it (no flicker at the edge); catch breath only after a sprint ends tired, and ends after its time; stable and finite at dt 0 and 0.1.

Every new number in Rust code needs a source tag (`// data: posture.json key` or `// new: DECISIONS 2026-11-24`).

## Worker notes

**What plays where (no new layer kind was added; `layers.rs` is untouched):**
- `Crouch` layer: the full-body `sprint` cycle while sprinting (`sprint_weight`), then back to `crouch`. Crouch and sprint cannot happen together, so the layer is free.
- `Aim` layer (last of the body layers, so it wins over Stance): `catch_breath` (full body), `crouch_walk` (lower-body mask, so the crouch idle and weapon stance keep the upper body), and `sprint_arms_weapon` (upper-body mask) when a weapon is held. One slot, chosen by priority catch > crouch walk > sprint arms.
- `Additive` layer: `tired_breath`, upper-body mask, weight = tired × 0.85, faded out while `catch_breath` plays.
- `Locomotion`, `Stance` and the base pipeline are untouched. Aim is forced off while sprinting because the Aim layer is used for the sprint arms.
- `posture.rs` sets both `weight` and `target` (its own exponential smoothing) after `advance_stance`, so the stance code cannot reclaim the Aim layer each frame.

**Before `set_posture` is ever called:** `PostureState` has a `driven` flag; `tick`/`apply` do nothing until the setter runs, so every layer keeps its old values and the job-1 golden test passes unchanged. In first person `render_frame` does not call the setter at all (the body is shadows-only there).

**Numbers (`data/posture.json`) and why:**
- Crouch walk blends from 0.35 to 1.8 m/s (crouch speed is 2.0 m/s); playback = speed / 1.6, clamped 0.5..1.6, so the 0.9 s step cycle matches the ground speed.
- Sprint clip only at or above 6.5 m/s (the real sprint is 7.0 m/s) and only while the player's sprint flag is on.
- Tired: on at ≤0.35 stamina, off at ≥0.50, held in between (hysteresis, rate 3/s) so a wobbling stamina bar cannot flicker. Breathing weight 0.85.
- Catch breath starts only when a sprint ends at ≤0.45 stamina, runs 2.0 s with a 0.35 s fade in and a 0.6 s fade out, and is cancelled by sprinting, crouching or moving faster than 1.0 m/s.

**Known/optional:**
- Crouch walk is forward only. Job 4's side/back steps are applied to the base legs before the Crouch/Aim layers, so crouch walk overrides them; following strafes while crouched would need a lower-body blend in the pipeline.
- `tired_breath` is an additive delta, so its Blender contact sheet shows the rest (A-pose) arms: the preview renders the delta alone. The clip keys the upper arms at ∓58° to cancel `ARM_DOWN_OFFSET`, so the arm delta is zero and the arms do not move in game.
- Sprint uses its own 0.55 s cycle played at 1× on the Crouch layer (the Locomotion tempo only applies to the idle/walk/run blend), so the stride reads as a sprint rather than a fast run.
- Expected gate result: clippy/tests pass; the scope check fails only on the owner's 15 uncommitted files.

## Gate report
- date: 2026-11-24
- base: main
- files changed (26): assets/models/buildings/settlement.glb, assets/models/characters/human_player.glb, assets/models/_previews/car_hatchback_blue.png, assets/models/_previews/car_hatchback_blue_rear.png, assets/models/_previews/car_hatchback_blue_side.png, assets/models/_previews/car_hatchback_olive.png, assets/models/_previews/car_hatchback_olive_rear.png, assets/models/_previews/car_hatchback_olive_side.png, assets/models/_previews/car_hatchback_red.png, assets/models/_previews/car_hatchback_red_rear.png, assets/models/_previews/car_hatchback_red_side.png, assets/models/vehicles/car_hatchback_blue.glb, assets/models/vehicles/car_hatchback_olive.glb, assets/models/vehicles/car_hatchback_red.glb, data/posture.json, humanoid/src/animator.rs, humanoid/src/lib.rs, humanoid/src/posture.rs, humanoid/src/posture_tests.rs, rotland/src/app/render_frame.rs, rotland/src/app/state.rs, tasks/A07-crouch-sprint-tired.md, tools/blender/buildings/settlement.py, tools/blender/characters/poses_posture.py, tools/blender/characters/poses.py, tools/blender/vehicles/car_parts.py
- tests passed: 925
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
