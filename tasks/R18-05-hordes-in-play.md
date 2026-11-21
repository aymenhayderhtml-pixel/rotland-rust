# R18-05: smarter zombies in play: director, hordes, special zombies

Read first: `tasks/R18-common.md`, `game/src/director/`, `game/src/horde/`, `data/director.json`, `data/zombies.json` (brute, screamer, crawler), `rotland/src/play.rs` (`spawn_tick`, ~line 302), `rotland/src/zombies/` (all), `game/src/spawner.rs`, `game/src/flowfield.rs`, `game/src/zombie/` (zombie brain/behaviour).
Files: rotland/src/play.rs, rotland/src/zombies/*, game/src/director/*, game/src/horde/*, game/src/spawner.rs, game/src/spawner_config.rs, game/src/zombie/*, game/tests/all/director_tests.rs, game/tests/all/horde_tests.rs, game/tests/all/spawner_city_tests.rs, game/tests/all/zombie_*.rs, data/director.json, data/zombies.json, data/spawning.json, tasks/R18-05-hordes-in-play.md
Test: horde

- The director ticks every frame (inputs: night, player health, in city, in camp, recent kills, zombies near) and its multipliers scale the spawner's rate and cap.
- Wave events spawn a horde: N zombies out of view at ~70-90 m, grouped, moving as a `Horde` toward the player (or, for the night `CampAttack`, toward the camp gate). Keep zombie count limits: total active zombies never above today's cap x the director's multiplier, and never above 60.
- Special zombies spawn by `specialChance` (night/city): `brute` uses `zombie_brute` model and its stats; `screamer` when it first sees the player plays a scream and calls zombies within `callRadius` to the player's position (they switch to chasing); `crawler` uses its model (its clips are named `zombie_crawler_*`: make the zombie animator pick those clips for this type, data-driven: an optional `clips` map per type in zombies.json).
- Expose two events for the HUD/camp (Claude hooks them): `HordeIncoming { size }` and `CampAttack { size }` as a small Vec of events returned from your tick (note the exact call in Worker notes).
- FPS: hordes must reuse the shared flow field (no per-zombie paths). Note anything that might cost frames.
- Startup flag idea: `--horde-test` spawns a 12-zombie horde 60 m away at start (note it for Claude if you can't add it in your files).

## Worker notes

