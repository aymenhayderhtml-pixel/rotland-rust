# R28-03: screenshot tool for animations

Files: rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/anim_shot*.rs, rotland/src/app/mod.rs, rotland/src/app/frame_actors.rs, rotland/src/app/render_frame.rs, tools/README_TOOLS.md, tasks/R28-03-pose-shots.md
Rules: `tasks/R27-common.md`.

**Goal:** Claude needs to check animations with a screenshot, without playing. Add dev flags used with `--shot=<png>`:
- `--pose=<clip>[@<seconds>]`: the player's animator plays that clip on the right layer (upper-body clips like `block_2h`, `melee_2h_b`, `eat` as an action/held pose; full-body clips like deaths as full body), frozen at that time (default: the middle of the clip), for the screenshot.
- `--weapon=<id>`: put that weapon in the player's hand at start (bat, axe, pistol, rifle...).
- `--cam-yaw=<degrees>`: orbit the third-person camera around the player (0 = behind, 90 = right side, 180 = front) so the pose can be seen from any side.
- `--block`: the player holds the melee block (right mouse held), and `--sprint` with `--autowalk`: the player sprints forward.
All flags are off by default and change nothing in normal play. Put the parsing and the "what to apply" logic in a small module with unit tests; wire it into startup and the frame. Document the flags with one example each in `tools/README_TOOLS.md`. Check it yourself with the gate only (don't run the game).

## Worker notes
- **What changed:**
  - Added `rotland/src/app/anim_shot.rs` with CLI parsing (`AnimShotState::from_args`), clip layer classification (`is_full_body_clip` checking whether any lower-body bone moves away from rest pose), timestamp resolution (defaults to `duration * 0.5`, clamped to `0..=duration`), pose freezing (`apply_pose`, `apply_pending_pose`), weapon lookup (`weapon_hand_change`), camera yaw orbit (`orbit_camera_yaw`), action overrides (`apply_actions`), and 6 unit tests.
  - Added `pub mod anim_shot` in `rotland/src/app/mod.rs` and `anim_shot: AnimShotState` in `SessionState` (`rotland/src/app/state.rs`).
  - Wired flag parsing and `--weapon` / `--cam-yaw` third-person setup into `rotland/src/app/startup.rs`.
  - Wired `--block`, `--sprint` (with `--autowalk`), `--cam-yaw` camera orbiting, and `--pose` guard into `rotland/src/app/frame_actors.rs`.
  - Wired one-shot `--pose` application (after `pending_weapon_model`) and `dt = 0.0` freeze + posture/IK skip into `rotland/src/app/render_frame.rs`.
  - Documented all five flags with examples in `tools/README_TOOLS.md`.
- **How to see it in the game:**
  - Pass any of `--pose=<clip>[@<seconds>]`, `--weapon=<id>`, `--cam-yaw=<degrees>`, `--block`, or `--autowalk --sprint` alongside `--shot=/tmp/<name>.png --save-dir=/tmp/scratch` (see `tools/README_TOOLS.md` for one example per flag).
- **Wiring notes / anything unsure:**
  - Upper-body `--pose` clips are played via `play_full_body` with `LayerId::FullBody.mask = MaskId::Upper` and `fade_in = 0 / fade_out = 0 / hold_last = true` instead of `play_action`, so any timestamp (`0.0` to `duration`) renders at full weight `1.0` while preserving `idle` lower-body locomotion and keeping `melee_2h_b` / `block_2h` / `eat` faithful to the clip.
  - Not run live (gate-only per task instructions); verified via unit tests in `rotland/src/app/anim_shot.rs`.
- **Final gate output (last 5 lines):**
```
test: PASS (1039 passed)
scope: PASS (7 changed paths)
numbers: PASS (0 checked)
unused: PASS
GATE: PASS
```

## Gate report
- date: 2026-11-24
- base: main
- files changed (7): rotland/src/app/anim_shot.rs, rotland/src/app/frame_actors.rs, rotland/src/app/mod.rs, rotland/src/app/render_frame.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, tools/README_TOOLS.md
- tests passed: 1039
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
