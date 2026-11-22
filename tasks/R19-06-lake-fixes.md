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
