# R39 round: New Halden towers. Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/WORLD.md, tasks/R39-*
Rules: `tasks/LEAD.md` + `tasks/R39-common.md` (owner decisions + "R39 shared names": follow them EXACTLY).

## Phase 0 (lead)
Baseline: `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` -> `/tmp/r39/perf_before.txt`. Shots before: `--metro=core`, `--metro=edge`, `--metro=top` (hour 10) -> `/tmp/r39/before_*.png`.

## Tasks (8)
| Task | What | after |
|---|---|---|
| `R39-01-tower-plan.md` | styles data, tower style/size on TowerSpot, `tower_parts` (stacking, setbacks, landmarks, lobby facing the street) | none |
| `R39-02-kit-brick-stone.md` | Blender: Brick + Stone styles (base/floor/floor_b/crown x sizes + LOD1) | builds after 04 + 05 |
| `R39-03-kit-glass-slab.md` | Blender: Glass + Slab styles | builds after 04 + 05 |
| `R39-04-kit-deco-landmarks.md` | Blender: Deco style + 4 landmark crowns + setback sections | none |
| `R39-05-lobbies.md` | Blender: lobby props, shared lobby shell (`towers/shell.py`), new palette colours, `build_r39_models.py` | none (first) |
| `R39-06-tower-render.md` | rotland: draw tower parts instanced per chunk with LODs, shadows only LOD0, colliders (walk-in lobby), remove near placeholders | after 01 |
| `R39-07-skyline-curbs.md` | skyline window bands + haze tint; raised curb/sidewalk mesh replacing jagged paint | none |
| `R39-08-flags-wiring.md` | `--metro=lobby|landmark`, `--tower-style=`, `--map`; perf scene; README lines; R38 leftovers (bus_city_side underside) | after 01-07 |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r39-saves`, pictures to /tmp/r39/)
- `--metro=core`, `--metro=edge` at hour 10 and 18: street canyons look like a real dead New York (cornices, fire escapes, broken windows, water tanks), no floating or gapped floors, no z-fighting flicker.
- `--metro=landmark`: the landmark crown reads from the street; `--metro=top`: varied skyline with setbacks.
- `--metro=lobby`: walk-in lobby with desk, elevators, mailboxes, rubble; door faces the street; you can walk in and out (no invisible wall).
- `--tower-style=brick|stone|glass|slab|deco` with `--metro=core`: one picture each.
- `--metro=highway` and `--metro=town_view`: the skyline now has window bands and reads as a city in the haze.
- Sidewalks: straight raised curbs, no jagged edges.
- `--map`: New Halden outline + streets on the big map.
- Camp, Ashford (`--at=-384,1792`), `--town=blackwater` look as before.
- FPS: `tools/perf_scenes.sh --set all --seconds 20` -> `/tmp/r39/perf_after.txt`; every scene's 1% low >= 20 (metro included). If metro is under 20, lower `towers.lod0Distance` first, then LOD1 triangles; never strip the close detail.

## Lead notes
- **Tasks done (8/8):** R39-01 (`styles.rs`, `tower_parts.rs`, `towers.rs`, `config.rs`, `data/metro.json`), R39-02 (`brick.py`/`brick_parts.py`, `stone.py`/`stone_parts.py`, 32 GLBs), R39-03 (`glass.py`, `slab.py`, 32 GLBs), R39-04 (`deco.py`, `landmarks.py`, `roof_parts.py`, 24 GLBs), R39-05 (`shell.py`, `lobby_props.py`/`lobby_props_b.py`, `stack_preview.py`, `build_r39_models.py`, 6 lobby GLBs), R39-06 (`parts.rs`, `near.rs`, `hulls.rs`, `props.rs`, `streaming.rs`), R39-07 (`curbs.rs`, `curb_mesh.rs`, `skyline.rs`, `boxes.rs`, `paint.rs`), R39-08 (`metro_flags.rs`, `test_flags.rs`, `README_TOOLS.md`, `jam_bus.py`).
- **Review & lead fixes:**
  - `world/src/metro/tower_parts.rs`: clamped `snap_setback_size` to `style.sizes` so Deco setbacks never emit non-existent `tw_deco_16_floor`.
  - `rotland/src/streaming.rs`: split `metro_world()` borrow before `self.shapes_of(pack, id)` and wired `curbs` + `place_chunk_towers`.
  - `rotland/src/metro_render/boxes.rs`, `near.rs`, `rotland/src/app/metro_flags.rs`: resolved 3 clippy warnings (`Detail::Near` `#[allow(dead_code)]` for R38 tests, `place_chunk_towers` taking `&MetroWorld`, lifetime elision).
  - `world/src/metro/curbs.rs`: fixed `single_street_layout` test helper so `l.grid` stays in sync with `l.blocks`.
  - `rotland/src/app/test_flags.rs`: fixed `--metro=lobby` spawn height (`ground_y + 0.15`) so the player spawns inside the ground-floor lobby instead of on the roof collider.
  - `tools/blender/palette.py`: added the 13 R39 `COLORS_C` entries and regenerated `assets/textures/palette*.png`.
- **Blender & baker:** `tools/blender/build_r39_models.py` (`R39 BUILD OK`, all 10 `stack_*.png` gap-free), `tools/blender/metro/jam_bus.py` (0 Traceback), `cargo run --release -p baker` (379 models -> `assets/models.pack`, 52.91 MB).
- **Last 5 gate lines (`tools/gate.sh --task tasks/R39-round.md`):**
  - `clippy: PASS`
  - `test: PASS (1385 passed)` (including `world/tests/golden.rs` unedited)
  - `scope: FAIL (1 outside task Files: assets/textures/palette_c.png)` — `base_files` reads `main:tasks/R39-round.md` which omitted `assets/textures/*`, though `R39-05` required regenerating `palette_c.png` for the new `COLORS_C` tower colors.
  - `numbers: PASS (16 checked)`
  - `unused: PASS`
- **FPS before -> after (`tools/perf_scenes.sh --set all --seconds 20`, `/tmp/r39/perf_before.txt` -> `/tmp/r39/perf_after.txt`):**
  - `metro`: 96.1 avg / 21.8 1% low -> 51.1 avg / **27.8** 1% low (107 draws, 38 shadow)
  - `city`: 71.8 avg / 28.0 1% low -> 78.2 avg / **34.3** 1% low
  - All 12 scenes >= 26.4 FPS 1% low (min `camp` 26.4).
- **Pictures (`/tmp/r39/`):** `before_{core,edge,top}.png`, `after_{core_10,core_18,edge_10,edge_18,landmark,top,lobby,style_brick,style_stone,style_glass,style_slab,style_deco,highway,town_view,map,camp,blackwater}.png`; model previews in `assets/models/_previews/{tw_*,lobby_*,stack_*}.png`.
- **For Claude to check:**
  - `assets/textures/palette_c.png` is updated for the 13 R39 colors (note `R39-round.md` `Files:` on `main` lacked `assets/textures/*`).
  - Re-bake `assets/models.pack` after merging `r39` into `main`.

## Gate report
- date: 2026-11-26
- base: main
- files changed (145): assets/models/buildings/lobby_bench.glb, assets/models/buildings/lobby_desk.glb, assets/models/buildings/lobby_elevators.glb, assets/models/buildings/lobby_mailboxes.glb, assets/models/buildings/lobby_plant_dead.glb, assets/models/buildings/lobby_rubble.glb, assets/models/buildings/tw_brick_16_base.glb, assets/models/buildings/tw_brick_16_base_lod1.glb, assets/models/buildings/tw_brick_16_crown.glb, assets/models/buildings/tw_brick_16_crown_lod1.glb, assets/models/buildings/tw_brick_16_floor_b.glb, assets/models/buildings/tw_brick_16_floor_b_lod1.glb, assets/models/buildings/tw_brick_16_floor.glb, assets/models/buildings/tw_brick_16_floor_lod1.glb, assets/models/buildings/tw_brick_24_base.glb, assets/models/buildings/tw_brick_24_base_lod1.glb, assets/models/buildings/tw_brick_24_crown.glb, assets/models/buildings/tw_brick_24_crown_lod1.glb, assets/models/buildings/tw_brick_24_floor_b.glb, assets/models/buildings/tw_brick_24_floor_b_lod1.glb, assets/models/buildings/tw_brick_24_floor.glb, assets/models/buildings/tw_brick_24_floor_lod1.glb, assets/models/buildings/tw_deco_24_base.glb, assets/models/buildings/tw_deco_24_base_lod1.glb, assets/models/buildings/tw_deco_24_crown.glb, assets/models/buildings/tw_deco_24_crown_lod1.glb, assets/models/buildings/tw_deco_24_floor_b.glb, assets/models/buildings/tw_deco_24_floor_b_lod1.glb, assets/models/buildings/tw_deco_24_floor.glb, assets/models/buildings/tw_deco_24_floor_lod1.glb, assets/models/buildings/tw_deco_32_base.glb, assets/models/buildings/tw_deco_32_base_lod1.glb, assets/models/buildings/tw_deco_32_crown.glb, assets/models/buildings/tw_deco_32_crown_lod1.glb, assets/models/buildings/tw_deco_32_floor_b.glb, assets/models/buildings/tw_deco_32_floor_b_lod1.glb, assets/models/buildings/tw_deco_32_floor.glb, assets/models/buildings/tw_deco_32_floor_lod1.glb, assets/models/buildings/tw_glass_24_base.glb, assets/models/buildings/tw_glass_24_base_lod1.glb, assets/models/buildings/tw_glass_24_crown.glb, assets/models/buildings/tw_glass_24_crown_lod1.glb, assets/models/buildings/tw_glass_24_floor_b.glb, assets/models/buildings/tw_glass_24_floor_b_lod1.glb, assets/models/buildings/tw_glass_24_floor.glb, assets/models/buildings/tw_glass_24_floor_lod1.glb, assets/models/buildings/tw_glass_32_base.glb, assets/models/buildings/tw_glass_32_base_lod1.glb, assets/models/buildings/tw_glass_32_crown.glb, assets/models/buildings/tw_glass_32_crown_lod1.glb, assets/models/buildings/tw_glass_32_floor_b.glb, assets/models/buildings/tw_glass_32_floor_b_lod1.glb, assets/models/buildings/tw_glass_32_floor.glb, assets/models/buildings/tw_glass_32_floor_lod1.glb, assets/models/buildings/tw_landmark_glass_top.glb, assets/models/buildings/tw_landmark_glass_top_lod1.glb, assets/models/buildings/tw_landmark_mast.glb, assets/models/buildings/tw_landmark_mast_lod1.glb, assets/models/buildings/tw_landmark_spire.glb, assets/models/buildings/tw_landmark_spire_lod1.glb, assets/models/buildings/tw_landmark_stepped.glb, assets/models/buildings/tw_landmark_stepped_lod1.glb, assets/models/buildings/tw_slab_24_base.glb, assets/models/buildings/tw_slab_24_base_lod1.glb, assets/models/buildings/tw_slab_24_crown.glb, assets/models/buildings/tw_slab_24_crown_lod1.glb, assets/models/buildings/tw_slab_24_floor_b.glb, assets/models/buildings/tw_slab_24_floor_b_lod1.glb, assets/models/buildings/tw_slab_24_floor.glb, assets/models/buildings/tw_slab_24_floor_lod1.glb, assets/models/buildings/tw_slab_32_base.glb, assets/models/buildings/tw_slab_32_base_lod1.glb, assets/models/buildings/tw_slab_32_crown.glb, assets/models/buildings/tw_slab_32_crown_lod1.glb, assets/models/buildings/tw_slab_32_floor_b.glb, assets/models/buildings/tw_slab_32_floor_b_lod1.glb, assets/models/buildings/tw_slab_32_floor.glb, assets/models/buildings/tw_slab_32_floor_lod1.glb, assets/models/buildings/tw_stone_24_base.glb, assets/models/buildings/tw_stone_24_base_lod1.glb, assets/models/buildings/tw_stone_24_crown.glb, assets/models/buildings/tw_stone_24_crown_lod1.glb, assets/models/buildings/tw_stone_24_floor_b.glb, assets/models/buildings/tw_stone_24_floor_b_lod1.glb, assets/models/buildings/tw_stone_24_floor.glb, assets/models/buildings/tw_stone_24_floor_lod1.glb, assets/models/buildings/tw_stone_32_base.glb, assets/models/buildings/tw_stone_32_base_lod1.glb, assets/models/buildings/tw_stone_32_crown.glb, assets/models/buildings/tw_stone_32_crown_lod1.glb, assets/models/buildings/tw_stone_32_floor_b.glb, assets/models/buildings/tw_stone_32_floor_b_lod1.glb, assets/models/buildings/tw_stone_32_floor.glb, assets/models/buildings/tw_stone_32_floor_lod1.glb, assets/textures/palette_c.png, data/metro.json, rotland/src/app/metro_flags.rs, rotland/src/app/render_frame.rs, rotland/src/app/test_flags.rs, rotland/src/metro_play.rs, rotland/src/metro_render/boxes.rs, rotland/src/metro_render/curb_mesh.rs, rotland/src/metro_render/hulls.rs, rotland/src/metro_render/mod.rs, rotland/src/metro_render/near.rs, rotland/src/metro_render/parts.rs, rotland/src/metro_render/skyline.rs, rotland/src/metro_render/tests.rs, rotland/src/props.rs, rotland/src/streaming.rs, tasks/R39-01-tower-plan.md, tasks/R39-02-kit-brick-stone.md, tasks/R39-03-kit-glass-slab.md, tasks/R39-04-kit-deco-landmarks.md, tasks/R39-05-lobbies.md, tasks/R39-06-tower-render.md, tasks/R39-07-skyline-curbs.md, tasks/R39-08-flags-wiring.md, tasks/R39-round.md, tools/blender/build_r39_models.py, tools/blender/metro/jam_bus.py, tools/blender/palette.py, tools/blender/towers/brick_parts.py, tools/blender/towers/brick.py, tools/blender/towers/deco.py, tools/blender/towers/glass.py, tools/blender/towers/landmarks.py, tools/blender/towers/lobby_props_b.py, tools/blender/towers/lobby_props.py, tools/blender/towers/roof_parts.py, tools/blender/towers/shell.py, tools/blender/towers/slab.py, tools/blender/towers/stack_preview.py, tools/blender/towers/stone_parts.py, tools/blender/towers/stone.py, tools/README_TOOLS.md, world/src/lib.rs, world/src/metro/config.rs, world/src/metro/curbs.rs, world/src/metro/mod.rs, world/src/metro/paint.rs, world/src/metro/styles.rs, world/src/metro/tower_parts.rs, world/src/metro/towers.rs, world/src/metro/towers_tests.rs
- tests passed: 1385
- clippy: PASS
- test: PASS
- checks: FAIL
- gate: FAIL (checks)
