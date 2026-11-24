# R27-01 (A15): wounded body language

Files: humanoid/src/wound_anim*.rs, humanoid/src/lib.rs, humanoid/src/animator.rs, data/wound_anim.json, rotland/src/app/render_frame.rs, tools/blender/characters/poses_wound*.py, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, assets/models/_previews/human_player_*, tasks/R27-01-wounds.md
Rules: `tasks/R27-common.md`.

**Goal (animation only):** when the player is hurt the body shows it. Below a health threshold the player limps (legs), holds a hand on the side when not using the arms, and the posture droops at very low health; it all fades back smoothly as health recovers. Thresholds, weights and fade rates live in `data/wound_anim.json`.
- Study first: `humanoid/src/posture.rs` (A07 tired body: copy its style), `humanoid/src/action_anim.rs` (config + small state), `humanoid/src/animator.rs` (layers), clips already in `tools/blender/characters/poses_wound.py` (e.g. `limp_walk`, `hold_side`: check names and quality on their preview PNGs; improve a weak one in a new `poses_wound_v2.py` merged after the old one).
- API: one new call `animator.set_health(frac: f32)` fed in `rotland/src/app/render_frame.rs` next to `set_posture` (find the player's health fraction; `actors.combat` has `health`). If `set_health` is never called, the animator behaves exactly as before.
- Wounds never fight with: sprint, swimming, ladder, car, full-body clips, melee swings/blocking, dying (fade out the wound layer there).
- Tests: config parses and equals Default; weights 0 at full health, rise below thresholds, smooth over time, 0 while dying/swimming.

## Gate report
- date: 2026-11-24
- base: main
- files changed (7): data/wound_anim.json, humanoid/src/animator.rs, humanoid/src/lib.rs, humanoid/src/wound_anim.rs, humanoid/src/wound_anim_tests.rs, rotland/src/app/render_frame.rs, tasks/R27-01-wounds.md
- tests passed: 1020
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS

## Worker notes
- **Files changed:**
  - `data/wound_anim.json`: thresholds (`limpThreshold`, `holdSideThreshold`, `droopThreshold`), max weights, limp speed ramp, and blend/fade-out rates.
  - `humanoid/src/wound_anim.rs` & `humanoid/src/wound_anim_tests.rs`: `WoundAnimConfig`, `WoundContext`, `WoundAnimState` (blending `limp_walk_left` on `MaskId::Lower`, `wounded_idle` on `MaskId::Spine`, and `hold_side` on `MaskId::LeftArm`), and unit tests.
  - `humanoid/src/lib.rs` & `humanoid/src/animator.rs`: added `HumanoidAnimator::set_health(frac: f32)` and `wound_state()`, ticked and applied wound blending in `update()` and `recompute_skin()`, suppressing wounds during sprint, crouch, swim, ladder, car, full-body clips, arm actions/aiming/blocking, and dying.
  - `rotland/src/app/render_frame.rs`: fed `actors.combat` health fraction to `pv.animator.set_health(health_frac)` next to `set_posture`.
- **How to see it in the game:** Switch to third-person view and take damage below 55% health while walking to see the limp, below 45% (when not aiming/swinging/holding a 2H weapon) to see the left hand held to the side, and below 25% to see the spine droop; heal back up to watch it fade out smoothly.
- **Notes:** Existing clips in `tools/blender/characters/poses_wound.py` (`limp_walk_left`, `hold_side`, `wounded_idle`) were already well-formed and registered in `poses.py`, so no Blender clip changes were needed. Preview PNGs for `human_player` were not present in `assets/models/_previews/` in this worktree, so visual inspection in-game was not run.
