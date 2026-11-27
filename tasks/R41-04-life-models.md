# R41-04: Silent life models (poles, wires, billboards, flags, plastic, crows)

Owns:
- `tools/blender/life/` (NEW folder: `life.py` with `build_all()`, plus helper files such as `poles.py`, `billboards.py`, `flappers.py`, `crows.py`)
- the new .glb files in `assets/models/props/` and `assets/models/animals/` (only the names below)
- their PNGs in `assets/models/_previews/`
- `tools/blender/build_r41_models.py` (NEW, the round's one Blender entry; it calls `life.build_all`, then the R41-07 `items_r41.build_all`, and prints `R41 BUILD OK`)

Not owned: every other Blender script and model.
Read: tasks/R41-common.md ("Life models": the exact names), the `blender-art` skill notes in `tools/blender/README*` and an R40 script to copy the style of (`tools/blender/traces/traces.py`, `tools/blender/traces/graffiti_font.py` for painted letters, `tools/blender/overgrowth/og_util.py`), the palette and budget rules used there.

## Rules
- Origin at the bottom centre unless said otherwise. Front = +Y. Use the palette materials only (flat style).
- Budgets:
  - poles: 1500
  - wire span / wire down: 600
  - billboard frames: 4000
  - faces: 400
  - flag pole: 300
  - each flag / plastic frame: 300
  - each crow: 400
  - Each model has a `_lod1` at about 1/10. Crows, flags and plastic get no lod1: they are only drawn near.
- **Poles:** `power_pole` is a wooden pole 9 m tall with a cross arm, 3 insulators, a transformer can on some (separate model is not needed; put it on `power_pole`), and climbing spikes. `power_pole_leaning` is the same, baked leaning 12 degrees toward +Y. `power_pole_fallen` lies along +Y, snapped at the base: a short stump stays at the origin, splinters at the break.
- **Wires:** `power_wire_span` runs from x=0 to x=20 at z=8.5 (the cross-arm insulators), 3 wires with a 1.2 m sag (a real curve, 12+ segments), thin (4 cm). `power_wire_down` is 3 wires lying in loose S-curves on the ground over 20 m, 2 cm above it.
- **Billboards:** frames are rusty steel (`billboard_roof_*`: legs, back bracing, catwalk with rail, 2-3 broken lamp arms; `billboard_street`: one thick pole, catwalk). Faces (`billboard_face_a..d`) are flat panels 4 cm in front of the frame's face plane, with the same origin as the frame, so placing both at the same transform lines them up. For the roof pair, the face is 12 x 4 m. The street face is SCALED by 0.66 when placed: write that in Worker notes, or make the faces fit the roof frame and state the street scale.
  - Faded colours, torn strips peeling down, made-up ads with big painted words: "SUMMER SALE" (a sun), "CALL NOW" (a phone shape), "FLY AWAY" (a plane over sea), "SAFE HOME" (a house + shield). Copy the paint-letter approach of `graffiti_font.py`. No real brands or names.
- **Flags:** `flag_pole_wall` is a 2.5 m pole with a wall bracket; its origin is at the bracket. `flag_torn_f0..f3` are 4 frames of one torn, faded flag (made-up stripes and a star, ragged trailing edge, 1.5 x 1 m). The pivot is at the pole tip, and the flag hangs along +X. The frames are a smooth wave cycle (f3 flows back into f0).
- **Plastic:** `plastic_flap_f0..f3` is a torn grey-blue tarp sheet, 1.2 x 1.6 m, hanging from its top edge (pivot at the top centre), in 4 frames of a flutter cycle.
- **Crows** (category `animals`, matte black with a dark blue-grey sheen, yellow-grey beak):
  - `crow_perched`: standing.
  - `crow_peck`: head down.
  - `crow_fly_f0..f3`: wings up, mid, down, mid. The pivot is the body centre for flying and the feet for perched/peck.
  - 0.45 m long, wings 0.9 m span.

## Blender run and checks
- Run (you MAY run Blender this way): `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r41_models.py`. If R41-07's `items_r41.py` is not there yet, guard the import so the life part still runs.
- Read the log: no `Traceback`, no `FAILED`, ends with `R41 BUILD OK`.
- Look at EVERY preview PNG:
  - Billboard frame + face together: no gap, no z-fight.
  - Flag frames side by side: the wave reads.
  - Crow frames: a wing beat.
  - Poles with a span: the wire meets the insulators.
- Fix and run again until they look right.

## Done when
All listed models exist within budget, the previews look right, and `build_r41_models.py` ends with `R41 BUILD OK`. Fill `## Worker notes
- **Files created:**
  - `tools/blender/life/life.py`, `poles.py`, `billboards.py`, `flappers.py`, `crows.py`
  - `tools/blender/build_r41_models.py` (guards `items_r41` import until R41-07 lands; prints `R41 BUILD OK`)
  - 34 `.glb` files in `assets/models/props/` + 6 `.glb` files in `assets/models/animals/` + 40 PNGs in `assets/models/_previews/`
- **Street billboard face scale:** `0.66` around `(0, 0, 0)` (place `billboard_face_a..d` at the exact same transform as `billboard_street` with uniform scale `0.66`; roof billboards `billboard_roof_a/b` use scale `1.0`).
- **Pivots & alignment:**
  - `power_pole`, `power_pole_leaning` (12° toward `+Y`), `power_pole_fallen` (snapped stump at origin, trunk along `+Y`): bottom center `(0, 0, 0)`. Cross-arm runs along `Y` with 3 insulators at `y = -0.90, 0.0, +0.90, z = 8.50`.
  - `power_wire_span` (`x = 0..20, z = 8.50`, 1.2 m sag, 14 segs) & `power_wire_down` (`x = 0..20, z = 0.02`, S-curves, 14 segs): pivot `(0, 0, 0)` so placing `power_wire_span` at a `power_pole`'s origin meets its 3 insulators at `y = -0.90, 0.0, +0.90, z = 8.50`.
  - `billboard_roof_a/b`, `billboard_street`, `billboard_face_a..d`: pivot `(0, 0, 0)` at bottom center. Roof backing at `y = 0.20`, face at `y = 0.24` (4 cm in front); street backing at `y = 0.122`, scaled face at `y = 0.1584` (~3.6 cm in front).
  - `flag_pole_wall`: pivot `(0, 0, 0)` at wall bracket; pole extends 2.5 m at 45° toward `+Y/+Z`, tip at `(0, 1.768, 1.768)`.
  - `flag_torn_f0..f3`: pivot `(0, 0, 0)` at pole tip, hangs along `+X` (`x = 0..1.5, z = -1.0..0.0`).
  - `plastic_flap_f0..f3`: pivot `(0, 0, 0)` at top centre (`x = -0.6..0.6, z = -1.6..0.0`).
  - `crow_perched`, `crow_peck`: pivot `(0, 0, 0)` at feet (`z = 0`).
  - `crow_fly_f0..f3`: pivot `(0, 0, 0)` at body centre (0.45 m long, 0.90 m wingspan).
- **Tri counts (all within budget):**
  - `power_pole` / `power_pole_leaning`: 464 (budget 1500), `_lod1`: 64 (budget 150)
  - `power_pole_fallen`: 240 (budget 1500), `_lod1`: 44 (budget 150)
  - `power_wire_span` / `power_wire_down`: 348 (budget 600), `_lod1`: 60 (budget 60)
  - `billboard_roof_a`: 516 (budget 4000), `_lod1`: 72 (budget 400)
  - `billboard_roof_b`: 432 (budget 4000), `_lod1`: 60 (budget 400)
  - `billboard_street`: 268 (budget 4000), `_lod1`: 44 (budget 400)
  - `billboard_face_a..d`: 94, 56, 54, 74 (budget 400), `_lod1`: 4 each (budget 40)
  - `flag_pole_wall`: 60 (budget 300), `_lod1`: 24 (budget 30)
  - `flag_torn_f0..f3`: 92 each (budget 300); `plastic_flap_f0..f3`: 76 each (budget 300)
  - `crow_perched`, `crow_peck`, `crow_fly_f0..f3`: 140 each (budget 400)` (the face scale for the street billboard, the pivots).

## Worker notes
- **Files created:**
  - `tools/blender/life/life.py`, `poles.py`, `billboards.py`, `flappers.py`, `crows.py`
  - `tools/blender/build_r41_models.py` (guards `items_r41` import until R41-07 lands; prints `R41 BUILD OK`)
  - 34 `.glb` files in `assets/models/props/` + 6 `.glb` files in `assets/models/animals/` + 40 PNGs in `assets/models/_previews/`
- **Street billboard face scale:** `0.66` around `(0, 0, 0)` (place `billboard_face_a..d` at the exact same transform as `billboard_street` with uniform scale `0.66`; roof billboards `billboard_roof_a/b` use scale `1.0`).
- **Pivots & alignment:**
  - `power_pole`, `power_pole_leaning` (12° toward `+Y`), `power_pole_fallen` (snapped stump at origin, trunk along `+Y`): bottom center `(0, 0, 0)`. Cross-arm runs along `Y` with 3 insulators at `y = -0.90, 0.0, +0.90, z = 8.50`.
  - `power_wire_span` (`x = 0..20, z = 8.50`, 1.2 m sag, 14 segs) & `power_wire_down` (`x = 0..20, z = 0.02`, S-curves, 14 segs): pivot `(0, 0, 0)` so placing `power_wire_span` at a `power_pole`'s origin meets its 3 insulators at `y = -0.90, 0.0, +0.90, z = 8.50`.
  - `billboard_roof_a/b`, `billboard_street`, `billboard_face_a..d`: pivot `(0, 0, 0)` at bottom center. Roof backing at `y = 0.20`, face at `y = 0.24` (4 cm in front); street backing at `y = 0.122`, scaled face at `y = 0.1584` (~3.6 cm in front).
  - `flag_pole_wall`: pivot `(0, 0, 0)` at wall bracket; pole extends 2.5 m at 45° toward `+Y/+Z`, tip at `(0, 1.768, 1.768)`.
  - `flag_torn_f0..f3`: pivot `(0, 0, 0)` at pole tip, hangs along `+X` (`x = 0..1.5, z = -1.0..0.0`).
  - `plastic_flap_f0..f3`: pivot `(0, 0, 0)` at top centre (`x = -0.6..0.6, z = -1.6..0.0`).
  - `crow_perched`, `crow_peck`: pivot `(0, 0, 0)` at feet (`z = 0`).
  - `crow_fly_f0..f3`: pivot `(0, 0, 0)` at body centre (0.45 m long, 0.90 m wingspan).
- **Tri counts (all within budget):**
  - `power_pole` / `power_pole_leaning`: 464 (budget 1500), `_lod1`: 64 (budget 150)
  - `power_pole_fallen`: 240 (budget 1500), `_lod1`: 44 (budget 150)
  - `power_wire_span` / `power_wire_down`: 348 (budget 600), `_lod1`: 60 (budget 60)
  - `billboard_roof_a`: 516 (budget 4000), `_lod1`: 72 (budget 400)
  - `billboard_roof_b`: 432 (budget 4000), `_lod1`: 60 (budget 400)
  - `billboard_street`: 268 (budget 4000), `_lod1`: 44 (budget 400)
  - `billboard_face_a..d`: 94, 56, 54, 74 (budget 400), `_lod1`: 4 each (budget 40)
  - `flag_pole_wall`: 60 (budget 300), `_lod1`: 24 (budget 30)
  - `flag_torn_f0..f3`: 92 each (budget 300); `plastic_flap_f0..f3`: 76 each (budget 300)
  - `crow_perched`, `crow_peck`, `crow_fly_f0..f3`: 140 each (budget 400)
