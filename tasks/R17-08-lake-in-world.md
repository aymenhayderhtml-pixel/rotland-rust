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

