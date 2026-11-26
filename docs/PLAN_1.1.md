# Plan v1.1: New Halden, the big dying city

Agreed with the owner 2026-11-26. Every choice is in `docs/DECISIONS.md` (2026-11-26 rows). Rounds are run by one Hydrogen lead prompt (`tasks/LEAD.md`): quality first, about 8 subagents, review, many screenshots with fix-and-reshoot.

## The city in short
- **New Halden:** about 2 km wide, about 6 km from camp, in its own area where the swamp meets the snow (grey, damp, cold). Ashford stays as it is.
- **Towers:** most are 10-30 floors (40-120 m). 3-5 landmark skyscrapers are about 150 m. Every tower has an open lobby. 3-5 towers can be climbed to the roof.
- **Look:** about 10 years after the fall, heavily overgrown, in a light grey-green haze. Nights are pitch dark. It is modelled on the city scenes of a well-known zombie TV show: the evacuation jam, the failed army line, survivor traces, a silent city with herds. No names or characters are taken.
- **Danger:** zombie numbers rise toward the center.
- **Rewards:** the best loot in the game, city-only items, a rooftop safe house, the radio goal and rooftop survivors.

## Speed rule (new, whole game)
- Detail first. A round fails only if the slowest moments drop **under 20 FPS** at 1080p on the owner's laptop.
- Raise the budgets in `data/budgets.json` to about 1.4M frame triangles, about 700k shadow triangles and about 400 draw calls. Model caps: buildings/towers 80k, vehicles 25k, props 15k, nature 15k. (Bench: about 1.1-1.8M triangles at 30 FPS, see `docs/BUDGETS.md`.)
- Ways to keep a big city drawable: a far **skyline layer** (one batch of cheap tower blocks drawn far past the 190 m load ring, under the haze), close / medium / far versions of every tower, and real shadows only from nearby towers.
- No speed passes unless the owner asks.

## Rounds
| Round | What it does | Done when |
|---|---|---|
| **R38 Groundwork** | Raise budgets and update `docs/BUDGETS.md`. Place New Halden in the world layout and flatten its ground. Lay a street grid with avenues. Skyline layer + city haze. Simple tower blocks as placeholders. A highway from the nearest town with the **evacuation jam**: cars with open doors, spilled luggage, a bus on its side, dead stuck in cars. Add `--city=new_halden` style test flags and show the city on the big map. | Screenshots: skyline seen from the highway and from the nearest town; jam on the highway; worst 1% at least 20 FPS at the city edge |
| **R39 Towers** | Detailed Blender tower kit: lobby, repeated floors, roof tops, window frames, AC units, fire escapes, pipes, rooftop water tanks, signs, awnings, broken glass and rust. 8-10 tower types with colour variants, plus the 3-5 landmark skyscrapers. Close / medium / far versions plus a skyline block for each. Lobbies you can walk into. | Street-level "canyon" shots; tower previews approved |
| **R40 Nature + survivor traces** | Grass through cracked roads, vines on walls, trees on roofs and in streets, moss on cars, parks gone wild. The fallen tower across a street, the leaning tower and the burnt block. Streets walled off with stacked cars, spray-painted warnings, missing-person posters, old rooftop camps, bodies under sheets. | Shots look like the show's dead city; the ground stays green |
| **R41 Zombies, loot, silent life** | Danger rising toward the center, many runners at night, **herds walking slowly down avenues**. Crows, torn flags and plastic flapping, fallen power lines, faded billboards. Loot tables by district. City-only items: army gear (body armor, helmet, assault rifle, rations), night-vision goggles (battery), rare medicine (antibiotics, surgery kit, adrenaline shot), rare gun and car parts. | Herd shot; items in `data/items.json` with tests |
| **R42 Big places + army line** | Hospital (body-bag pile), police station, fire station. The **failed army line** in an avenue: tank, burnt trucks, crashed helicopter, sandbags, wire, evacuation tents, loudspeakers. Top loot, heavy danger. | Each place screenshotted inside and out |
| **R43 Climbing** | 3-5 climbable towers: stairwell to the roof, a few stop-off floors with loot and zombies. Rooftop planks, ladders and fire escapes. Radio at the top of the tallest tower (a goal). | Climb to the roof works; rooftop view shot |
| **R44 Rooftop life** | Claim a rooftop safe house (bed, storage, pull-up ladder), fast travel between camp and safe house, a rooftop survivor group with a trader, and people to rescue for the camp. | Claim, travel, trade, rescue all work in a test run |
| **R45 Wild park + stadium** | A big park grown into a forest between the towers (a calm spot), and a stadium where a horde lives. | Shots; horde spawns and leaves at night |
| **R46 Subway** | 2-3 stations joined by dark tunnels under the center, a flashlight-only shortcut. The biggest job. | Walk station to station in a test run |
| **R47 Polish + play test** | Fixes from the owner's play test, then tag v1.1. | Owner says it's good |

## Rules that stay
No sounds (the owner adds their own). Never write to real save folders. Heavy commands go under `flock /tmp/rotland-heavy.lock`. Never `pkill -f`. Only `main` remains after each round.
