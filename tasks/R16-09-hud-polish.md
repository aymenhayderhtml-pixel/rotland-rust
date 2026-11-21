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

