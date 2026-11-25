# World Layout & Biomes

Rotland uses an endless procedural world built around a preserved inner **core** and four directional **outer biome sectors** with increasing **danger tiers**.

## 1. Layout Structure

- **Core (`0 .. 2200 m` from the starting camp):**
  - Keeps the original terrain heights, grass colors, scatter placement, starting camp, ruined city (Ashford), and lake unchanged so existing saves remain compatible.
  - `outer = 0.0`, `tier = 0`.
- **Blend Band (`2200 .. 2600 m`):**
  - `outer` smoothly transitions from `0.0` to `1.0` across `blend_band` (`400 m`), blending the core terrain into the outer biome sector.
- **Outer Sectors (`> 2200 m`):**
  - Angles are measured counter-clockwise when viewed from above (`+Y` up), starting from `reference_dir` (the vector from the starting camp to Ashford, or `+X` if no city is attached):
    - **`0°` (beyond Ashford):** Desert
    - **`90°`:** Swamp
    - **`180°` (beyond the lake):** Snow
    - **`270°`:** Mountains
  - **Warped & Blended Borders:** Sector boundaries bend organically using low-frequency simplex noise (`warp_deg`, `warp_frequency`). Within `sector_blend_deg` (`14°`) of a boundary, `secondary` records the neighboring biome and `blend` rises smoothly from `0.0` to `0.5` right on the border.
- **Danger Tiers (`1 .. 5`):**
  - Beyond `core_radius`, danger increases by 1 tier every `tier_band` (`1500 m`):
    `tier = min(5, 1 + floor((distance - core_radius) / tier_band))`.
  - Higher tiers increase zombie spawn density (`zombie_mult`) and grant extra loot rolls (`loot_rolls_bonus`) in crates and POIs.

## 2. Data Files

| File | What it controls |
| --- | --- |
| `data/biomes.json` | All 6 biome definitions (`plains`, `forest`, `desert`, `snow`, `swamp`, `mountains`): display name, map/fog colors, ground RGB colors (`base`, `alt`, `slope`, `peak`, `peak_height`), heightfield parameters (`base`, `amplitude`, `frequency`, `octaves`, `ridged`, `dune`), scatter rules, `zombie_density`, `loot_table`, `animals`, `temperature_day`/`temperature_night`, `thirst_mult`, and `footstep`. |
| `data/world_layout.json` | Radial ring distances (`core_radius`, `blend_band`), sector directions (`sectors`), border warping/blending (`sector_blend_deg`, `warp_deg`, `warp_frequency`), and danger tier multipliers (`tier_band`, `tiers`). |
| `data/scatter.json` | Core Plains/Forest prop rules, visibility distances, and crate placement defaults. |
| `data/spawning.json` | Base spawn caps, distances, day/night zombie weights, and camp/city density modifiers. |
| `data/loot_tables.json` | Loot roll tables referenced by each biome's `loot_table` key. |
| `data/poi.json` | Point-of-interest building definitions and placement rules. |
| `data/settlements.json`, `data/city.json`, `data/lake.json` | Core camp placement, Ashford city layout, and lake/village generation. |

## 3. How to Add a New Biome

1. **Enum (`world/src/biome/mod.rs`):** Add the new `#[repr(u8)]` variant to `BiomeId`, update `BiomeId::COUNT` and `BiomeId::ALL`, and add its snake_case string to `BiomeId::key()` and `BiomeId::from_key()`.
2. **Biome Data (`data/biomes.json`):** Add an entry matching the new `key` with ground colors, height settings, scatter rules, temperatures, and gameplay multipliers. (`BiomeRegistry::load()` validates that every `BiomeId` variant is present.)
3. **World Sector (`data/world_layout.json`):** Add `{ "biome": "<key>", "center_deg": <angle> }` to `sectors` and adjust neighboring sector angles.
4. **Assets & Gameplay Tables:** Export any new prop `.glb` models in `tools/blender/` + `assets/models/props/` and add corresponding loot entries in `data/loot_tables.json`.
5. **Verify:** Run `cargo test -p world` to check registry loading and layout invariants.
