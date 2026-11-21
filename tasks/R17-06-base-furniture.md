# R17-06: player-built base furniture and defences

Read first: `tasks/R17-common.md`, `data/build_parts.json`, `game/src/building/state.rs` (`world_transform`), `tools/blender/props/crate_loot.py` + `barrel_rusty.py` (prop style), `tools/blender/common.py`.
Files: tools/blender/building_parts/base_bed.py, tools/blender/building_parts/base_storage_box.py, tools/blender/building_parts/base_workbench.py, tools/blender/building_parts/base_light.py, tools/blender/building_parts/base_spikes.py, tools/blender/building_parts/base_barricade.py, assets/models/building_parts/base_bed*, assets/models/building_parts/base_storage_box*, assets/models/building_parts/base_workbench*, assets/models/building_parts/base_light*, assets/models/building_parts/base_spikes*, assets/models/building_parts/base_barricade*, assets/models/_previews/base_bed*, assets/models/_previews/base_storage_box*, assets/models/_previews/base_workbench*, assets/models/_previews/base_light*, assets/models/_previews/base_spikes*, assets/models/_previews/base_barricade*, tasks/R17-06-base-furniture.md

Another worker (R17-05) creates the folder `tools/blender/building_parts/` and its `build_parts.py`; you only add your own scripts (Claude adds them to the list). Use category `building_parts` the same way (if the folder/category doesn't exist in `common.py`, write the files to `assets/models/building_parts/` the way R17-05 will: read `common.run_model` and pass what it needs; explain in notes).
- `base_bed` (mattress on a pallet frame, blanket, pillow) 400; `base_storage_box` (wooden chest, lid as a child mesh `lid` hinged at the back) 300; `base_workbench` (rough plank bench, saw, hammer, nails box) 600; `base_light` (hanging lantern on a post, `light_1` empty at the flame) 300; `base_spikes` (row of sharpened stakes 2 m wide, angled outward) 400; `base_barricade` (planks nailed across 2 m, tyres and a door piece) 500.
- Sizes fit the 2 m grid cell.

## Worker notes
- Created 6 player-built base furniture and defence models in `tools/blender/building_parts/`:
  - `base_bed` (`tools/blender/building_parts/base_bed.py`): 348 tris (budget 400). Wooden pallet frame (3 runners, 9 blocks, 8 top slats), mattress with piped rim, pillow at head (+Y), and blue blanket draped over sides and foot with folded white cuff. Has convex collider `base_bed_bed-convcolonly`.
  - `base_storage_box` (`tools/blender/building_parts/base_storage_box.py`): 216 tris (budget 300). Wooden chest with bottom skids, iron corner brackets, perimeter strap, brass latch, side handles. Separate child mesh `lid` exported as node `lid` with wood slats, metal straps, front clasp, and back hinges at Y=+0.31, Z=0.42. Has convex collider `base_storage_box_box-convcolonly`.
  - `base_workbench` (`tools/blender/building_parts/base_workbench.py`): 396 tris (budget 600). Heavy wooden workbench with 4 legs, under-shelf with planks, 3 thick tabletop planks, backstop board, side woodworking vise with screw & T-handle, handsaw with teeth strip, claw hammer, and open nails box with hardware. Has convex collider `base_workbench_bench-convcolonly`.
  - `base_light` (`tools/blender/building_parts/base_light.py`): 238 tris (budget 300). Wooden cross base on ground, upright post (1.75m), overhead arm with 45-degree diagonal brace, hook, and hanging hurricane lantern with wire bail, red cap, glass globe, flame, red fuel tank base, and wire guards. Contains `light_1` empty marker at flame `(0.20, 0.0, 1.40)` and convex collider `base_light_post-convcolonly`.
  - `base_spikes` (`tools/blender/building_parts/base_spikes.py`): 252 tris (budget 400). 2m wide defensive spike row. Heavy base timber beam, 4 rear ground kickstand braces, 6 sharpened wooden stakes angled outward (-Y) with whittled pyramid tips and subtle organic jitters, crossbar binding rail, and lashed metal strap bindings. Has convex collider `base_spikes_spikes-convcolonly`.
  - `base_barricade` (`tools/blender/building_parts/base_barricade.py`): 388 tris (budget 500). 2m wide survivor barricade. 2 vertical posts with rear angled kickstands, framing rails, salvaged red door panel with recessed panels and brass doorknob, rough planks nailed across (horizontal, diagonal, kicker, jagged top), 2 low-poly hollow car tyres (`tire_black`) stacked/leaning at left base, rusty reinforcement plate and bolt studs. Has convex collider `base_barricade_barrier-convcolonly`.
- Wiring notes for Claude:
  - All 6 models use category `building_parts` and were exported to `assets/models/building_parts/<name>.glb`.
  - Previews generated in `assets/models/_previews/<name>.png`.
  - Sizes fit the 2m grid cell; origins grounded at Z=0.
  - When R17-05 adds `tools/blender/building_parts/build_parts.py`, these 6 scripts can be added to its build list.
- Verification:
  - `python3 -I tools/gate_checks.py --base main --task tasks/R17-06-base-furniture.md`: scope PASS, numbers PASS, unused PASS.


