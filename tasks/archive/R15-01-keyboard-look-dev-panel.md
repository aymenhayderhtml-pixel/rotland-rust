# R15-01: arrow keys turn the camera; steady dev panel stats

Read first: `tasks/R13-common.md` ("Rust/data lanes" part), `game/src/input.rs`, `game/src/keybinds/actions.rs` and `game/src/keybinds/mod.rs`, `rotland/src/app/frame_input.rs` (around `Actions::from_input`, ~lines 165-190), `rotland/src/dev_panel/draw.rs`, `rotland/src/dev_panel/mod.rs`.
Files: game/src/input.rs, game/src/keybinds/*, game/tests/all/keybinds_tests.rs, game/tests/all/input_look_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_input.rs, rotland/src/dev_panel/*, tasks/R15-01-keyboard-look-dev-panel.md
Test: keybind

The owner sometimes plays without a mouse.

## 1. Arrow keys look around
- New rebindable actions `look_left`, `look_right`, `look_up`, `look_down`, default ArrowLeft/ArrowRight/ArrowUp/ArrowDown, added the same way the existing actions are (so they show in the controls menu if the menu lists actions from that table). Check no existing action already uses the arrow keys.
- Turn speed: 120 degrees per second at mouse sensitivity 1.0 (`// new: DECISIONS 2026-11-21`). The camera uses 0.003 radians per mouse pixel, so convert the key turn into the same "pixels" unit and ADD it to `actions.look_delta` in `frame_input.rs` (needs `dt`, which is there), BEFORE the sensitivity/invert-Y scaling, so the settings apply to it too. Left arrow turns the view left, Up arrow looks up (same direction as moving the mouse up with invert-Y off).
- Unlike mouse look, arrow look must work even when the cursor is NOT captured (that's the no-mouse case), but never while a menu/screen is open (`ui.screen_open()`), and never while a dev panel text field has keyboard focus (see 3).
- It must also work in the car camera (it uses `actions.look_delta` too).
- Put the pure part (held keys + dt -> look delta in pixels) in a function in `game/src/input.rs` with tests in a new `game/tests/all/input_look_tests.rs` (add `mod input_look_tests;` to `main.rs`): 1 s of Left = 120 degrees worth of pixels; Left+Right cancel; Up gives a negative y delta like moving the mouse up; nothing held = zero.

## 2. Dev panel stats must not jump
In the dev panel's "Performance & Stats" section the text wraps to another line when a number gets one digit longer (e.g. 9.9 -> 10.1 ms), so the panel jumps up and down every frame. Fix: each stat on its own row (FPS + ms on one row, draw calls + shadow draws on one row, triangles on one row, position on one row), numbers padded to a fixed width (`{:>6.1}` style), in monospace, and give the window a fixed width so nothing ever wraps. Also update the shown FPS/ms text at most 4 times per second (keep the graph per frame) so it's readable.

## 3. Typing in the dev panel must not move the player
While any dev panel text field has keyboard focus (egui `ctx.wants_keyboard_input()`), game keys (movement, hotbar numbers, jump, arrow look, and other game hotkeys except Esc/F1) are ignored for that frame. Find the cleanest place in `frame_input.rs` to apply this.

## Worker notes
- **Files changed**:
  - `game/src/input.rs`: Added `look_left`, `look_right`, `look_up`, `look_down` fields to `InputBindings` (defaults `KeyCode::ArrowLeft`, `ArrowRight`, `ArrowUp`, `ArrowDown`); added `KEYBOARD_LOOK_SPEED_DEG = 120.0` with `// new: DECISIONS 2026-11-21`; added pure function `keyboard_look_delta(left, right, up, down, dt) -> (f32, f32)` converting turn speed (120 deg/s) to mouse pixel units via `camera_rig::MOUSE_SENSITIVITY`.
  - `game/tests/all/input_look_tests.rs`: Integration tests for pure keyboard look math (1 s left = 120 deg worth of pixels, left/right cancellation, up gives negative y delta, down gives positive y delta, up/down cancellation, nothing held = zero, non-positive dt = zero, dt linear scaling).
  - `game/tests/all/main.rs`: Added `mod input_look_tests;`.
  - `game/tests/all/keybinds_tests.rs`: Added assertions verifying default arrow look bindings in `keybinds_input_bindings_conversion`.
  - `rotland/src/app/frame_input.rs`: Added `dev_typing` detection (`ui.dev_panel.open && ui.dev_panel.wants_keyboard_input`); ignored game keys (movement, actions, hotbar, hub keys, dog stay, autowalk) when `dev_typing` is active while preserving Esc and F1; added arrow look delta to `actions.look_delta` when cursor is not captured or captured (disabled during menus/screens or dev typing), before sensitivity/invert-Y scaling. Works in both on-foot and car camera.
  - `rotland/src/dev_panel/mod.rs`: Added `stats_timer`, `shown_fps`, `shown_ms`, `wants_keyboard_input` to `DevPanelState`. Throttled shown FPS and ms text updates to at most 4 times per second (0.25 s interval) while preserving per-frame history for the graph. Set `wants_keyboard_input` via `ctx.egui_wants_keyboard_input()`. Passed `shown_fps` and `shown_ms` to `DevPanelView`.
  - `rotland/src/dev_panel/draw.rs`: Fixed dev panel window width to 360.0 (`resizable(false)`, `min_width`, `max_width`, `default_width`) to prevent wrapping; formatted stats on distinct rows with fixed-width padded numbers (`{:>6.1}`, `{:>5}`, `{:>7}`, `{:>4.0}`) for FPS + ms, draw calls + shadow draws, triangles, and position + yaw.
- **How to test**:
  - `tools/gate.sh --task tasks/R15-01-keyboard-look-dev-panel.md` (all checks pass).
  - `tools/gate.sh --test input_look` (all 7 pure look tests pass).
  - `tools/gate.sh --test keybind` (all keybinding integration tests pass).
- **Wiring notes for Claude**:
  - Everything is wired directly into `frame_input.rs` and `dev_panel`. No additional wiring needed.
- **Known issues**:
  - None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (8): game/src/input.rs, game/tests/all/input_look_tests.rs, game/tests/all/keybinds_tests.rs, game/tests/all/main.rs, rotland/src/app/frame_input.rs, rotland/src/dev_panel/draw.rs, rotland/src/dev_panel/mod.rs, tasks/R15-01-keyboard-look-dev-panel.md
- tests passed: 7
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
