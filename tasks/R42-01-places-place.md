# R42-01: Place the hospital, police station and fire station

Owns:
- `world/src/metro/places.rs` (NEW, pure), `world/src/metro/places_data.rs` (NEW: serde for `data/places/*.json`), `world/src/metro/places_tests.rs` (NEW)
- `world/src/metro/paint.rs` (ONLY: ground paint for the place lots, see below)

Lines for the lead:
- `pub mod places; mod places_data;` + `#[cfg(test)] mod places_tests;` in `world/src/metro/mod.rs`
- the calls in `world/src/generator_contents.rs` (POIs into the chunk's POI list, props, crates into `chunk_crates_from`)

Write the exact lines in Worker notes.
Not owned: `data/places/*.json` (written by R42-03/04/05). While those tasks run, test with small fixture JSON written inside your tests, plus one test that loads the real files and is skipped when a file is missing (`if !path.exists() { return; }`).
Read: tasks/R42-common.md (whole), `world/src/metro/layout.rs`, `world/src/metro/state.rs`, `world/src/metro/traces_camps.rs` (the per-chunk prop style), `world/src/metro/loot.rs` (LootSpot ids/tables), `world/src/poi.rs` (`PoiInstance`), `world/src/generator_contents.rs` (`append_town_contents`, `chunk_crates_from`), `world/src/metro/paint.rs`.

## Sites (`place_sites`)
- For each reserved kind `Hospital`, `Police`, `FireStation`, find its 2x2 group of blocks.
- `main_lot` is the block of the four that touches an AVENUE (if several, the one nearest the city centre; if none, the first in grid order).
- `yaw` makes the building front (local -Z) face that avenue.
- `yard_lots` are the other three in a fixed order:
  - `a`: the lot beside the main lot along the same avenue
  - `b`: the lot diagonally opposite
  - `c`: the last one
- `pos` is the main lot's centre, pushed 2 m back from the avenue side so the front has a forecourt.
- Deterministic: no rng needed.

## Per chunk (`places_in_chunk`)
- Load `data/places/<kind>.json` once (cache it in a `OnceLock` keyed by the repo path, or pass a loaded struct through `MetroWorld` if that is cleaner; state it).
- **POI:** one `PoiInstance` per place in the chunk that holds `pos`.
  - id `metro_<kind>`, model from the file, y from `height`, yaw = site yaw.
  - Loot table: `metro_hospital` / `metro_police` / `metro_fire` (R42-08 adds them).
- **Props:** `props` (main-lot local) and `yards.a/b/c` (each lot's local frame; the lot's front faces the same way as the main building). Transform to world, y = `height(x,z) + entry.y`, and keep each one only in the chunk that holds its position.
- **Crates:** `crates` + `yardCrates` -> `LootSpot` with `table`, `model`, id `metro_<kind>_c<idx>`, tier 0.
- Shadows: props under 0.05 m2 cast none (the R40 rule: the lead wires it the same way as the other metro props).

## Ground paint (`paint.rs`)
- Place lots are paved like a plaza:
  - the main lot = sidewalk grey with a darker forecourt
  - yards = asphalt (parking/bays)
  - the hospital triage yard = dirt (`yards.b`)

  Use the existing colours in `metro.json` `look`; no new colours unless needed (then add them to `look`).
- The Park stays as it is.

## Tests (`places_tests.rs`)
- Three sites exist, one per kind. Each `main_lot` touches an avenue, and the front direction points at that avenue.
- Yards are the other three lots of the group (no duplicates; all inside the reserved group).
- Every prop/crate/POI lands inside the place's 2x2 group (with a 6 m margin for the forecourt into the street).
- A prop is emitted by exactly one chunk.
- Same seed gives the same result.
- A chunk far away gives an empty `PlaceChunk`.
- Fixture JSON round-trips; and when the real files exist, every model listed there exists as `assets/models/**/<name>.glb`.
- `world/tests/golden.rs` untouched.

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
