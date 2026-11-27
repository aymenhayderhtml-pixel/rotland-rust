# R43-01: The climb plan (which towers, stair modules, stop-off floors, colliders)

Owns:
- `world/src/metro/climb.rs` (NEW, pure), `world/src/metro/climb_hulls.rs` (NEW, pure: `climb_hull_boxes`), `world/src/metro/climb_tests.rs` (NEW)
- `data/metro.json` (ONLY the new key `climb`)
- `world/src/metro/config.rs` (ONLY: serde for `climb`, `#[serde(default)]` so old JSON loads)

Lines for the lead (write them in Worker notes):
- `pub mod climb; mod climb_hulls;` + `#[cfg(test)] mod climb_tests;` in `world/src/metro/mod.rs`, plus the `pub use` list
- the `climb_in_chunk` call in `world/src/generator_contents.rs` (R43-08 wires it)

Not owned: models (R43-03/04/05), `data/climb_floors.json` (R43-04; until it exists, use fixture crates in tests and an empty list at runtime), rotland/* (R43-08).
Read: tasks/R43-common.md (whole), `world/src/metro/towers.rs`, `tower_parts.rs`, `styles.rs`, `state.rs`, `places.rs` (`place_sites_cached`, `LotFrame`, `push_poi`, the crate/prop pushing), `world/src/poi.rs`, `world/src/loot.rs`, `rotland/src/metro_render/hulls.rs` + `parts.rs` (how `is_tallest_landmark` is found; copy that rule), `tools/blender/towers/landmarks.py` (the mast: deck height at the crown base, leg positions, platform at +18 m, the hut and its door).

## Do this first
Write `roof_deck_y` and the `ClimbTower`/`ClimbChunk`/`ClimbLadder`/`ClimbBox` types first, so R43-02 can import them.
- Normal towers: `crown_y + roofDeckHeight`.
- Landmarks: the deck height read from `landmarks.py`.

## `climb` data (`data/metro.json`)
```json
"climb": {
  "_why": "R43: climbable towers. Counts per district + the tallest landmark; floors counted from 0 above the lobby.",
  "towers": { "midtown": 2, "core": 2 },
  "minSize": 24, "minSpacingBlocks": 3,
  "stopFloors": { "midtown": 2, "core": 3, "landmark": 4 },
  "stopKinds": [ { "kind": "office", "weight": 0.45 }, { "kind": "flat", "weight": 0.35 }, { "kind": "camp", "weight": 0.2 } ],
  "stairBChance": 0.3, "stairZombieChance": 0.12,
  "shaftCentreZShare": 0.25, "shaftSize": [4.4, 6.4],
  "hatch": [3.4, 0.0],
  "mast": { "platformY": 18.0, "platformHalf": 3.0, "ladderFoot": [0.0, 0.0], "ladderTop": [0.0, 0.0] }
}
```
- Fill `mast.ladderFoot`/`ladderTop` (crown-local x/z) from `landmarks.py`: a ladder on the inside face of one leg, from the roof deck to the platform edge. If no leg face fits, put the ladder up the platform's centre post and say so.
- The `hatch` z is added to the shaft centre z.

## Picking (`climb_towers`)
- **Candidates:**
  - Standing towers only: not `damage.tower_is_replaced`, not in the burnt block (all `_floor_c`), not `landmark`, not the `SafeHouseTower` reserved block.
  - `size >= minSize`, district Midtown/Core by kind.
- **Choice:**
  - Pick `towers.midtown` + `towers.core` with a seeded rng (the site seed, as `towers.rs` `mix_seed`).
  - Each pick at least `minSpacingBlocks` (Chebyshev, in blocks) from the others.
  - Prefer towers whose lobby faces an avenue (a tie-break, not a rule).
  - Then add the tallest landmark as `ClimbKind::Landmark`.
- **Stop floors:**
  - `stopFloors[kind]` floors spread evenly between 25% and 85% of `floors`, never floor 0, never two adjacent.
  - Plus the top floor `floors-1`, always.
  - Kinds are weighted from `stopKinds`, seeded per tower. The top floor of the tallest landmark is always `Camp` (the last stand by the radio).
- Deterministic: same seed, same result. Memoise per layout like `place_sites_cached`.

## Per chunk (`climb_in_chunk`), for each climb tower whose CENTRE is in `chunk`
All props use the shaft frame (common). Module `y` = floor `i` start; the lobby has two modules at `ground_y` and `ground_y + 3.8`.
- **Stair modules (props):**
  - Lobby modules: `tw_stair_open` (bottom) and `tw_stair_flight`.
  - Floor `i` < top: `tw_stair_open` on stop floors; else `tw_stair_flight` or `tw_stair_flight_b` (`stairBChance`).
  - Some floors become a stairwell POI instead (`stairZombieChance`): a `PoiInstance` with building `tw_stair_flight_z`, id `metro_stair_<tower>_<i>`. It is NOT also emitted as a prop.
  - The top floor gets `tw_stair_top`.
- **Floor numbers:** two `stair_digit_*` props on the landing wall beside the door. The number shown is `i + 2`; one digit under 10.
- **Lobby sign:** `stair_sign_roof` at the lobby front, beside the door (base-local; read `tools/blender/towers/shell.py` for the door gap).
- **Stop floors:**
  - Each is a `PoiInstance`: building `tw_interior_<kind>_<size>` (size 24 or 32; landmark setback floors use the setback module size there, as in `tower_parts.rs`), pos = tower centre at the floor start, yaw = tower yaw, id `metro_floor_<tower>_<i>`.
  - Loot table: `metro_tower_office` / `metro_tower_flat` / `metro_tower_camp` (R43-06).
  - Crates come from `data/climb_floors.json` (R43-04) for that model, transformed like `places.rs`. Crate ids `metro_floor_<tower>_<i>_c<n>`.
- **Roof hatch:**
  - Ladder `ClimbLadder` from the top floor at the hatch spot (foot) to the deck (deck point 1.0 m past the hatch toward the shaft-free side).
  - `roof_ladder_seg` props stacked every 1.0 m from the top floor up to the deck.
  - A `roof_hatch` prop on the deck.
- **Mast** (tallest landmark only):
  - The ladder from the deck to the platform, with `roof_ladder_seg` props.
  - `ClimbBox`es for the platform slab and its 4 rail sides.
  - A `radio_console` prop beside the hut door, plus the radio crate: a `LootSpot` id `metro_radio_crate`, model `army_crate`, table `metro_radio`.

## Colliders (`climb_hull_boxes`): replaces the single upper box of a climb tower
Tower-local boxes, frame and origin as `box_points` (the base centre on the ground). From `baseHeight` to the roof deck:
- **Solid bands** between stop floors: 4 boxes around the shaft rect (front, back, left, right of it), so the shaft column stays hollow.
- **Each stop floor (3.8 m):**
  - a floor slab 0.3 m thick, minus the shaft rect
  - the 4 outer walls, 0.3 m thick, full height
  - The interior model provides inner walls and furniture.
- **Above the top floor:** a roof slab from the top floor ceiling to the deck, minus the shaft rect AND a 1.4 x 1.4 m hatch hole.
- **Setbacks (landmark):** use the module size of each band.
- **Lobby:** nothing new. The lobby base colliders stay; the bottom module walls sit inside the lobby.

## Tests (`climb_tests.rs`, plus `climb_hulls` unit tests)
- **Picking:**
  - The right counts per district + 1 landmark.
  - Spacing holds.
  - None is replaced / burnt / the safe house.
  - Deterministic.
- **Stop floors:**
  - The count per kind.
  - The top floor is always there; none adjacent; none at floor 0.
  - The tallest landmark's top is a Camp.
- **Modules:**
  - Exactly one module (prop or stairwell POI) per storey, from the lobby to the top.
  - Every stop floor has `tw_stair_open`; the top has `tw_stair_top`.
  - No module is both a prop and a POI.
- **Hull boxes:**
  - No box overlaps the shaft rect, nor the hatch hole above the top floor.
  - Every non-stop storey is fully solid outside the shaft.
  - Every stop floor has a slab + 4 walls.
  - The top of the highest box = `roof_deck_y`.
  - The mast platform box is at `platformY` above the crown base.
- **Ladders:**
  - The hatch ladder's foot is on the top floor and its deck at `roof_deck_y`.
  - Mast ladder foot on the deck, top at the platform.
- **Shaft fit:** the shaft rect fits inside every module size of every climb tower (setbacks too).
- **No change elsewhere:** a non-climb tower is unchanged. `world/tests/golden.rs` untouched.

## Done when
Code + data + tests written (the lead builds and tests). Fill `## Worker notes
- **Files changed:**
  - `world/src/metro/climb.rs` (NEW): `ClimbKind`, `StopKind`, `ClimbTower`, `ClimbLadder`, `ClimbBox`, `ClimbChunk`, `FloorCrateEntry`, `ClimbConfig`, `roof_deck_y`, `climb_towers`, `climb_in_chunk`, `climb_in_chunk_with`.
  - `world/src/metro/climb_hulls.rs` (NEW): `climb_hull_boxes` (tower-local boxes from `baseHeight` to `roof_deck_y`, hollow shaft, stop-floor slab + 4 walls, roof hatch hole, landmark setbacks).
  - `world/src/metro/climb_tests.rs` (NEW): unit tests for picking, stop floors, modules, ladders, mast, hull boxes, and shaft fit.
  - `data/metro.json`: added `climb` key.
  - `world/src/metro/config.rs`: added `#[serde(default)] pub climb: crate::metro::climb::ClimbConfig` to `MetroConfig`.
- **Mast ladder note:** In `tools/blender/towers/landmarks.py`, the 4 mast legs taper inward from `+-5.5` at `z=0.2` to `+-2.2` at `z=18.0` and the 6x6 m platform (`-3.0..+3.0`) overhangs them, so no leg face fits a vertical ladder to the platform edge; and `(0.0, 0.0)` is occupied by `radio_hut` and the upper mast. Placed `mast.ladderFoot` at `[0.0, 3.2]` and `mast.ladderTop` at `[0.0, 2.6]` (clear `+Z` platform edge, with a matching gap in the `+Z` rail collider).
- **Lines for `world/src/metro/mod.rs`:**
  ```rust
  pub mod climb;
  mod climb_hulls;
  #[cfg(test)]
  mod climb_tests;
  ```
  and `pub use`:
  ```rust
  pub use climb::{
      ClimbBox, ClimbChunk, ClimbConfig, ClimbKind, ClimbLadder, ClimbTower, FloorCrateEntry,
      StopKind, climb_hull_boxes, climb_in_chunk, climb_in_chunk_with, climb_towers, roof_deck_y,
  };
  ```
- **Lines for `world/src/generator_contents.rs`** (inside `if let Some(metro) = self.metro_world()`, alongside `places_in_chunk` / `army_line_in_chunk`):
  ```rust
  let climb = crate::metro::climb_in_chunk(metro, chunk, hfn);
  pois.extend(climb.pois);
  props.extend(climb.props);
  ````.

## Worker notes
- **Files changed:**
  - `world/src/metro/climb.rs` (NEW): `ClimbKind`, `StopKind`, `ClimbTower`, `ClimbLadder`, `ClimbBox`, `ClimbChunk`, `FloorCrateEntry`, `ClimbConfig`, `roof_deck_y`, `climb_towers`, `climb_in_chunk`, `climb_in_chunk_with`.
  - `world/src/metro/climb_hulls.rs` (NEW): `climb_hull_boxes` (tower-local boxes from `baseHeight` to `roof_deck_y`, hollow shaft, stop-floor slab + 4 walls, roof hatch hole, landmark setbacks).
  - `world/src/metro/climb_tests.rs` (NEW): unit tests for picking, stop floors, modules, ladders, mast, hull boxes, and shaft fit.
  - `data/metro.json`: added `climb` key.
  - `world/src/metro/config.rs`: added `#[serde(default)] pub climb: crate::metro::climb::ClimbConfig` to `MetroConfig`.
- **Mast ladder note:** In `tools/blender/towers/landmarks.py`, the 4 mast legs taper inward from `+-5.5` at `z=0.2` to `+-2.2` at `z=18.0` and the 6x6 m platform (`-3.0..+3.0`) overhangs them, so no leg face fits a vertical ladder to the platform edge; and `(0.0, 0.0)` is occupied by `radio_hut` and the upper mast. Placed `mast.ladderFoot` at `[0.0, 3.2]` and `mast.ladderTop` at `[0.0, 2.6]` (clear `+Z` platform edge, with a matching gap in the `+Z` rail collider).
- **Lines for `world/src/metro/mod.rs`:**
  ```rust
  pub mod climb;
  mod climb_hulls;
  #[cfg(test)]
  mod climb_tests;
  ```
  and `pub use`:
  ```rust
  pub use climb::{
      ClimbBox, ClimbChunk, ClimbConfig, ClimbKind, ClimbLadder, ClimbTower, FloorCrateEntry,
      StopKind, climb_hull_boxes, climb_in_chunk, climb_in_chunk_with, climb_towers, roof_deck_y,
  };
  ```
- **Lines for `world/src/generator_contents.rs`** (inside `if let Some(metro) = self.metro_world()`, alongside `places_in_chunk` / `army_line_in_chunk`):
  ```rust
  let climb = crate::metro::climb_in_chunk(metro, chunk, hfn);
  pois.extend(climb.pois);
  props.extend(climb.props);
  ```
