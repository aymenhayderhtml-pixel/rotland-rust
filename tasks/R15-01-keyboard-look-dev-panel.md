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

