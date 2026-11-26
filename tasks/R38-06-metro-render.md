# R38-06: New Halden look (street ground, near towers, far skyline)

Owns: rotland/src/metro_render/ (new folder: mod.rs, boxes.rs (pure box and window-grid mesh), near.rs (per-chunk near tower meshes), hulls.rs (per-tower collider hulls), skyline.rs (far batch), tests.rs; each file under ~300 lines), world/src/metro/paint.rs (new, pure ground colour; tests inside), data/metro.json "look" key (new top-level key, only this task edits it). Engine: engine/src/render/ only if a real gap is found (report it first). Shared lines belong to the lead: report them, do not edit `rotland/src/streaming.rs`, `rotland/src/app/*`, `world/src/terrain_paint.rs`, `world/src/lib.rs`.
Read: tasks/R38-common.md ("Towers", "Ground", "Budgets"), tasks/R38-02-metro-towers.md (`TowerSpot`, `TowerKind`), tasks/R38-07-haze-map.md (`skyline_fog` in rotland_ui/src/metro_ui/haze.rs, read only the signature once merged), world/src/metro/layout.rs (`MetroLayout`, `height_at`, `street_at`, `district_at`), world/src/city/ground.rs (`CityGround::paint`: how Ashford paints streets, sidewalks and lots), world/src/terrain_paint.rs (`TerrainPaintCtx` fields and the `city_ground.paint` call around lines 28-40 and 160-165), data/city_ground.json (`asphalt`, `dirt` keys, read-only), rotland/src/terrain.rs (`chunk_vertices`, `srgb_byte`, `chunk_of`: how chunk ground vertices are built), rotland/src/streaming.rs (`LoadedChunk`, `build` of a chunk with `GpuMesh::new` and compound colliders around lines 250-332), rotland/src/props.rs (`PropLibrary`, `Placed`: do not use it for towers), engine/src/render/mesh.rs (`GpuMesh::new`, `ColorVertex`), engine/src/render/lit.rs (`DrawItem { mesh, offset }`, `LitRenderer::draw`, `Lighting`), engine/src/render/camera.rs (far plane, to make sure the skyline is not clipped at 6000 m), rotland/src/light.rs (night lighting, no city lights).

Goal: the city reads as a big dying grey-green place from far away (the skyline) and close up (dense box towers with window grids, streets with asphalt and sidewalks). The grass OUTSIDE the city stays green (owner rule).

## 1. Ground colour (`world/src/metro/paint.rs`, pure)
- `pub fn metro_ground_color(base: Color, wx: f32, wz: f32, layout: &MetroLayout, look: &MetroLook) -> Color`: outside the skirt (`layout.near` false) returns `base` untouched. Inside: avenue or street surface = asphalt (`look.asphalt`), a sidewalk band `look.sidewalk_width` m along each street edge = sidewalk colour, lot blocks = `look.lot_concrete` (Core and Midtown) or `look.lot_dirt` (Edge); reserved Park blocks keep `base` (grass).
- In the skirt, blend from the city colour to `base` with `smooth_step` over `skirtWidth` (same idea as `CityGround::paint`).
- `MetroLook` is loaded from the `look` key in `data/metro.json` by the parser in the same file (`from_json`, tested). Missing key: defaults in code are NOT allowed; a missing key is a load error reported to the caller.
- Lead line (report only): `TerrainPaintCtx` gets `metro: Option<&MetroLayout>` and `metro_ground_color` runs after `city_ground.paint` in `world/src/terrain_paint.rs`.

## 2. Box mesh (`boxes.rs`, pure)
- `pub fn tower_mesh(spot: &TowerSpot, look: &MetroLook, detail: Detail) -> Vec<ColorVertex>`, with `enum Detail { Near, Far }`.
- Near (close-up): four walls, a darker base band (`look.base_color`, bottom 6 m), a window grid by vertex colour: per floor, per face, one wall quad plus window strips (`look.glass_color` for the lit-looking panes, a few darker `look.glass_dark` panes); about 1 in 8 window bays is a broken dark patch (`look.broken_color`). Rooftop box (a 4 m high cap, half of the footprint, `look.roof_color`) and a rooftop parapet strip. Landmarks get a stepped top (one smaller box on top).
- Far (skyline): four walls and a roof only, one colour (`look.far_color`), no windows. Triangles per tower = `skyline.trisPerTower` from data (60 expected).
- Triangle budgets (test-enforced): near tower at most 1,500 tris; far tower at most `skyline.trisPerTower`.
- Yaw and position come from `TowerSpot` (`pos`, `yaw`), the mesh is in tower-local space (origin at the base centre); the draw applies the transform.

## 3. Near meshes per chunk (`near.rs`)
- `pub fn near_tower_mesh(towers: &[TowerSpot], coord: IVec2, look: &MetroLook) -> Option<GpuMesh>`: only the towers whose `pos` is inside chunk `coord` (`chunk_of` rule), merged into one vertex buffer per chunk in world space (so one draw per chunk). `None` when the chunk has no towers.
- Built only for loaded chunks, from the streaming build step (the lead adds a `towers: Option<GpuMesh>` field to `LoadedChunk`; report the line). Each loaded chunk builds once, never every frame.

## 4. Colliders (`hulls.rs`)
- `pub fn tower_hulls(towers: &[TowerSpot], coord: IVec2) -> Vec<(Vec3, f32, Vec<[f32; 3]>)>`: one box hull per tower in that chunk (8 points at `half` and `height`), with position and yaw for `physics.add_static_deferred`. The lead adds them in streaming next to the prop colliders (report the line). Towers are solid: the player must not walk through them.

## 5. Skyline (`skyline.rs`)
- `pub struct SkylineBatch { mesh: Option<GpuMesh>, loaded_key: u64 }` and `pub fn rebuild_if_needed(&mut self, device, towers, loaded: &HashSet<IVec2>, cfg) -> bool`: the batch holds ALL towers whose chunk is NOT in the loaded set (so near towers and far towers never draw at the same spot). Rebuilt only when the loaded set changes (key = hash of the sorted loaded set). Far meshes are cached per tower (built once), the batch concatenates them.
- Distance: towers within `skyline.skylineDistance` (6000) of the player only; the batch is rebuilt if the player crosses a 512 m cell boundary, not every frame.
- `pub fn draw_items(&self) -> Vec<DrawItem>` for the MAIN pass only (never the shadow pass). The draw uses a `Lighting` copy whose fog comes from `skyline_fog` in rotland_ui/src/metro_ui/haze.rs (R38-07). Report if `LitRenderer::draw` needs one Lighting per call, and the exact line the lead needs in render_frame.rs.
- `engine/src/render/camera.rs` uses an infinite reverse-Z projection (no far plane), so 6000 m is not clipped; just confirm it in Worker notes.

## Data (`data/metro.json` "look", new key)
- `asphalt`, `sidewalk`, `lot_concrete`, `lot_dirt`, `sidewalk_width` (2.5), `base_color`, `glass_color`, `glass_dark`, `broken_color`, `roof_color`, `far_color` (colours as `[r, g, b]` 0-255 like the other data files). Each Rust number from data is tagged `// data: metro.json look.<key>`.

## Tests (`rotland/src/metro_render/tests.rs`, `world/src/metro/paint.rs` tests)
- Ground: a point outside the skirt returns `base` exactly (grass stays green); a street point is asphalt; a sidewalk point is sidewalk; a reserved Park block keeps `base`.
- Boxes: near tower tris ≤ 1,500; far tower tris ≤ `trisPerTower`; base band darker than the walls; roof box present (top vertices at `height + 4`).
- Near batch: only towers in the chunk; `None` for an empty chunk; deterministic for the same input.
- Hulls: 8 points, extents match `half` and `height`, one per tower in the chunk.
- Skyline: a tower in a loaded chunk is not in the batch; a tower outside is; the key does not change when the set is the same (no rebuild); the batch is empty when all towers are loaded.
- `world/tests/golden.rs` untouched and passing.

## Done when
- Ground, near towers, colliders and the skyline batch are implemented with the tests above; no tower draws both near and far at the same spot.
- Skyline is main pass only and uses the R38-07 skyline fog; grass outside the city is unchanged in tests.
- The lead lines (LoadedChunk field, streaming collider call, terrain_paint ctx field, render_frame draw) are written under Worker notes; no file outside Owns is changed.

## Worker notes
(fill in: the shared-file lines for the lead, triangles per near and far tower measured by the tests, any engine gap found (far plane, one Lighting per draw), questions)
