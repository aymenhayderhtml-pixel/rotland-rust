# Decisions log

Note: The game is now built in Rust (custom wgpu engine, see docs/archive/RUST_PLAN.md) and Godot-specific tech notes are history.

## Locked
| Date | Decision | Why |
|---|---|---|
| 2026-11-11 | Solo offline game, with NPC survivors instead of other players | Much simpler, and no servers to pay for |
| 2026-11-11 | Godot 4.7 .NET + C# (binary: `~/.local/bin/godot-mono`) | Free, free, light, works well on Linux and the laptop |
| 2026-11-11 | 3D, low-poly stylized, Mobile renderer | Integrated Intel GPU; easy for AI to make the art |
| 2026-11-11 | .glb models through Blender Python scripts | Already installed; AI can script models |
| 2026-11-11 | All code by AI: Opus orchestrates, Gemini 3.8 Flash and Haiku are workers | Owner's choice |
| 2026-11-11 | Camera: third + first person, switch with a key | Owner's choice |
| 2026-11-11 | Death: lose all carried items; keep skills XP and bank | Owner's choice (hardcore) |
| 2026-11-11 | No asset packs: AI makes all models, including rigged characters (blocky part-based rigs + procedural animation) | Owner's choice |
| 2026-11-11 | Hunger + thirst | Owner's choice |
| 2026-11-11 | Currency: dollars ($) | Owner's choice |
| 2026-11-11 | Night: more and faster zombies + night-only type | Owner's choice |
| 2026-11-11 | Horse: tame wild or buy; add pet dog companion | Owner's choice |
| 2026-11-11 | FPS: 30 minimum, 60 nice to have | Owner's choice |
| 2026-11-11 | Working title "Rotland"; GitHub repo aymenhayderhtml-pixel/rotland | Name can change later |
| 2026-11-11 | Dog death is permanent | Owner's choice |
| 2026-11-19 | `mesh` uses CW terrain fronts; `skinned` uses CCW glTF/procedural model fronts | Terrain winding differs from baked model winding; baker tests guard CCW outward geometry |
| 2026-11-19 | Export vehicles as separate body, wheel, driver-door, and tailgate `.glb` parts | Rust can instance wheels and animate wheels, door, and tailgate without skeletal skinning |
| 2026-11-20 | 0.2 starts with new places; the ruined city comes first | Owner's choice (see docs/PLAN_0.2.md) |
| 2026-11-20 | Places are big regions: travel between them is a real trip and the car matters | Owner's choice |
| 2026-11-20 | Base: camp upgrades first, player-built base later in 0.2 | Owner's choice |
| 2026-11-20 | Spare frame time (~3x over the 30 FPS minimum) goes to better-looking models | Owner's choice |
| 2026-11-20 | Workers run `tools/gate.sh` (checks + tests + scope + number sources) and loop until PASS; max 4 workers at once | Write-only workers produced ~35 compile errors that Opus fixed (JESTER #35) |
| 2026-11-21 | Ruined city: 1.5 km from camp, 640 m square (10x10 blocks of 64 m), 2x plains zombies, about 1 in 4 street pieces blocked for cars; all other city numbers per docs/CITY_SPEC.md | Owner's choice |
| 2026-11-21 | The first city is named Ashford | Owner's choice |
| 2026-11-21 | Arrow keys turn the camera (for playing without a mouse): about 120 degrees per second, scaled by the mouse sensitivity setting; up/down arrows tilt | Owner request |
| 2026-11-21 | 0.2 push (11-22 to 11-27): all four camp upgrade lines (walls & gates, guards & towers, workshop, traders & services); next place lake & coast; frontend: new menus, big map, inventory/crafting, HUD polish; features: more weapons, player-built base, smarter zombies, guild. Plan: docs/PLAN_0.2_PUSH.md | Owner's choice |
| 2026-11-21 | Camp upgrades are a steady goal: first level after about 1 hour of play, top level after many hours; building an upgrade takes a few in-game hours. Camp walls hold against normal zombies; only big night waves damage them, and the player repairs them. The lake & coast sits opposite the city, about 1.5 km from the camp. Water: shallow water slows the player, deep water = slow swimming, no weapons, stamina drains | Owner's choice |
| 2026-11-24 | Melee v2 (A13): 2-3 hit combos (chain of different swings, last hit a bit harder, resets when you stop clicking); right mouse with a melee weapon: tap = shove (push + stagger, costs stamina), hold = block (front bites hurt less, drains stamina); heavy weapons much slower with big knockback and a pause after a miss, knives fast; base damage numbers unchanged. Spear keeps right-click throw. All numbers in `data/melee_moves.json` + `data/melee_guard.json` | Owner's choice |
| 2026-11-25 | Downed + rescue (owner): at 0 health the player is knocked down first, crawls slowly, can't fight, and bleeds out after 30 s (zombie bites shorten it); a following teammate within 12 m comes and picks you up (4 s), or you hold E 6 s to use a bandage; up again at 25 health; downed again within 60 s = death; drowning still kills at once. Recruits: survivors trapped in some houses with zombies near; when no zombie is within 12 m, [E] Rescue: they join the team for free and follow you (at most 6 in the world, normal team-size limit applies). Numbers are Claude's first guess, in data/downed.json and data/rescue.json. |
| 2026-11-25 | R33 towns/roads/rivers (owner said "R33"; names and layout are Claude's choice): towns Dusthollow (desert), Frostmere (snow), Blackwater (swamp), Ridgeback Station (mountains) at 3.4-4.2 km; highways start at signposted trailheads on the home-land edge, a ring trail joins the towns; 3 rivers flow from the mountains/snow toward swamp/desert, never into the core; bridges where roads cross. The home core stays unchanged. Numbers in data/towns.json, roads.json, rivers.json. |
| 2026-11-25 | Detail over FPS (owner): the game runs ~100+ FPS in many places; the owner wants much more visual detail and accepts **30-60 FPS** (still min 30 at 1080p on the UHD laptop). R32 keeps today's FPS; **R35 = all-out detail round** (many prompts): denser grass/foliage, more and richer models, better ground, longer view distance, nicer lighting/shadows. Weapons moved from R35 to **R36**. |
| 2026-11-25 | "0.3 World" (owner, replaces weapons as R31; weapons moved to R35): world generator rebuilt in 4 rounds (R31 engine + desert/snow/swamp/mountains, R32 weather + light temperature + biome zombies, R33 places/rivers/roads, R34 map/polish/speed). **Rings layout**: today's world (camp, plains, forest, Ashford, lake) stays exactly the same inside a 2.2 km core; outside it each direction is a biome (desert beyond Ashford, snow beyond the lake, swamp and mountains to the sides) and danger tiers rise every 1.5 km (more zombies, more loot). This REPLACES the older "No danger rings" line. **Endless world** (no edge). **Light temperature** (cold/heat bar, campfire/clothes help, never a quick death) in R32. Numbers: data/biomes.json, data/world_layout.json. |
| 2026-11-25 | Weapons + gun feel (owner picked R31, now R35): view kick + muzzle flash on every gun; new revolver (6 shots, strong), hunting rifle (scoped, 4 shots), crossbow (quiet, reload 2 s, bolts can be picked up), grenade (2.5 s fuse, 6 m blast, hurts the player too, wakes zombies 80 m); backpack visible on the player. Stats are Claude's first guess in tasks/R31-03 / data. |

## Open questions (ask the owner)
- Final game name

## Provisional (Claude chose overnight; owner to confirm)
| Date | Choice |
|---|---|
| 2026-11-11 | Player has 100 HP; respawn 3 s after death |
| 2026-11-11 | Walker zombie: 60 HP, wander 0.8 m/s, chase 2.6 m/s, sight 18 m / 120°, bite 10 dmg every 1.2 s, senses you within 2.5 m even from behind |
| 2026-11-11 | Bat 25 dmg / 12 stamina, machete 35 dmg / 10 stamina; stamina 100, regen 18/s after 0.8 s; attacks aim where the camera looks |
| 2026-11-11 | Pistol: 30 dmg, x2 headshot, 12-round mag, 36 reserve, 1.6 s reload, semi-auto, noise radius 35 m; aiming = over-shoulder zoom, 60% move speed, no sprint |
| 2026-11-11 | Hunger empties in about 40 min, thirst in about 25 min (×1.5 while sprinting); empty = 1 / 1.5 HP/s; both ≥ 80 = heal 0.5 HP/s; sprint 15 stamina/s, jump 8; respawn with needs at 75 |
| 2026-11-11 | Backpack: 24 slots, 40 kg max; start kit = bat, pistol, 36 ammo, 2 beans, 2 water, 1 bandage; item base prices in dollars are in data/items.json |
| 2026-11-11 | Hotbar = first 5 backpack slots; press again to holster; pick-up range 2.5 m, item must be in front of you and the camera |
| 2026-11-11 | Backpack screen does NOT pause the game (survival tension); you can still walk while it's open |
| 2026-11-12 | World: 64 m chunks, 2 m grid, load radius 3 (about 450 m), fog 120–190 m; random seed per new game; plains = flat light grass, forest = hilly dark grass |
| 2026-11-12 | One save slot; autosave every 2 min and on quit; auto-continue on start; F5/F9/F8 |
| 2026-11-11 | Skills: max level 20; XP to next level = 100 × 1.35^(level−1); zombie kill 25 XP (×1.5 headshot); 1 agility XP per 20 m sprinted; Combat +3% melee / +2% gun damage per level; Agility +3% stamina / +1% speed per level |

## Performance notes
- 2026-11-11 benchmark (empty test scene, uncapped): Mobile 1080p = ~88 FPS; 3D scale 0.75 = ~117; shadow map 2048 + low soft-shadow filter = ~115; Compatibility renderer = no gain (~86). Vsync caps at 60.
- Applied defaults: `scaling_3d/scale=0.8`, directional shadow size 2048, soft shadow filter 1. These become options in the settings menu later. Owner saw 40 FPS once (power-saving mode or battery is likely).

## 2026-11-13: owner answers on direction
- **Endless survival**, no ending.
- **Horse later** (after lakes and the boat), as quiet travel. **Motorbike cut** (Fable + Opus suggestion; the owner didn't object; confirm when we get there).
- **Camp upgrades: yes.** They are the main long-term goal.
- **No music; minimal sound.**
- **Action over horror.**
- **Difficulty curve: hard from the start, easier as the player grows.** (Danger rings outside the home core were added 2026-11-25, see the "0.3 World" row.) The player's power grows through skills, gear, camp upgrades, the dog and the car.
