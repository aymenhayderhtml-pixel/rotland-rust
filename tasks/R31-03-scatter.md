# R31-03: trees, rocks and props per biome; POIs per biome; build each chunk once

Owns: world/src/scatter.rs, world/src/scatter/* (new, if split), world/src/poi.rs, world/src/loot.rs, data/scatter.json (read-only unless a key must move; say so), data/poi.json, rotland/src/streaming.rs, rotland/src/props.rs (only the per-model draw-range/LOD/shadow tables), world/tests/biome_scatter_tests.rs (new)
Files: world/src/scatter.rs, world/src/poi.rs, world/src/loot.rs, data/scatter.json, data/poi.json, rotland/src/streaming.rs, rotland/src/props.rs, world/tests/biome_scatter_tests.rs
Read: tasks/R31-round.md, tasks/LEAD.md, world/src/scatter.rs, world/src/poi.rs, world/src/loot.rs, rotland/src/streaming.rs, rotland/src/props.rs, world/tests/golden.rs
API: world/src/generator.rs, world/src/lib.rs, world/src/settlement.rs, world/src/biome.rs, rotland/src/crates.rs, rotland/src/poi_links.rs, game_core/src/loot/table.rs, rotland/src/app/state.rs
Data: data/scatter.json, data/poi.json
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY. Don't edit R31-01's or R31-02's files. Call `WorldGenerator::biome_sample` and `BiomeRegistry::get`.

1. **Scatter per biome:**
   - Today `ScatterConfigData::rules()` picks the forest or plains list with an if/else, and coverage is measured on an 8×8 grid by `biome == X`.
   - **In the core:** exactly today's behaviour (same RNG calls in the same order), so the golden props still match.
   - **Outside the core:** the rules come from `BiomeRegistry::get(primary).scatter`. Coverage = the share of the 8×8 samples whose primary is that biome, so border chunks mix both biomes' props naturally.
   - A chunk is "core" when its whole square is inside `core_radius`. Chunks crossing the edge use the outer path only for candidates beyond `core_radius`, and today's path for the rest. Explain how you kept the golden test safe.
2. **POIs per biome:** `data/poi.json` `buildings[].biome` weights get keys for the new biomes. Read the current format first. Suggested: cabin snow 0.6 / mountains 0.3; shed swamp 0.5; gas_station desert 0.6; house desert 0.3 / snow 0.3; barn swamp 0.2; watchtower mountains 0.4 / desert 0.3. The POI code must read the weight by `BiomeId::key()`, not a fixed list. Core POIs unchanged.
3. **Danger tier → loot:** crates and POI loot get `loot_rolls_bonus` extra rolls from the tier (`WorldGenerator::tier`, tiers data from `WorldLayout`; read it via the generator, and ask the lead in your notes if no accessor exists). `LootSpot` gets a `tier: u8` field. Find who uses `LootSpot.biome` and keep it working.
4. **Build each chunk once:** `rotland/src/streaming.rs` calls `chunk_contents` and then `chunk_crates`, which calls `chunk_contents` AGAIN. Add a `chunk_crates_from(&self, coord, &pois, &props)` (or similar) and use it in streaming. Keep the old function for tests and the golden test.
5. **Draw tables:** add the new model names to `rotland/src/props.rs`'s draw range / LOD / no-shadow tables, following the closest existing model:
   - trees (`pine_snow`, `tree_swamp`, `tree_desert_dead`) like pines, with LODs;
   - `boulder_big` / `rock_cliff_01` like rocks, but drawn to tree range because they're big;
   - `reeds`, `lily_mud_patch`, `desert_shrub`, `snow_shrub` like grass/bush (short range, no shadow);
   - cacti like bushes.
   Also check how `streaming.rs` decides "tree" (felling + its own collider): the names start with `tree` or `pine_`. `cactus_*` should NOT be a felled tree, but should have a collider like rocks.
6. **Tests** (`biome_scatter_tests.rs`):
   - a desert chunk at 4000 m in the desert sector has cacti and no `pine_tall`;
   - a snow chunk has `pine_snow`;
   - a core chunk's props are identical to today's (compare with the old code path, or rely on golden and say so);
   - `chunk_crates_from` gives the same crates as `chunk_crates`;
   - tier-3 crates roll more items than tier-1 with the same seed.

## Worker notes
