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
| `data/weather.json` | Weather spell durations (`min_spell_seconds`, `max_spell_seconds`), transition `fade_seconds`, per-biome weighted weather tables (`biomes`), and per-kind gameplay/render properties (`fog_mult`, `visibility`, `noise_mask`, `wind`, `particles_per_m3`, `tint`). |
| `data/scatter.json` | Core Plains/Forest prop rules, visibility distances, and crate placement defaults. |
| `data/spawning.json` | Base spawn caps, distances, day/night zombie weights, and camp/city density modifiers. |
| `data/loot_tables.json` | Loot roll tables referenced by each biome's `loot_table` key. |
| `data/poi.json` | Point-of-interest building definitions and placement rules. |
| `data/settlements.json`, `data/city.json`, `data/lake.json` | Core camp placement, Ashford city layout, and lake/village generation. |

## 3. How to Add a New Biome

1. **Enum (`world/src/biome/mod.rs`):** Add the new `#[repr(u8)]` variant to `BiomeId`, update `BiomeId::COUNT` and `BiomeId::ALL`, and add its snake_case string to `BiomeId::key()` and `BiomeId::from_key()`.
2. **Biome Data (`data/biomes.json`):** Add an entry matching the new `key` with ground colors, height settings, scatter rules, temperatures, and gameplay multipliers. (`BiomeRegistry::load()` validates that every `BiomeId` variant is present.)
3. **World Sector (`data/world_layout.json`):** Add `{ "biome": "<key>", "center_deg": <angle> }` to `sectors` and adjust neighboring sector angles.
4. **Weather Table (`data/weather.json`):** Add a weighted weather list under `biomes.<key>` and a `_biomes_why.<key>` note. (`WeatherData::load()` validates that every `BiomeId` variant has a non-empty list.)
5. **Assets & Gameplay Tables:** Export any new prop `.glb` models in `tools/blender/` + `assets/models/props/` and add corresponding loot entries in `data/loot_tables.json`.
6. **Verify:** Run `cargo test -p world` to check registry loading, layout invariants, and weather rules.

## 4. Weather

Weather is simulated deterministically in `world::weather` (`WeatherKind`, `WeatherState`, `WeatherData`) using `data/weather.json`:

- **Kinds:** `clear`, `rain`, `snow`, `sandstorm`, and `fog`. Each kind defines `fog_mult`, `visibility`, `noise_mask`, `wind` speed, `particles_per_m3`, and normalized RGB `tint`.
- **Biome Spells:** A spell lasts between `min_spell_seconds` (`240 s`) and `max_spell_seconds` (`720 s`) of game time. When a spell ends, the next kind is drawn from the current biome's weighted list using `DotnetRandom` seeded from the world seed.
- **Smooth Transitions & Biome Crossings:** Switching kinds fades `intensity` smoothly to `0` over `fade_seconds` (`40 s`), swaps `kind`, and fades back to `1`. Entering a biome whose list excludes the active weather (such as bringing a desert `sandstorm` into `snow`) starts fading out immediately and picks a valid weather kind for the new biome.
- **Gameplay Outputs & Wind:** `fog_mult()`, `visibility()`, and `noise_mask()` blend linearly between `clear` and the current kind by `intensity`. Horizontal `wind` (`glam::Vec2` in m/s on XZ) drifts slowly in direction (`~2–4°/min`) with magnitude `kinds[k].wind * intensity`. `WeatherState` derives `Serialize`/`Deserialize` for save files.

## 5. New Halden (Metro City)

New Halden is generated deterministically in `world::metro` from `data/metro.json` and placed on the outer ring connected by the jammed highway:

- **Overgrowth (`world::metro::overgrowth`):** Vines on tower walls, weeds in street cracks, asphalt trees, moss patches, wild lot shrubs/saplings/tall grass, and roof bushes/trees, densest in `edge` and sparser in `core`.
- **Major Damage (`world::metro::damage`):** One fallen tower section lying across a midtown street with walkable rubble mounds, one leaning core tower, and one burnt-out midtown block with blackened floor overlays, ash patches, and burnt car piles.
- **Survivor Traces (`world::metro::traces`):** Stacked-car street barricades with a walkable gap, facade spray-painted warnings (`graffiti_*`), missing-person posters (`poster_wall`), rooftop survivor camps, sidewalk bodies under sheets, and lobby interior dressing.
- **Tower Variety (`world::metro::tower_parts`):** Damaged `_floor_c` modules (`towers.damagedShare = 0.25`, `DamageMode::All` in the burnt block) and rebuilt crowns with NY water tanks, machinery, stair huts, and antennas.
- **Test Flags:** `--metro=<edge|core|highway|town_view|top|lobby|landmark|fallen|leaning|burnt|carwall|camp|herd|crows|crows_up|army|night>` and `--nvg`.
- **R41 (Zombies, Loot & Silent Life):**
  - **Metro Spawner & Herds:** District-scaled zombie density/caps (`edge` < `midtown` < `core`), night runner dominance (`data/spawning.json` `metro`), and slow avenue walker herds (`world::metro::herds` + `game::StreetHerds`) that aggro when noticing the player.
  - **District Loot:** 7 `metro_*` loot tables (`edge`, `midtown`, `core`, `army`, `medical`, `camp`, `car`) with `army_crate`, `metro_locker`, `med_cabinet`, and `camp_stash` containers plus 12 city-only items (armor, `army_rifle`, `night_vision`, rare meds, repair parts).
  - **Silent Life:** Power poles with sagging/fallen wires, weathered billboards, flapping torn flags/plastic, and startling crow flocks (`world::metro::life`, `game::crows`).
- **Perf Rule:** Measured with `tools/perf_scenes.sh --set metro --seconds 20`; pass requires average FPS >= 20 (1% low logged for information only).

### R42: Big Places & Failed Army Line
- **Hospital, Police Station, Fire Station:** Placed on their 2x2 reserved block groups (`world/src/metro/places.rs`, `data/places/{hospital,police,fire_station}.json`). Each has a walk-in ground floor building facing an avenue (`metro_hospital`, `metro_police`, `metro_fire_station`) and three themed yards (ambulance bay/triage/parking, police lot/barricade/impound, training/staging/overgrown).
- **Failed Army Line:** Placed on the `army_line` 4x1 strip and its avenue (`world/src/metro/army_line.rs`, `data/places/army_line.json`): sandbag/concrete barrier with razor wire and a breach, tank, burnt trucks, crashed helicopter, evac tents, loudspeakers, and a once-per-day brute.
- **Looks & Danger:** Four themed zombie looks (`zombie_soldier`, `zombie_police`, `zombie_medic`, `zombie_firefighter`) assigned per POI building via `zombieModels` and `spawn_kind_look`; `metro.placeDanger` boosts spawn density and special chance within 90 m.

### R43: Climbing, Rooftops, Fire Escapes & Radio
- **Climbable Towers (`world::metro::climb`):** 4 standing towers (2 Midtown, 2 Core) plus the tallest landmark have walk-in stairwells (`tw_stair_open`, `tw_stair_flight`, `tw_stair_top`), dressed stop-off floors (`tw_interior_{office,flat,camp}_{24,32}` with `metro_tower_*` loot and height-gated POI zombies via `wakeHeightM`), and a roof hatch ladder to the deck.
- **Rooftops & Fire Escapes (`world::metro::rooftops`, `fire_escapes`):** Plank bridges (`roof_plank_{4,8,12}`), step crates, and wall ladders link climbable/fire-escape roofs to nearby roofs. 8 Edge/Midtown brick towers have working fire escapes (drop ladder, zig-zag stair colliders, gooseneck roof ladder).
- **Radio Goal (`game_world::radio`, `data/radio.json`):** On the tallest landmark's mast platform (+18 m), `[E]` at the `radio_console` plays the survivor broadcast, unlocks `metro_radio_crate`, and saves `systems.extra["metro_radio"]`. Big map labels show `"Radio Mast"` and `"Stairs"`.
