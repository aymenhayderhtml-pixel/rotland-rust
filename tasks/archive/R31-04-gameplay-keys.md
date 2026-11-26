# R31-04: one biome key everywhere; zombies, animals and loot per biome + danger tier

Owns: rotland/src/play.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/startup.rs, rotland/src/start_world.rs, game_core/src/loot/*, game_world/src/animals/*, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, engine/src/cli.rs (only biome strings), data/spawning.json, data/animals.json, data/loot_tables.json, game/tests/all/biome_keys_tests.rs (new)
Files: rotland/src/play.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/startup.rs, rotland/src/app/start_world.rs, game_core/src/loot/difficulty.rs, game_core/src/loot/mod.rs, game_core/src/loot/roller.rs, game_core/src/loot/state.rs, game_core/src/loot/table.rs, game_core/src/loot/world_items.rs, game_world/src/animals/ai.rs, game_world/src/animals/ai_states.rs, game_world/src/animals/butcher.rs, game_world/src/animals/combat.rs, game_world/src/animals/config.rs, game_world/src/animals/herd.rs, game_world/src/animals/hunting.rs, game_world/src/animals/instance.rs, game_world/src/animals/loot.rs, game_world/src/animals/manager.rs, game_world/src/animals/manager_update.rs, game_world/src/animals/mod.rs, game_world/src/animals/motion.rs, game_world/src/animals/pool.rs, game_world/src/animals/rng.rs, game_world/src/animals/senses.rs, game_world/src/animals/spawner.rs, game_world/src/animals/state.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, engine/src/cli.rs, data/spawning.json, data/animals.json, data/loot_tables.json, game/tests/all/biome_keys_tests.rs, game/tests/all/main.rs
Read: tasks/R31-round.md, tasks/LEAD.md, rotland/src/play.rs, rotland/src/app/frame_sim_actors.rs, rotland/src/app/startup.rs, rotland/src/app/start_world.rs, game_core/src/loot/difficulty.rs, game_core/src/loot/mod.rs, game_core/src/loot/roller.rs, game_core/src/loot/state.rs, game_core/src/loot/table.rs, game_core/src/loot/world_items.rs, game_world/src/animals/ai.rs, game_world/src/animals/ai_states.rs, game_world/src/animals/butcher.rs, game_world/src/animals/combat.rs, game_world/src/animals/config.rs, game_world/src/animals/herd.rs, game_world/src/animals/hunting.rs, game_world/src/animals/instance.rs, game_world/src/animals/loot.rs, game_world/src/animals/manager.rs, game_world/src/animals/manager_update.rs, game_world/src/animals/mod.rs, game_world/src/animals/motion.rs, game_world/src/animals/pool.rs, game_world/src/animals/rng.rs, game_world/src/animals/senses.rs, game_world/src/animals/spawner.rs, game_world/src/animals/state.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, engine/src/cli.rs, game/tests/all/main.rs
API: world/src/biome.rs, world/src/generator.rs, world/src/lib.rs
Data: data/spawning.json, data/animals.json, data/loot_tables.json, data/items.json
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Don't edit `world/` (R31-01/02/03 own it).

1. **Kill the string forks.** Today about 11 places do `if forest_weight > 0.5 { "forest" } else { "plains" }` or match `"forest"`/`"plains"`. Grep: `rg -n '"forest"|"plains"|forest_weight' --type rust`. Replace each with `generator.biome(x, z).key()` (or pass `BiomeId`). Map tiles (`game_camp/src/map/tiles.rs`) belong to R31-05; skip them. Leave tests that check old behaviour alone unless they break.
   - `game_core` and `game_zombie` already depend on `world` (`game_core/src/loot/table.rs` imports `world::BiomeId`), so pass `world::BiomeId` where it fits, and use `.key()` only where JSON keys are needed.
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

## Neon notes

* **String forks removed:**
* `rotland/src/play.rs`: replaced `if world_gen.forest_weight(player.x, player.z) > 0.5 { "forest" } else { "plains" }` with `world_gen.biome(player.x, player.z).key()`.
* `rotland/src/app/frame_sim_actors.rs`: replaced the 2-arm `match g.biome(player.x, player.z)` with `g.biome(player.x, player.z).key()`.
* `game_core/src/loot/table.rs`: replaced the 2-arm `match biome` in `LootTableRegistry::for_biome` with a `BiomeKey` trait accepting both `BiomeId` and `&str`/`String` (preserving existing callers while supporting `for_biome("desert")`), resolving via `BiomeId::from_key` and falling back to `"plains"` (then `"generic"`) with `log::warn!`.
* `rotland/src/app/startup.rs`, `rotland/src/app/start_world.rs`, and `engine/src/cli.rs` were inspected and left untouched because their `"forest"` references are the `--scene=forest` benchmark scene name, not biome forks (`--biome=` teleport belongs to R31-05).


* **Data added for the 4 new biomes:**
* `data/spawning.json`: added `desert: 0.6`, `snow: 0.8`, `swamp: 1.3`, `mountains: 0.5` to `biomeDensity`.
* `data/animals.json`: added `desert: ["rabbit"]`, `snow: ["deer", "rabbit"]`, `swamp: ["boar"]`, `mountains: ["deer", "rabbit"]`. Note: `"deer_buck"` is a `buckModel` variant on the `"deer"` species rather than a top-level species key in `data/animals.json`, so using `"deer"` keeps `AnimalConfigData::validate()` passing while spawning bucks via `buckChance`.
* `data/loot_tables.json`: added `desert`, `snow`, `swamp`, and `mountains` tables using only item IDs present in `data/items.json`.


* **Danger tier → zombie density:**
* Added `pub tier_mult: f32` (default `1.0`) and `with_tier_mult` / `with_biome_id` to `SpawnContext` in `game_zombie/src/spawner.rs`, and `density_for_biome(BiomeId)` to `SpawnConfig` in `game_zombie/src/spawner_config.rs`. `Spawner::calculate_cap` multiplies `day_cap` (and city cap) by `tier_mult` (capped by `hard_cap`).
* Wired `world_gen.tier(player.x, player.z)` in `rotland/src/play.rs`. Because `R31-round.md` did not name a tier-multiplier accessor on `WorldGenerator`, `PlayState` reads `tiers[tier - 1].zombie_mult` from `data/world_layout.json` (defaulting to `1.0` for tier `0` or if missing). Once R31-02 lands, if `WorldGenerator` exposes a direct tier-multiplier accessor, `PlayState::tier_zombie_mult` can delegate to it.


* **Caller / Lead checks:**
* Verify `log` is in `game_core/Cargo.toml` (used by `log::warn!` in `game_core/src/loot/table.rs`).
* If any test outside the files shown here constructs `SpawnContext` via a struct literal instead of `SpawnContext::new(...)`, add `tier_mult: 1.0`.
* Compiles against R31-01/02 once `BiomeId::ALL`, `BiomeId::key`, `BiomeId::from_key`, and `WorldGenerator::tier` are merged into `world/`.
