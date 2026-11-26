# R31-01: biome list, biome data, ring/sector layout, danger tiers (pure logic)

Owns: world/src/biome/* (new folder; delete `world/src/biome.rs`), data/biomes.json (new), data/world_layout.json (new), world/tests/biome_layout_tests.rs (new), docs/WORLD.md (new), world/Cargo.toml (move `paths` from dev-dependencies to dependencies), world/src/lib.rs (the `pub mod`/`pub use` lines for the new biome module only)
Files: world/src/biome/mod.rs, world/src/biome/registry.rs, world/src/biome/layout.rs, data/biomes.json, data/world_layout.json, world/tests/biome_layout_tests.rs, docs/WORLD.md, world/Cargo.toml, world/src/lib.rs
Read: tasks/R31-round.md, tasks/LEAD.md, world/src/biome.rs, world/src/lib.rs, world/src/generator.rs, world/tests/terrain_unit_tests.rs, world/Cargo.toml
API: world/src/scatter.rs, world/src/noise.rs, paths/src/lib.rs
Data: data/scatter.json, data/spawning.json
Rules: `tasks/LEAD.md` subagent rules. Use the "R31 shared names" in `tasks/R31-round.md` EXACTLY.

**Goal:** the base every other R31 task builds on.

1. **`world/src/biome/mod.rs`**: the `BiomeId` enum from the shared names (keep the derive list the old `biome.rs` had, plus `Hash` if missing), with `ALL`, `key()` and `from_key()`. `pub mod registry; pub mod layout;`. Re-export what `world/src/lib.rs` already re-exports (`BiomeId`); report any new `pub use` lines for `world/src/lib.rs` to the lead.

2. **`registry.rs`**: `BiomeRegistry`, `BiomeDef`, `BiomeGround`, `BiomeHeight` with serde, loading `data/biomes.json`. Scatter rules: reuse the existing scatter rule type from `world/src/scatter.rs` (read it; don't copy it). `get()` must never panic for a valid `BiomeId`: the loader checks that all 6 keys are present and returns `Err` naming any missing key.

3. **`data/biomes.json`**: all 6 biomes. Plains and forest get colours equal to today's (`PLAINS_GRASS` / `FOREST_GRASS` in `world/src/generator.rs`, as 0-255) and today's zombie density (`data/spawning.json` `biomeDensity`). Pick the new biomes' values to look and play right, with a `"_why"` string per biome. The look:
   - **desert:** warm sand (light/dark), sandstone on slopes, gentle dunes (`dune` > 0 means long stretched ridges along one direction), height 2-10 m. Scatter: cactus_tall, cactus_round, desert_shrub (most), rock_sandstone_01/02, cattle_skull (rare), tree_desert_dead (rare). Hot days, cool nights, thirst ×1.6.
   - **snow:** white snow with blue-grey shadow and slope colours, rolling hills like the forest (amplitude ~18). pine_snow (dense groves), snow_shrub, rock_ice_01, snow_drift, boulder_big. Cold (-12 day / -22 night), thirst ×0.9.
   - **swamp:** dark green-brown mud, almost flat (amplitude ~1.5, base ~0.6), with low wet spots darker. tree_swamp, reeds (many), swamp_log_mossy, swamp_stump, lily_mud_patch. Mild and humid. The most zombies of the new biomes (1.3).
   - **mountains:** `ridged: true`, amplitude ~110, low frequency (~0.0016), rock grey on slopes, `peak` = snow_white above `peak_height` ~70 m. boulder_big, rock_cliff_01, pine_snow (sparse), rock_01-03. Cold, few zombies (0.5).
   The colours must use the same RGB as the sheet C colours R31-06 adds (sand_light etc.). Copy the values from R31-06's list below, so ground and models match:
   `sand_light 226,200,148 · sand_dark 196,165,110 · sandstone 190,120,80 · sandstone_dark 150,90,60 · snow_white 236,240,245 · snow_shadow 190,200,215 · ice_blue 170,205,230 · swamp_moss 70,90,45 · swamp_mud 75,62,42 · reed_tan 170,150,95`.

4. **`layout.rs` + `data/world_layout.json`**:
   ```
   { "core_radius": 2200.0, "blend_band": 400.0,
     "sectors": [ {"biome":"desert","center_deg":0.0}, {"biome":"swamp","center_deg":90.0},
                  {"biome":"snow","center_deg":180.0}, {"biome":"mountains","center_deg":270.0} ],
     "sector_blend_deg": 14.0, "warp_deg": 22.0, "warp_frequency": 0.0004,
     "tier_band": 1500.0,
     "tiers": [ {"zombie_mult":1.0,"loot_rolls_bonus":0}, {"zombie_mult":1.25,"loot_rolls_bonus":0},
                {"zombie_mult":1.5,"loot_rolls_bonus":1}, {"zombie_mult":1.8,"loot_rolls_bonus":1},
                {"zombie_mult":2.2,"loot_rolls_bonus":2} ] }
   ```
   Angles are measured from `reference_dir` (camp → Ashford), counter-clockwise when seen from above (+Y up). So the desert lies beyond Ashford and the snow beyond the lake (the lake is opposite the city). `warp_deg` bends the sector borders with low-frequency noise (use `crate::noise` with the seed + a NEW offset not used anywhere: grep `seed +` / `seed.wrapping_add` and pick an unused one, e.g. 201, using `wrapping_add`). Within `sector_blend_deg` of a border, `secondary` = the neighbour and `blend` rises to 0.5 at the border. `outer` = smoothstep from `core_radius` to `core_radius + blend_band`. Tier = 0 in the core, else `1 + floor((d - core_radius) / tier_band)` capped at the number of tiers.

5. **Tests** (`world/tests/biome_layout_tests.rs`; the world crate's tests live in `world/tests/`, check how they're organised and follow that):
   - the registry loads the real data, has all 6 keys, and `key`/`from_key` round-trip;
   - a missing biome gives `Err`;
   - `outer` = 0 inside the core;
   - each sector centre at 4000 m gives that biome, with blend 0;
   - the border gives blend near 0.5;
   - tiers 1..5 at the right distances, capped at 5;
   - the same seed gives the same result, and a different seed moves the borders a bit.

6. **`docs/WORLD.md`** (short, simple English): how the world is laid out (core, rings, sectors, tiers), which data file holds what, and how to add a new biome.

## Worker notes

## Neon notes

* **Deleted file:** Remove the old `world/src/biome.rs` file now that it has been replaced by the `world/src/biome/` module (`mod.rs`, `registry.rs`, `layout.rs`).
* **Shared-file re-exports (`world/src/lib.rs`):** Updated `pub use biome::BiomeId;` to:
```rust
pub use biome::{
    BiomeDef, BiomeGround, BiomeHeight, BiomeId, BiomeRegistry, BiomeSample, SectorConfig,
    TierConfig, WorldLayout, WorldLayoutData,
};

```


* **API & Design Summary:**
* `BiomeId` (`#[repr(u8)]`) keeps `Plains = 0` and `Forest = 1` and adds `Desert = 2`, `Snow = 3`, `Swamp = 4`, `Mountains = 5`, `BiomeId::ALL`, `key()`, and `from_key()`.
* `BiomeRegistry` (`load()`, `load_from_file()`, `from_json()`, `get()`) validates that all 6 biome keys exist in `data/biomes.json` and returns an `Err` naming any missing keys; `get(BiomeId)` indexes a fixed `[BiomeDef; 6]` and cannot panic. Reuses `crate::scatter::ScatterRule`.
* `WorldLayout` (`new()`, `sample()`, `direction_at_deg()`) computes `outer` via `smooth_step(core_radius, core_radius + blend_band, dist)`, `tier` (`0` in core, `1..=5` outside), counter-clockwise sector angles from `reference_dir` (+Y up), noise-warped borders using `GodotNoise::simplex(seed.wrapping_add(201), warp_frequency)`, and smooth border blending up to `0.5` across `sector_blend_deg`.


* **Downstream Wiring:** R31-02 (`WorldGenerator`) can now instantiate `BiomeRegistry` and `WorldLayout` (passing camp pad center as `origin` and `city_center - camp` or `Vec2::X` as `reference_dir`) to implement `WorldGenerator::biome_sample()` and `WorldGenerator::tier()`.
