# R29 round (lead file for the gate)

Files: game/src/downed.rs, data/downed.json, game/tests/all/*, rotland/src/combat.rs, rotland/src/downed_play.rs, rotland/src/app/*, rotland/src/hud/*, game/src/rescue.rs, game/src/rescue_place.rs, data/rescue.json, rotland/src/rescue_play.rs, rotland/src/rescue_view.rs, game/src/guild/save.rs, game/src/guild/recruits.rs, game/src/lib.rs, rotland/src/main.rs, tools/blender/buildings/settlement_parts.py, assets/models/buildings/settlement.glb, tasks/R29-*
Rules: `tasks/R29-common.md` (lead section).

Tasks (run at the same time; they don't share files except the lead's shared files):
- `tasks/R29-01-downed.md`: knocked down, crawl, revive
- `tasks/R29-02-rescue.md`: rescue survivors in the world
- `tasks/R29-03-tower.md`: solid guard towers (issue 003)

Run the gate as `tools/gate.sh --task tasks/R29-round.md`.

## Lead notes
- **R29-01 (downed, crawl, revive):** Done. Pure logic in `game/src/downed.rs` + `data/downed.json` + `game/tests/all/downed.rs`; wired into `rotland/src/downed_play.rs`, `combat.rs`, `frame_actors.rs`, `frame_team.rs`, `hud/downed.rs`, `frame_input.rs`, `frame_world.rs`, `frame_water.rs`, `frame_sim.rs`, `frame_ui.rs`, `start_world.rs`, `state.rs`, `main.rs`.
- **R29-02 (rescue survivors):** Done. Seeded placement + state in `game/src/rescue_place.rs`, `game/src/rescue.rs`, `data/rescue.json`, `game/src/guild/recruits.rs`, `game/tests/all/rescue.rs`; wired into `rotland/src/rescue_play.rs`, `rescue_view.rs`, `frame_world.rs`, `frame_sim.rs`, `render_frame.rs`, `start_world.rs`, `save_load.rs`, `state.rs`, `main.rs`.
- **R29-03 (solid camp guard towers):** Done. Added `k.solid(..., base="platform")` to the 4 legs and 4 rail segments in `tools/blender/buildings/settlement_parts.py` (`guard_platform()`) while keeping the ladder opening clear; rebuilt `assets/models/buildings/settlement.glb` cleanly (0 tracebacks, 6002 tris).
- **Final gate (last 5 lines):**
  ```
  test: PASS (1061 passed)
  scope: PASS (34 changed paths)
  numbers: PASS (14 checked)
  unused: PASS
  GATE: PASS
  ```
- **For Claude to check:**
  1. Re-bake `assets/models.pack` (`cargo run --release -p baker`) so the updated `assets/models/buildings/settlement.glb` colliders are packed into runtime assets (`models.pack` is outside `R29-round.md` `Files:`).
  2. `rescued_ids` save/load uses `game::rescue::RescueSave` (`#[serde(default)]`) inside `systems.extra["guild"]` (`rotland/src/rescue_play.rs`) instead of adding a field to `GuildState` in `game/src/guild/save.rs`, because `GuildManager::to_state()` (`game/src/guild/manager.rs`) constructs `GuildState` without `..Default::default()` and `manager.rs` was outside `R29-round.md` `Files:`.
  3. Teammate revive stands beside the downed player with `CompanionAnim::Idle` (`npc_guard`/`npc_scout` lack a `use`/bandage clip). Visual check in-game recommended for `downed_crawl`, survivor rescue prompt/NPC, and tower leg/rail collision + ladder climb.

## Gate report
- date: 2026-11-25
- base: main
- files changed (34): assets/models/buildings/settlement.glb, data/downed.json, data/rescue.json, game/src/downed.rs, game/src/guild/recruits.rs, game/src/lib.rs, game/src/rescue_place.rs, game/src/rescue.rs, game/tests/all/downed.rs, game/tests/all/main.rs, game/tests/all/rescue.rs, rotland/src/app/frame_actors.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_sim.rs, rotland/src/app/frame_team.rs, rotland/src/app/frame_ui.rs, rotland/src/app/frame_water.rs, rotland/src/app/frame_world.rs, rotland/src/app/render_frame.rs, rotland/src/app/save_load.rs, rotland/src/app/start_world.rs, rotland/src/app/state.rs, rotland/src/combat.rs, rotland/src/downed_play.rs, rotland/src/hud/downed.rs, rotland/src/hud/mod.rs, rotland/src/main.rs, rotland/src/rescue_play.rs, rotland/src/rescue_view.rs, tasks/R29-01-downed.md, tasks/R29-02-rescue.md, tasks/R29-03-tower.md, tasks/R29-round.md, tools/blender/buildings/settlement_parts.py
- tests passed: 1061
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
