# R42-02: Place the failed army line

Owns:
- `world/src/metro/army_line.rs` (NEW, pure) + `world/src/metro/army_line_tests.rs` (NEW)
- `data/places/army_line.json` (NEW: the whole layout, written by you)

Lines for the lead: `pub mod army_line;` + `#[cfg(test)] mod army_line_tests;` in `world/src/metro/mod.rs`, and the generator call (with R42-01's, same pattern).
Not owned: the models (R42-06 makes them; use EXACTLY the names in the common file), `places.rs` (R42-01; you may import its `PlaceChunk` struct; if it is not written yet, define the same struct shape and tell the lead).
Read: tasks/R42-common.md (whole), `world/src/metro/layout.rs` (`reserved`, streets, avenues), `world/src/metro/traces_walls.rs` (car-wall placement across a street: copy its "across the street" math), `world/src/metro/herds*.rs` (herds must not path through the line: write in Worker notes what R42-09 must add to the herd `blocked` list), `world/src/metro/loot.rs`.

## Where
- The army line's strip is reserved `army_line` (4x1 blocks).
- Find the avenue that runs along the strip's long side. The strip centre point on that avenue is the line's origin; the avenue direction is local +x.
- If no avenue touches the long side, use the street along it, and say so in Worker notes.

## Layout (`data/places/army_line.json`, local frame from the common file)
Write the composition as data, so the owner can tweak it:
- **The barrier across the avenue:**
  - It sits at the strip's middle (`x` around 0).
  - `sandbag_wall` pieces and `concrete_barrier`s fill the full road width plus sidewalks, with ONE breach where the wall is blown open (rubble + a `car_burnt_pile`).
  - Two `razor_wire` coils run in front (on the outside, toward the city centre).
- **The tank** sits behind the barrier in the avenue, slightly turned. **Burnt army trucks** (2-3) are parked askew along the avenue behind it, plus one `army_truck_jam` still standing.
- **The crashed helicopter** lies on the strip's lot nearest the centre, nose down, tilted. A separate `heli_rotor_blade` lies 10-20 m away, with a scorched patch (`ash_patch`).
- **Evacuation camp** on the other strip lots:
  - 4-6 `evac_tent`s in rows, `cot`s inside some of them (R42-03 makes `cot`)
  - a `field_kitchen`, `flood_light`s
  - 2-3 `loudspeaker_pole`s along the sidewalk, a `checkpoint_booth` at the barrier end
  - `army_gear_scatter`, `body_bag`s and a `body_bag_pile` near the tents
  - `sandbag_corner` nests at the corners
- **Crates:** 6-8 `army_crate` (`metro_army` table), plus 2 `med_cabinet`s in a tent (`metro_medical`).

## Rust (`army_line.rs`)
- `pub fn army_line_site(layout) -> Option<ArmyLineSite { origin: Vec2, dir: Vec2, yaw: f32, strip: Vec<(i32,i32)> }>`.
- `pub fn army_line_in_chunk(metro, chunk, height) -> PlaceChunk`: transform the data entries to world coordinates (`x` along `dir`, `z` across, toward the strip), y = `height + entry.y`. Each entry belongs to exactly one chunk.
- POI `metro_army_line` at the barrier middle (spawn points come from R42-07's data, not a model).
- `pub fn army_line_blocked_points(...) -> Vec<Vec2>`: points that herds and the car-wall placement must avoid (the barrier ends and the middle).

## Tests (`army_line_tests.rs`)
- The site exists, and its origin is on a street (`layout.street_at`).
- The barrier entries cover the full road width (the outermost barrier pieces are within 1 m of each curb), except one breach gap of 3-6 m.
- Nothing is placed inside a tower footprint.
- Every entry is emitted by exactly one chunk.
- Same seed gives the same result.
- Every model named in the data file exists in `assets/models/**` (skip a name if R42-06 has not made it yet, but list the missing ones in the test output; the lead's run must find none).

## Done when
Code + data + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
