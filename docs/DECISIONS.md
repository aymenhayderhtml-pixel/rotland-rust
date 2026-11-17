# Decisions log

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
- **Difficulty curve: hard from the start, easier as the player grows.** No danger rings; the player's power grows through skills, gear, camp upgrades, the dog and the car.
