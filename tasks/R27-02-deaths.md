# R27-02 (A16): player deaths by direction and cause

Files: game/src/death_anim*.rs, game/src/lib.rs, game/tests/all/death_anim_tests.rs, game/tests/all/main.rs, data/death_anim.json, humanoid/src/animator.rs, rotland/src/app/frame_sim.rs, rotland/src/combat.rs, tools/blender/characters/poses_death*.py, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, assets/models/_previews/human_player_*, tasks/R27-02-deaths.md
Rules: `tasks/R27-common.md`.

**Goal (animation only):** today the player always plays one `die` clip (`animator.play_die()`, called in `rotland/src/app/frame_sim.rs` on `ev.died`). Pick the death clip by cause and direction: killed by a zombie from the front → fall backward; from behind → fall forward; from the side → twist and fall sideways; drowned → slow sink/limp; other → current `die`. The final pose holds until respawn (`reset_alive`). Downed/revive is NOT part of this job.
- Reuse: direction from `game::hit_side` (game/src/hit_react.rs, A14) and the attacker position already found in frame_sim (`hit_from`); zombies' directional deaths (A17: `game/src/zombie/anim_fade.rs`, `tools/blender/characters/poses_zombie_v2.py`, `death_clip` in rotland/src/zombies) show how a whole-body fall is keyed. Check `poses_hurt.py` / `poses_wound.py` for player death clips that already exist.
- API: `animator.play_die_clip(name)` (or extend `play_die`) so the chosen clip plays once and holds the last frame.
- Clip names, speeds and the cause → clip table in `data/death_anim.json`.
- Tests: config parses; each side and cause picks the right clip; unknown → `die`.

## Worker notes

- **What changed:**
  - Added `data/death_anim.json` with clip names and speeds for `default` (`die`), `causes.drowned` (`death_drown`), and `zombieBySide` (`front` → `death_back`, `back` → `death_forward`, `left` → `death_right`, `right` → `death_left`).
  - Added `game/src/death_anim.rs` (`DeathCause`, `DeathClipSpec`, `ZombieDeathClips`, `DeathAnimConfig`, `DeathChoice`) reusing `game::hit_side`, exported via `game/src/lib.rs`, with unit tests in `game/tests/all/death_anim_tests.rs` (registered in `game/tests/all/main.rs`).
  - Extended `HumanoidAnimator` in `humanoid/src/animator.rs` with `play_die_clip(&mut self, clip: &str, speed: f32)` (falls back to `"die"` if a clip is missing, holds last frame) and tracked `death_clip` so `reset_alive()` clears any death clip on respawn.
  - Wired `rotland/src/app/frame_sim.rs` to pick directional zombie death clips (using `actors.player`, `player_yaw`, and `hit_from`) and `DeathCause::Drowned` when drowning.
  - Added `death_drown` clip in `tools/blender/characters/poses_death.py`, registered in `tools/blender/characters/poses.py` (`PLAYER_ONLY_CLIPS`), and rebuilt `assets/models/characters/human_player.glb` via Blender (`3030 tris OK`, preview `assets/models/_previews/human_player_death_drown.png`).
- **How to see it in the game:**
  - Let a zombie kill the player from the front (falls backward), behind (falls forward), or left/right (twists and falls away); or exhaust stamina in deep lake water to drown (slowly sinks limp). The final pose holds until respawn.
- **Notes / unsure:**
  - None. `assets/models.pack` was not rebaked by this lane (outside `Files:`); `baker` will pick up `human_player.glb` on next pack bake.

## Gate report
- date: 2026-11-24
- base: main
- files changed (11): assets/models/characters/human_player.glb, data/death_anim.json, game/src/death_anim.rs, game/src/lib.rs, game/tests/all/death_anim_tests.rs, game/tests/all/main.rs, humanoid/src/animator.rs, rotland/src/app/frame_sim.rs, tasks/R27-02-deaths.md, tools/blender/characters/poses_death.py, tools/blender/characters/poses.py
- tests passed: 1019
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
