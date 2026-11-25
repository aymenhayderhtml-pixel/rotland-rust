# R31-04: one biome key everywhere; zombies, animals and loot per biome + danger tier

Owns: rotland/src/play.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/startup.rs, rotland/src/start_world.rs, game_core/src/loot/*, game_world/src/animals/*, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, engine/src/cli.rs (only biome strings), data/spawning.json, data/animals.json, data/loot_tables.json, game/tests/all/biome_keys_tests.rs (new)
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Don't edit `world/` (R31-01/02/03 own it).

1. **Kill the string forks.** Today about 11 places do `if forest_weight > 0.5 { "forest" } else { "plains" }` or match `"forest"`/`"plains"`. Grep: `rg -n '"forest"|"plains"|forest_weight' --type rust`. Replace each with `generator.biome(x, z).key()` (or pass `BiomeId`). Map tiles (`game_camp/src/map/tiles.rs`) belong to R31-05; skip them. Leave tests that check old behaviour alone unless they break.
   - `game_zombie` and `game_core` don't depend on `world`. Don't add that dependency; keep passing the `&str` key (or a small enum in game_core if one already exists). Read the Cargo.toml files first.
2. **Data for the new biomes:**
   - `spawning.json` `biomeDensity`: desert 0.6, snow 0.8, swamp 1.3, mountains 0.5 (the same values as `data/biomes.json` `zombie_density`; R31-01 writes those).
   - `animals.json` `biomes`: desert rabbit, snow deer + rabbit, swamp boar, mountains deer_buck + rabbit. Only use animals that already exist.
   - `loot_tables.json`: add `desert`, `snow`, `swamp` and `mountains` tables using only existing item ids. Desert leans to water/canned food, snow to food/cloth/bandages, swamp to medicine, mountains to ammo/tools. Make sure `LootTables::for_biome` (`game_core/src/loot/table.rs`) maps all 6 keys and falls back to plains with a warning log for an unknown key.
3. **Danger tier → zombies:** the spawner's density gets multiplied by `zombie_mult` for the player's tier. Get the tier from `WorldGenerator::tier(x, z)` in `rotland/src/play.rs`, where the biome string is set today. Pass the multiplier through the spawn context as a plain `f32` (`tier_mult`, default 1.0). The tiers come from `world::biome::layout`; the generator exposes them, and you read them in rotland.
4. **Tests** (`biome_keys_tests.rs`):
   - every biome key has a spawning density, an animals entry and a loot table;
   - `for_biome("desert")` gives the desert table;
   - an unknown key falls back to plains;
   - `tier_mult` 2.2 gives more spawns than 1.0 with the same RNG.

## Worker notes
