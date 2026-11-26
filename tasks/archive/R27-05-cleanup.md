# R27-05: small fixes: "Task ready" overlap, split specials.rs

Files: game/src/zombie/specials*.rs, game/src/zombie/mod.rs, rotland/src/hud/*, rotland/src/hud_*.rs, tasks/R27-05-cleanup.md
Rules: `tasks/R27-common.md`.

1. **HUD overlap:** the guild "Task ready" notice is drawn on top of the "IN CAMP" label at the top centre of the screen. Find both (grep `Task ready`, `IN CAMP` in rotland/src) and move the notice below the label so they never overlap at 1920x986 and 1280x720. Layout numbers get `// new: DECISIONS 2026-11-24`.
2. **Split `game/src/zombie/specials.rs` (531 lines)** into files under ~300 lines (e.g. `specials.rs` + `specials_brute.rs` + `specials_screamer.rs`...). Pure move: no behaviour change, same public API, all tests unchanged and passing.

## Worker notes
- **What changed:**
  - `rotland/src/hud/notification.rs`: moved `draw_notifications` `base_y` from `65.0 * scale` to `screen.min.y + 126.0 * scale` (`// new: DECISIONS 2026-11-24`) so notification pills (including `"Task ready: return to the guild"`) sit below the compass strip (`10..50 * scale`), `"IN CAMP"` (`74 * scale`), `"CAMP"` + distance (`74..102 * scale`), `"Safe zone"` (`96..118 * scale`), and waypoint distance (`114 * scale`) at both `1920x986` and `1280x720`.
  - Split `game/src/zombie/specials.rs` (531 lines) into `specials.rs` (~300 lines, shared types + `SpecialState` core), `specials_brute.rs`, `specials_screamer.rs`, and `specials_crawler.rs`, wired in `game/src/zombie/mod.rs` with the exact same public API and no behaviour change.
- **How to see it in the game:**
  - Complete or advance a guild task while inside camp (or trigger any HUD notification): the notification pill now renders cleanly below `"IN CAMP"` and `"Safe zone"` instead of overlapping them.
- **Wiring notes for Claude / unsure:**
  - None; no new wiring needed.

## Gate report
- date: 2026-11-24
- base: main
- files changed (7): game/src/zombie/mod.rs, game/src/zombie/specials_brute.rs, game/src/zombie/specials_crawler.rs, game/src/zombie/specials.rs, game/src/zombie/specials_screamer.rs, rotland/src/hud/notification.rs, tasks/R27-05-cleanup.md
- tests passed: 1015
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
