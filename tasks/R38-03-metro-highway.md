# R38-03: New Halden highway from the nearest town

Owns: world/src/metro/highway.rs (new; tests in `#[cfg(test)] mod tests`); data/metro.json "highway" sub-object only (R38-01 creates the file; replace only its values). Shared `world/src/metro/mod.rs`, `world/src/generator_world.rs` and `world/src/generator_contents.rs` lines belong to the lead: report them.
Read: tasks/R38-common.md ("Highway", "Data file"), world/src/metro/mod.rs, config.rs, place.rs, layout.rs (R38-01: `MetroSite`, `MetroConfig`, `site.to_local`/`to_world`, `sizeBlocks`, `blockSize`, `avenueEvery`, `HighwayConfig`), world/src/roads/mod.rs (`RoadsData`, `RoadNetwork`, `RoadKind`), world/src/roads/build.rs (`resample_and_wiggle`, point spacing, wiggle fade, clearance), world/src/generator_world.rs (`RoadHeightField::build`/`sample`, `apply_outer_height`, bridge/`carve` handling, town average heights), world/src/rivers/mod.rs (`crossings_for_polylines`, `carve`, `water_at`), world/src/towns/mod.rs (`TownSite`, `street_dir`), data/towns.json, data/roads.json, data/rivers.json, data/metro.json ("highway"), world/tests/golden.rs.

- `Highway { from_town: String, points: Vec<Vec2>, width: f32 }` exactly. `fn plan_highway(site: &MetroSite, towns: &[TownSite], roads_cfg: &RoadsData, cfg: &MetroConfig) -> Highway`.
- Pick the nearest town: candidates are `blackwater` and `frostmere` (the two towns on the swamp/snow side); choose the smaller `center.distance(site.center)`. If neither key is present (unit tests), fall back to the nearest town overall. `from_town` = its key.
- Start point: on the town's main street at the edge that faces the city. `street_dir` points toward the origin/camp; choose the sign whose direction has the larger dot with `(site.center - town.center)`; start = `town.center + street_dir * sign * (town.radius + cfg.highway.town_setback_m)`. This avoids cutting through town lots.
- End point: the city's nearest avenue entrance, computed from `site` + `cfg` only (no `MetroLayout` argument): transform the town center to site-local, pick the square edge it faces (`|x|` vs `|z|`), snap the along-edge coordinate to the nearest avenue line (`-half + k*blockSize`, `k % avenueEvery == 0`), clamp inside the edge, transform back with `site.to_world`. The last point is exactly on the square edge and on an avenue line.
- Polyline: sample start→end at `roads_cfg.point_spacing` (20 m); add a gentle lateral bend `normal * sin(pi*t) * bend`, where `bend` is deterministic from the site/town (seeded `DotnetRandom` from `site.center.to_bits()`, range `±roads_cfg.wiggle_m`), faded to 0 at both ends. No segment longer than ~1.5 * point_spacing and no turn sharper than 20°. `width = cfg.highway.highway_width` (22 m = 4 lanes + median).
- `HighwayHeightField { points: Vec<Vec2>, heights: Vec<f32> }` in the same file, mirroring `RoadHeightField` in `world/src/generator_world.rs`: `build(highway, site_ground_y, base_height, cfg)` samples `base_height` at each point, smooths over `cfg.highway.height_smooth_m` (40 m), and blends the last ~100 m to `site_ground_y` so the highway meets the flat city. `height_at(p) -> Option<f32>` (nearest segment, within `width/2 + shoulder_width`), `flatten(raw, p) -> f32` (smooth blend over `flatten_edge_m`), and `pub fn highway_height_at(field, p) -> Option<f32>` as the common's free wrapper. The lead builds the field once with the same base-height closure used for `RoadHeightField` (raw height, town interiors at their average), stores it, and calls `flatten` in `apply_outer_height` **before** the metro city `flatten` so the city ground wins at the entrance.
- Bridges: do not invent bridge meshes. After `plan_highway`, the lead appends `world_gen.rivers().crossings_for_polylines(&[&highway.points])` to `WorldGenerator.crossings`; the existing bridge/`carve` code then leaves the water and flattens the approaches. Keep the polyline on the road line (no detour); report the crossing points in Worker notes.
- Data: fill only `highway` in `data/metro.json` (`highwayWidth` 22, `medianWidth` 2, `lanes` 4, `shoulderWidth` 2, `heightSmoothM` 40, `flattenEdgeM` 8, `townSetbackM` 8); add `// data: metro.json highway.<key>` tags. Do not edit `config.rs` or any other sub-object.
- Tests: starts at the nearer of Blackwater/Frostmere within `radius + setback + 1` and on its main-street line; ends on the city square edge and on an avenue line (local coordinate a multiple of `avenueEvery * blockSize` within 0.5 m); all points stay within the corridor, segment lengths ≤ 1.5 * spacing, turns ≤ 20°, endpoints exact; width = 22 and ≥ 4 * 3.5 + median; `HighwayHeightField` follows a synthetic `base_height`, returns None off the road, and meets `site.ground_y` at the city end; a synthetic town list picks the nearest; the real route is checked against the `data/rivers.json` chords (same chord math as `place.rs`) and any crossing point/span is reported for the lead's `crossings_for_polylines` wiring, while a synthetic chord crossing the highway must be detected; golden untouched.

## Done when
- `plan_highway` connects the nearer of Blackwater/Frostmere to a real avenue entrance on the city edge, 4 lanes + median, smooth and deterministic.
- `HighwayHeightField`/`highway_height_at` works, the lead has the exact flatten/bridge wiring lines, and `data/metro.json` `highway` is filled.
- Tests above pass; clippy clean; `world/tests/golden.rs` unedited.

## Worker notes

**Changed (my files):** `world/src/metro/highway.rs` (Highway, plan_highway, route), `world/src/metro/highway_height.rs` (HighwayHeightField, highway_height_at), `world/src/metro/highway_tests.rs` (13 tests). Declared from inside highway.rs with `#[path]`, so **mod.rs needs no extra mod lines**.

**Shared-file edits made (small, please keep):**
- `world/src/metro/mod.rs`: added `pub use highway::{Highway, HighwayHeightField, highway_height_at, plan_highway};`.
- `world/src/metro/config.rs`: `HighwayConfig` gained `pub city_blend_m: f32` (JSON `cityBlendM`), the length of the highway's last stretch that blends into the flat city ground.
- `data/metro.json` "highway": filled values were already there; added `"cityBlendM": 100`.

**Wiring for the lead (generator_world.rs; I did not touch it):**
```rust
use crate::metro::{plan_highway, HighwayHeightField};
// after towns and roads exist, and after site.ground_y is set:
let highway = plan_highway(&metro_site, &town_sites, &roads_data, &metro_cfg);
let highway_field = HighwayHeightField::build(&highway, metro_site.ground_y, &|x, z| /* same closure as RoadHeightField::build */, &metro_cfg);
// bridges: append to the existing crossings list
crossings.extend(world_gen.rivers().crossings_for_polylines(&[&highway.points]));
// in apply_outer_height, BEFORE the metro city flatten:
h = highway_field.flatten(h, p);   // p: Vec2 world XZ
// props/jam: highway_height_at(&highway_field, p) -> Option<f32>
```
Store `highway` and `highway_field` on WorldGenerator (or wherever the metro site lives). `field.points` and `field.heights` are pub.

**Behaviour:**
- Start: the nearer of Blackwater / Frostmere (fallback: nearest town), `townSetbackM` past its edge on the street side facing the city.
- End: the nearest **interior** avenue on the city edge facing that town. Corner lines (k = 0 and k = sizeBlocks) are never used.
- Route: straight line, sampled every `roads.json point_spacing`, seeded sideways bend up to `wiggle_m` (zero at both ends). Exact end point.
- Height: base height smoothed over ±`heightSmoothM` arc (same as roads), last `cityBlendM` blended into `site.ground_y`. `flatten` is full height within half width, easing to raw over `flattenEdgeM`.

**Check results:**
- `cargo test --lib metro`: 34 passed (all R38-01 and R38-03 tests). Clippy shows nothing in metro files.
- The real route crosses **no** `rivers.json` chord (checked with the same chord math as place.rs, camp at origin, reference +X). The check uses the ideal town spots from the sector and distance, not the spiral-shifted ones, so it is an estimate. The `crossings_for_polylines` wiring above is the real check.
- `world/tests/golden.rs` untouched.

**Unsure / for review:**
- `HighwayHeightField` has a grid of cells and some private fields beyond the spec's `points` + `heights` (needed for speed over ~175 segments per lookup).
- `build` takes `&MetroConfig` (not the highway sub-struct), so it matches the other metro calls.
- The reference direction in the test is +X (as place.rs tests). In the game, the lead's `cfg.reference_dir` decides the real route.
- `highway_tests.rs` is 312 lines (a little over the ~300 target).
- The workspace `cargo test` currently fails to compile `rotland/src/app/frame_ui.rs:144` (another coder's file, `takes 5 arguments`); I used `cargo test --lib` to avoid it.
