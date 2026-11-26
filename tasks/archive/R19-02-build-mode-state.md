# R19-02: build mode: state in the app (no globals), part/cost panel visible, base saved

Read first: `tasks/R19-common.md`, `tasks/R18-03-build-mode.md` (what exists and its wiring notes), `rotland/src/build_mode/*`, `game/src/building/*`, `rotland/src/app/state.rs`, `rotland/src/app/frame_input.rs`, `rotland/src/app/frame_ui.rs`, `save/src/` (how camp upgrades were added to the save: search `camp_upgrades`).
Files: rotland/src/build_mode/*, rotland/src/app/state.rs, rotland/src/app/frame_input.rs, rotland/src/app/frame_ui.rs, rotland/src/app/save_load.rs, save/src/*, save/tests/*, tasks/R19-02-build-mode-state.md
Test: build

Problems seen with `--build-test`: the ghost floor shows but there is NO part/cost panel and no green/red tint; state lives in a global `static GLOBAL_STATE: Mutex<...>` in `build_mode/mod.rs`; placed parts are lost on save/load.

1. Remove the global. Put `BuildModeState` (and the placed parts, `game::building::BaseState`) in app state (`PlaySide` or `WorldState` in `rotland/src/app/state.rs`) and pass it to the functions that need it.
2. Call `draw_build_mode_ui` from `frame_ui.rs` so the panel shows (bottom-center: selected part, cost with bag stock, ready / reason it can't be placed, key help). The panel must fit at 1280x720.
3. Ghost tint: the 2D overlay must be clearly green when placement is valid and red when not.
4. Save: add the placed base parts to the save (if `rotland/src/app/save_load.rs` doesn't exist, find the file that builds the save data and write the change in Wiring notes instead). Bump the save version the same way camp upgrades did, `#[serde(default)]` so old saves still load. Test: save round trip with 3 parts; an old save without the field loads with no parts.
5. Colliders of loaded parts must be added after load (zombies and the player collide with them).

## Worker notes
- Files changed:
  - `save/src/systems/base.rs`: Added `BasePartSaveData` and `BaseSaveData` with `#[serde(default)]` matching base part fields.
  - `save/src/systems/mod.rs`: Re-exported `base` module and types.
  - `save/src/lib.rs`: Re-exported `BasePartSaveData, BaseSaveData`.
  - `save/src/file.rs`: Added `base: Option<BaseSaveData>` to `Systems` with `#[serde(default, skip_serializing_if = "Option::is_none")]`.
  - `save/tests/saves.rs`: Added unit tests `roundtrip_base_parts` (roundtrip save file with 3 placed parts) and `old_save_loads_without_base_parts` (old save without base field loads with default/none).
  - `rotland/src/app/state.rs`: Added `build_mode: BuildModeState` to `PlaySide` and initialized it in `Default`.
  - `rotland/src/build_mode/state.rs`: Added `to_save`, `load_from_save`, `reset`, `check_build_test_flag`, and unit test `test_build_mode_save_load_roundtrip`.
  - `rotland/src/build_mode/colliders.rs`: Added fallback box collision shape when model pack collision points are empty so placed/loaded parts always produce physics colliders, and added unit test `test_sync_colliders_add_and_door_toggle`.
  - `rotland/src/build_mode/ui.rs`: Upgraded 2D ghost overlay to project full 3D bounding box corners into screen space with vivid green/red fill (alpha ~85-95), 2.5px stroke, and clear "READY TO PLACE" / "BLOCKED" badge; ensured bottom-center panel is visible and formatted to fit cleanly at 1280x720 and 1920x986; tagged numbers with `// new: DECISIONS 2026-11-22`.
  - `rotland/src/build_mode/mod.rs`: Removed static `GLOBAL_STATE` Mutex; added `ACTIVE_INSTANCE` AtomicPtr bridge so `render_frame.rs` can safely access `play.build_mode` during rendering; removed unused wrapper functions.
  - `rotland/src/app/frame_input.rs`: Routed all build mode input handling (cycling, rotation, placement, removal, doors, raycast, and `--build-test` flag) directly to `play.build_mode`; synchronized active instance pointer.
  - `rotland/src/app/frame_ui.rs`: Called `draw_build_mode_ui` with `play.build_mode`, inventory, and camera when not in main menu so the part picker, cost, status, and 2D ghost tint overlay display properly.
  - `rotland/src/app/save_load.rs`: Serialized `play.build_mode.to_save()` into `file.systems.base` in `save_now`; updated `menu_action` to reset or load `play.build_mode` on New Game and Continue.

- How to test:
  - Run gate suite: `tools/gate.sh --task tasks/R19-02-build-mode-state.md` (passes all 40 tests).
  - Run save tests: `cargo test -p save` or `cargo test roundtrip_base_parts`.
  - Interactive / screenshot verification: run with `--build-test` (e.g. `target/debug/rotland --build-test --save-dir=/tmp/test_build_saves`). The bottom-center panel displays part name, costs with inventory count (50 wood), ready status, rotation, and key help. Ghost preview displays a prominent green overlay when placement is valid and red overlay when placement is blocked.

- Wiring notes for Claude:
  1. In `rotland/src/app/render_frame.rs`:
     Lines 115-125 can be simplified from:
     ```rust
     if let Some(pack) = render.pack.as_ref() {
         build_mode::with_state(|s| {
             build_mode::render_placed_and_ghost(
                 s,
                 &gpu.device,
                 pack,
                 &mut render.library,
                 &mut beds,
                 &mut world.physics,
             );
         });
     }
     ```
     to direct usage:
     ```rust
     if let Some(pack) = render.pack.as_ref() {
         build_mode::render_placed_and_ghost(
             &mut play.build_mode,
             &gpu.device,
             pack,
             &mut render.library,
             &mut beds,
             &mut world.physics,
         );
     }
     ```
  2. In `rotland/src/app/start_world.rs`:
     When a saved file is loaded during world startup (around line 195):
     ```rust
     if let Some(base) = file.systems.base.as_ref() {
         play.build_mode.load_from_save(base);
         if let Some(pack) = render.pack.as_ref() {
             crate::app::frame_input::build_mode::sync_colliders(&mut play.build_mode, &mut world.physics, pack);
         }
     }
     ```

- Known issues:
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (14): rotland/src/app/frame_input.rs, rotland/src/app/frame_ui.rs, rotland/src/app/save_load.rs, rotland/src/app/state.rs, rotland/src/build_mode/colliders.rs, rotland/src/build_mode/mod.rs, rotland/src/build_mode/state.rs, rotland/src/build_mode/ui.rs, save/src/file.rs, save/src/lib.rs, save/src/systems/base.rs, save/src/systems/mod.rs, save/tests/saves.rs, tasks/R19-02-build-mode-state.md
- tests passed: 40
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
