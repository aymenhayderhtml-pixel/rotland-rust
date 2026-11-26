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
Status: tests pass. `cargo test --lib metro` (world): 57 pass. rotland `metro` filter in a scratch copy: 13 pass. The real worktree did not compile at the time because of another coder's `rotland/src/app/frame_ui.rs:144` (region_banner.update takes 5 args). The scratch copy used HEAD versions of frame_ui.rs and region_banner.rs; my files were copied unchanged. clippy shows only "never used" warnings in metro_render, which go away once R38-08 wires the calls.

Files: world/src/metro/paint.rs (MetroLook, from_json/load_from_file, metro_ground_color, tests); world/src/metro/mod.rs (added `pub use paint::{MetroLook, metro_ground_color};`); data/metro.json `look` key (new, last top-level key); rotland/src/metro_render/{mod.rs, boxes.rs, near.rs, hulls.rs, skyline.rs, tests.rs}.

Triangles (measured by tests): near tower max 1,372 (landmark, 42 floors, footprint 40) under the 1,500 budget; far tower max 20 (landmark with step) under trisPerTower 60. Winding: every triangle is clockwise seen from outside (test `every_triangle_is_clockwise_seen_from_outside`), matching the `mesh` pipeline's FrontFace::Cw. Not verified in a screenshot yet.

Signatures (differ from the brief because the GPU and ground height need passing in):
- `near::near_tower_mesh(device: &wgpu::Device, towers: &[TowerSpot], coord: IVec2, ground_y: f32, look: &MetroLook) -> Option<GpuMesh>`
- `hulls::tower_hulls(towers: &[TowerSpot], coord: IVec2, ground_y: f32) -> Vec<(Vec3, f32, Vec<[f32; 3]>)>`
- `skyline::SkylineBatch::default()`, then `rebuild_if_needed(&mut self, device: &wgpu::Device, inputs: &SkylineInputs, loaded: &HashSet<IVec2>, player: Vec2) -> bool`, `draw_items(&self) -> Vec<DrawItem>`, `tower_count()`. `SkylineInputs { towers, ground_y, cfg: &MetroConfig, look: &MetroLook }`.
- `ground_y` is `layout.site.ground_y`. `look` = `MetroLook::load_from_file(paths::repo_path("data/metro.json"))`, loaded once at startup.
- `paint::metro_ground_color(base, wx, wz, &layout, &look)`: wire it in terrain_paint after city_ground.paint.

Lines for the lead (shared files, not edited by me):
- rotland/src/streaming.rs LoadedChunk: add `pub towers: Option<GpuMesh>,` (and `tower_colliders` if you want them in the list). In `upload`, after building the ground: `let towers = near_tower_mesh(device, &self.metro_towers, b.coord, ground_y, &look);`, then for colliders: `let hulls = tower_hulls(&self.metro_towers, b.coord, ground_y); let h = physics.add_static_compound_deferred(hulls.iter().map(|(p, yaw, pts)| (std::slice::from_ref(pts), *p, *yaw, 1.0)));` and push h if valid. (`pts` is `Vec<[f32;3]>`, so `from_ref(pts)` gives `&[Vec<[f32;3]>]`; check the element type matches the compound API.)
- world/src/terrain_paint.rs: `TerrainPaintCtx` gets `metro: Option<&MetroLayout>` and a `look: Option<&MetroLook>`, then after `city_ground.paint`: `if let (Some(m), Some(l)) = (ctx.metro, ctx.look) { c = metro_ground_color(c, wx, wz, m, l); }`.
- Skyline in render_frame: `let fog = rotland_ui::metro_ui::haze::skyline_fog(&haze_cfg, base_sky_color)` gives (colour, begin, end). Make a Lighting copy with fog_color/fog_begin/fog_end set from it, then pass `batch.draw_items()` to the main pass.
- Streaming: keep a `HashSet<IVec2>` of loaded chunk coords (the ones with towers) and call `skyline.rebuild_if_needed(device, &inputs, &loaded, player_xz)` once per frame.

ENGINE GAP (not edited, needs the lead's decision): `LitRenderer::draw` takes ONE `Lighting` and ONE `items` list. Those items are also drawn in the sun shadow pass (`record_items(... visible_scratch ...)`), so the skyline would cast shadows, which the spec forbids. Also the skyline needs its own fog (`skyline_fog`, which keeps the far towers visible), while the scene uniform (`write_scene`) applies the same fog to the whole frame. Proposed fix: add a `main_only: &[DrawItem]` parameter to `draw()` that is recorded in the "world" pass after `record_items` and never in the shadow pass, plus a second fog uniform (or, simplest, draw the skyline with the same fog and accept a stronger far fade). Please decide; I did not change the engine.

Camera: engine/src/render/camera.rs uses an infinite reverse-Z projection (line 39), so 6000 m is not clipped.

Deviations: far meshes are generated per rebuild, not cached per tower (about 60 tris per tower, cheap, rebuilds are rare). Skyline batch key = sorted loaded set + player cell (512 m), so it also rebuilds on a cell crossing.

Tests (rotland/src/metro_render/tests.rs, world/src/metro/paint.rs): budgets, winding, base band darker, rooftop box and landmark step, near batch per chunk and deterministic, None for empty chunk, hulls (8 points, extents), skyline excludes loaded chunks and distant towers, key stable within a cell and changes with set or cell.

Not done (not in my ownership): `skyline_fog` from R38-07 is not called inside my code. The lead wires it in render_frame, as noted above.
