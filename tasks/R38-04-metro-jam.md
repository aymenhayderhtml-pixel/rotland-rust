# R38-04: New Halden evacuation jam (placement, props per chunk)

Owns: world/src/metro/jam.rs (new: `JamProp`, `plan_jam`, lane and car rules; tests in `#[cfg(test)] mod tests`), world/src/metro/jam_chunks.rs (new: `jam_props_in_chunk`, tests inside), data/metro.json "jam" sub-object only (R38-01 creates the file; R38-05 adds `jam.seatOffsets`, do not touch it). Shared `world/src/metro/mod.rs`, `world/src/generator_contents.rs` and `world/src/lib.rs` lines belong to the lead: report them.
Read: tasks/R38-common.md ("Jam", "Jam models", "Data file"), tasks/R38-03-metro-highway.md (`Highway`, `HighwayHeightField`), world/src/metro/config.rs (`JamConfig`, `WeightedModel`), world/src/metro/layout.rs (`MetroLayout`, `MetroStreet { avenue }`, `street_at`), world/src/metro/place.rs (`MetroSite::to_local`/`to_world`), world/src/city/place.rs (`in_chunk`, `dir_yaw`, `place_street_props`: the pattern for per-chunk prop lists and the wreck rule at lines ~300-322), world/src/scatter.rs (`PropInstance { model, position, yaw, scale }`, `PropInstance::new`), world/src/generator_contents.rs (`chunk_contents`, lines ~30-100: where town and city props are appended after `keep_scattered_prop`), world/src/towns/clutter.rs (`place_in_chunk`), world/src/random.rs (`DotnetRandom`), rotland/src/streaming.rs (non-tree static props become one compound collider: `add_static_compound_deferred` around line 311; shapes come from `shapes_of`), world/src/generator.rs (`WorldGenerator::height` or `raw_height` for ground y).

## Placement (`jam.rs`, pure)
- `JamProp { model: String, pos: Vec3, yaw: f32, pitch: f32, roll: f32 }` exactly as the shared names. `pos` is site-world x/z with y = 0 (the chunk fn sets the real height). `pitch` and `roll` are always 0 in R38: `PropInstance` has no pitch or roll, and bus_city_side is baked lying down (report this).
- `fn plan_jam(highway: &Highway, layout: &MetroLayout, cfg: &MetroConfig) -> Vec<JamProp>`. Deterministic: seed from `layout.site.key` and `layout.site.center.to_bits()` (FNV-1a, like `derive_town_seed`), never the wall clock.
- Lanes: walk the highway polyline (`highway.points`) from the city end backwards. Lane centres sit at lateral offsets `±(medianWidth/2 + 1.75)` and `±(medianWidth/2 + 5.25)` from the centre line. The OUTBOUND pair is the side that carries traffic from the city toward the town (right-hand traffic from the tangent; report the sign you chose). Outbound lanes: bumper-to-bumper for the last `jamLength` m before the city. Spacing per car from `carSpacingM` (random in range, plus the car's own length). Inbound lanes: `inboundChance` per slot, mostly empty, a few wrecks.
- First avenues: the 2 avenues (`avenue == true`) whose segment end is nearest to the city entrance, inside the square. Cars on both sides at `avenueWidth` lanes, same spacing, `avenueJamChance` per slot.
- Each car: model picked by `carModels` weights; `burntChance` swaps to `<id>_burnt`; yaw = the lane tangent (`dir_yaw` rule); `body_seated` in ~`bodyChance` (1 in 4) of the car slots that have seats (not buses or trucks), seat index chosen by the RNG; body pos = car pos + seat offset (from `jam.seatOffsets[car id]`, rotated by the car yaw with the props' yaw convention), body yaw = car yaw.
- `luggageChance` per car: `luggage_pile` or `suitcase_open` on the lane edge beside the car (1.5 to 3 m outside the car's side); `clothes_spill` on the asphalt in about 1 in 3 luggage spots. `body_sheet` beside about 1 in 12 cars on the shoulder.
- Buses: `busCount` random in range; exactly ONE of them is `bus_city_side` (the first in the outbound lane nearest the city), the rest `bus_city`, placed in the outbound lanes, not on the median.
- Army trucks: `armyTruckCount` random in range, `army_truck_jam` in the outbound lanes.
- One `highway_sign_gantry` at the far end of the jam, centred over the carriageway, yaw = the highway tangent (it is the only prop allowed off the lanes).
- Data (`data/metro.json` "jam"): keep `jamLength` 900, `carSpacingM` [6, 9], `carModels`, `busCount` [1, 2], `armyTruckCount` [1, 2], `bodyChance` 0.25, `luggageChance` 0.2, `burntChance` 0.08. Add `inboundChance` 0.04, `avenueJamChance` 0.6, `shoulderBodyChance` 0.08 (new keys, tagged `// data: metro.json jam.<key>`). Each number must come from data; no number in Rust except `// new: DECISIONS 2026-11-26` on structural values.
- Config: the new `JamConfig` fields (`inbound_chance`, `avenue_jam_chance`, `shoulder_body_chance`) are needed in `world/src/metro/config.rs`; do NOT edit config.rs. Report the exact field lines for the lead.

## Per chunk (`jam_chunks.rs`)
- `pub fn jam_props_in_chunk(plan: &[JamProp], coord: IVec2, height: &dyn Fn(f32, f32) -> f32) -> Vec<PropInstance>`: keeps the props whose x/z falls in the chunk (same rule as `in_chunk` in `world/src/city/place.rs`), sets `position.y = height(x, z)`, sets `yaw` and `scale = 1.0`. Pure, no GPU.
- Colliders: no new collider code. Jam props are ordinary non-tree static props, so `rotland/src/streaming.rs` gives them compound colliders the same way as the Ashford wrecks (`car_wreck*`). Confirm that in the Worker notes; if a prop needs a collider but is not in `shapes`, report it.
- The lead calls `jam_props_in_chunk` in `generator_contents.rs` after `keep_scattered_prop` (like the city props), so road-side filters do not remove them.

## Tests
- Same site gives identical `Vec<JamProp>`; a different site gives a different one.
- No car, body, luggage or truck has `|lateral offset|` below `medianWidth/2 + 0.9` (no prop on the median). Only the gantry is exempt.
- Count: the outbound car count is within 0.8x to 1.2x of `2 * jamLength / mean(carSpacingM)`; total buses 1 to 2 with exactly one `bus_city_side`; army trucks in `armyTruckCount`.
- Every prop is on the highway (within `width/2`) or on one of the first 2 avenues (within `avenueWidth/2`) of the square.
- Every body sits inside its car (distance from the car origin < 2.5 m) and is on a car with seats; about 1 in 4 cars has a body (range test, deterministic).
- Burnt ids only from `burntChance`; all model ids exist in `data/metro.json` `carModels` or the fixed list.
- `jam_props_in_chunk`: the union over all chunks equals the plan (no loss, no duplicate); y comes from `height`.
- `world/tests/golden.rs` untouched and passing.

## Done when
- `plan_jam` gives a dense, deterministic outbound jam with open-door cars, luggage, one bus_city_side, buses and trucks, seated bodies in about 1 in 4 cars, sparse inbound lanes.
- `jam_props_in_chunk` splits the plan per chunk; colliders come from the existing compound path.
- Tests above pass; clippy clean; no file outside Owns changed.

## Worker notes
(fill in: lateral sign chosen for outbound, exact `config.rs` field lines needed, colliders confirmation, any prop model missing from R38-05, lines for the lead)
