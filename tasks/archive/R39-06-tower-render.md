# R39-06: Draw tower parts (instanced, LODs, shadows, colliders, lobby props)

Owns: rotland/src/metro_render/near.rs (rewritten: the instanced part list per chunk; the old near mesh code is removed), rotland/src/metro_render/hulls.rs, rotland/src/metro_render/mod.rs, rotland/src/metro_render/tests.rs (the near/hull tests only; the skyline tests stay), rotland/src/metro_render/parts.rs (new: part placement helpers, pure where possible), rotland/src/props.rs (ONLY a small tower entry point: a per-model LOD switch distance and a "no shadow above LOD0" flag; the other rules stay as they are), rotland/src/streaming.rs (ONLY the New Halden block around the current `near_tower_mesh` / `tower_hulls` lines, and the `LoadedChunk` tower fields). Shared files (`rotland/src/main.rs`, `rotland/src/app/*`, `Cargo.toml`) belong to the lead: report the exact lines.
Not owned: `rotland/src/metro_render/boxes.rs` and `skyline.rs` (the far skyline stays on boxes.rs until R39-07 changes it).
Read: tasks/R39-common.md ("How towers are built", "Drawing", "Colliders"), tasks/archive/R38-06-metro-render.md (Worker notes), rotland/src/metro_render/near.rs (`towers_in_chunk`, `tower_chunk`, `base_point`), rotland/src/metro_render/hulls.rs (`tower_hulls`, `box_points`), rotland/src/metro_render/mod.rs, rotland/src/metro_play.rs (`MetroPlay`, `update_skyline`), rotland/src/props.rs (`lod_fade`, `FADE`, `BUILDING`, `BUILDING_LOD1`, `load_model`, `PropModel`, `PropLibrary::get`, `place`, `batches`, `shadow_lod_for`, `casts_shadow`, `Placed`), rotland/src/streaming.rs (`Streamer`, `LoadedChunk`, the build step that makes `placed` and `colliders`, the `shapes_of` / `add_static_compound_deferred` calls, the New Halden block near line 327), rotland/src/terrain.rs (`chunk_vertices`), physics/src/world.rs (static compound colliders), world/src/metro/tower_parts.rs and world/src/metro/styles.rs (from R39-01: `tower_parts`, `TowerPart`, `PartKind`, `StyleDef`), world/src/metro/config.rs (`TowersConfig.lod0_distance`, `lobby_props`), data/budgets.json (tower budgets), data/metro.json.

## What to build
- **Placement**: for every loaded chunk, each tower's parts (from `tower_parts`) become `Placed` instances through `PropLibrary::get(device, pack, model_id, building)` and `place(slot, pos, yaw, 1.0)`. Use the chunk the tower centre stands in (`tower_chunk`), as R38 does. Parts of one tower share the tower's yaw.
- **LOD**: each tower model has LOD0 (its own file) and `_lod1` (from the pack, about 1/10 triangles). LOD0 is drawn out to `towers.lod0Distance` (90 m, data), then LOD1 out to the load ring. The switch uses the same `lod_fade` / `FADE` dither as the other props (`lod_fade(dist, &[lod0_distance, range], FADE)`). Add a small entry point in `props.rs` (for example `PropLibrary::get_tower` or a `ends` override) so it does NOT change the building or tree rules.
- **Shadows**: only LOD0 of tower models casts a sun shadow. The shadow list takes LOD0 only: skip shadow instances for LOD1 of tower models (add the rule next to `shadow_lod_for`). Tower shadows stop at the usual shadow distance.
- **Lobby interior**: `PartKind::Interior` parts are placed with the same path (they have no LOD, one model each, range = load ring). Interior props are not solid, except `lobby_desk` may be solid (optional, keep it simple: no collider).
- **Colliders**: per tower in a loaded chunk, (a) the base model's own wall colliders from the pack, so the lobby walls, door opening and floor are walkable (use the same shape path as other props: `shapes_of(pack, id)` in `streaming.rs`), and (b) one solid box from the TOP of the base (`base_height`) to the roof (`TowerSpot.height`), covering the footprint, from `tower_hulls` (change its box to start at `base_height`, not 0, so the doorway stays open). Add both to the chunk's static compound with `add_static_compound_deferred`, as the R38 hulls do now.
- **Remove the near placeholder**: `near_tower_mesh` / `near_tower_vertices` and the `Detail::Near` path in `boxes.rs` are no longer called for the near ring (remove the near helpers; leave `boxes.rs` alone, the skyline uses `Detail::Far`).
- **Data knobs only** if the FPS floor is at risk: first `towers.lod0Distance`, then the LOD1 triangle count in the Blender kit. Never remove close detail (lobby props, LOD0 windows) to win FPS.

## Budget and performance
- Instances: at most one `Placed` per part. Parts of towers beyond `lod0Distance` use LOD1. Count per chunk in a test (`tests.rs`), and keep the old batch scratch reuse (`BatchScratch`).
- Expect the `metro` perf scene to drop from 150 FPS. The rule: its 1% low must be at least 20 FPS (`tools/perf_scenes.sh --set all --seconds 20`, run by the lead).

## Tests (in `rotland/src/metro_render/tests.rs` and a `parts.rs` test module)
- A chunk with one tower gives its parts as `Placed` instances with the tower yaw; a chunk with none gives none.
- LOD switch: LOD0 at 89 m, LOD1 at 91 m, the fade band uses the `FADE` width.
- Shadow list: a tower LOD1 instance is never in the shadow list; LOD0 inside the shadow distance is.
- `tower_hulls` boxes start at `base_height` and end at `height`, footprint = `half`, one hull per tower.
- The base collider uses the base model's shapes (the model id is `tw_<style>_<size>_base`).
- Lobby interior parts are inside the base footprint.
- Existing tests for skyline, far boxes and triangle budgets still pass.

## Done when
- Towers are drawn from instanced parts with LOD0 to `lod0Distance`, LOD1 to the load ring, shadows only from LOD0, and the walk-in lobby has walls and a door opening.
- The solid box starts at the base top; the lobby is walkable; you can walk in and out (no invisible wall).
- The near placeholder meshes are gone from the near ring; the far skyline still draws.
- `tools/gate.sh --task tasks/R39-06-tower-render.md` ends with `GATE: PASS` (workers only).
- Shared-file lines for the lead (`rotland/src/main.rs` mod lines, `LoadedChunk` use, any render-frame draw call) are in Worker notes.

## Worker notes
Files changed:
- `rotland/src/metro_render/parts.rs` (new): pure part assembly (`tallest_landmark_index`, `base_model_id`, `tower_load_ring_range`, `chunk_tower_parts`) plus unit tests.
- `rotland/src/metro_render/near.rs`: removed `near_tower_vertices` and `near_tower_mesh`; added `place_chunk_towers` using `PropLibrary::get_tower` + `place`. Kept `tower_chunk`, `towers_in_chunk`, `base_point` for `hulls.rs` and `skyline.rs`.
- `rotland/src/metro_render/hulls.rs`: updated `tower_hulls` / `box_points` to start at `style.base_height` (`7.6` fallback) up to `t.height.max(base_height)` so the walk-in lobby stays open; added `tower_bases` returning `("tw_<style>_<size>_base", pos, yaw)`.
- `rotland/src/metro_render/mod.rs`: added `pub mod parts;` and updated module docs.
- `rotland/src/props.rs`: added `shadow_lod0_only: bool` on `PropModel`, `PropLibrary::get_tower` (with `lod0_distance` and `range`), `load_tower_model`, and `shadow_lod_for_rule` (tower models cast shadows only from LOD0). Existing building/tree/prop rules are untouched.
- `rotland/src/streaming.rs`: removed `LoadedChunk.towers: Option<GpuMesh>`; updated the New Halden block in `upload()` to push instanced tower parts into `placed` via `place_chunk_towers` and combine `tower_bases` pack shapes (`shapes_of`) + `tower_hulls` upper boxes into `physics.add_static_compound_deferred`.
- `rotland/src/metro_render/tests.rs`: added `style`/`size` to `spot()`, replaced near-mesh tests with `chunk_tower_parts`, `tower_lod_switch_uses_lod0_distance_and_fade_band`, `tower_shadows_only_cast_from_lod0`, `lobby_interior_parts_stay_inside_the_base_footprint`, and updated `hulls_are_one_box_per_tower_starting_at_base_height`.

Shared-file lines for the lead:
- `rotland/src/app/render_frame.rs` (around lines 175-183): `LoadedChunk.towers` was removed (tower parts now live in `LoadedChunk.placed`, which line 217 already iterates). Replace lines 175-183 with:
  ```rust
    let mut items: Vec<DrawItem> = streamer
        .chunks
        .values()
        .map(|c| DrawItem { mesh: &c.ground, offset: c.origin, fog_scale: 1.0 })
        .collect();
  ```

How to see it:
- Once R39-02..05 models are baked into `assets/models.pack` and `render_frame.rs` is updated, run with `--metro=core` or `--metro=lobby` to see instanced LOD0/LOD1 tower kits, LOD0-only shadows, and walkable ground-floor lobbies.

Notes / unsure:
- Did not run `cargo` or `gate.sh` per subagent rules.
- `tower_load_ring_range()` computes `(UNLOAD_RADIUS + 1.0) * WorldGenerator::CHUNK_SIZE * SQRT_2` (~452.5 m) so every loaded chunk's towers stay visible until the chunk unloads and `SkylineBatch` takes over.
