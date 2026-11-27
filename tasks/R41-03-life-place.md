# R41-03: Silent life placement (power lines, billboards, flags, plastic, crow perches)

Owns:
- `world/src/metro/life.rs` (NEW), `world/src/metro/life_wires.rs` (NEW: poles and wire spans), `world/src/metro/life_tests.rs` (NEW)
- `world/src/metro/config.rs` (ONLY the new optional `life: LifeConfig` field, `#[serde(default)]`)
- `data/metro.json` (ONLY the new top-level key `life`)

Lines for the lead: `pub mod life; mod life_wires;` + `#[cfg(test)] mod life_tests;` in `world/src/metro/mod.rs`, and the call in `world/src/generator_contents.rs` next to `traces_props_in_chunk` (write the exact line in Worker notes).
Not owned: the damage files (R41-10 owns them), `rotland/*`, the models (R41-04).
Read: tasks/R41-common.md ("Life placement", "Life models"), `world/src/metro/traces.rs` + `traces_camps.rs` (style to copy, `roof_top_y`), `world/src/metro/overgrowth.rs` (district density pattern), `world/src/metro/tower_parts.rs`, `world/src/metro/layout.rs`, `world/src/metro/state.rs`, `world/src/generator_contents.rs` (how metro props are added per chunk).

## Data (`data/metro.json` -> `life`)
```json
"life": {
  "_why": "R41: silent life. Poles on some avenues; per 64 m block chances below.",
  "poleSpacingM": 20.0,
  "poleAvenueShare": 0.35,
  "poleFallenChance": 0.15,
  "poleLeaningChance": 0.2,
  "billboardRoofChance": 0.18,
  "billboardStreetChance": 0.12,
  "flagChance": 0.35,
  "plasticChance": 0.5,
  "crowPerchChance": { "edge": 0.55, "midtown": 0.45, "core": 0.35 },
  "crowsPerPerch": [3, 9]
}
```

## What to place
- **Power lines** (`life_wires.rs`):
  - Pick `poleAvenueShare` of the avenues. Along each, put poles on one sidewalk every `poleSpacingM`, offset 0.6 m from the curb.
  - Between standing neighbours, add a `power_wire_span` at pole-top height (8.5 m on the model). It is scaled in X to the span length and yawed along the street.
  - A pole can be `power_pole_leaning` (wire still spans, a bit lower) or `power_pole_fallen` (lies across the road). A fallen pole breaks the spans on both sides; those become `power_wire_down` lying across the street.
  - Skip poles where a lobby door, a car wall, the fallen tower or a reserved block is.
- **Billboards**:
  - `billboard_roof_a/b` on low roofs (Brick/Slab towers, 4-10 floors). They stand at `roof_top_y` (re-use the R40 roof-deck height function), facing the nearest avenue.
  - `billboard_street` in wild lots and at corners, facing the street.
  - Every billboard gets ONE face model (`billboard_face_a..d`, picked by rng) at the same transform. The face's pivot matches the frame (R41-04 makes them line up).
- **Flags**: `flag_pole_wall` on some lobby fronts at 4-6 m height, sticking out of the facade (2-5 cm into the wall, never floating). Each one also gives a `Flapper { kind: Flag }` at the pole tip.
- **Plastic**: a `Flapper { kind: Plastic }` on the top edge of some car walls and on wild-lot fences, hanging.
- **Crow perches** (`crow_perches_in_chunk`), chance per block by district:
  - `Roof`: the roof edge of a low tower.
  - `Wire`: on a span, at the sag height.
  - `Car`: on the roof of a jam/street car.
  - `Body`: next to a `body_sheet`/`body_sheet_b` (the crows peck).
  - `Street`: on the pavement.

  `count` is from `crowsPerPerch`.
- Every number has a data tag; deterministic per seed and chunk; a prop belongs to the chunk that holds its position (no doubles across chunk edges).

## Tests (`life_tests.rs`)
- Same seed gives the same props, flappers and perches; another seed differs.
- Poles stand on sidewalks: never on a road centre unless fallen, never inside a tower footprint.
- Every span's two ends are at standing poles, and the span length matches its X scale (within 5 cm).
- Every billboard has exactly one face at the same position and yaw.
- Flags sit at a lobby front, 4-6 m high.
- Perch counts are in range, and Core has fewer perches per block than Edge over the whole city.
- Nothing is placed outside New Halden (a chunk far away gives empty lists).
- `world/tests/golden.rs` is untouched and passes.

## Done when
Code + tests written (the lead builds and tests). Fill `## Worker notes`.

## Worker notes
