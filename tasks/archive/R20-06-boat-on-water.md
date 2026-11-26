# R20-06: rowboat sits in the water, not above it (issue 008)

Read first: `tasks/R20-common.md`, `docs/ISSUES.md` row 008 (+ pictures `docs/issues/008-*.png`), `world/src/lake/place.rs` (~line 60-85, buoy and rowboat at `lake.water_level`), `engine/src/render/water.wgsl` (waves move the surface), `tools/blender/props/rowboat.py`, `data/lake.json`.
Files: world/src/lake/*, data/lake.json, tools/blender/props/rowboat.py, assets/models/props/rowboat.glb, assets/models/_previews/rowboat*, tasks/R20-06-boat-on-water.md
Test: lake

Claude checked: the rowboat model's lowest point is at y = 0 (its keel), and it is placed exactly at the water level, so it sits ON the water with no depth in it, and in the picture it looks like it floats above. The hull also looks thin/open from above.
1. Sink floating props by a draft from `data/lake.json` (new key, e.g. `boatDraft` ~0.18 m for the rowboat, `buoyDraft` for the buoy) so the waterline is partway up the hull. Make sure the wave height (`wave_amplitude` in the water settings) cannot lift the water above the gunwale or drop it below the keel.
2. Check the rowboat hull in `rowboat.py`: every hull face must be closed and face outward (inside faces too, so you see the inside from above). Fix holes/flipped faces; keep under its triangle budget. Rebuild and look at the preview.
3. Test: rowboat and buoy y are below `water_level` by their draft.

## Worker notes
- **Files changed**:
  - `data/lake.json`: Added `boatDraft` (0.18 m) and `buoyDraft` (0.20 m).
  - `world/src/lake/config.rs`: Added `boat_draft` and `buoy_draft` fields to `LakeConfigData` with serde defaults and validation.
  - `world/src/lake/place.rs`: Placed rowboat at `lake.water_level - lake.config.boat_draft` and buoy at `lake.water_level - lake.config.buoy_draft`.
  - `world/src/lake/tests.rs`: Updated `lake_config_load` to check `boat_draft` and `buoy_draft`; added unit test `lake_props_boat_and_buoy_draft` verifying buoy and rowboat elevations are below water level by their draft, and that wave amplitude (0.05 m) neither submerges the gunwale nor drops below the keel.
  - `tools/blender/props/rowboat.py`: Fixed open bow stem seam by welding the bow stem vertex; solidified the hull by 0.04 m to provide closed thickness, interior faces with inward-facing normals visible from above, and gunwale rim caps; painted inner and rim faces; enabled `top_preview=True`.
  - `assets/models/props/rowboat.glb`: Rebuilt model (252 / 500 tris, 0 open edges, manifold & consistent normals).
- **How to test**:
  - Automated tests: `tools/gate.sh --task tasks/R20-06-boat-on-water.md` runs unit tests and gate checks.
  - Model verification: `python3 tools/blender/check_normals.py assets/models/props/rowboat.glb` reports 0 open edges, 0 non-manifold edges, and normals consistent.
  - In-game: Teleport to the lake dock east of camp; observe rowboat floating naturally in the water with its waterline ~0.18 m up the hull, keel fully submerged, gunwales clear above wave peaks, and interior hull faces clearly visible from above without holes or missing faces.
- **Wiring notes for Claude**:
  - None; lake prop placement reads drafts directly from `data/lake.json` and `lake.config`.
- **Known issues**:
  - None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (7): assets/models/props/rowboat.glb, data/lake.json, tasks/R20-06-boat-on-water.md, tools/blender/props/rowboat.py, world/src/lake/config.rs, world/src/lake/place.rs, world/src/lake/tests.rs
- tests passed: 29
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
