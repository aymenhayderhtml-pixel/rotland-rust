# Rotland 0.2 push: 6 days, 2026-11-22 to 11-27

Owner decisions (2026-11-21): use the token budget fully. All four camp upgrade lines; next place = **lake & coast**; frontend = new menus look, big map screen, better inventory/crafting, HUD polish; features = more weapons, player-built base, smarter zombies, guild building.

## How we work this week
- **2 rounds a day, 8-9 Gemini 3.8 Flash lanes per round** (morning round, afternoon round). Each lane: one task file, its own worktree, `tools/gate.sh` until PASS (Blender lanes: renders + scope check). Opus writes the round's tasks, reviews every diff and picture, wires, merges, runs one release build + city/camp FPS check, pushes, cleans worktrees.
- **Neon (AI Studio, owner pastes):** one big UI screen a day (menus, inventory, crafting, map, camp board, build menu). Neon gets the pack from `tools/neon_pack.py`; a Gemini lane applies and gates it.
- **Haiku:** only for jobs that must wait on long runs (if a Gemini lane stalls).
- **Opus budget:** about 13-15% of the weekly limit a day (today was 6%).
- **Rules kept:** never under 30 FPS (city core and camp measured every round), the ground stays green, only `main` after each round, numbers in `data/*.json` with source tags, owner asked on anything that changes the feel.
- **Order:** foundations first (pure logic + data + tests, which parallelize well), then models (Blender, checked by render), then UI and wiring. A feature isn't done until it's wired, playable and in a screenshot.

## The tracks (what "done" means)
| Track | Backend (logic, data, engine) | Frontend (UI, look) |
|---|---|---|
| A. Camp upgrades | `game/src/camp_upgrades/`: 4 lines x 3-4 levels (walls & gates, guards & towers, workshop, traders & services), costs, build time, effects; save/load; camp walls block zombies | Camp board screen (Neon), upgrade models (walls wood/metal/concrete, gates, towers, workshop, doctor tent, mechanic garage) |
| B. Player-built base | Build mode: grid/snapping, placeable parts (wall, door, window, floor, roof, stairs, bed, storage box, workbench, light), costs, health, zombies attack walls, save/load | Build menu + ghost preview (green/red), part models |
| C. Weapons | Bow + arrows (projectile, pickup), spear (thrust, throw), dagger (fast), new melee (hammer, pipe, fire axe); data in `weapons.json`; crafting recipes | Weapon models + icons, first-person views, ammo HUD for arrows |
| D. Smarter zombies | Flow field toward the player/noise; hordes that move together; a director that paces waves (calm / tension / wave), night hordes in the city; 3 special zombies (brute, screamer that calls others, crawler) | Special zombie models, horde warning on HUD |
| E. Lake & coast | Region placer (reuses the city's region tech): a big lake + coastline; water surface rendering (cheap shader, waves by vertex), shore paint, swimming/wading slow-down, docks, fishing village, fishing (rod + fish items + cooking) | Water look, dock/boathouse/fish shack/pier models, fishing UI (bite bar) |
| F. Guild | Guild building in the camp: task board (generated tasks: kill, fetch, explore), recruit teammates (follow, guard the camp), buy items | Guild screen (Neon), guild building + recruit models |
| G. Frontend | | New main/pause/settings menus (Neon), inventory with drag and drop + crafting screen with icons and categories (Neon), big map: city streets, roads, lake, discovered places, player markers, HUD polish: bars, damage direction, pickup popups, notifications |
| H. Speed & health | Stutter fix (from the owner's `--spikes` recording), dithered LOD fade for buildings, shadow-triangle budget, GitHub #19 (move logic out of `rotland/app`), #17 (dirty water + uses for scrap/antlers), #12 + #1 (model faces/character parts), #8 (profile portrait) | |

## Day by day (lanes per round; F = Gemini Flash, N = Neon)
| Day | Morning round | Afternoon round | Neon (owner pastes) |
|---|---|---|---|
| 1 (11-22) | Foundations, 9 F: A camp upgrades core; B build-mode core (pure); C weapons data + bow projectile logic; D flow field + horde core (pure); E water region placer (pure); E water shader prototype (engine); G big-map data layer (streets, roads, discovered); H stutter fix from recording; H #17 data | Models + wiring, 9 F: A wall/gate tiers; A towers + workshop + doctor + mechanic; C bow/spear/dagger/hammer/pipe/fire axe models + icons; D brute/screamer/crawler models; B base part models (walls, door, floor, roof, stairs); B furniture (bed, box, bench, light); E docks/pier/boathouse/fish shack; G HUD polish (bars, popups); H LOD fade | Main + pause + settings menus |
| 2 (11-23) | Wire A (upgrade effects in game: safe zone, guards, recipes, traders), wire B (build mode in play, save/load), wire C (bow, spear in combat), wire D (hordes in spawner), wire E (lake in world gen + water draw), G big map screen, H #19 split part 1, tests for all | Fixes from day 1 play; E fishing mechanic + fish items; D director (calm/tension/wave); C crafting recipes for new weapons/arrows; A camp board screen apply (from Neon); F guild data (tasks, recruits) | Camp board (upgrades) screen |
| 3 (11-24) | F guild logic (task board, recruits follow/guard); B zombies attack player walls; D screamer calls, brute breaks doors; E swimming/wading; G inventory drag-drop apply (Neon); H #12/#1 model fixes; H #8 portrait | Lake/coast props + village layout; F guild building + NPC models; G damage direction + notifications; balance pass data (costs, loot, zombie numbers); save format tests for A/B/F | Inventory + crafting screens |
| 4 (11-25) | Polish round: every track's open bugs from Opus's play checks; FPS pass at camp, city, lake (budget fixes, LODs for all new models) | Second balance + content round: more base parts, 2 more weapons, more guild tasks, lake loot tables | Guild screen + build menu |
| 5 (11-26) | Integration: one full playthrough script (`--seconds` runs at camp, city, lake; screenshots of every new screen); fix lanes | Fix lanes; docs (STATUS, DECISIONS, CODEMAP) | Map screen polish |
| 6 (11-27) | Owner play test (release build) | Fixes from the play test, release, cleanup (only `main`) | |

## Questions for the owner along the way (asked when the round needs them)
- Day 1: camp upgrade costs feel (cheap and fast vs slow grind); can zombies break the camp walls?
- Day 1: lake distance and side (opposite the city?); can the player swim?
- Day 2: build mode: anywhere, or only near the camp/owned land? Do zombies attack the player's base when the player is away?
- Day 3: guild teammates: can they die? Do they cost upkeep?
- Day 3: special zombies: how often, and are they in the countryside too or only the city/night?

## Risks and the plan for them
- **FPS:** water, hordes and more models all cost frames. Every round measures camp + city core (+ lake from day 2); anything that drops the worst 1% under 30 is fixed before new content merges.
- **Merge conflicts:** lanes own separate files; shared files (`generator.rs`, `frame_sim.rs`, `data/items.json`, `weapons.json`) get one owner per round, others leave wiring notes.
- **Gemini quality:** the gate (clippy, tests, scope from main, number tags, unused code) plus Opus's diff and picture review. Mistakes go to `docs/JESTER.md`.
- **Save files:** every new system adds to the save with a version bump and a load test from an old save.
