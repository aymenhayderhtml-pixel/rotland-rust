# R45-02: The wild park (placement, data, park loot)

Runs at the same time as R45-01, 03, 04. Read first: `tasks/R45-common.md` (whole). Style: `tasks/R44-03-safehouse-placement.md`.

Owns:
- `world/src/metro/park.rs` (NEW, pure): `ParkLayout`, `default_park()`, `ParkSite`, `park_site`, `park_contains`, `park_in_chunk`, `park_pond_world`.
- `world/src/metro/park_tests.rs` (NEW, `#[cfg(test)]`).
- `world/src/metro/park_scatter.rs` (NEW, pure, only if `park.rs` would pass ~300 lines: the deterministic tree, bush and grass placement; otherwise keep it in `park.rs`).
- `data/park.json` (NEW).
- `data/loot_tables.json` (ONLY two new keys: `metro_park` and `metro_stadium`; add them next to `metro_camp`, do not edit other tables). R45-03 uses `metro_stadium` by name.

Not owned: `world/src/generator_contents.rs` (R45-05 wires `park_in_chunk`), `world/src/metro/mod.rs` (lead adds `pub mod park;` and `#[cfg(test)] mod park_tests;`), `world/src/metro/paint.rs`, `curbs.rs` (the park ground stays as it is), `data/metro.json` (the reserved park area stays 3x3), models (R45-01).

Read (in this order):
- `tasks/R45-common.md` "What is true today" (the reserved park, why blocks are empty).
- `world/src/metro/army_line.rs` (`ArmyLineSite::to_world`, `army_line_yaw`, `army_line_in_chunk`, the `OnceLock` loader `default_army_line`, `load_army_line` validation). Copy this shape.
- `world/src/metro/place.rs` `MetroSite` (`center`, `yaw`, `to_world`, `to_local`).
- `world/src/metro/layout.rs` `MetroLayout::blocks`, `block_at`, `street_at` (line ~218), `MetroBlock.rect` (site-local min/max corners), `MetroStreet`.
- `world/src/metro/loot_helpers.rs` (`in_chunk`, `make_spot`, `mix_seed`), `world/src/metro/places.rs` (`PlaceChunk` at line ~76).
- `world/src/scatter.rs` `PropInstance::new(model, pos, yaw, scale)`; `world/src/biome/` (how forest trees and bushes get their LOD model names: grep for `tree_broadleaf` / `_lod1`; reuse that naming so the existing LOD code applies).
- `data/loot_tables.json` (`metro_army` at line ~507 is the shape to copy).

## What the park is (the layout)
- The park = the union of the `Reserved::Park` blocks (3x3 blocks, one rect). `park_site` returns the centre (world XZ, from the site's to_world of the rect centre), `yaw` = the site yaw, `half` = half the rect size (site-local).
- **Streets inside the park:** a grid line may run through the 3x3 area. Check it with `layout.street_at`. Props and trees must NOT sit on a street surface (test it). Park paths are laid where the park has no street, or the path is simply left out where it would cross one (say which in Worker notes).
- The pond, the fountain, the bandstand, the playground, the statue and the kiosk are placed in PARK-LOCAL metres (offset from the park centre, x along the site's +X, z along the site's +Y, data yaw relative to the site yaw). Convert with `site.to_world` and add `site.yaw` to the data yaw (as `army_line_yaw` does).

## Data file `data/park.json` (R45-02 creates it)
```json
{
  "_why": "R45: the wild park on the reserved 3x3 park blocks. Park-local metres from the park centre, x along the site, z across it; yaw relative to the site. Claude defaults 2026-11-28.",
  "mapLabel": "Park",
  "pond": { "x": 0.0, "z": 10.0, "radiusM": 9.0, "reedModel": "reeds", "reedCount": 28, "reedRingM": 6.5 },
  "treeModels": ["tree_broadleaf_01", "tree_birch", "tree_oak_big", "tree_pine_01", "tree_autumn"],
  "treeCount": 260,
  "bushModels": ["bush_01", "shrub_wild", "fern"],
  "bushCount": 320,
  "groundModels": ["grass_tall_patch", "leaves_litter", "mushrooms", "moss_patch", "flowers_white", "flowers_yellow", "log_fallen", "stump", "rock_02"],
  "groundCount": 180,
  "clearRadiusM": 4.0,
  "paths": [
    { "from": [-80.0, -80.0], "to": [80.0, 80.0] },
    { "from": [-80.0, 80.0], "to": [80.0, -80.0] }
  ],
  "pathSlabSpacingM": 1.4,
  "pathSlabModel": "park_path_slab",
  "pathCoverShare": 0.5,
  "props": [
    { "model": "park_fountain_dry", "x": -20.0, "z": -20.0, "yaw": 0.0 },
    { "model": "park_bandstand", "x": 30.0, "z": -30.0, "yaw": 0.0 },
    { "model": "park_playground", "x": -40.0, "z": 35.0, "yaw": 0.0 },
    { "model": "park_statue", "x": 0.0, "z": -50.0, "yaw": 0.0 },
    { "model": "park_cafe_kiosk", "x": 45.0, "z": 15.0, "yaw": 0.0 },
    { "model": "bench_wood", "x": -12.0, "z": -15.0, "yaw": 0.0 },
    { "model": "street_lamp", "x": -6.0, "z": -6.0, "yaw": 0.0 }
  ],
  "crates": [
    { "x": 47.0, "z": 17.0, "yaw": 0.0, "model": "barrel_rusty", "table": "metro_park" },
    { "x": 31.0, "z": -27.0, "yaw": 0.0, "model": "army_crate", "table": "metro_park" }
  ]
}
```
- The numbers above are a starting layout (Claude default). Fix any that land on a street, outside the park rect, or inside the pond. The worker must keep every `props`/`crates` entry inside the park rect and add a validation test for it.
- Scatter (`treeCount`, `bushCount`, `groundCount`, `pathSlab*`) is placed by a deterministic `DotnetRandom` seeded from the layout (`mix_seed`-style, as `rooftops.rs` does), once per park, then each chunk takes its own entries. Trees and bushes keep out of `clearRadiusM` of every `props` entry, the pond disc, the paths (path width 3 m) and streets.
- `pathCoverShare` 0.5: half the path slabs get a `leaves_litter` prop on top (old paths half covered). Use the `groundModels` list for the rest.
- Use the forest LOD naming (the tree and bush models get their `_lod1` automatically, if the forest code does that: check and say in Worker notes).

## Code to write (`world/src/metro/park.rs`)
1. `ParkLayout` (serde, camelCase, `#[serde(default)]` on optional blocks), `from_json` (validate: at least one tree model, counts >= 0, every props/crates entry inside the park rect, crate tables non-empty). `default_park()` loads `data/park.json` with `paths::repo_path` (`OnceLock`, panic with a clear message like `default_army_line`).
2. `park_site(layout) -> Option<ParkSite>`: None when there is no Park block.
3. `park_contains(site, p) -> bool`: world point inside the park rect (use the site's `to_local` and the half size).
4. `park_in_chunk(metro, chunk, height) -> PlaceChunk`: the props, the pond reeds and the crates that fall in `chunk` (`in_chunk`). `pois` stays empty (no POI for the park in R45). `height` gives the ground y at each point (as `army_line_in_chunk` does).
5. `park_pond_world(metro) -> Option<Vec2>`: the world centre of the pond (used by the flags, R45-06).
6. The `park_pond` model is placed at the pond centre, y = ground + 0.05 (not in the list above; add it as the first prop of the pond block in code: `park_pond`, scale 1).

## Number tags
Every value read from `park.json` is `// data: park.json <key>`. Code constants (path width 3 m, the `_ = 1e-3` tolerance) are `// new: DECISIONS 2026-11-28`.

## Tests (`world/src/metro/park_tests.rs`, use a fixture `MetroWorld` like `army_line_tests.rs` / `climb_tests.rs`)
- `default_park()` loads; `ParkLayout::from_json` rejects an entry outside the park rect (a small inline JSON).
- `park_site` finds the park; `park_contains` is true at the centre and false 300 m away.
- Every prop and crate position from `park_in_chunk` over all the chunks of the park is inside the park rect and off every street (`street_at` is None).
- The trees are deterministic: the same seed and layout give the same list (two calls equal).
- The tree count across the whole park equals `treeCount` (within the chunks walked) — say how you walk the chunks.
- The pond reeds form a ring (every reed is within `reedRingM` of the pond centre) and no tree is inside `radiusM` of the pond centre.
- Crates use the `metro_park` table (names exist in `data/loot_tables.json`: a test loads the file and checks `metro_park` and `metro_stadium` exist with at least one entry).
- No park-only code path changes the reserved layout: `layout.blocks` count is unchanged (assert against a fixture).

## Done when
Everything above is written; the gate passes (`tools/gate.sh --task tasks/R45-02-park-placement.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` lines for the lead (`world/src/metro/mod.rs`: `pub mod park;`, plus `pub mod park_scatter;` if you added it; `#[cfg(test)] mod park_tests;`).
- The street check result: did any grid street cross the park? what you did about it.
- The tree LOD rule you relied on.
- The `park_pond` / `park_path` prop names (no collider rule; R45-01 notes the prefix rule).
- Any number that you changed in the starting layout, and why.

## Worker notes
