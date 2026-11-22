# R19-05: HUD warnings for hordes, the director's waves and camp attacks

Read first: `tasks/R19-common.md`, `rotland/src/hud/mod.rs` (`notify`, `NotificationKind`), `rotland/src/hud/notification.rs`, `rotland/src/zombies/mod.rs` (`ZombieFrame`, `hordes`), `game/src/horde/*`, `game/src/director/*`, `rotland/src/play.rs` (where the director runs), `rotland/src/app/frame_sim.rs`, `data/hud.json`.
Files: rotland/src/zombies/mod.rs, rotland/src/zombies/types.rs, rotland/src/app/frame_sim.rs, rotland/src/hud/*, rotland/src/play.rs, data/hud.json, game/src/director/*, game/tests/all/director_tests.rs, tasks/R19-05-hud-horde-events.md
Test: hud

1. `ZombieFrame` gets events: `HordeFormed { size, position }`, `HordeNear { size, distance }` (first time a horde of 5+ comes within 60 m of the player; distance from `data/hud.json`), `SpecialSeen { kind }` (first sighting of a brute / screamer / crawler within 30 m), `CampAttacked` (a zombie inside the camp wall radius while the player is in the camp).
2. The director's phase changes (calm -> tension -> wave) produce an event too (from `play.rs` or `game/src/director`).
3. `frame_sim.rs` turns events into HUD notifications with short texts ("A horde is coming from the north-east", "Screamer nearby!", "The camp is under attack!", "Wave incoming"). Direction words from the angle (8 directions, pure fn + test). Each event type has a cooldown (seconds in `data/hud.json`) so the screen isn't spammed.
4. Horde warning kind uses a red/orange color; add a small screen-edge arrow toward a near horde if it fits cleanly (optional).
5. Tests: direction words, cooldown, `HordeNear` fires once per horde.

## Worker notes
