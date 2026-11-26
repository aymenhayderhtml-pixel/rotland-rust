# R39 round: New Halden towers. Lead file for the gate

Files: world/*, rotland/*, rotland_ui/*, engine/*, physics/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/WORLD.md, tasks/R39-*
Rules: `tasks/LEAD.md` + `tasks/R39-common.md` (owner decisions + "R39 shared names": follow them EXACTLY).

## Phase 0 (lead)
Baseline: `tools/perf_scenes.sh --set all --seconds 20 --bin target/play/rotland` -> `/tmp/r39/perf_before.txt`. Shots before: `--metro=core`, `--metro=edge`, `--metro=top` (hour 10) -> `/tmp/r39/before_*.png`.

## Tasks (8)
| Task | What | after |
|---|---|---|
| `R39-01-tower-plan.md` | styles data, tower style/size on TowerSpot, `tower_parts` (stacking, setbacks, landmarks, lobby facing the street) | none |
| `R39-02-kit-brick-stone.md` | Blender: Brick + Stone styles (base/floor/floor_b/crown x sizes + LOD1) | builds after 04 + 05 |
| `R39-03-kit-glass-slab.md` | Blender: Glass + Slab styles | builds after 04 + 05 |
| `R39-04-kit-deco-landmarks.md` | Blender: Deco style + 4 landmark crowns + setback sections | none |
| `R39-05-lobbies.md` | Blender: lobby props, shared lobby shell (`towers/shell.py`), new palette colours, `build_r39_models.py` | none (first) |
| `R39-06-tower-render.md` | rotland: draw tower parts instanced per chunk with LODs, shadows only LOD0, colliders (walk-in lobby), remove near placeholders | after 01 |
| `R39-07-skyline-curbs.md` | skyline window bands + haze tint; raised curb/sidewalk mesh replacing jagged paint | none |
| `R39-08-flags-wiring.md` | `--metro=lobby|landmark`, `--tower-style=`, `--map`; perf scene; README lines; R38 leftovers (bus_city_side underside) | after 01-07 |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r39-saves`, pictures to /tmp/r39/)
- `--metro=core`, `--metro=edge` at hour 10 and 18: street canyons look like a real dead New York (cornices, fire escapes, broken windows, water tanks), no floating or gapped floors, no z-fighting flicker.
- `--metro=landmark`: the landmark crown reads from the street; `--metro=top`: varied skyline with setbacks.
- `--metro=lobby`: walk-in lobby with desk, elevators, mailboxes, rubble; door faces the street; you can walk in and out (no invisible wall).
- `--tower-style=brick|stone|glass|slab|deco` with `--metro=core`: one picture each.
- `--metro=highway` and `--metro=town_view`: the skyline now has window bands and reads as a city in the haze.
- Sidewalks: straight raised curbs, no jagged edges.
- `--map`: New Halden outline + streets on the big map.
- Camp, Ashford (`--at=-384,1792`), `--town=blackwater` look as before.
- FPS: `tools/perf_scenes.sh --set all --seconds 20` -> `/tmp/r39/perf_after.txt`; every scene's 1% low >= 20 (metro included). If metro is under 20, lower `towers.lod0Distance` first, then LOD1 triangles; never strip the close detail.

## Lead notes

## Gate report
