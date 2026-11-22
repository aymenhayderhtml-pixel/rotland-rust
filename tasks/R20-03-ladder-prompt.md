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
