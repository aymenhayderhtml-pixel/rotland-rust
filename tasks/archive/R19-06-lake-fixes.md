# R19-06: lake fixes: docks at water height, smooth shore water, white line

Read first: `tasks/R19-common.md`, `world/src/lake/*` (`place.rs` docks, `layout.rs` 557 lines), `rotland/src/water_view.rs` (`lake_patches`: 8 m strips), `engine/src/render/water.rs`, `data/lake.json`.
Files: world/src/lake/*, rotland/src/water_view.rs, engine/src/render/water.rs, engine/src/render/water.wgsl, data/lake.json, tasks/R19-06-lake-fixes.md
Test: lake

Seen at `--at=528,-1478` (seed 42, the first dock):
1. Docks, piers and boathouse floors sit at ground height, under the water. Put dock/pier decks at `water_level + deckHeight` (new key in `data/lake.json`, about 0.4 m) and give dock posts enough length to reach the lake bed. Test: every dock's deck y is above water_level.
2. The water edge is blocky (8 m strips). Make the water mesh follow the shore: e.g. 2 m cells near the shore, bigger cells in the open water, or clip each strip to where `depth_at > 0` with the last cell edge moved onto the shore line. Keep the triangle count low (log it; under ~20k for the lake). The beach must stay dry and no water may float over land.
3. A thin white line shows across the screen at `--at=470,-1420`: probably a gap/edge between water strips (T-junctions or a seam showing the sky/fog color). Make the water mesh watertight (shared vertices, no T-junctions). Explain the cause in Worker notes.
4. Split `layout.rs` under 300 lines while you're there.
5. Tests: water mesh has no open edges inside the lake; no water vertex where `depth_at <= 0` except on the shore line.

## Worker notes
- files changed (13):
  - `data/lake.json`: Added `deckHeight: 0.4` and `waterGridCell: 5.0`.
  - `world/src/lake/config.rs`: Added `deck_height` and `water_grid_cell` to `DocksConfig` and `LakeConfigData` with source tags and validation.
  - `world/src/lake/types.rs`: Extracted `LakeZone`, `Dock` (with `deck_y`), `VillageLot` (with `floor_y`), `FishingSpot`, `FishingSpotKind`.
  - `world/src/lake/height.rs`: Extracted `sand_color`, `wet_sand_color`, `lake_bed_color`, `ground_height`, `paint`.
  - `world/src/lake/generate.rs`: Extracted `ellipse_radius`, `generate_village_lots`, `generate_docks`, `generate_fishing_spots`.
  - `world/src/lake/mesh.rs`: Watertight water mesh generation (`LakeWaterMesh`, `build_lake_water_mesh`) via 2D Marching Squares with root-finding on shoreline edge crossings.
  - `world/src/lake/layout.rs`: Trimmed to ~180 lines (< 300 lines), maintaining pure geometry queries and delegates.
  - `world/src/lake/place.rs`: Raised dock sections and dock ends to `dock.deck_y` (`water_level + deckHeight`).
  - `world/src/lake/mod.rs`: Re-exported all submodules and types for full backward compatibility.
  - `world/src/lake/tests.rs`: Added 4 unit tests verifying dock decks above water level, water mesh manifoldness (no open interior edges, all boundary edges on shore), no water vertices on dry land, and triangle count (< 20k).
  - `engine/src/render/water.rs`: Added `draw_all` and `custom_cache` to render cached custom water meshes in a single alpha-blended render pass with frustum culling.
  - `rotland/src/water_view.rs`: Replaced blocky rectangular strips with cached `LakeWaterMesh` triangle list.
- cause of thin white line at `--at=470,-1420`:
  `lake_patches` previously carved the lake into independent 8 m strips along Z and subdivided each strip's arbitrary length by 4 m cells. Because strips on adjacent rows had different starting and ending X coordinates, horizontal seams between rows had mismatched vertices (T-junctions). When vertex displacement waves animated in `water.wgsl` (`world.y += wave_height(...)`), vertices along one row shifted vertically while the adjacent row linearly interpolated across a straight edge between different vertices, tearing open a subpixel slit where the background sky/fog color shone through.
- watertight solution:
  The new water mesh generator in `world/src/lake/mesh.rs` uses a globally aligned 5 m grid clipped to the shoreline using Marching Squares. Edge crossings are computed once and indexed with a hash map so every triangle shares identical vertex indices with its neighbor. Every interior edge is shared by exactly two triangles (zero open interior edges, zero T-junctions, completely watertight). All outer boundary vertices lie exactly on the shoreline (`shore_distance.abs() < 0.05`, `depth_at == 0.0`), preventing water from floating over land and keeping the beach dry.
- triangle count:
  Lake mesh produces ~13,400 triangles (well under the 20,000 budget), logged via `log::info!`.
- how to test:
  Run `tools/gate.sh --task tasks/R19-06-lake-fixes.md`.
- wiring notes for Claude:
  - In `world/src/generator.rs` (around line 202): when placing village lot POIs in chunks, for `lot.building_id == "boathouse"` use `lot.floor_y` (`let y = if lot.building_id == "boathouse" { lot.floor_y } else { self.height(...) };`) so the boathouse floor sits at deck height above water level.
  - In `tools/blender/props/dock_section.py` and `dock_end.py`: current posts extend down -3.0 m; for deeper lake sections, extending posts down to -6.0 m or -8.0 m ensures posts reach lake bed.
- known issues: None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (13): data/lake.json, engine/src/render/water.rs, rotland/src/water_view.rs, tasks/R19-06-lake-fixes.md, world/src/lake/config.rs, world/src/lake/generate.rs, world/src/lake/height.rs, world/src/lake/layout.rs, world/src/lake/mesh.rs, world/src/lake/mod.rs, world/src/lake/place.rs, world/src/lake/tests.rs, world/src/lake/types.rs
- tests passed: 22
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
