# R20-03: show "[E] Climb ladder" at ladders (issue 003, part a)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` row 003 (+ picture), `game/src/ladders/*` (`LadderEnd::prompt()` in `end.rs`), `rotland/src/app/frame_world.rs` (E-prompt code for crates, NPCs, cars, corpses, fires), the ladder climb tick (search `climb_tick` in `rotland/src`).
Files: game/src/ladders/*, rotland/src/app/frame_world.rs, rotland/src/ladder_play.rs, game/tests/all/*, tasks/R20-03-ladder-prompt.md
Test: ladder

1. When the player stands at the bottom or top of a ladder (in reach of a rail, not yet climbing), show the prompt from `LadderEnd::prompt()` (e.g. "[E] Climb ladder" / "[E] Climb down"), using the same prompt mechanism as crates and NPCs. While climbing, show how to get off if the Godot game did (check `ladders/` comments for the Godot file; port the text, don't invent).
2. The prompt must not fight the other prompts: pick the nearest thing like the existing code does.
3. Test the pure part: which prompt (if any) for a player position near a ladder end, far away, and while climbing.
4. Part (b) of the issue (tower legs have no colliders) is NOT in this task: that model has someone's unsaved work.
(`rotland/src/ladder_play.rs` may not exist: create it only if the wiring needs its own file.)

## Worker notes
- Files changed:
  - `game/src/ladders/end.rs`: Added `nearest_ladder_end` and `ladder_prompt` to find the nearest ladder end within reach (`GRAB_RANGE`) when not climbing.
  - `game/src/ladders/mod.rs`: Re-exported `nearest_ladder_end` and `ladder_prompt`.
  - `game/src/ladders/tick.rs`: Refactored `climb_tick` grab detection to use `nearest_ladder_end`.
  - `rotland/src/app/frame_world.rs`: Integrated `nearest_ladder_end` into interaction prompt selection in `update_world_interactions`. The nearest interactable (ladder vs crate/item/NPC/car/fire/dog) wins so prompts don't fight. When climbing or far away, no ladder prompt is shown (matching Godot).
  - `game/tests/all/ladders_tests.rs`: Added comprehensive unit tests for `ladder_prompt` and `nearest_ladder_end` covering ladder foot, deck, out-of-range positions, climbing state, and multi-rail proximity.
- How to test:
  - Stand at the foot of any ladder (e.g. guard tower): "[E] Climb ladder" is displayed.
  - Climb onto the deck: on deck in reach of the ladder top, "[E] Climb down" is displayed.
  - While climbing, no ladder prompt is shown.
  - Automated tests: `tools/gate.sh --task tasks/R20-03-ladder-prompt.md`
- Wiring notes for Claude:
  - None; wiring is complete in `rotland/src/app/frame_world.rs`.
- Known issues:
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (6): game/src/ladders/end.rs, game/src/ladders/mod.rs, game/src/ladders/tick.rs, game/tests/all/ladders_tests.rs, rotland/src/app/frame_world.rs, tasks/R20-03-ladder-prompt.md
- tests passed: 6
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
