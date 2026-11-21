# City spec: "Ashford" ruined city (0.2, day 3)

Status: SIGNED OFF by the owner 2026-11-21 (distance 1.5 km, size 640 m, 2x zombies, 1 in 4 streets blocked). "Ashford" is a working name. Next, every number moves to `data/city.json`, covered by the DECISIONS 2026-11-21 row, so code can tag it `// data: city.json <key>`.

## What exists today (facts this spec builds on)
- The world is endless, made of 64 m chunks. The game loads 3 chunks around the player (7x7 chunks, ~190 m sight).
- Two biomes: plains and forest. The camp (`camp_1`) is placed near the start, within 6 chunks.
- Loose buildings (cabin, shed, house_small, barn, watchtower, gas_station) are placed per chunk from `data/poi.json`, each with a loot table and 0-2 zombies.
- The car tops out at 90 km/h (25 m/s). Zombies: walker (2.6 m/s chase), runner (5.2 m/s).
- Speed budget: 228k triangles per frame, 120 draw calls, 12k triangles per building model.

## 1. Where the city is
| Thing | Proposal |
|---|---|
| Distance from camp to the city edge | **1.5 km**. About 1 minute by car, 6-7 minutes on foot |
| Direction | Picked from the seed, so every world is different; never toward a cliff or into deep forest |
| City size | **640 m x 640 m = 10 x 10 chunks**. Walking across takes ~3 minutes |
| Shape | A square grid, with ragged edges: corner blocks are dropped at random so it doesn't look like a box |
| Ground | Flattened to one gentle slope (at most 3 m height change across the city). Grass stays outside and in parks; streets are asphalt; lots are dirt/concrete |
| Road to the city | One old two-lane road from near the camp to the city edge, with wrecks along it. This makes the trip readable ("follow the road") |

## 2. Road grid and blocks
- A block = one chunk (64 m). Streets run along the chunk edges: **10 m wide** (7 m road + 1.5 m sidewalk each side). That leaves a **54 m x 54 m lot area** per block.
- Every 3rd street is a wider avenue (14 m) so the car has a main route.
- **Blocked streets:** 25% of street pieces have a barricade, a pile-up of wrecks or rubble. The car can't pass them; on foot you can climb through. The car is useful, but you can't just drive to every door.
- No traffic lights, no working lamps (no power). Lamps are props only.

## 3. Districts (from the edge inward)
| District | Where | Buildings | Danger | Loot |
|---|---|---|---|---|
| Outskirts | Outer 2-block ring | houses, gas station, small shops, fenced yards | low (like the plains) | tier 1 |
| Downtown | Middle ring | apartments, shops, offices, a pharmacy | medium | tier 2 |
| Core | Center 2x2 blocks | police station, hospital, ruins | high, many runners at night | tier 3 |
| Industrial | One side strip (2 x 6 blocks) | warehouses, a fuel depot, a junkyard | medium | tier 2 + car parts and fuel |
| Parks | 1-2 blocks | trees, benches, a dry fountain | low | none; a breather spot |

## 4. Buildings (8 new models, Neon writes the Blender scripts, Flash makes the variants)
| Model | Lot (m) | Enterable? | Triangle cap (LOD0 / LOD1) |
|---|---|---|---|
| apartment (3 floors) | 20 x 14 | ground floor only | 6k / 800 |
| shop (row of 2-3 fronts) | 18 x 10 | yes | 4k / 500 |
| office (4 floors, broken) | 18 x 18 | ground floor only | 6k / 800 |
| pharmacy | 12 x 10 | yes | 3k / 400 |
| police station | 24 x 18 | yes | 8k / 1k |
| hospital | 30 x 22 | ground floor | 10k / 1.2k |
| warehouse | 26 x 16 | yes (one big room) | 4k / 500 |
| ruin (collapsed shell) | 16 x 12 | yes (open) | 4k / 500 |
- Reused from today: house_small, gas_station, shed.
- Each building has 2 color variants (paint and window colors) so streets don't repeat.
- **LOD:** past 70 m, switch to the cheap LOD1 box. Past 190 m, the chunk isn't loaded. Upper floors are closed (boarded or collapsed stairs) so there's no interior to draw.

### Will it stay fast? (worst case, standing at the core)
- 7x7 chunks visible, about 3 buildings per block, so ~150 buildings. About 20 are close (LOD0, ~5k each = 100k); 130 are far (LOD1, ~600 each = 78k). Plus streets and props ~30k. **Total ~210k, under the 228k budget.**
- Draw calls: one per model type and LOD (instanced): ~40.
- Hard rule check: the first city build gets measured with `--bench`/`--stats` at the core with `--at=X,Z`. If the worst 1% drops under 30 FPS, the LOD distance comes down before anything else ships.

## 5. Street props (Flash, checked by render)
Asphalt, sidewalk curbs, street lamp, bus stop, mailbox, trash bags and dumpster, barricade, rubble pile, car wrecks (reuse + 2 new), fallen sign, broken fence. Props cap: 3k triangles each (today's rule).

## 6. Loot
| Tier | Where | What gets more likely |
|---|---|---|
| 1 | outskirts | food, water, bandages, wood and scrap (like today's house table) |
| 2 | downtown, industrial | medicine, ammo, tools, fuel cans, car parts |
| 3 | core (police, hospital) | guns, lots of ammo, first-aid kits, rare crafting parts |
- New loot tables: `apartment`, `shop`, `office`, `pharmacy`, `police`, `hospital`, `warehouse`, `ruin`. Items come only from the existing `data/items.json` (no new items in this round).
- Looted buildings stay empty (same rule as today's buildings).

## 7. Zombies
| Rule | Proposal |
|---|---|
| Danger level | **City has 2x the zombies of the plains** |
| Max zombies near you at once | 32 in the city (today 20; night x1.6 kept, so up to ~50). Must pass the FPS check; if it fails, cap at 28 |
| Inside buildings | 1-3 per building (police/hospital 3-5) |
| Core at night | 60% runners (today 40%) |
| New zombie types | None this round (later: a "brute" or "screamer" for the city) |
- The road trip itself has normal plains density, so the city feels like a change.

## 8. Map and minimap
- Streets drawn on the minimap as dark gray lines; the city outline on the big map once discovered.
- A road sign at the camp-road start points to the city.

## 9. Work split for days 4-6 (at most 4 workers at once)
| Lane | Work | Who | Proven by |
|---|---|---|---|
| a | `world/src/city/`: city placement, street grid, blocked streets, districts, lots (pure code + tests), plus the road from camp | DeepSeek (or Haiku if DeepSeek is out) | gate tests: same seed gives the same city; city never overlaps the camp; distance from camp is right |
| b | 8 building scripts + LOD | Neon (scripts) -> Flash (variants, renders) | baker `--report-budgets` passes; render previews checked by Opus |
| c | Street props + asphalt/sidewalk ground paint | Flash | renders; a screenshot shows the grass outside the city is still green |
| d | `data/city.json`, loot tables, spawn density, minimap streets | Flash (data) + DeepSeek (minimap) | gate; data checker |
| Opus | Wiring, a screenshot at the city core (`--at`), FPS check, merge | Opus | screenshot + `--stats` |

## 10. Not in this round
Enterable upper floors, working cars parked in the city, new zombie types, NPCs or traders in the city, water, the second city.
