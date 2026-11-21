# R18-03: build mode in play: place your own base parts

Read first: `tasks/R18-common.md`, `game/src/building/` (all: `BuildConfig`, `BuildPlan::check`, `BaseState`, `world_transform`), `data/build_parts.json`, `rotland/src/props.rs` (how placed models are drawn as instances), `rotland/src/app/frame_input.rs` (keys), `rotland/src/app/render_frame.rs`, `physics/src/world.rs` (adding static colliders), `rotland/src/streaming.rs` (how chunk props/colliders are added: base parts are like player-placed props).
Files: rotland/src/build_mode/*, rotland/src/app/frame_input.rs, rotland/src/app/render_frame.rs, game/src/building/*, game/tests/all/building_tests.rs, game/src/keybinds/*, game/tests/all/keybinds_tests.rs, tasks/R18-03-build-mode.md
Test: building

- New rebindable action `build_mode` (key B). In build mode: a part picker (number keys or mouse wheel cycle through parts; show the part name + cost in a small egui panel bottom-center), a ghost preview of the selected part (`base_<part>` model) snapped to the grid in front of the player (raycast from the camera), green tint when `BuildPlan::check` is OK, red when not (with the reason shown); R rotates 90 degrees; left click places (removes the cost from the bag), right click on a placed part removes it (refund); Esc/B leaves build mode. Weapons can't be used in build mode.
- Placed parts draw with their `base_<part>` models and get physics colliders (walls block the player and zombies), doors toggle with E.
- `BaseState` lives in the app state (in your own module; tell Claude the one line to add to `state.rs` and to the save in Worker notes; DON'T edit `state.rs` or the save crate yourself: another lane owns them).
- Tint for the ghost: if the lit shader can't tint, draw the ghost with a simple colored transparent box instead (and note it).
- Startup flag `--build-test` (parse it inside your module from the args the app passes, or note it for Claude) that starts in build mode with 50 wood in the bag for screenshots.

## Worker notes

