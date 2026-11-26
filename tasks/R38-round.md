# R38 round: New Halden groundwork (skyline, streets, highway jam). Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, save/*, game/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/BUDGETS.md, docs/WORLD.md, tasks/R38-*
Rules: `tasks/LEAD.md` + `tasks/R38-common.md` (owner decisions + "R38 shared names": follow them EXACTLY).

## Phase 0 (lead, before subagents)
1. Raise `data/budgets.json` and add the paragraph to `docs/BUDGETS.md` (numbers in R38-common "Budgets").
2. Baseline: `cargo build --profile play`, then `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` and save it to `/tmp/r38/perf_before.txt`.

## Tasks (8 subagents)
| Task | What | after |
|---|---|---|
| `R38-01-metro-layout.md` | `world/src/metro/`: config, placement (~6 km, swamp/snow seam), block grid, streets, districts, reserved blocks, flat ground | none |
| `R38-02-metro-towers.md` | tower spots: heights per district, 3-5 landmarks | none (use shared names) |
| `R38-03-metro-highway.md` | 4-lane highway from the nearest town, flattened ground | none (use shared names) |
| `R38-04-metro-jam.md` | evacuation jam placement (pure) + per-chunk props + colliders | none (use shared names) |
| `R38-05-jam-models.md` | Blender: jam cars (open doors, burnt), buses, army truck, luggage, bodies, gantry sign | none |
| `R38-06-metro-render.md` | street ground, near placeholder towers + colliders, far skyline batch | none (use shared names) |
| `R38-07-haze-map.md` | grey-green city haze, big map + minimap, "Entering New Halden" banner | none |
| `R38-08-metro-wiring.md` | wire it all into rotland, `--metro=` flags, perf scene `metro` | after 01-07 report |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r38-saves`, pictures to /tmp/r38/)
- `--metro=town_view`: from the nearest town the skyline rises through the haze, landmarks clearly taller.
- `--metro=highway` at hour 10 and hour 18: the jam is dense and looks like a real evacuation (open doors, luggage, a bus on its side, bodies in some cars); skyline ahead.
- `--metro=edge`, `--metro=core`: street canyons, sidewalks, towers have colliders (walk into one: you stop), the grass outside the city stays green.
- `--metro=top`: whole city from above, avenues, ragged edge, reserved blocks empty.
- `--metro=core --hour=23`: pitch dark, no city lights.
- Each new model preview in `assets/models/_previews/` looked at.
- Ashford, camp and the 4 towns look the same as before (`--town=blackwater`, camp spawn).
- Big map shows New Halden; "Entering New Halden" banner appears.
- FPS: `tools/perf_scenes.sh --set all --seconds 20` -> `/tmp/r38/perf_after.txt`. **Every scene's 1% low must be at least 20 FPS**, including the new `metro` scene. Detail is fine, under 20 is not.
- An old save loads (make one with the before-binary in Phase 0).

## Lead notes

## Gate report
