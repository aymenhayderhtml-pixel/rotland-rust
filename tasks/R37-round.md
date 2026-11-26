# R37 round: weapon shop (gunsmith). Lead file for the gate

Files: game_core/*, game_camp/*, game_player/*, game_world/*, game/*, rotland/*, rotland_ui/*, world/*, save/*, data/*, tools/blender/*, assets/models/*, assets/icons/*, assets/models.pack, tasks/R37-*, docs/WORLD.md
Rules: `tasks/LEAD.md` + `tasks/R37-common.md` (owner decisions + "R37 shared names").

## Tasks (8 subagents)
| Task | What | after |
|---|---|---|
| `R37-01-wear.md` | per-weapon condition: wear, jams, spread, broken; save | none |
| `R37-02-attachments.md` | attachment data + stat effects (pure) | none |
| `R37-03-gunsmith-logic.md` | shop logic: stock per camp tier / town, prices in scrap, buy, sell, repair, fit | none (use shared names for 01/02) |
| `R37-04-gunsmith-ui.md` | the shop screen (4 tabs) | none |
| `R37-05-models-shop.md` | `gunsmith_bench`, `gun_shop` building | none |
| `R37-06-models-attachments.md` | 9 attachment models + icons, mount points on guns | none |
| `R37-07-places.md` | gunsmith in camp (NPC + bench, camp upgrade tiers), a gun shop + shopkeeper in each town | none |
| `R37-08-wiring.md` | rotland: talk to NPC → screen, actions → logic, wear on fire, attachments change stats and show on the gun, condition bar on hotbar | after 01-04 report |

## Checks after the gate (target/play/rotland, `--save-dir=/tmp/r37-saves`, pictures to /tmp/r37/)
R37-08 adds test flags: `--gunsmith=<tab>` (opens the shop at the camp gunsmith), `--gunsmith-town=<key>`, `--give=<item>[:count]`, `--wear=<weapon>:<0..1>`, `--mods=<weapon>:<id,id>`.
- Shop screen, each tab: `--gunsmith=buy|sell|repair|upgrade --give=scrap_metal:60` (and one in a town: `--gunsmith-town=dusthollow`).
- Camp gunsmith bench + NPC (third person, looking at it); Dusthollow gun shop outside and inside.
- Guns with attachments, third AND first person: `--weapon=rifle --mods=rifle:scope_4x,suppressor`, `--weapon=pistol --mods=pistol:red_dot,ext_mag`, `--weapon=ak47 --mods=ak47:foregrip,drum_mag`, `--weapon=crossbow --mods=crossbow:scope_2x`. Attachments must sit ON the gun (no floating, no clipping into the hand), and the 4x scope must change the aim zoom.
- A worn gun: `--weapon=pistol --wear=pistol:0.1` shows a low condition bar.
- `tools/perf_scenes.sh --set all --seconds 20`: no scene slower than R36 by more than 3 FPS.
- An old save (copy any save from /tmp/r36-saves) still loads.

## Lead notes
