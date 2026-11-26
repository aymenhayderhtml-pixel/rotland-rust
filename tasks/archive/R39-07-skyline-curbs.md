# R39-07: Skyline window bands and haze tint; raised curbs replace the painted sidewalk edge

Owns: rotland/src/metro_render/skyline.rs, rotland/src/metro_render/boxes.rs (the FAR path only: `Detail::Far`, `tower_mesh` far branch and its helpers; do not touch the near code, R39-06 owns that), rotland/src/metro_render/curb_mesh.rs (new: GPU upload of the curb chunk mesh), world/src/metro/curbs.rs (new: pure mesh builder), world/src/metro/paint.rs (the sidewalk band colour only), world/src/metro/paint_tests.rs if it exists (else the tests in paint.rs), data/metro.json (ONLY the `look` keys `curbHeight`, `curbWidth`, `skylineBandShare` and the haze keys you need; `skyline` is not changed). Shared files (`world/src/metro/mod.rs`, `world/src/lib.rs`, `rotland/src/streaming.rs`, `rotland/src/main.rs`, `rotland/src/app/*`, `Cargo.toml`) belong to the lead: report the exact lines.
Read: tasks/R39-common.md ("Skyline", "Sidewalks"), tasks/archive/R38-06-metro-render.md (Worker notes), rotland/src/metro_render/skyline.rs (`SkylineBatch::rebuild_if_needed`, `skyline_vertices`, `draw_items(fog_scale)`), rotland/src/metro_render/boxes.rs (`tower_mesh`, `Detail::Far`, `roof`, `closed_box`, `to_world`), rotland/src/metro_play.rs (`MetroPlay::haze`, `update_skyline`), rotland_ui/src/metro_ui/haze.rs (`MetroHazeConfig` fields `color`, `skyline_fog_end`, `load_default`), world/src/metro/paint.rs (`MetroLook`, `city_surface`, `metro_ground_color`, its tests), world/src/metro/layout.rs (`MetroStreet { a, b, width, avenue }`, `contains_point`, `closest_point`, `MetroLayout::streets`, `MetroLayout::block_at_point`), world/src/metro/state.rs (`MetroWorld::paint_ground`), world/src/terrain_paint.rs (line ~184, where the metro paint is applied), rotland/src/streaming.rs (`terrain::chunk_vertices`, `LoadedChunk`), rotland/src/terrain.rs (`chunk_vertices`), physics/src/mover.rs (`max_step_height`, line ~25: 0.35 m), world/src/metro/tower_parts.rs and styles.rs (R39-01: `StyleDef.color`), data/metro.json.

## Part A: skyline window bands and haze tint (`skyline.rs`, far path of `boxes.rs`)
- Far towers get dark **window bands**: one band per floor (use the `floors` count; each floor band is a darker strip on each face, with a gap between floors). Still ONE batch (`SkylineBatch`, one mesh, one draw item). Keep the triangle cap `skyline.trisPerTower` (60 in data): bands are drawn as quads per face, not per window, so the cap holds. If a tower exceeds the cap, use fewer bands (every second floor), never more triangles.
- Wall colour: the style colour `StyleDef.color` (from R39-01), instead of the single `look.far_color`. Landmarks keep the Deco colour.
- **Haze tint**: each far colour is blended toward the haze colour (`MetroHazeConfig.color`, Vec3 sRGB from `rotland_ui::metro_ui`) by a share in data (`look.skylineHazeShare`, start 0.5). The haze colour comes in through `SkylineInputs` (add a `haze: Vec3` field) so the skyline stays pure. Tint is baked into the vertex colours, so there is no new shader.
- Skyline stays out of the shadow pass and stays inside `draw_items`; `rebuild_if_needed` keeps its key logic.
- `--metro=highway` and `--metro=town_view` must now read as a city with window bands in the haze.

## Part B: raised curb and sidewalk mesh (`world/src/metro/curbs.rs`, `curb_mesh.rs`)
- **Pure builder** `build_curbs(layout: &MetroLayout, look: &MetroLook) -> Vec<CurbTri>` (or positions and colours, whichever matches `ColorVertex` use). For every `MetroStreet`, two sidewalk strips run along both edges. Each strip is the band from the asphalt edge (`width/2 - sidewalkWidth`) to the street edge (`width/2`), raised by `curbHeight` (0.15 m, data) above the ground of the street.
  - Straight edges: a top quad of the sidewalk, plus a curb face (vertical, `curbHeight` tall) on the asphalt side and a thin kerb top stone.
  - Corners (where two streets meet, or a street ends at a block): the sidewalk stops at the intersection; each open end gets a **ramp**: the top slopes down to ground over `curbWidth` (data, about 1.0 m). Avenues (`avenue: true`) use the same rule.
  - Kerb stones: thin dark lines every `kerbStoneLength` (data, about 1.0 m) on the curb face, so the edge reads as stones.
  - Do not build sidewalk over a park block (`Reserved::Park`): the park edge keeps the grass.
- Output is positions, normals and colours in world space for the chunk (clip to the chunk box, as the ground does with its triangles). Colour = `look.sidewalk` for the top, a darker share for faces.
- **Chunk mesh**: `curb_mesh.rs` builds one `GpuMesh` per loaded chunk from `build_curbs` filtered to the chunk, uploaded like `LoadedChunk.ground`. It is drawn with the chunk's ground (same draw path, no new shader).
- **Colliders**: none for the curbs. Reason (check and write it in Worker notes): the player's step height is `max_step_height` 0.35 m (`physics/src/mover.rs`, default in `CharacterMover`), and 0.15 m is below it, so the player steps up with no collider.

## Part C: remove the jagged painted sidewalk (`paint.rs`)
- In `city_surface`, the sidewalk band no longer paints `look.sidewalk`. The band is painted the lot colour or the base colour under it, so the painted asphalt stays (dist <= `width/2 - sidewalkWidth`) and the raised curb mesh gives the clean edge. Keep the asphalt rule unchanged.
- Update the test `the_sidewalk_band_is_sidewalk` (it tested the old paint) to the new rule: the band is not `look.sidewalk`, and the asphalt centreline is still asphalt. The other paint tests stay.

## Tests
- `curbs.rs` (pure, in-module or sibling test file): a straight street gives two strips with the right length, edges and heights (top at `curbHeight`, base at 0); a corner gives ramps at the open ends; a park block gets no sidewalk; every triangle is in the street band; the total is deterministic.
- Skyline: one batch per rebuild; window bands add no more than `trisPerTower` triangles per tower; the tint moves a colour toward the haze colour (a tower at haze colour stays haze colour); landmarks use Deco.
- `paint.rs`: the updated sidewalk test; asphalt centreline test unchanged.
- The existing skyline tests in `rotland/src/metro_render/tests.rs` (owned by R39-06 for the near part) still pass; if a test breaks because of the new band colours, change only that expected value and say so in Worker notes.
- `world/tests/golden.rs` passes UNEDITED.

## Done when
- The far skyline has window bands per floor, style colours, and a haze tint, still in one draw.
- Sidewalks are straight raised 15 cm curbs with ramps at the open ends and kerb lines; no jagged painted edge; asphalt paint stays.
- The step-height reason for having no curb colliders is written in Worker notes.
- `tools/gate.sh --task tasks/R39-07-skyline-curbs.md` ends with `GATE: PASS` (workers only).
- Shared-file lines for the lead (`world/src/metro/mod.rs`: `pub mod curbs;`; `rotland/src/streaming.rs` and the draw call that draws `curb_mesh` with the ground; `rotland/src/metro_render/mod.rs`: `pub mod curb_mesh;`) are in Worker notes.

## Worker notes
### What changed
- `data/metro.json`: added `look.curbHeight` (`0.15`), `look.curbWidth` (`1.0`), `look.kerbStoneLength` (`1.0`), and `look.skylineHazeShare` (`0.5`).
- `rotland/src/metro_render/boxes.rs` (far path only): added `far_tower` and `far_window_bands` using `StyleDef.color` (Deco for landmarks) and dark `look.glass_dark` window bands per floor, stepping floors when needed to stay within `skyline.trisPerTower` (60).
- `rotland/src/metro_render/skyline.rs`: added `haze: Vec3` to `SkylineInputs`, `tint_toward_haze` / `tint_skyline_vertices` applied in `SkylineBatch::rebuild_if_needed`, and unit tests for window bands, Deco landmarks, and haze tinting.
- `world/src/metro/curbs.rs` (new): pure `build_curbs` and `build_curbs_in_chunk` builders emitting raised sidewalk tops (`curbHeight`), vertical asphalt-side curb faces with `kerbStoneLength` seam lines and thin kerb top stones, `curbWidth` ramps at open intersection/block ends, and skipping `Reserved::Park` edges; includes unit tests.
- `rotland/src/metro_render/curb_mesh.rs` (new): `curb_chunk_vertices` and `curb_chunk_mesh` uploading one `GpuMesh` per chunk.
- `world/src/metro/paint.rs`: added the 4 new `MetroLook` fields + validation, removed `look.sidewalk` painting in `city_surface` (leaving asphalt unchanged and letting the sidewalk band take the lot/base colour), and updated `the_sidewalk_band_is_sidewalk`.

### Shared-file lines for the lead
1. `world/src/metro/mod.rs`:
   - `pub mod curbs;`
   - `pub use curbs::{CurbTri, build_curbs, build_curbs_in_chunk};`
2. `rotland/src/metro_render/mod.rs`:
   - `pub mod curb_mesh;`
3. `rotland/src/metro_play.rs` (in `MetroPlay::update_skyline`, inside `SkylineInputs { ... }`):
   - `haze: self.haze.color,`
4. `rotland/src/streaming.rs`:
   - Import: `use crate::metro_render::{curb_mesh::curb_chunk_mesh, hulls::tower_hulls, near::near_tower_mesh};`
   - `LoadedChunk` field: `pub curbs: Option<GpuMesh>,`
   - In `Streamer::finish_chunk` (next to `towers`/`hulls`):
     `let curbs = self.world_gen.metro_world().and_then(|m| curb_chunk_mesh(device, &m.layout, &m.look, b.coord));`
     and `curbs,` inside `LoadedChunk { ... }`.
5. `rotland/src/app/render_frame.rs` (in the chunk `flat_map` around line 178):
   - `let curbs = c.curbs.as_ref().map(|mesh| DrawItem { mesh, offset: glam::Vec3::ZERO, fog_scale: 1.0 });`
   - `std::iter::once(ground).chain(curbs).chain(towers)`

### Why curbs have no colliders
`CharacterMover::default().max_step_height` is `0.35` m (`physics/src/mover.rs:25`), and `look.curbHeight` is `0.15` m (`0.15 < 0.35`), so the player steps onto the raised sidewalk automatically without needing curb colliders.

### Notes
- Did not run `cargo` or `tools/gate.sh` (per subagent rules).
