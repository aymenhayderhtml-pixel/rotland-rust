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
