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
- Files changed:
  - `game/src/keybinds/actions.rs`: Added 32nd rebindable action `build_mode` (default KeyCode::KeyB, Godot code 66).
  - `game/tests/all/keybinds_tests.rs`: Updated count to 32, added assertions for `build_mode` key code and key name.
  - `game/src/building/state.rs`: Added `raycast_part` to `BaseState` to find aimed base part along ray.
  - `game/tests/all/building_tests.rs`: Added unit tests for `raycast_part`.
  - `rotland/src/build_mode/types.rs`: Defined `PARTS_ORDER`, display names, model names (`base_<part>`).
  - `rotland/src/build_mode/state.rs`: Defined `BuildModeState` with part cycling, rotation, selection, and unit tests.
  - `rotland/src/build_mode/colliders.rs`: Synchronizes physics colliders for placed base parts; removes doorway colliders when doors are opened.
  - `rotland/src/build_mode/logic.rs`: Raycasting, grid snapping, `BuildPlan::check`, placing (deducting bag costs), removing (refunding materials), door toggling.
  - `rotland/src/build_mode/render.rs`: Instances all placed parts and ghost preview model into the render pipeline.
  - `rotland/src/build_mode/ui.rs`: egui bottom-center panel showing selected part, resource costs and bag stock, placement readiness/error reason, controls legend, plus 2D projected tint overlay over the ghost preview.
  - `rotland/src/build_mode/mod.rs`: Re-exports public API and global accessors, plus `--build-test` startup flag handler.
  - `rotland/src/app/frame_input.rs`: Wired build mode toggle (B key), Esc exit, weapon disable during build mode, part cycling (wheel / 1-9), R rotation, LMB placement, RMB removal, E door toggle, raycast update.
  - `rotland/src/app/render_frame.rs`: Wired placed base parts and ghost preview rendering into render pass.

- How to test:
  - Run unit and integration tests: `tools/gate.sh --task tasks/R18-03-build-mode.md`.
  - Interactive / screenshot verification: Run with `--build-test` flag to start directly in build mode with 50 wood added to inventory.
  - Controls: Press B to toggle build mode, number keys 1-9 or mouse wheel to cycle parts, R to rotate 90°, LMB to place, RMB to remove and refund, E to open/close doors, Esc/B to exit.

- Wiring notes for Claude:
  1. In `rotland/src/main.rs`: add `mod build_mode;` (can replace `#[path = "../build_mode/mod.rs"] pub mod build_mode;` in `frame_input.rs`).
  2. In `rotland/src/app/frame_ui.rs`: inside `draw_ui`: call `crate::build_mode::draw_build_mode_ui(ctx, play.play.as_ref().map(|p| &p.inventory), &render.camera);`.
  3. In `rotland/src/app/state.rs`: add `pub base: Option<game::building::BaseState>` (or directly inside `PlayState` / save data) to persist placed base parts across saves.
  4. Tint for ghost preview: The lit pipeline does not support per-instance tint uniforms (only dither cross-fade). The ghost model renders with 50% dither opacity in 3D, and `draw_build_mode_ui` renders a projected translucent bounding box overlay (green for valid, red for invalid with error reason).

- Known issues:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (14): game/src/building/state.rs, game/src/keybinds/actions.rs, game/tests/all/building_tests.rs, game/tests/all/keybinds_tests.rs, rotland/src/app/frame_input.rs, rotland/src/app/render_frame.rs, rotland/src/build_mode/colliders.rs, rotland/src/build_mode/logic.rs, rotland/src/build_mode/mod.rs, rotland/src/build_mode/render.rs, rotland/src/build_mode/state.rs, rotland/src/build_mode/types.rs, rotland/src/build_mode/ui.rs, tasks/R18-03-build-mode.md
- tests passed: 28
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
