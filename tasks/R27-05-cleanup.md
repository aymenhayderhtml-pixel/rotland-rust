# R27-05: small fixes: "Task ready" overlap, split specials.rs

Files: game/src/zombie/specials*.rs, game/src/zombie/mod.rs, rotland/src/hud/*, rotland/src/hud_*.rs, tasks/R27-05-cleanup.md
Rules: `tasks/R27-common.md`.

1. **HUD overlap:** the guild "Task ready" notice is drawn on top of the "IN CAMP" label at the top centre of the screen. Find both (grep `Task ready`, `IN CAMP` in rotland/src) and move the notice below the label so they never overlap at 1920x986 and 1280x720. Layout numbers get `// new: DECISIONS 2026-11-24`.
2. **Split `game/src/zombie/specials.rs` (531 lines)** into files under ~300 lines (e.g. `specials.rs` + `specials_brute.rs` + `specials_screamer.rs`...). Pure move: no behaviour change, same public API, all tests unchanged and passing.
