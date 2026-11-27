# R41 round: New Halden, zombies, loot, silent life. Lead file for the gate

Files: world/*, game_zombie/*, game_core/*, game_player/*, game_world/*, game/*, rotland/*, rotland_ui/*, engine/*, physics/*, save/*, data/*, tools/blender/*, tools/perf_scenes.sh, tools/README_TOOLS.md, assets/models/*, assets/icons/*, assets/textures/*, assets/models.pack, docs/WORLD.md, tasks/R41-*
Rules: `tasks/LEAD.md` + `tasks/R41-common.md` (owner decisions + "R41 shared names": follow them EXACTLY).

## Phase 0 (lead)
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r41/perf_before.txt` (metro ONLY; do not run `--set all`).
- Shots before, at hour 10, saved to `/tmp/r41/before_*.png`:
  - `--metro=core`
  - `--metro=edge`
  - `--metro=core --hour=23`

## Tasks (10)
| Task | What | after |
|---|---|---|
| `R41-01-metro-zombies.md` | metro spawn profile: density/cap by district, runners at night | none |
| `R41-02-street-herds.md` | herd routes along avenues (world) + herd runtime (game_zombie) | none |
| `R41-03-life-place.md` | power lines, billboards, flags, plastic, crow perches placement | none |
| `R41-04-life-models.md` | Blender: poles, wires, billboards + faces, flag/plastic frames, crows; `build_r41_models.py` | none |
| `R41-05-crows-flappers.md` | crow flocks + flapping frames (pure) and their drawing (`metro_life.rs`) | none (uses 03/04 names from common) |
| `R41-06-city-items.md` | 12 city items, army rifle, armour, boosts, night vision, parts repair | none |
| `R41-07-item-models.md` | Blender: item models, army rifle, crates, icons | none |
| `R41-08-district-loot.md` | metro loot spots + 7 district tables, table/model override on `LootSpot` | none (uses 06/07 names from common) |
| `R41-10-splits-fallen-climb.md` | split traces/damage/streaming, fallen tower climbable | none |
| `R41-09-wiring-flags.md` | wiring, night-vision look, HUD boosts, saves, flags, README | after all others |

Start 01-08 and 10 at once (9 subagents). R41-10 only MOVES code in traces/damage/streaming; tell 03, 08 and 09 to call those modules by their public names. Start R41-09 when the rest have reported.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r41-saves`, pictures to `/tmp/r41/`)
- `--metro=herd`: a herd of 12-30 walkers coming slowly down an avenue. It reads as a herd, not a scattered crowd. Take a second shot ~10 s later (`--shot-frame`) to show they moved along the street.
- `--metro=night`: more zombies than the day, mostly runners. Compare with `--metro=core` at hour 10 and with `--metro=edge` at hour 23 (Edge has fewer).
- `--metro=night --nvg`: a green night-vision view. You can see the street and the zombies, with a little noise and vignette. The HUD shows the battery.
- `--metro=crows`: crows on a body, pecking. `--metro=crows_up` (or the startle shot): crows in the air.
- `--metro=core` and `--metro=top`: power poles with sagging wires along some avenues, one or more fallen poles with wires down, billboards with faded made-up ads on roofs and lots, torn flags on lobby fronts. Two shots a few frames apart show the flags in different frames.
- `--metro=army`: an army crate with its loot. `--give=army_rifle,ammo_556` with a first-person shot of the rifle and one firing shot.
- `--metro=fallen`: walk up the fallen section (a shot from its top).
- Zombies outside the city are unchanged: `--scene=` forest shot, same zombie count look as before.
- No z-fighting on billboard faces, flags or wires (look closely).
- FPS after: `--set metro` -> `/tmp/r41/perf_after.txt`. Pass = **average at least 20**; write the 1% low too, for information. If the average is under 20, lower crow counts/draw radius and the herd count in data first; never strip close detail.

## Lead notes

## Gate report
