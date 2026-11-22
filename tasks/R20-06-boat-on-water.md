# R20-06: rowboat sits in the water, not above it (issue 008)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` row 008 (+ pictures `docs/issues/008-*.png`), `world/src/lake/place.rs` (~line 60-85, buoy and rowboat at `lake.water_level`), `engine/src/render/water.wgsl` (waves move the surface), `tools/blender/props/rowboat.py`, `data/lake.json`.
Files: world/src/lake/*, data/lake.json, tools/blender/props/rowboat.py, assets/models/props/rowboat.glb, assets/models/_previews/rowboat*, tasks/R20-06-boat-on-water.md
Test: lake

Claude checked: the rowboat model's lowest point is at y = 0 (its keel), and it is placed exactly at the water level, so it sits ON the water with no depth in it, and in the picture it looks like it floats above. The hull also looks thin/open from above.
1. Sink floating props by a draft from `data/lake.json` (new key, e.g. `boatDraft` ~0.18 m for the rowboat, `buoyDraft` for the buoy) so the waterline is partway up the hull. Make sure the wave height (`wave_amplitude` in the water settings) cannot lift the water above the gunwale or drop it below the keel.
2. Check the rowboat hull in `rowboat.py`: every hull face must be closed and face outward (inside faces too, so you see the inside from above). Fix holes/flipped faces; keep under its triangle budget. Rebuild and look at the preview.
3. Test: rowboat and buoy y are below `water_level` by their draft.

## Worker notes
