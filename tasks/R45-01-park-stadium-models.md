# R45-01: Blender models for the park and the stadium

Runs at the same time as R45-02, 03, 04. Read first: `tasks/R45-common.md` (whole). Style: `tasks/R44-02-safehouse-models.md`. Load the skill `blender-art` before you write any script (Blender runs only via `blender -b --python`).

Owns:
- `tools/blender/build_r45_models.py` (NEW): the one build script. It calls the modules below and prints `R45 BUILD OK` at the end.
- `tools/blender/park/park_props.py` (NEW): the park models (fountain, bandstand, playground, statue, kiosk, pond, path slab).
- `tools/blender/stadium/stadium_props.py` (NEW): the stadium models (stand, floodlight, gate, pitch, goal).
- `assets/models/props/park_*.glb`, `assets/models/props/stadium_*.glb` and their `_palette.png` files only if the look needs one (copy `tools/blender/palette.py` pattern; reuse an existing palette sheet if one fits).

Not owned: Rust, data, `assets/models.pack` (the lead bakes it; do NOT run `baker`), `tools/README_TOOLS.md` (R45-06 adds the line).

Read (for style, do not edit):
- `tools/blender/build_r44_models.py` (the `build_all()` pattern, export, the end line, the triangle print).
- `tools/blender/places/army_line.py` and `tools/blender/places/place_kit.py` (how a prop with a palette is made), `tools/blender/common.py` (export, budgets, previews).
- `tools/README_TOOLS.md` (the blender lock: run under `flock /tmp/rotland-blender.lock`).

## Models (names EXACT; each file is `assets/models/props/<name>.glb`)
| Name | Size (m, W x H x D) | What it is | Colliders |
|---|---|---|---|
| `park_fountain_dry` | 3.0 x 1.2 x 3.0 | A round stone basin with a cracked rim, a dry centre pillar with a broken bowl on top. Origin at the base centre. | Convex |
| `park_bandstand` | 6.0 x 4.5 x 6.0 | An octagonal rusty iron bandstand: 8 posts, a domed roof with a finial, a rail with gaps, a wooden floor. Origin at the floor centre. | Convex (posts + roof only: keep the floor open) |
| `park_playground` | 6.0 x 3.0 x 3.0 | A slide with a ladder, a swing frame with 2 empty seats, a rusty climbing bar. Origin at the centre. | Convex per part |
| `park_statue` | 1.2 x 2.8 x 1.2 | A worn stone figure (no face detail) on a square plinth, arm raised. Origin at the base centre. | Convex |
| `park_cafe_kiosk` | 3.0 x 2.8 x 2.5 | A small park kiosk: a flat roof with a striped awning, a serving hatch, a dead menu board (no letters), a bench in front. Origin at the base centre, front = local -Z. | Convex |
| `park_pond` | 10.0 x 0.1 x 10.0 | A flat dark-green-black water disc (one flat surface, a ring of mud at the edge). It sits at 0.05 m above the ground. Origin at the centre. | NO collider (name prefix `park_pond`: tell the lead in Worker notes) |
| `park_path_slab` | 2.0 x 0.05 x 1.2 | One flat concrete path slab, cracked, a little moss at one edge. Origin at its centre. | NO collider (name prefix `park_path`: tell the lead) |
| `stadium_stand` | 20.0 x 9.0 x 8.0 | One curved stand section: stepped concrete tiers with rows of plastic seats (flat colours), a back wall with broken windows, a roof edge. Front = local -Z (the pitch side). Origin at the base centre. | Convex (the back wall and the tiers) |
| `stadium_floodlight` | 2.0 x 30.0 x 2.0 | A tall lattice mast with a flat 2 x 1.5 m lamp head at the top (two lamps lit grey, not glowing). Origin at the base centre. | Convex (the mast) |
| `stadium_gate` | 9.0 x 6.0 x 2.0 | An entrance arch with two concrete piers, a rusty sign board (no letters) and two steel gates half open. Front = local -Z. Origin at the base centre. | Convex (piers) |
| `stadium_pitch` | 100.0 x 0.05 x 64.0 | The pitch as a flat dusty dirt-grey plane with worn grass patches (flat colours, no texture). White lines are NOT part of the model. Origin at the centre. | NO collider (name prefix `stadium_pitch`: tell the lead). NO shadow (name prefix `stadium_pitch`: say so in notes) |
| `stadium_goal` | 7.3 x 2.4 x 2.0 | A football goal frame, rusty, with a torn net (flat colour). Origin at the base centre, opening facing local +Z. | Convex |

Keep every model under 1500 triangles, except `stadium_stand` (up to 4000) and `stadium_floodlight` (up to 2000).

## Style rules
- Low-poly, flat colours, dirty and worn. Each part is a separate object; one material per colour.
- Origins as in the table. Export with `+Y up`, metres, no animation, no cameras, no lights.
- Nothing should z-fight: details sit at least 2 cm off the faces they sit on. `park_pond` and `park_path_slab` are flat: keep their top 5 cm above the ground origin, never below.
- Run `python3 tools/find_coplanar.py <file>` (plain python, no Blender) on every `.glb` and fix any flicker-prone face pairs listed. Note the count in Worker notes.

## Script rules
- `tools/blender/build_r45_models.py` imports `park.park_props.build_all()` and `stadium.stadium_props.build_all()`, exports each name above to `assets/models/props/`, re-imports each file once (no error), prints each triangle count, and prints `R45 BUILD OK` at the end, exit 0. It prints no `Traceback`.
- Run it as: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r45_models.py`. Check the output for `R45 BUILD OK` and for zero `Traceback`.

## Checks you do yourself
- All 12 names exist as `.glb` under `assets/models/props/`.
- Triangle count per file printed; limits above.
- `python3 tools/blender/check_palette.py` (plain python, no Blender) passes if you added a palette sheet.

## Done when
The script runs and prints `R45 BUILD OK`; all 12 `.glb` files exist; the gate passes (`tools/gate.sh --task tasks/R45-01-park-stadium-models.md`). The gate has no Rust here, so it must still pass on an unchanged build. Fill `## Worker notes` with:
- Files changed.
- The triangle count of each model.
- The exact name-prefix rules you relied on (`park_pond`, `park_path`, `stadium_pitch` have no collider; `stadium_pitch` has no shadow) so the lead can put them in the collider and shadow lists (`rotland/src/props/rules.rs`, like R44 did for `rope_ladder_` and `green_smoke_`).
- The `mod` lines the lead needs (none for Rust; say so).

## Worker notes
