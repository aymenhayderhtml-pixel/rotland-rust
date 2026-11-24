# R27-01 (A15): wounded body language

Files: humanoid/src/wound_anim*.rs, humanoid/src/lib.rs, humanoid/src/animator.rs, data/wound_anim.json, rotland/src/app/render_frame.rs, tools/blender/characters/poses_wound*.py, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, assets/models/_previews/human_player_*, tasks/R27-01-wounds.md
Rules: `tasks/R27-common.md`.

**Goal (animation only):** when the player is hurt the body shows it. Below a health threshold the player limps (legs), holds a hand on the side when not using the arms, and the posture droops at very low health; it all fades back smoothly as health recovers. Thresholds, weights and fade rates live in `data/wound_anim.json`.
- Study first: `humanoid/src/posture.rs` (A07 tired body: copy its style), `humanoid/src/action_anim.rs` (config + small state), `humanoid/src/animator.rs` (layers), clips already in `tools/blender/characters/poses_wound.py` (e.g. `limp_walk`, `hold_side`: check names and quality on their preview PNGs; improve a weak one in a new `poses_wound_v2.py` merged after the old one).
- API: one new call `animator.set_health(frac: f32)` fed in `rotland/src/app/render_frame.rs` next to `set_posture` (find the player's health fraction; `actors.combat` has `health`). If `set_health` is never called, the animator behaves exactly as before.
- Wounds never fight with: sprint, swimming, ladder, car, full-body clips, melee swings/blocking, dying (fade out the wound layer there).
- Tests: config parses and equals Default; weights 0 at full health, rise below thresholds, smooth over time, 0 while dying/swimming.
