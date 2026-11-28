# R44-03: World placement (safe house tower, rooftop camp, floor rescue seeds)

Runs at the same time as R44-01, 02, 04. Read first: `tasks/R44-common.md` (whole). Style: `tasks/R43-02-rooftops-fire-escapes.md`.

Owns:
- `world/src/metro/safehouse.rs` (NEW, pure): `SafeHouseLayout` and its sub-structs, `safe_house_tower`, `rooftop_camp_tower`, `SafeHouseSpots`, `safe_house_spots`, `safehouse_in_chunk`, `RooftopNpc`, `rooftop_camp_in_chunk`, `ClimbRescueSeed`, `rescue_floor_seeds`.
- `world/src/metro/safehouse_tests.rs` (NEW, `#[cfg(test)]`). The lead adds `#[cfg(test)] mod safehouse_tests;` to `world/src/metro/mod.rs`.
- Nothing else. Do not edit `world/src/metro/mod.rs`, `climb.rs`, `fire_escapes.rs`, `generator_contents.rs` or any `game_*`/`rotland` file. Lines for the lead go in Worker notes.

Not owned: wiring the chunk into the world (R44-05 does that), data (`data/safe_house.json` is R44-01's, read it as written in `R44-common.md`), models (R44-02).

Read (in this order):
- `world/src/metro/fire_escapes.rs` (`fire_escape_towers`, `fire_escape_ladders`, `to_world` helper, `mix_seed`, `DotnetRandom` use). Copy the tower-local -> world helper; do not change the file.
- `world/src/metro/climb.rs`: `ClimbTower`, `ClimbKind`, `StopKind`, `climb_towers`, `roof_deck_y`, `ClimbChunk`, `climb_in_chunk_with` (how stop-floor POI ids `metro_floor_<tower>_<i>`, building ids `tw_interior_<kind>_<size>`, floor heights and positions are computed). Reuse the same helpers; do not re-derive numbers.
- `world/src/metro/life_helpers.rs` `in_chunk` (`pub(super)`), `world/src/metro/towers.rs` `TowerSpot`, `world/src/poi.rs` (`PropInstance` fields), `world/src/metro/climb_tests.rs` (how a test `MetroWorld` is built and how `height` is passed).
- `rotland/src/app/metro_flags_r43.rs` `FLOOR_IN_DIST_M` (3.0 m in from the shaft door, on the floor) for the rescue seed position.

## What to write
1. **`SafeHouseLayout`** (serde, `#[serde(rename_all = "camelCase")]`, `from_json(&str) -> Result<Self, String>`):
   - Fields from the JSON blocks `shack`, `ladder`, `rooftopCamp` in `R44-common.md`. Unknown keys are fine.
   - Also add `pub fn safe_house_layout() -> Option<&'static SafeHouseLayout>`: a `OnceLock` loader that reads `paths::repo_path("data/safe_house.json")` (the same pattern as `generator_contents.rs` for `climb_floors.json`). Log a warning and return `None` on error.
   - `ShackLayout`, `LadderLayout`, `RooftopCampLayout` (`map_label` from `rooftopCamp.mapLabel`), `PropSpec`, `NpcSpec` as in the common file.
   - Tests use an inline JSON copy of the blocks (not the file). One test calls `safe_house_layout()` and checks it is `Some`.
2. **`safe_house_tower(metro) -> Option<usize>`**: `fire_escape_towers(metro)`, the pick nearest to `metro.layout.site.center` (flat XZ distance, `Vec2`). Ties: lowest index. Memoise nothing (the list is already memoised).
3. **`rooftop_camp_tower(metro) -> Option<usize>`**: among `climb_towers(metro)`, keep `kind == ClimbKind::Core` (if none, keep all). Pick the one farthest (flat XZ) from the safe house tower's centre `towers[idx].pos`. Ties: lowest index. Return `None` when there is no safe house tower.
4. **`safe_house_spots(metro, layout, height) -> Option<SafeHouseSpots>`**:
   - `tower = safe_house_tower(metro)?`. `t = metro.towers[tower]`, `style = style_for(default_styles(), t.style)`, `ground_y = metro.layout.site.ground_y`, `deck_y = roof_deck_y(t, style, ground_y)`.
   - Shack, bed, chest, sign and travel point: tower-local `(lx, lz)` from `layout.shack.*`, `y = deck_y`, converted with the R43 tower frame.
   - Ladder foot: local `(ladder.centre_x, _, half + ladder.foot_out_m)`, `y = height(wx, wz)` (the street), on the BACK face (local +Z).
   - Ladder deck: local `(ladder.centre_x, _, half - ladder.deck_inset_m)`, `y = deck_y`.
   - `half = t.half.x` (as `fire_escapes.rs` uses).
   - The ladder's back face must not be the fire-escape face: assert in a test that the foot's local `z > half`.
5. **`safehouse_in_chunk(metro, layout, chunk, height) -> ClimbChunk`**:
   - Only when the safe house tower centre is in `chunk` (`in_chunk`).
   - `props`: `safehouse_shack` (at `shack.centre_local`, yaw = tower yaw), `safehouse_bed` (`bed_local`), `safehouse_chest` (`chest_local`), `travel_sign` (`sign_local`), all at `y = deck_y`.
   - `safehouse_in_chunk` takes `ladder_down: bool`. When false, emit ONE `rope_ladder_coil` on the deck at the ladder top instead of the segments. When true, `rope_ladder_seg` props: one per `segment_m` from the street up to the deck, on the ladder's line (local `centre_x`, back face). Their `y` runs from the foot to the deck.
   - `pois`, `crates`, `ladders`, `boxes` stay empty. Set `ClimbChunk` fields accordingly.
6. **`RooftopNpc`** (struct with the fields in `R44-common.md`) and **`rooftop_camp_in_chunk(metro, layout, chunk, height) -> (ClimbChunk, Vec<RooftopNpc>)`**:
   - Only when the rooftop camp tower centre is in `chunk`. Its roof is `roof_deck_y` for that tower.
   - `props` from `rooftop_camp.props` (`model`, `local`, `yaw`), at `y = deck_y`.
   - `npcs` from `rooftop_camp.npcs`, at `y = deck_y`, yaw as given. `role` is `Option<String>` in the layout and `String` in `RooftopNpc` (use `""` for none).
7. **`rescue_floor_seeds(metro) -> Vec<ClimbRescueSeed>`**:
   - Candidate floors: stop floors of every `climb_towers` entry, EXCEPT the top floor and any `StopKind::Camp`.
   - Pick exactly 3 with `DotnetRandom::new(mix_seed(...))` (the `mix_seed` pattern of `fire_escapes.rs`, label `b"rescue_floors"`), shuffled as `fire_escapes` shuffles. Take the first 3 that are on different towers; if fewer than 3 towers qualify, take any remaining candidates.
   - Each seed: `poi_id = metro_floor_<tower>_<i>`, `building_id = tw_interior_<kind>_<size>` (the same name `climb_in_chunk_with` uses), `position` = the floor surface 3.0 m in from the shaft door (`FLOOR_IN_DIST_M`, the R43 flag spot), `yaw` = tower yaw.
   - Sorted by `poi_id` so the order is stable.

## Number tags
No Rust numbers except `0`, `1`, `2` counts, the R43 `FLOOR_IN_DIST_M = 3.0` (tag `// new: DECISIONS 2026-11-27`), and the 3 rescue seeds (tag `// data: rescue.json floorSurvivors`: that value lives in `data/rescue.json` and R44-04 adds it; for this task write `const RESCUE_FLOOR_SEEDS: usize = 3; // data: rescue.json floorSurvivors`). All layout numbers come from the JSON through `SafeHouseLayout`.

## Tests (`safehouse_tests.rs`, use a fixture `MetroWorld` like `climb_tests.rs`)
- `safe_house_tower` is in `fire_escape_towers(metro)` and is the nearest of them to the site centre (brute-force check).
- The safe house tower is NOT a climb tower and is NOT replaced / burnt.
- `rooftop_camp_tower` is a `Core` climb tower (when one exists) and is farthest from the safe house among the Core ones; it is not the safe house tower.
- `safe_house_spots`:
  - `deck_y == roof_deck_y(...)`.
  - The ladder foot local `z > half` (back face) and the deck local `z < half`.
  - The bed, chest and shack are inside the roof (`|lx|`, `|lz|` < `half`).
- `safehouse_in_chunk`: for the chunk that contains the safe house centre, exactly 4 non-rope props are emitted (`safehouse_shack`, `safehouse_bed`, `safehouse_chest`, `travel_sign`), and every other chunk returns none.
- `safehouse_in_chunk(.., false)`: one `rope_ladder_coil`, no segments.
- `safehouse_in_chunk(.., true)`: `rope_ladder_seg` count is `ceil((deck_y - ground_y) / segment_m)` or within 1 of it.
- `rooftop_camp_in_chunk`: 5 props, 4 NPCs, exactly one NPC with `role == "trader"`, the others `role == ""`.
- `rescue_floor_seeds`: exactly 3; none is a top floor; none is a `Camp` floor; each `poi_id` appears in `climb_in_chunk` POIs of its tower; same seed -> same output.
- A non-safe-house, non-rooftop-camp tower's chunk gets no safe-house props (the chunk counts above are the only change).
- `world/tests/golden.rs` is NOT edited and still passes.

## Done when
The files above exist; the tests pass in the gate (`tools/gate.sh --task tasks/R44-03-safehouse-placement.md`). Fill `## Worker notes
- Files: `world/src/metro/safehouse.rs` (~330 lines), `safehouse_tests.rs` (12 tests); lead added the mod lines.
- `rooftop_camp_in_chunk(metro, layout, chunk)` has no height argument (all on the roof).
- JSON yaw values are treated as tower-relative.
- Stop-floor building name: `tw_interior_<office|flat|camp>_<size>` (same as `climb_in_chunk_with`).
- Rescue seed y = site ground + floor height + 0.3 slab + 0.05 clearance.
- Open: rope leans out over the street per data; R44-05 to check in a picture.` with:
- Files changed.
- The `mod` line for the lead (`#[cfg(test)] mod safehouse_tests;` and `pub mod safehouse; pub use ...` for `world/src/metro/mod.rs`).
- Any function signature you changed from the common file, and why.
- The name of the stop-floor interior building you used, and the rope segment count for the repo's safe house tower (if you can compute it in a test).

## Worker notes

## Gate report
- date: 2026-11-28
- base: main
- files changed (3): world/src/metro/mod.rs, world/src/metro/safehouse.rs, world/src/metro/safehouse_tests.rs
- tests passed: 1539
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
