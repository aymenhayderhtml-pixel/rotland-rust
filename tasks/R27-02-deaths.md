# R27-02 (A16): player deaths by direction and cause

Files: game/src/death_anim*.rs, game/src/lib.rs, game/tests/all/death_anim_tests.rs, game/tests/all/main.rs, data/death_anim.json, humanoid/src/animator.rs, rotland/src/app/frame_sim.rs, rotland/src/combat.rs, tools/blender/characters/poses_death*.py, tools/blender/characters/poses.py, assets/models/characters/human_player.glb, assets/models/_previews/human_player_*, tasks/R27-02-deaths.md
Rules: `tasks/R27-common.md`.

**Goal (animation only):** today the player always plays one `die` clip (`animator.play_die()`, called in `rotland/src/app/frame_sim.rs` on `ev.died`). Pick the death clip by cause and direction: killed by a zombie from the front → fall backward; from behind → fall forward; from the side → twist and fall sideways; drowned → slow sink/limp; other → current `die`. The final pose holds until respawn (`reset_alive`). Downed/revive is NOT part of this job.
- Reuse: direction from `game::hit_side` (game/src/hit_react.rs, A14) and the attacker position already found in frame_sim (`hit_from`); zombies' directional deaths (A17: `game/src/zombie/anim_fade.rs`, `tools/blender/characters/poses_zombie_v2.py`, `death_clip` in rotland/src/zombies) show how a whole-body fall is keyed. Check `poses_hurt.py` / `poses_wound.py` for player death clips that already exist.
- API: `animator.play_die_clip(name)` (or extend `play_die`) so the chosen clip plays once and holds the last frame.
- Clip names, speeds and the cause → clip table in `data/death_anim.json`.
- Tests: config parses; each side and cause picks the right clip; unknown → `die`.
