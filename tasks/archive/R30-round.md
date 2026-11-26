# R30 round (lead file for the gate)

Files: rotland/src/app/*, rotland/src/hud/*, rotland/src/combat*.rs, rotland/src/main.rs, rotland/src/companion_view.rs, rotland/src/companion_model.rs, rotland/src/rescue_play.rs, rotland/src/streaming.rs, game/src/guild/companion/*, game/tests/all/*, engine/src/app.rs, engine/src/timing*.rs, engine/src/render/*, physics/src/world.rs, tools/blender/characters/poses_revive.py, tools/blender/characters/poses.py, assets/models/characters/*, tools/README_TOOLS.md, docs/PERF_R30.md, tasks/R30-*
Rules: `tasks/R30-common.md` (lead section).

Tasks:
- Phase 1, at the same time:
  - `tasks/R30-01-test-flags.md`: test flags for downed / team / rescue screenshots
  - `tasks/R30-02-team-fixes.md`: rescued survivors start at their spot; teammates kneel to revive (new clip, lead rebuilds characters once)
  - `tasks/R30-03-stutter.md`: measure and fix the stutter (lead runs the game twice: before and after)
- Phase 2, after phase 1 is in:
  - `tasks/R30-04-split.md`: split the 5 biggest files (code moves only)

Run the gate as `tools/gate.sh --task tasks/R30-round.md`.

## Lead notes
- **R30-01 (test flags):** Done. Added `--down-at`, `--team`, `--tp-rescue`, `--hold-e` in `rotland/src/app/test_flags.rs` (with unit tests), wired in `startup.rs`, `frame_actors.rs`, `state.rs`, `mod.rs`, `main.rs`, and documented in `tools/README_TOOLS.md`.
- **R30-02 (team fixes):** Done.
  - Rescued survivors now spawn at their rescue spot (`CompanionSquad::set_spawn_position` in `game/src/guild/companion/squad.rs`, wired in `rescue_play.rs` and `frame_world.rs`, unit tests in `game/tests/all/companion_rescue_start.rs`).
  - Teammates kneel to revive using new shared clip `revive_kneel` (`tools/blender/characters/poses_revive.py`, `poses.py`). Rebuilt all 15 character models (`TRACEBACKS: 0`, `CHARACTERS: 15`) and verified `assets/models/_previews/npc_guard_revive_kneel.png` (knee and foot planted on the ground).
  - Note for Claude: `game/src/team_anim.rs` was not in `R30-round.md`'s base `Files:` scope, so instead of adding `CompanionAnim::Revive` (which would fail scope check), `TeamFigure.reviving: bool` triggers `revive_kneel` in `companion_model.rs`.
- **R30-03 (stutter):** Done. Fixed per-collider `update_broad_phase()` thrashing during chunk streaming (`physics/src/world.rs` deferred methods + `rotland/src/streaming.rs` batching + `MAX_APPLIED_PER_FRAME` 3 -> 1).
  - Before (`--autowalk --seconds=40`): `avgFps=117.7`, `p95FrameMs=14.54`, `worst1PercentFps=31.9`, 148 spike rows (>20ms), 26 frames >33ms, worst frame `100.00ms` (frame 1) / `89.36ms` (frame 6), `world streaming` max `25.56ms`.
  - After (`--autowalk --seconds=40`): `avgFps=138.4`, `p95FrameMs=10.29`, `worst1PercentFps=54.2`, 39 spike rows (>20ms), 10 frames >33ms, worst frame `152.54ms` (frame 1 startup upload) / `64.68ms` post-startup (frame 393 `physics=61.28ms`), `world streaming` max `2.80ms`.
- **R30-04 (file split):** Done. Split `frame_sim.rs`, `hud/mod.rs`, `frame_actors.rs`, `combat.rs`, and `frame_team.rs` into topic files under ~350 lines each with no behavior change.
- **Final gate (last 5 lines):**
  ```
  test: PASS (1068 passed)
  scope: PASS (46 changed paths)
  numbers: PASS (1 checked)
  unused: PASS
  GATE: PASS
  ```
- **For Claude to check:** Run screenshot checks with `--down-at`, `--team`, `--tp-rescue`, `--hold-e` to verify downed/revive/rescue visuals; remaining `surface acquire` (Mailbox + Poll in `engine/src/gpu.rs`) and `frame_sim` physics spikes are documented in `docs/PERF_R30.md`.

## Gate report
- date: 2026-11-25
- base: main
- files changed (46): assets/models/characters/human_player.glb, assets/models/characters/npc_banker.glb, assets/models/characters/npc_guard.glb, assets/models/characters/npc_mechanic.glb, assets/models/characters/npc_scout.glb, assets/models/characters/npc_survivor.glb, assets/models/characters/npc_trader.glb, assets/models/characters/npc_villager.glb, docs/PERF_R30.md, game/src/guild/companion/squad.rs, game/tests/all/companion_rescue_start.rs, game/tests/all/main.rs, physics/src/world.rs, rotland/src/app/frame_actors_combat.rs, rotland/src/app/frame_actors_move.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/frame_sim_player.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_sim_zombies.rs, rotland/src/app/frame_team_revive.rs, rotland/src/app/frame_team.rs, rotland/src/app/frame_team_view.rs, rotland/src/app/frame_world.rs, rotland/src/app/mod.rs, rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/test_flags.rs, rotland/src/combat_bash.rs, rotland/src/combat_damage.rs, rotland/src/combat.rs, rotland/src/companion_model.rs, rotland/src/hud/draw.rs, rotland/src/hud/manager.rs, rotland/src/hud/mod.rs, rotland/src/hud/state.rs, rotland/src/main.rs, rotland/src/rescue_play.rs, rotland/src/streaming.rs, tasks/R30-01-test-flags.md, tasks/R30-02-team-fixes.md, tasks/R30-03-stutter.md, tasks/R30-round.md, tools/blender/characters/poses.py, tools/blender/characters/poses_revive.py, tools/README_TOOLS.md
- tests passed: 1068
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
