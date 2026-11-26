# R39-01: Tower styles, sizes and tower parts (pure world logic)

Owns: world/src/metro/styles.rs (new), world/src/metro/tower_parts.rs (new), world/src/metro/towers.rs, world/src/metro/towers_tests.rs, world/src/metro/config.rs (ONLY the `TowersConfig` struct and its new fields; nothing else in that file), world/src/metro/mod.rs (ONLY the new `mod`/`pub use` lines for this task), data/metro.json (ONLY the keys `styles`, and the new keys under `towers`: `lod0Distance`, `setbacks`, `lobbyProps`). Shared files (`world/src/lib.rs`, `world/src/generator*.rs`, `rotland/*`, `Cargo.toml`) belong to the lead: report the exact lines.
Read: tasks/R39-common.md ("How towers are built", "R39 shared names", "Budgets per model"), tasks/archive/R38-01-metro-layout.md (Worker notes), world/src/metro/towers.rs (`plan_towers`, `plan_block`, `single_tower`, `pick_landmarks`, `usable_rect`, `cell_grid`, `grid_fits`, `to_spot`, `pick_floors`, `district_towers`, `kind_for`, `mix_seed`), world/src/metro/towers_tests.rs (every test; keep the ones that still hold), world/src/metro/config.rs (`TowersConfig`, `DistrictTowers`, `from_json`, `validate`), world/src/metro/layout.rs (`MetroDistrict`, `MetroBlock`, `MetroStreet::closest_point`, `MetroLayout::streets`), world/src/metro/place.rs (`MetroSite::to_world`, `to_local`, `yaw`), world/src/metro/state.rs (`MetroWorld::in_tower_footprint`, which reads `TowerSpot.pos/half/yaw`), world/src/random.rs (`DotnetRandom::next`, `next_range`, `next_double`), world/src/metro/paint.rs (for the `MetroLook` colours only), world/tests/golden.rs (must stay unedited), data/metro.json.

## What to build
- **`styles.rs`**: `TowerStyle { Brick, Stone, Glass, Slab, Deco }` (serde snake_case, `ALL` const, `key()` returns "brick" etc. for `--tower-style=`), and `StyleDef { key, sizes: Vec<u32>, districts: Vec<MetroDistrict>, floors: [u32; 2], floor_height: f32, base_height: f32, crown_height: f32, color: [u8; 3] }`. `color` is the wall colour (sRGB) that R39-07 reads for the skyline. `load_styles(json) -> Result<Vec<StyleDef>, String>` reads the `styles` array of `data/metro.json`. Validate: sizes only 16, 24 or 32; floors ascending; at least one style per district (Edge, Midtown, Core) and one `deco` style for landmarks.
- **Style pick** (in `towers.rs`): for an ordinary lot, the candidates are the styles whose `districts` contain the block's district. Pick one with the block's RNG (`mix_seed`), so the same site always gives the same city. Landmarks always use `Deco`.
- **Size snap**: a lot's side is the old random pick (`fp[0]..fp[1]`). Snap it DOWN to the largest of the style's `sizes` that is `<=` side. If none fits, skip the lot. The snapped size is stored as `TowerSpot.size` (u32) and the footprint is `size x size` (`half = size/2`).
- **Floors**: take the district `floors` range as today, then clamp it to the style's `floors`.
- **`TowerSpot`** gains `style: TowerStyle` and `size: u32`. Keep every other field. `height` = `base_height + floors * floor_height + crown_height` (style values; the global `towers.floorHeight` is used only when a style does not give one). Write this down in the doc comment.
- **Front facing**: `yaw` = site yaw + k * 90 degrees, where k is the quarter turn that points the lobby (the front, Blender +Y) at the nearest street (`MetroStreet::closest_point` on `layout.streets`). Streets run on the site axes, so a quarter turn keeps every footprint square and axis-aligned. Check the sign of the front axis against how the baker exports Blender +Y (glTF -Z is the usual result of Blender's Y-up export); the test below must prove it.
- **Landmarks** (`Deco`): stack setback sections. `towers.setbacks` is a list of `{ "atFloor": <share of floors 0..1>, "footprintShare": <0..1> }`. From each setback up, the footprint is `size * footprintShare` snapped to 16/24/32 when the style has one, else the nearest smaller size, and the parts use the smaller module.
- **`tower_parts.rs`**: `TowerPart { model: String, pos: Vec3, yaw: f32, kind: PartKind }`, `PartKind { Base, Floor, Crown, Setback, LandmarkCrown, Interior }`. `pub fn tower_parts(spot: &TowerSpot, style: &StyleDef, ground_y: f32, rng_seed: u64) -> Vec<TowerPart>`, pure and deterministic. Model ids follow R39-common: `tw_<style>_<size>_base`, `_floor` / `_floor_b` (alternate by seed, with a random start), `_crown`, and for landmarks `tw_landmark_spire|mast|stepped|glass_top` (pick by seed, the `mast` only on the tallest landmark). `pos` is world-space (`ground_y` plus stacked heights); `yaw` = `spot.yaw`.
  - Base at `ground_y`, then floors at `ground_y + base_height + i * floor_height`, then the crown on top. Setback sections sit on the floor stack at their floor, then their own floors, then the landmark crown.
  - Lobby interior parts (`PartKind::Interior`) come from `towers.lobbyProps`: each entry `{ "model", "x", "z", "yaw" }` in footprint shares (-0.5..0.5 of `size`), rotated by `spot.yaw`, placed at `ground_y`.
- **Config** (`config.rs`, `TowersConfig` only): add `lod0_distance: f32` (`towers.lod0Distance`, 90), `setbacks: Vec<SetbackSpec>`, `lobby_props: Vec<LobbyProp>`. Parse with `#[serde(rename_all = "camelCase")]` and `#[serde(default)]` on the new fields only, so an older metro.json still loads.
- Every new number gets `// data: metro.json <key>` or `// new: DECISIONS 2026-11-26`.

## Data (`data/metro.json`)
- `styles`: five entries (brick, stone, glass, slab, deco). Sizes per R39-common ("Brick: 4-12 floors, Edge and Midtown", etc.). Colours from the wall palette in `look` (brick = 118,96,80; stone = 128,128,124; glass = 88,100,108; slab = 128,128,124 lighter; deco = 112,110,104). Floor counts keep the R38 district ranges.
- `towers.lod0Distance`: 90.
- `towers.setbacks`: two entries, e.g. `{atFloor 0.45, footprintShare 0.7}` and `{atFloor 0.75, footprintShare 0.5}`.
- `towers.lobbyProps`: one entry per lobby model (`lobby_desk`, `lobby_elevators`, `lobby_mailboxes`, `lobby_bench`, `lobby_rubble`, `lobby_plant_dead`), with x/z shares that keep them inside the footprint, away from the front door (front = +Y, so keep z < 0.3).

## Tests (in `towers_tests.rs`; split into a sibling file if the file grows past ~300 lines)
- Stacked parts have no gaps: each floor's `pos.y` equals the previous top (`base_height` and `floor_height` from the style).
- Heights match floors: the last floor top + crown height equals `TowerSpot.height`.
- Same site and seed give identical parts; a different block seed gives a different `_floor` / `_floor_b` order somewhere.
- Front faces a street: the front direction (yaw applied to +Y mapped as the baker does) points at the nearest street within 1 m.
- Landmarks have setbacks: their parts include `Setback` kinds, and the setback footprint is smaller than the base.
- Snap: every `size` is 16, 24 or 32 and each lot fits its block's usable rect.
- Every ordinary block gets a style whose `districts` contain its district. Landmarks are `deco`.
- Config: `styles` parses; `validate` rejects a style with size 20.
- The existing R38 tests (same site identical, lots off the streets, no overlaps, landmark count, density) still pass, except where the new size/style fields change them (update only those).
- `world/tests/golden.rs` passes UNEDITED.

## Done when
- `styles.rs` and `tower_parts.rs` exist, each under ~300 lines, with tests; `plan_towers` uses styles and snapped sizes.
- Every tower's parts stack without gaps and match `height`; landmarks have setbacks and a landmark crown; lobbies face the nearest street.
- `tools/gate.sh --task tasks/R39-01-tower-plan.md` ends with `GATE: PASS` (workers only; the worker never runs cargo itself).
- Shared-file lines (lib.rs `pub use` names for `TowerStyle`, `StyleDef`, `TowerPart`, `PartKind`, `tower_parts`) are in Worker notes.

## Worker notes
