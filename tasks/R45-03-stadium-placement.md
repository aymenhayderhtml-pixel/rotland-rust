# R45-03: The stadium (placement, data, relief camp on the pitch)

Runs at the same time as R45-01, 02, 04. Read first: `tasks/R45-common.md` (whole). Style: `tasks/R44-03-safehouse-placement.md`.

Owns:
- `world/src/metro/stadium.rs` (NEW, pure): `StadiumLayout`, `default_stadium()`, `StadiumSite`, `stadium_site`, `stadium_contains`, `stadium_in_chunk`.
- `world/src/metro/stadium_tests.rs` (NEW, `#[cfg(test)]`).
- `data/stadium.json` (NEW).

Not owned: `data/loot_tables.json` (R45-02 adds `metro_stadium`; read it, do not edit it), `world/src/metro/mod.rs` (lead), `generator_contents.rs` (R45-05 wires `stadium_in_chunk`), `data/stadium_horde.json` and the horde (R45-04), models (R45-01), any `game_*` / `rotland` file.

Read (in this order):
- `tasks/R45-common.md` "What is true today" (the reserved Stadium is 2x2 at ix 13, iz 4; reserved blocks have no towers).
- `world/src/metro/places.rs`: `group_origin` (line ~243), `place_site` (line ~93) and `PlaceSite::to_world`-style use, `block_centre` (line ~252), `PlaceChunk` (line ~76). The stadium is a 2x2 reserved group like the big places: copy `group_origin` / the 2x2 check (do NOT add Stadium to `PLACE_KINDS`: that would change the place paint and places tests).
- `world/src/metro/army_line.rs` (`ArmyLineSite` frame, `army_line_in_chunk`, the loader and validation pattern, `default_army_line`).
- `world/src/metro/place.rs` `MetroSite` (`center`, `yaw`, `to_world`).
- `world/src/metro/loot_helpers.rs` (`in_chunk`, `make_spot`), `world/src/scatter.rs` (`PropInstance::new`), `world/src/poi.rs` (not needed unless you add a POI: do NOT add one in R45).
- `data/places/army_line.json` (shape of a props / crates list in a local frame), `data/loot_tables.json` `metro_medical`.

## What the stadium is (the layout)
- `stadium_site(layout)`: the union of the `Reserved::Stadium` blocks (2x2). `centre` = world XZ of the rect centre, `yaw` = site yaw. Returns None when the group is not all there (like `place_site`).
- Stadium-local frame: origin at `centre`, x along the site's +X, z along its +Y, data yaw relative to the site yaw (as the army line does). Convert with `site.to_world` and add the site yaw to the data yaw.
- The oval: `outerHalf` = [x, z] half sizes of the outer boundary; `stadium_contains` is an ellipse test with that outer half. Keep `outerHalf` inside the block group rect minus 6 m (half a street) on each side: write the check in a test.
- `gates`: 4 gate points on the outer boundary, world XZ, in the order north, east, south, west (stadium-local). The horde leaves through them (R45-04 uses the index). `gates` are part of `StadiumSite`.

## Data file `data/stadium.json` (R45-03 creates it)
```json
{
  "_why": "R45: the stadium on the reserved 2x2 Stadium group. Stadium-local metres from the centre: x along the site, z across it. The pitch is a failed relief camp. Claude default 2026-11-28.",
  "mapLabel": "Stadium",
  "outerHalf": [52.0, 38.0],
  "pitchHalf": [45.0, 27.0],
  "stands": [
    { "model": "stadium_stand", "x": -40.0, "z": -36.0, "yaw": 0.0 },
    { "model": "stadium_stand", "x": -20.0, "z": -36.0, "yaw": 0.0 },
    { "model": "stadium_stand", "x": 0.0, "z": -36.0, "yaw": 0.0 },
    { "model": "stadium_stand", "x": 20.0, "z": -36.0, "yaw": 0.0 },
    { "model": "stadium_stand", "x": 40.0, "z": -36.0, "yaw": 0.0 },
    { "model": "stadium_stand", "x": -40.0, "z": 36.0, "yaw": 3.1416 },
    { "model": "stadium_stand", "x": -20.0, "z": 36.0, "yaw": 3.1416 },
    { "model": "stadium_stand", "x": 0.0, "z": 36.0, "yaw": 3.1416 },
    { "model": "stadium_stand", "x": 20.0, "z": 36.0, "yaw": 3.1416 },
    { "model": "stadium_stand", "x": 40.0, "z": 36.0, "yaw": 3.1416 },
    { "model": "stadium_stand", "x": -52.0, "z": -12.0, "yaw": 1.5708 },
    { "model": "stadium_stand", "x": -52.0, "z": 12.0, "yaw": 1.5708 },
    { "model": "stadium_stand", "x": 52.0, "z": -12.0, "yaw": -1.5708 },
    { "model": "stadium_stand", "x": 52.0, "z": 12.0, "yaw": -1.5708 }
  ],
  "floodlights": [
    { "x": -58.0, "z": -42.0, "yaw": 0.0 },
    { "x": 58.0, "z": -42.0, "yaw": 0.0 },
    { "x": -58.0, "z": 42.0, "yaw": 0.0 },
    { "x": 58.0, "z": 42.0, "yaw": 0.0 }
  ],
  "gates": [
    { "x": 0.0, "z": -38.0, "yaw": 0.0 },
    { "x": 52.0, "z": 0.0, "yaw": -1.5708 },
    { "x": 0.0, "z": 38.0, "yaw": 3.1416 },
    { "x": -52.0, "z": 0.0, "yaw": 1.5708 }
  ],
  "pitch": { "model": "stadium_pitch", "x": 0.0, "z": 0.0, "yaw": 0.0 },
  "goals": [
    { "model": "stadium_goal", "x": -44.0, "z": 0.0, "yaw": 1.5708 },
    { "model": "stadium_goal", "x": 44.0, "z": 0.0, "yaw": -1.5708 }
  ],
  "reliefCamp": [
    { "model": "evac_tent", "x": -28.0, "z": -14.0, "yaw": 0.3 },
    { "model": "evac_tent", "x": -28.0, "z": 14.0, "yaw": -0.2 },
    { "model": "triage_tent", "x": 10.0, "z": -16.0, "yaw": 0.0 },
    { "model": "bus_city", "x": 24.0, "z": 8.0, "yaw": 0.2 },
    { "model": "bus_city_side", "x": 14.0, "z": 20.0, "yaw": -0.3 },
    { "model": "field_kitchen", "x": 30.0, "z": -15.0, "yaw": 0.4 },
    { "model": "sandbag_wall", "x": 0.0, "z": -22.0, "yaw": 0.0, "len": 6.0 },
    { "model": "army_truck_burnt", "x": -12.0, "z": 18.0, "yaw": 0.5 },
    { "model": "fire_barrel", "x": -6.0, "z": -8.0, "yaw": 0.0 },
    { "model": "body_bag", "x": 8.0, "z": -6.0, "yaw": 0.3 }
  ],
  "crates": [
    { "x": -22.0, "z": -16.0, "yaw": 0.0, "model": "army_crate", "table": "metro_stadium" },
    { "x": -20.5, "z": -15.0, "yaw": 0.3, "model": "army_crate", "table": "metro_stadium" },
    { "x": 12.0, "z": -16.0, "yaw": 0.0, "model": "med_cabinet", "table": "metro_medical" },
    { "x": 30.0, "z": 11.0, "yaw": 0.0, "model": "army_crate", "table": "metro_stadium" }
  ]
}
```
- All numbers are a Claude starting layout. Keep every stand, light, gate, tent and crate inside `outerHalf`, and every relief-camp prop inside `pitchHalf`. Gates sit on the outer boundary. The worker adds a test that checks these.
- A `stands` entry's model is 20 m long (`stadium_stand` size): a stand at x = -52 or 52 (the ends) is rotated 90 deg. Fix any overlap the test finds; say what you changed.

## Code to write (`world/src/metro/stadium.rs`)
1. `StadiumLayout` (serde, camelCase, `#[serde(default)]` on the lists): `outer_half: [f32; 2]`, `pitch_half`, `gate_half`, `stands`, `floodlights`, `gates`, `pitch: StadiumEntry`, `goals`, `relief_camp`, `crates`, `map_label: String`. `StadiumEntry { model, x, z, yaw, len (optional) }`. `from_json` validates (lists non-empty for stands/gates/crates; `outer_half` > `pitch_half`; every crate table non-empty).
2. `default_stadium()`: the `OnceLock` loader of `data/stadium.json` with `paths::repo_path`.
3. `stadium_site(layout: &MetroLayout) -> Option<StadiumSite>`: the 2x2 group. `StadiumSite { centre: Vec2, yaw: f32, gates: [Vec2; 4] (world XZ), pitch_half: Vec2, outer_half: Vec2 }`.
4. `stadium_contains(site, p) -> bool`: ellipse test on `outer_half`, in stadium-local coordinates.
5. `stadium_in_chunk(metro, chunk, height) -> PlaceChunk`: the stands, floodlights, gates, pitch, goals, relief camp props that fall in the chunk (each by its own position: `in_chunk`), and the crates via `make_spot("stadium", pos, yaw, table, Some(model))`. No POIs.

## Number tags
Every value from `stadium.json` is `// data: stadium.json <key>`. Code constants are `// new: DECISIONS 2026-11-28`.

## Tests (`world/src/metro/stadium_tests.rs`, fixture `MetroWorld` like `army_line_tests.rs`)
- `default_stadium()` loads; a bad JSON (outer half smaller than pitch half, empty gates) is rejected (inline JSON).
- `stadium_site` returns Some for the seed fixture and None when a block of the 2x2 is missing.
- The 4 gates are on the outer boundary (`stadium_contains` is true just inside and false just outside, 1 m each way, for each gate direction), and each gate is inside the stadium group rect minus 6 m.
- Every stand, floodlight, relief-camp prop and crate is inside `outer_half` and every relief-camp prop is inside `pitch_half` (use the layout data, stadium-local).
- The pitch centre is the stadium centre.
- Chunks: the union of `stadium_in_chunk` over the chunks that cover the stadium equals the full prop count (no prop lost or doubled at a chunk edge: count the props once per chunk, each in exactly one chunk).
- Every crate table name in `stadium.json` is a non-empty string. (The existence of `metro_stadium` in `data/loot_tables.json` is tested by R45-02, which adds it; do not test it here, because this worktree does not have that key until R45-02 is merged.)

## Done when
Everything above is written; the gate passes (`tools/gate.sh --task tasks/R45-03-stadium-placement.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` lines for the lead (`world/src/metro/mod.rs`: `pub mod stadium;` and `#[cfg(test)] mod stadium_tests;`).
- The gate positions in world XZ for the default seed (R45-04 and R45-06 need them; print them in a test or note them).
- Any `stands` entry you moved to avoid overlap, and why.
- Whether the stadium group touches an avenue (the front the gates should face) and what you did if it does not.

## Worker notes
