# R44-02: Blender models for the safe house, rooftop camp and rope ladder

Runs at the same time as R44-01, 03, 04. Read first: `tasks/R44-common.md` (whole). Style: `tasks/R43-03-stair-models.md`. Load the skill `blender-art` before you write any script (Blender runs only via `blender -b --python`).

Owns:
- `tools/blender/build_r44_models.py` (NEW): the one build script. It calls small modules you add under `tools/blender/towers/` (names below).
- `tools/blender/towers/safehouse.py` (NEW): shack, bed, chest, travel sign, rope ladder segment.
- `tools/blender/towers/rooftop_camp.py` (NEW): survivor tent, green smoke barrel, green smoke plume, trader stall.
- `assets/models/props/safehouse_*.glb`, `travel_sign.glb`, `rope_ladder_seg.glb`, `rope_ladder_coil.glb`, `survivor_tent.glb`, `green_smoke_barrel.glb`, `green_smoke_plume.glb`, `rooftop_trader_stall.glb` and `assets/models/props/*_palette.png` if the house style needs one (copy the pattern of `roof_ladder_seg.glb`).

Not owned: Rust, data, `assets/models.pack` (the lead bakes it at wiring time; do NOT run `baker`).

Read (for style, do not edit):
- `tools/blender/towers/rooftop_kit.py` and `tools/blender/build_r43_models.py` (the `build_all()` pattern, export, the `R43 BUILD OK` end line).
- `tools/blender/towers/evac_tent.py` if it exists (or the `evac_tent` source), `barrel` / `fire_barrel` source, to match the tent and barrel look.
- `docs/` is off limits; `tools/README_TOOLS.md` for the blender lock.

## Models (names EXACT; each file is `assets/models/props/<name>.glb`)
| Name | Size (m) | What it is | Colliders |
|---|---|---|---|
| `safehouse_shack` | 4.0 x 3.0 x 3.2 | A tarp shack on the roof: four posts, a sagging blue-grey tarp roof, one open side (the door side = local -Z), a plank back, a rope and a tin can hung on the post. Origin at floor centre. | Convex from the prop path (the walls) |
| `safehouse_bed` | 2.0 x 0.6 x 0.5 | A camp cot with a grey blanket and a pillow. Lying flat (top at 0.5 m). Origin at its centre. | Convex (small) |
| `safehouse_chest` | 0.9 x 0.6 x 0.6 | A metal storage chest with a lid and a padlock. Origin at its centre, base on the floor. | Convex |
| `travel_sign` | 0.9 x 1.6 x 0.1 | A painted board on two posts, with a big white arrow and two plain bars. No letters. Origin at the base centre. | Convex (the posts) |
| `rope_ladder_seg` | 0.5 x 1.0 x 0.1 | One segment of a rope ladder: two ropes and 3 rungs, 1.0 m long, hung flat. Origin at the top centre. | NO collider (name prefix rule) |
| `rope_ladder_coil` | 0.7 x 0.4 x 0.7 | The rope ladder pulled up: a rolled bundle of rope and rungs tied to a hook on the parapet. Origin at the base centre. | NO collider (name prefix rule) |
| `survivor_tent` | 2.4 x 1.4 x 2.2 | A small dome tent, dirty olive, a zip door, a rope guy line. Origin at the base centre. | Convex |
| `green_smoke_barrel` | 0.6 x 0.9 x 0.6 | A rusty oil barrel with a bright green (flat colour) stain, a lid with a hole. Origin at the base centre. | Convex |
| `green_smoke_plume` | 1.2 x 3.0 x 1.2 | A column of 5-7 lumpy low-poly puffs rising from the barrel, flat green-grey, no alpha. Origin at the base centre. | NO collider and NO shadow (name prefix `green_smoke_`; tell the lead in Worker notes) |
| `rooftop_trader_stall` | 2.2 x 1.6 x 1.4 | A trestle table with a tarp awning, two stacked crates, a small hand-drawn price board (no letters). Origin at the table centre, base on the floor. | Convex |

## Style rules
- Low-poly, flat colours, dirty and worn. Keep each model under 1500 triangles (the props budget is 15k per model cap; these are small).
- Each part is a separate object; use one material per colour. No textures other than the palette PNG pattern of existing props.
- Origins as in the table. Export with `+Y up`, metres, no animation, no cameras, no lights.
- Nothing should z-fight: details sit at least 2 cm off the faces they sit on (lesson from R43).

## Script rules
- `tools/blender/build_r44_models.py` runs `safehouse.build_all()` and `rooftop_camp.build_all()`, then exports each name above to `assets/models/props/`.
- It prints `R44 BUILD OK` at the end and exits with 0. It prints no `Traceback`.
- Run it as: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r44_models.py`. Check the output for `R44 BUILD OK` and for zero `Traceback`.

## Checks you do yourself
- Every name in the table exists as a `.glb` under `assets/models/props/`.
- Each file opens (the build script re-imports it once: no errors).
- Triangle count per file is printed by the script; each is under 1500 (`safehouse_shack` may be up to 2500).

## Done when
The script runs and prints `R44 BUILD OK`; all 9 `.glb` files exist; the gate passes (`tools/gate.sh --task tasks/R44-02-safehouse-models.md`). Fill `## Worker notes` with:
- Files changed.
- The triangle count of each model.
- The exact `green_smoke_` rule you relied on (the lead needs it for the no-collider and no-shadow list).
- The `mod` lines the lead needs (none for Rust; say so).

## Worker notes
