# R32-06: frozen, husk and bloater zombies (behaviour + data + spawn mix)

Owns: game_zombie/src/zombie/specials_biome.rs (new), game_zombie/src/zombie/specials.rs, game_zombie/src/zombie/defs.rs, game_zombie/src/zombie/mod.rs (only the `mod`/`pub use` lines), game_zombie/src/spawner.rs, game_zombie/src/spawner_config.rs, data/zombies.json, data/special_zombies.json, data/spawning.json, game/tests/all/biome_zombies_tests.rs (new)
Files: game_zombie/src/zombie/specials_biome.rs, game_zombie/src/zombie/specials.rs, game_zombie/src/zombie/defs.rs, game_zombie/src/zombie/mod.rs, game_zombie/src/spawner.rs, game_zombie/src/spawner_config.rs, data/zombies.json, data/special_zombies.json, data/spawning.json, game/tests/all/biome_zombies_tests.rs
Read: tasks/R32-round.md, tasks/LEAD.md, game_zombie/src/zombie/specials.rs, game_zombie/src/zombie/defs.rs, game_zombie/src/zombie/mod.rs, game_zombie/src/zombie/brain.rs, game_zombie/src/spawner.rs, game_zombie/src/spawner_config.rs, game/tests/all/special_zombie_tests.rs
API: game_zombie/src/lib.rs
Data: data/zombies.json, data/special_zombies.json, data/spawning.json
Overlap: `game/tests/all/main.rs` is owned by R32-04. Do not write it: report the `mod biome_zombies_tests;` line to the lead instead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. Study how `brute`, `screamer` and `crawler` work (`specials*.rs`, `data/special_zombies.json`, `data/zombies.json` `types`) and follow the same pattern. Don't invent a new system.

1. **Data:**
   - `data/zombies.json` `types`: add `frozen`, `husk` and `bloater` with speed/health relative to the normal walker, as in the shared names. Copy the fields an existing type has.
   - `data/special_zombies.json`:
     - `frozen`: `{ "chillOnBite": 8.0, "speedMult": 0.75, "healthMult": 1.6 }`
     - `husk`: lunge numbers, about 80% of the crawler's (`speedMult` 1.3, `healthMult` 0.7)
     - `bloater`: `{ "cloudRadius": 4.0, "cloudSeconds": 6.0, "playerDps": 4.0, "zombieDps": 2.0, "speedMult": 0.7, "healthMult": 1.4, "burstOnFire": true }`
2. **Behaviour** (`specials_biome.rs`, under ~300 lines):
   - husk: reuses the crawler lunge logic with its own numbers. Don't copy it; take a parameter.
   - frozen: no special move; when its attack lands, the result says `chill: Some(amount)`.
   - bloater: when it dies, an event `ToxicBurst { pos, radius, seconds, player_dps, zombie_dps }`.
   - Follow how existing specials report events to the game (grep `Event` / `events` in `specials.rs`), and add the new event kinds there.
   - **Toxic clouds:** a pure `ToxicClouds` list with `spawn`, `update(dt)` and `damage_at(pos) -> (player_dps, zombie_dps)`. Clouds expire.
3. **Spawn mix:** `data/spawning.json` `biomeSpecials`:
   - snow: frozen 0.35
   - mountains: frozen 0.25
   - desert: husk 0.35
   - swamp: bloater 0.25
   - home biomes: none
   The spawner picks a biome special with that share, using the biome key it already receives (R31 passes `biome.key()`), BEFORE today's special roll. Today's specials stay as they are.
4. **Kind strings:** `rotland/src/play.rs:523` and `game_zombie/src/spawner.rs:336` hold `["brute", "screamer", "crawler"]` lists. In YOUR file (spawner.rs), make it a list that also includes the new kinds where that list means "special kinds". Report the play.rs line for the lead/R32-07.
5. **Tests** (`biome_zombies_tests.rs`; tell the lead to add the `mod` line):
   - the data loads;
   - with a seeded RNG, about 35% of 1000 snow spawns are frozen (±5%), and 0 in plains;
   - a bloater death makes a burst event;
   - a cloud damages within its radius and not outside, and expires after 6 s;
   - a frozen bite carries chill 8;
   - husk speed > walker speed.

## Worker notes
