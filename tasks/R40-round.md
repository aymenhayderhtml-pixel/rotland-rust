# R40 round: New Halden, nature takes it back + survivor traces. Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R40-*
Rules: `tasks/LEAD.md` + `tasks/R40-common.md` (owner decisions + "R40 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r40/perf_before.txt` (metro ONLY; do not run `--set all`).
- Shots before: `--metro=core`, `--metro=edge`, `--metro=lobby`, `--metro=top` (hour 10) -> `/tmp/r40/before_*.png`.

## Tasks (8)
| Task | What | after |
|---|---|---|
| `R40-01-overgrowth-place.md` | overgrowth placement per chunk (streets, lots, walls, roofs), density per district | none |
| `R40-02-overgrowth-models.md` | Blender: vines, crack weeds, asphalt trees, shrubs, moss, roof bushes/trees, leaves | none |
| `R40-03-damage-place.md` | fallen / leaning tower / burnt block sites, blocked street, skip replaced towers | none |
| `R40-04-damage-models.md` | Blender: fallen section, rubble fields, leaning tower, burnt overlays, burnt car pile, ash | none |
| `R40-05-traces-models.md` | Blender: car walls, graffiti, posters, rooftop camp, boarded doors, sandbags, body pose, lobby dressing | none |
| `R40-06-traces-place.md` | car walls, graffiti, posters, camps, bodies, lobby dressing placement | after 03 + 07 (uses their names) |
| `R40-07-tower-variety.md` | `_floor_c` damaged floors for all styles/sizes + better crowns/roof props | none |
| `R40-08-wiring-flags.md` | chunk props + colliders + shadow rules, `--metro=fallen|leaning|burnt|carwall|camp`, landmark view fix, README | after 01-07 |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r40-saves`, pictures to /tmp/r40/)
- `--metro=core`, `--metro=edge` at hour 10 and 18: it looks like nature is taking the city back (vines on walls, weeds in cracks, trees in the street, wild lots), still clearly a big city. The ground outside the city is still green.
- `--metro=fallen`, `--metro=leaning`, `--metro=burnt`: each reads at a glance, and you can climb the rubble.
- `--metro=carwall`: a stacked-car wall across a street with a way over. `--metro=camp`: a rooftop camp (camera high).
- `--metro=lobby`: a dressed lobby (columns, ceiling, debris, posters), not a bare hall.
- `--metro=landmark`: looking UP at the tallest landmark crown.
- `--metro=top`: roofs show water tanks and machinery, plus some roof greenery.
- No z-fighting flicker on vines, graffiti or posters (look closely).
- FPS after: `--set metro` -> `/tmp/r40/perf_after.txt`. Pass = **average at least 20**. Write the 1% low too, for information. If the average is under 20, lower the overgrowth density in data first; never strip close detail.

## Lead notes

## Gate report
