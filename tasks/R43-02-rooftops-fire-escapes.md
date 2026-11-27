# R43-02: Rooftop links (planks + ladders) and working fire escapes

Owns:
- `world/src/metro/rooftops.rs` (NEW, pure), `world/src/metro/fire_escapes.rs` (NEW, pure), `world/src/metro/rooftops_tests.rs` (NEW)
- `data/metro.json` (ONLY the new keys `rooftops` and `fireEscapes`)
- `world/src/metro/config.rs` (ONLY: serde for those two keys, `#[serde(default)]`. R43-01 edits the same file for `climb`: add your fields in your own block and don't touch theirs)

Lines for the lead: `pub mod rooftops; pub mod fire_escapes;` + `#[cfg(test)] mod rooftops_tests;` in `world/src/metro/mod.rs`; the `rooftops_in_chunk` call in `generator_contents.rs` (R43-08).
Not owned: `climb.rs` (R43-01 writes `roof_deck_y`, `climb_towers`, `ClimbChunk`, `ClimbLadder` first: import them, don't copy them), models, rotland/*.
Read: tasks/R43-common.md (whole), `world/src/metro/towers.rs`, `styles.rs`, `state.rs` (`towers_by_block`), `tower_parts.rs`, the crown scripts in `tools/blender/towers/` (`brick.py`, `stone.py`, `glass.py`, `slab.py`, `deco.py`, `roof_parts.py`: find the **parapet top height above the deck** of each style), `brick.py` + `brick_parts.py` (the fire escape: which face, `fe_u`, landing depth 1.2 m, the drop ladder on the base, `roof=True`), `game_player/src/ladders/rail.rs`.

## Data (`data/metro.json`)
```json
"rooftops": {
  "_why": "R43: plank bridges from climbable / fire-escape roofs to nearby roofs.",
  "maxGapM": 12.0, "minOverlapM": 3.0, "maxDropM": 8.0, "flatStepM": 0.6,
  "linksPerTower": 2, "plankLengths": [4, 8, 12],
  "parapetTop": { "brick": 0.0, "stone": 0.0, "glass": 0.0, "slab": 0.0, "deco": 0.0 }
},
"fireEscapes": {
  "_why": "R43: brick towers whose drawn fire escapes become walkable.",
  "count": 8, "districts": ["edge", "midtown"], "maxFloors": 12, "minSpacingBlocks": 2
}
```
Fill `parapetTop` (metres above each style's deck) from the crown scripts.

## Rooftop links (`roof_links`, `rooftops_in_chunk`)
- **Sources:** every climb tower (`climb_towers`) and every fire-escape tower.
- **Partners:** any standing non-landmark tower, not replaced.
- **Fit:**
  - The facing edges of the two square footprints (both rotated by their yaw; towers in one block share the site yaw up to quarter turns) are parallel.
  - The edge gap is `1.5 ..= maxGapM`, and the edges overlap by at least `minOverlapM`.
  - Take the nearest `linksPerTower` per source; never link the same pair twice.
- **Plank:**
  - Use the shortest `roof_plank_<L>` with `L >= gap + 2` (1 m resting on each side). It lies across the gap at the middle of the overlap, yaw along the gap.
  - Its height is the LOWER roof's `deck + parapetTop` (it rests on the parapet).
  - Put a `roof_step_crate` on each deck under each plank end when the parapet top is more than 0.3 m above that deck.
- **Height difference `dh`** (higher deck minus the plank top):
  - `dh <= flatStepM`: nothing more.
  - `flatStepM < dh <= maxDropM`: a `ClimbLadder` up the taller tower's wall from the plank end to its deck (the deck point 1 m inside), with `roof_ladder_seg` props every 1.0 m.
  - More than `maxDropM`: no link.
- **Colliders:** planks and crates have model colliders (R43-05). A plank must not cut through any tower hull box: test it.
- Output into `ClimbChunk` by the chunk of the plank's centre. Ids `metro_roof_<a>_<b>`.

## Fire escapes (`fire_escape_towers`, `fire_escape_ladders`)
- **Pick:** `count` brick towers in `districts`, standing, `floors <= maxFloors`, not climb towers, spread by `minSpacingBlocks`. Seeded like the other picks; deterministic.
- **Ladders** (tower-local from the brick script numbers, put into world space):
  - **Drop ladder:** street -> first landing.
  - **Roof ladder:** top landing -> over the parapet onto the deck.
- Each brick floor's zig-zag stairs get colliders from the brick models themselves (R43-05 adds them). Here you only give the ladders and the tower list. R43-08 loads part colliders for the listed towers only.
- If the brick fire escape sits on a face that R43-05 reports differently, follow R43-05's Worker notes (`fe_u`, face, sign).

## Tests (`rooftops_tests.rs`)
- **Links:**
  - Every link's gap and overlap are inside the limits, and its plank length covers the gap + 2.
  - No pair twice; at most `linksPerTower` per source.
  - Deterministic.
- **Planks:** no plank's footprint box overlaps any tower hull box, and every plank lies at the lower deck + parapet.
- **Ladders:** ladders only when `dh > flatStepM`; each foot is at the plank top and each deck at the higher roof's deck.
- **Fire escapes:**
  - `count` towers, all brick, floors in range, none a climb tower, spacing holds.
  - Two ladders each: drop ladder foot at street level, roof ladder deck at `roof_deck_y`.
- **No change elsewhere:** no other tower changes.

## Done when
Code + data + tests written. Fill `## Worker notes` (the parapet values and where you read them).

## Worker notes
