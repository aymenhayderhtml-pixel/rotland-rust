# R17-08: put the lake & coast into the world and draw its water

Read first: `tasks/R17-common.md`, `world/src/lake/` (the pure layout, done), `world/src/generator.rs` (how `attach_city`, `city_ground`, `sample`, `emit_triangle` and `chunk_contents` add the city: do the same for the lake), `world/src/city/place.rs` (how props are placed per chunk), `data/lake.json`, `data/poi.json`, `engine/src/render/water.rs` (`WaterPatch`, `WaterRenderer`), `rotland/src/water_view.rs` + its call in `rotland/src/app/render_frame.rs` (today it only draws a `--water-test` patch).
Files: world/src/generator.rs, world/src/lake/*, world/src/lib.rs, data/lake.json, data/poi.json, rotland/src/water_view.rs, rotland/src/app/render_frame.rs, rotland/src/main.rs, tasks/R17-08-lake-in-world.md
Test: lake

1. `WorldGenerator::from_data_dir` loads `data/lake.json` (if present) and attaches a `LakeLayout` after the city (opposite the city). Accessor `lake()`.
2. `sample`: apply `lake.ground_height(p, h)` (with the `contains_region` early-out, like the city). The lake must not overlap the city or the camp (add a test).
3. `emit_triangle`: paint inside the lake region only: beach = sand (light warm beige), wet sand near the waterline (darker), lake bed under water = muddy dark green-brown (seen through the water). Colors in `data/lake.json`. The grass outside the region stays EXACTLY as it is (the existing golden tests must still pass).
4. `chunk_contents`: no scatter props in water (trees/rocks/grass), keep them on land; the village lots become POIs (building ids picked per lot: `fish_shack`, `boathouse` (only for lots touching the water), `house_small`, `cabin`, `shed`; add `fish_shack` and `boathouse` to `poi.json` with `chancePerChunk` 0, loot tables `house`/`barn` for now, zombies [1,2]); each dock = `dock_section` props every 4 m along it + a `dock_end`; beach props (`rowboat_flipped`, `beach_umbrella`, `beach_chair`, `fishing_rack`) a few per 100 m of beach; `buoy` and `rowboat` on the water near docks. Prop ids are final even if a model is missing in the pack today (the game skips missing models).
5. Water drawing: replace the test-only patch with the real lake: `water_view` asks `world_gen.lake()` for its water patch(es) (one patch over the lake's bounding box at `water_level`; the terrain under the water is lower so only the lake shows) and draws them every frame when the lake is near (skip when the camera is > 600 m from the lake box). Keep `--water-test` working.
6. Tests (`lake_` names, in `world/src/lake/` test files): lake attached in `from_data_dir(42, data)`; far from city and camp; ground under deep water is below `water_level`; beach paint is sand-colored; a chunk in the water has no trees; dock props line up with the docks; same seed same lake.

## Worker notes
- files changed (11): `data/lake.json`, `data/poi.json`, `world/src/lake/config.rs`, `world/src/lake/layout.rs`, `world/src/lake/place.rs`, `world/src/lake/mod.rs`, `world/src/lib.rs`, `world/src/generator.rs`, `rotland/src/water_view.rs`, `world/src/lake/tests.rs`, `tasks/R17-08-lake-in-world.md`
- implemented lake & coast integration into the world:
  - `data/lake.json`: Added `sand`, `wetSand`, and `lakeBed` RGB color arrays.
  - `data/poi.json`: Added `fish_shack` and `boathouse` building definitions (`chancePerChunk: 0.0`, loot tables `house`/`barn`, zombies `[1, 2]`).
  - `world/src/lake/config.rs`: Added `sand`, `wet_sand`, and `lake_bed` fields to `LakeConfigData` with default fallback functions and source tags.
  - `world/src/lake/layout.rs`: Added `building_id: String` to `VillageLot`, implemented `water_bounds()` AABB query, added `sand_color()`, `wet_sand_color()`, `lake_bed_color()`, and terrain `paint()` method to blend lake bed under water into wet sand at the shoreline, dry warm sand along the beach, and smoothly blend into surrounding grass at the beach edge.
  - `world/src/lake/place.rs`: Placed dock sections every 4 m and dock ends at dock tips, placed buoys and rowboats on the water near docks, and scattered beach props (`rowboat_flipped`, `beach_umbrella`, `beach_chair`, `fishing_rack`) along the shoreline.
  - `world/src/lake/mod.rs` & `world/src/lib.rs`: Exported `place_lake_props` and wiring modules.
  - `world/src/generator.rs`: Loaded `data/lake.json` and attached `LakeLayout` after city in `from_data_dir()`, added accessor `lake()`, applied `lake.ground_height()` in `sample()`, applied `lake.paint()` in `emit_triangle()`, filtered wild POIs and props from water, attached village lot POIs, and placed lake props in `chunk_contents()`.
  - `rotland/src/water_view.rs`: Updated `draw()` to ask `world_gen.lake()` for its water patch over its bounding box at `water_level`, culling when camera is > 600 m away from the lake box, while keeping `--water-test` flag working.
  - `world/src/lake/tests.rs`: Added 7 comprehensive unit tests verifying lake attachment, separation from city and camp, underwater bed height, sand coloring on beach, absence of trees in water, dock prop alignment, and seed determinism.
- how to test:
  Run `tools/gate.sh --task tasks/R17-08-lake-in-world.md`
- wiring notes for Claude:
  - `WorldGenerator::lake()` exposes `&LakeLayout`.
  - `water_view::draw` dynamically draws the lake water patch at `water_level` when camera distance to lake box is <= 600m.
  - Village building definitions and prop instances use final IDs matching R17-07 (`fish_shack`, `boathouse`, `dock_section`, `dock_end`, `rowboat`, `rowboat_flipped`, `beach_umbrella`, `beach_chair`, `fishing_rack`, `buoy`).
- known issues: None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (11): data/lake.json, data/poi.json, rotland/src/water_view.rs, tasks/R17-08-lake-in-world.md, world/src/generator.rs, world/src/lake/config.rs, world/src/lake/layout.rs, world/src/lake/mod.rs, world/src/lake/place.rs, world/src/lake/tests.rs, world/src/lib.rs
- tests passed: 18
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
