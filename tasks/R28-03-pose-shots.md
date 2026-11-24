# R28-03: screenshot tool for animations

Files: rotland/src/app/startup.rs, rotland/src/app/state.rs, rotland/src/app/anim_shot*.rs, rotland/src/app/mod.rs, rotland/src/app/frame_actors.rs, rotland/src/app/render_frame.rs, tools/README_TOOLS.md, tasks/R28-03-pose-shots.md
Rules: `tasks/R27-common.md`.

**Goal:** Claude needs to check animations with a screenshot, without playing. Add dev flags used with `--shot=<png>`:
- `--pose=<clip>[@<seconds>]`: the player's animator plays that clip on the right layer (upper-body clips like `block_2h`, `melee_2h_b`, `eat` as an action/held pose; full-body clips like deaths as full body), frozen at that time (default: the middle of the clip), for the screenshot.
- `--weapon=<id>`: put that weapon in the player's hand at start (bat, axe, pistol, rifle...).
- `--cam-yaw=<degrees>`: orbit the third-person camera around the player (0 = behind, 90 = right side, 180 = front) so the pose can be seen from any side.
- `--block`: the player holds the melee block (right mouse held), and `--sprint` with `--autowalk`: the player sprints forward.
All flags are off by default and change nothing in normal play. Put the parsing and the "what to apply" logic in a small module with unit tests; wire it into startup and the frame. Document the flags with one example each in `tools/README_TOOLS.md`. Check it yourself with the gate only (don't run the game).
