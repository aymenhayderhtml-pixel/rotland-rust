# R42 round: New Halden big places + the failed army line. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R42-*
Rules: `tasks/LEAD.md` + `tasks/R42-common.md` (what is true today, owner decisions, "R42 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r42/perf_before.txt` (metro ONLY).
- Shots before: `--metro=core`, `--metro=top` (hour 10) -> `/tmp/r42/before_*.png`.

## Tasks (9)
| Task | What | after |
|---|---|---|
| `R42-01-places-place.md` | sites + per-chunk POIs/props/crates for hospital, police, fire station; plaza paint | none (uses 03-05 data names; fixtures until then) |
| `R42-02-army-line-place.md` | army line site + layout data + per-chunk placement | none |
| `R42-03-hospital-models.md` | Blender: `metro_hospital` (walk-in), hospital props, ambulance, body bags, triage tents; `place_kit.py`; `data/places/hospital.json` | none |
| `R42-04-police-models.md` | Blender: `metro_police`, police cars, cells, barriers; `data/places/police.json` | none (imports `place_kit.py`) |
| `R42-05-fire-station-models.md` | Blender: `metro_fire_station`, 3 fire trucks, props; `data/places/fire_station.json` | none (imports `place_kit.py`) |
| `R42-06-army-line-models.md` | Blender: tank, burnt trucks, crashed helicopter, barrier, camp; `build_r42_models.py` | none |
| `R42-07-zombie-looks.md` | 4 zombie looks + per-building looks + `spawn_kind_look` | none |
| `R42-08-loot-danger.md` | 3 loot tables, 4 POI entries, place danger bonus | none |
| `R42-09-wiring-flags.md` | wiring, flags, map labels, README | after all others |

Start 01-08 at once (8 subagents). Tell R42-04 and 05 that R42-03 creates `place_kit.py` first. Start R42-09 when the rest have reported.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r42-saves`, pictures to `/tmp/r42/`)
Each place, inside AND outside (hour 10; plus one night shot of the army line at hour 22 with `--nvg`):
- `--metro=hospital`, `--metro=hospital_in`:
  - the ER canopy, ambulances, triage tents and the body-bag pile read at a glance
  - the inside reads as a hospital (reception, ward beds with curtains, pharmacy cage)
- `--metro=police`, `--metro=police_in`: police cars, barricades; inside the desks, cells and armoury.
- `--metro=fire`, `--metro=fire_in`: three bays with trucks; inside the bay hall, gear and pole.
- `--metro=armyline`, `--metro=armyline_top`: the sandbag/T-wall barrier with wire and a breach, the tank, burnt trucks, crashed helicopter, evacuation tents, loudspeakers.
- Walk in: from the `_in` poses, a 3-second `--frames` run with a forward walk (or the existing walk flag) doesn't get stuck on invisible walls and doesn't fall through a floor. Doors are open gaps.
- Danger: after `--shot-frame=3000` at `--metro=hospital` the stats show more zombies near than at `--metro=core`. Medic/police/firefighter/soldier looks are visible in at least one shot each.
- Loot: one crate at each place opened (`--hold-e` style), showing its table's items.
- The big map (`--map`) shows the four labels.
- Nothing z-fights (signs, vines, stripes); no props floating or sunk.
- FPS after: `--set metro` -> `/tmp/r42/perf_after.txt`. Pass = **average at least 20**; write the 1% low for information. If under 20, lower the far LOD distances of the big buildings first; never strip close detail.

## Lead notes

## Gate report
