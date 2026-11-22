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
