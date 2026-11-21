# R16-09: HUD polish: damage direction, pickup popups, notifications, nicer bars

Read first: `tasks/R16-common.md`, `rotland/src/hud/mod.rs` and the files it uses (`vitals.rs`, `hint.rs`, `text.rs`), `game/src/hud_math.rs`, how the player takes damage (search `bites` in `rotland/src/app/frame_sim.rs`) and how items are picked up (search `pick` / `add_item` in `rotland/src/` to find the spot; don't change the logic there, only read the result).
Files: rotland/src/hud/*, game/src/hud_math.rs, game/tests/all/hud_math_tests.rs, game/tests/all/main.rs, data/hud.json, tasks/R16-09-hud-polish.md
Test: hud

Frontend polish (the owner wants a better-looking HUD):
1. **Damage direction:** when the player is hit, a red arc on the screen edge points toward the attacker (relative to the camera yaw), fades over 1.2 s; several at once allowed. Pure math in `hud_math.rs` (angle on screen from player pos, attacker pos, camera yaw; Godot yaw convention, AGENTS.md rule 7) with tests.
2. **Pickup popups:** "+2 Canned Beans" with the item icon, stacking on the right side above the hotbar, merging repeats ("+3"), fade after 2.5 s.
3. **Notifications:** a small queue (top center under the compass) for events: "Upgrade finished", "Horde incoming", "Bounty complete". API `HudNotifications::push(text, kind)` (kinds: Info, Warning, Danger with colors).
4. **Nicer bars:** health/food/water/stamina bars get rounded ends, a darker back, a smooth "recent damage" trail (white part that shrinks after a hit), and a pulse when low.
All timings/colors/sizes in a new `data/hud.json`. Wiring the events in: add a small public API (`hud.on_damage(from, now)`, `hud.on_pickup(item, count)`, `hud.notify(..)`) and call `on_damage`/`on_pickup` from the one place each happens ONLY if that place is in your Files; otherwise list the exact call sites in Worker notes for Claude.

## Worker notes
- **Files changed (10):**
  - `data/hud.json`: Full configuration for timings, colors, sizes, and thresholds (damage fade 1.2s, pickup popup 2.5s, notifications 4.0s, rounded ends corner radius 6.0px, dark backgrounds, trail colors, low thresholds, pulse speed).
  - `game/src/hud_math.rs`: Added pure math functions with source tags:
    - `damage_direction_angle_rad(player_x, player_z, attacker_x, attacker_z, camera_yaw_rad)`: Computes 2D angle relative to view direction with Godot yaw convention.
    - `damage_screen_direction(angle_rad)`: 2D screen direction unit vector (dx, dy).
    - `damage_screen_point(screen_w, screen_h, angle_rad, margin)`: Screen edge ellipse coordinate.
    - `damage_arc_points(cx, cy, rx, ry, angle, span, segments)`: Elliptical screen edge arc points.
    - `damage_alpha(elapsed, duration)`: Linear fade alpha calculation.
    - `vitals_trail_update(current, target, dt, rate)`: Smooth recent damage trail shrink.
    - `bar_pulse_factor(time, speed)`: Low-stat pulse sine wave factor.
    - Constants: `DEFAULT_DAMAGE_FADE_S`, `DEFAULT_PICKUP_FADE_S`, `DEFAULT_NOTIFICATION_DURATION_S`, `DEFAULT_TRAIL_SHRINK_RATE`, `DEFAULT_TRAIL_DELAY_S`, `DEFAULT_PULSE_SPEED` (tagged `// data: hud.json <key>`).
  - `game/tests/all/hud_math_tests.rs`: Comprehensive unit tests for all math functions covering all cardinal directions, camera yaw rotation, screen points, arc points, alpha fade, trail updates, and pulse factors.
  - `rotland/src/hud/config.rs`: `HudConfig` with `parse_hex_color`, `color_with_alpha`, and default loading from `data/hud.json`.
  - `rotland/src/hud/damage.rs`: `DamageIndicator` and `draw_damage_indicators` for rendering red screen edge arcs pointing toward attackers.
  - `rotland/src/hud/pickup.rs`: `PickupPopup`, `push_or_merge_pickup`, and `draw_pickup_popups` stacking above hotbar on the right with repeat count merging and icon rendering.
  - `rotland/src/hud/notification.rs`: `NotificationKind` (`Info`, `Warning`, `Danger`), `HudNotification`, `HudNotifications` queue, and `draw_notifications` top-center under compass.
  - `rotland/src/hud/vitals.rs`: `VitalsTrailState` and `draw_vital_bars_polished` / `draw_need_bars_polished` with rounded ends, darker back, white recent damage trail, and low-pulse animations.
  - `rotland/src/hud/mod.rs`: `Hud` manager struct with public API:
    - `hud.on_damage(from, now)`
    - `hud.on_pickup(item, count)` / `hud.on_pickup_at(item, count, now)`
    - `hud.notify(text, kind)` / `hud.notify_at(text, kind, now)`
    - `hud.update(dt, now, player_pos, camera_yaw)`
    - `hud.update_vitals(health, stamina, food_frac, water_frac, dt)`
    - `hud.populate_state(state)`
    - `hud.draw(ctx, state, icons, now)`
    - `HudState` extensions for damage indicators, pickup popups, notifications, and vitals trail.
    - Integrated into `draw_hud` with full backward compatibility.
    - Unit tests covering lifecycle, repeat merging, vitals trail, config, state population, and JSON roundtrip.
  - `tasks/R16-09-hud-polish.md`: Filled worker notes and gate report.

- **How to test:**
  - Run `tools/gate.sh --task tasks/R16-09-hud-polish.md` (passes all checks: clippy, unit tests, scope, numbers, unused).
  - Can also run `cargo test hud` to run all 21 HUD math, lifecycle, and rendering unit tests.

- **Wiring notes for Claude:**
  - `UiState` (`rotland/src/app/state.rs`): Add `pub hud: crate::hud::Hud` (initialized via `Hud::default()`).
  - Damage direction call site:
    In `rotland/src/app/frame_sim.rs:59-62` and `frame_sim.rs:382-402`:
    When zombies attack the player (`frame.bites`), record attacker position (e.g. zombie position from `actors.zombies` or hit position):
    Call `ui.hud.on_damage(attacker_pos, (play.play_time as f64));`.
  - Pickup popups call site:
    In `rotland/src/play.rs:347-360` (`PlayState::interact_tick`):
    When `InteractResult::PickedUpItem { item_id, count, .. }`:
    Call `ui.hud.on_pickup(&item_id, count);` (or pass via `play_state`).
    When `InteractResult::OpenedCrate { items, .. }`:
    For each stack in `items`: `ui.hud.on_pickup(&stack.item_id, stack.count);`.
  - Notifications call sites:
    Any gameplay event can call `ui.hud.notify("Upgrade finished", NotificationKind::Info)`, `ui.hud.notify("Horde incoming", NotificationKind::Danger)`, `ui.hud.notify("Bounty complete", NotificationKind::Warning)`.
  - Per-frame update & draw:
    In `rotland/src/app/frame_ui.rs:140-195`:
    Call `ui.hud.update(dt, time, p.position, p.yaw);` and `ui.hud.update_vitals(c.health.current(), p.stamina.current, state.food.map(|(f, _)| f), state.water.map(|(w, _)| w), dt);`.
    Call `ui.hud.populate_state(&mut state);`.
    `hud::draw_hud(ctx, &state, &mut ui.icons);` will then automatically render all polished elements.

- **Known issues:**
  - None. All 21 tests pass with clean clippy.

## Gate report
- date: 2026-11-21
- base: main
- files changed (10): data/hud.json, game/src/hud_math.rs, game/tests/all/hud_math_tests.rs, rotland/src/hud/config.rs, rotland/src/hud/damage.rs, rotland/src/hud/mod.rs, rotland/src/hud/notification.rs, rotland/src/hud/pickup.rs, rotland/src/hud/vitals.rs, tasks/R16-09-hud-polish.md
- tests passed: 21
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
