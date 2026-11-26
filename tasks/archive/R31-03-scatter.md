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

## Neon notes

### What was wired

* **`world/src/scatter.rs`**:
* Added `ScatterPlacer::is_core_chunk` (true when all 4 corners of the chunk square have `biome_sample(x, z).outer <= 0.0`).
* Added `ScatterPlacer::place_with_registry(generator, config, registry, coord)`:
* **Golden safety**: when `is_core_chunk` is true, it immediately returns `ScatterPlacer::place(generator, config, coord)`, executing the exact legacy Forest/Plains loop and `DotnetRandom` calls in the exact same order.
* **Edge-crossing chunks**: splits the 8×8 coverage grid into core samples (`outer <= 0.0`) and outer samples (`outer > 0.0`). Core rules run only for candidates with `outer <= 0.0`; outer rules from `BiomeRegistry::get(biome).scatter` run only for candidates with `outer > 0.0`, sharing `placed`/`spacing` so nothing overlaps across the boundary.
* **Outer chunks**: uses `BiomeRegistry::get(primary).scatter` weighted by each biome's share of the 8×8 coverage grid.


* Kept `ScatterPlacer::place` unchanged for backward compatibility and direct core-vs-legacy comparison tests.


* **`world/src/poi.rs` & `data/poi.json**`:
* `PoiPlacer::biome_weight` now looks up `def.biome.get(biome.key())` instead of hardcoding `"forest"`/`"plains"`. Core POIs remain identical.
* Added the new biome weights to `data/poi.json` (`cabin`, `shed`, `house_small`, `barn`, `watchtower`, `gas_station`).


* **`world/src/loot.rs`**:
* Added `#[serde(default)] pub tier: u8` to `LootSpot` (and `LootSpot::with_tier`; kept `LootSpot::new` with `tier: 0` so existing callers keep working). `LootSpot.biome` is unchanged.
* `LootPlacer::try_place` populates `tier` via `generator.tier(x, z)`.
* Added `loot_rolls_bonus_for_tier` and `roll_crate_item_count` helpers.


* **`rotland/src/streaming.rs`**:
* Switched worker build from `world_gen.chunk_crates(coord)` to `world_gen.chunk_crates_from(coord, &pois, &props)` so `chunk_contents` runs only once per chunk.
* Verified tree/collider routing: `PropInstance::is_tree()` matches `tree*` and `pine_*` (`pine_snow`, `tree_swamp`, `tree_desert_dead`), while `cactus_*` goes through `static_props` and receives compound colliders like rocks without being choppable.


* **`rotland/src/props.rs`**:
* Added `reeds`, `lily_mud_patch`, `desert_shrub`, `snow_shrub` to `is_ground` (short `GROUND` range, no shadow, no dither fade).
* Added `is_big_rock` (`boulder_big`, `rock_cliff_01`): `TREE` range, no shadow.
* Added `cactus_*` to `BUSH` range, no shadow.
* `pine_snow`, `tree_swamp`, `tree_desert_dead` already match `tree_`/`pine_` (`TREE` range, casts shadow, 3 LODs).
* Added unit tests for all new model rules.


* **`data/scatter.json`**: left read-only (no key needed to move).
* **`world/tests/biome_scatter_tests.rs`**: added all 5 required tests.

### Required lead / shared-file wiring (files outside R31-03 ownership)

1. **`world/src/generator.rs`** (owned by R31-02):
* In `chunk_contents(&self, coord: IVec2)`, call `ScatterPlacer::place_with_registry(self, &self.scatter_config, &self.biome_registry, coord)` (using R31-02's `BiomeRegistry` field on `WorldGenerator`).
* Add `chunk_crates_from` and delegate `chunk_crates` to it:
```rust
pub fn chunk_crates_from(
    &self,
    coord: IVec2,
    pois: &[PoiInstance],
    props: &[PropInstance],
) -> Vec<crate::loot::LootSpot> {
    let mut crates = crate::loot::LootPlacer::place(
        self,
        &self.scatter_config,
        &self.poi_config,
        coord,
        props,
        pois,
    );
    crates = crate::settlement::SettlementGround::remove_crates(
        &self.settlements,
        crates,
        &self.config,
    );
    // Keep any existing city/lake crate filtering that chunk_crates currently applies.
    crates
}

pub fn chunk_crates(&self, coord: IVec2) -> Vec<crate::loot::LootSpot> {
    let (pois, props) = self.chunk_contents(coord);
    self.chunk_crates_from(coord, &pois, &props)
}

```


* Add a danger-tier loot bonus accessor on `WorldGenerator`, e.g.:
```rust
pub fn loot_rolls_bonus(&self, tier: u8) -> i32 {
    self.layout.loot_rolls_bonus(tier) // 0 for tier == 0; tiers[tier - 1].loot_rolls_bonus outside
}

```




2. **Crate & POI loot rolling (`rotland/src/crates.rs` / `game_core/src/loot/`, outside R31-03 ownership / shared with R31-04)**:
* Pass `world_gen.loot_rolls_bonus(spot.tier)` when rolling crate loot for a `LootSpot`, and `world_gen.loot_rolls_bonus(world_gen.tier(poi.position.x, poi.position.z))` when rolling POI loot.
* `LootSpot.biome` is preserved unchanged for `LootTableRegistry::for_biome(spot.biome)`.



### Known risks

* `world/tests/biome_scatter_tests.rs` assumes `data/world_layout.json` has a `tiers` array with integer `"loot_rolls_bonus"` fields where tier 3 > tier 1, and that `WorldGenerator::from_data_dir` loads `data/biomes.json` and `data/world_layout.json`. If R31-01 uses camelCase (`lootRollsBonus`) in `data/world_layout.json`, update the JSON key lookup in `tier_3_crates_roll_more_items_than_tier_1_with_same_seed` to match.
* `WorldGenerator::chunk_crates_from` must be added in `world/src/generator.rs` before `rotland` and `biome_scatter_tests` compile.
