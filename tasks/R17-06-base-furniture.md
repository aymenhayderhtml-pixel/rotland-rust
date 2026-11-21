# R17-06: player-built base furniture and defences

Read first: `tasks/R17-common.md`, `data/build_parts.json`, `game/src/building/state.rs` (`world_transform`), `tools/blender/props/crate_loot.py` + `barrel_rusty.py` (prop style), `tools/blender/common.py`.
Files: tools/blender/building_parts/base_bed.py, tools/blender/building_parts/base_storage_box.py, tools/blender/building_parts/base_workbench.py, tools/blender/building_parts/base_light.py, tools/blender/building_parts/base_spikes.py, tools/blender/building_parts/base_barricade.py, assets/models/building_parts/base_bed*, assets/models/building_parts/base_storage_box*, assets/models/building_parts/base_workbench*, assets/models/building_parts/base_light*, assets/models/building_parts/base_spikes*, assets/models/building_parts/base_barricade*, assets/models/_previews/base_bed*, assets/models/_previews/base_storage_box*, assets/models/_previews/base_workbench*, assets/models/_previews/base_light*, assets/models/_previews/base_spikes*, assets/models/_previews/base_barricade*, tasks/R17-06-base-furniture.md

Another worker (R17-05) creates the folder `tools/blender/building_parts/` and its `build_parts.py`; you only add your own scripts (Claude adds them to the list). Use category `building_parts` the same way (if the folder/category doesn't exist in `common.py`, write the files to `assets/models/building_parts/` the way R17-05 will: read `common.run_model` and pass what it needs; explain in notes).
- `base_bed` (mattress on a pallet frame, blanket, pillow) 400; `base_storage_box` (wooden chest, lid as a child mesh `lid` hinged at the back) 300; `base_workbench` (rough plank bench, saw, hammer, nails box) 600; `base_light` (hanging lantern on a post, `light_1` empty at the flame) 300; `base_spikes` (row of sharpened stakes 2 m wide, angled outward) 400; `base_barricade` (planks nailed across 2 m, tyres and a door piece) 500.
- Sizes fit the 2 m grid cell.

## Worker notes

