# Rust migration: overnight work queue (2026-11-15)

Note (2026-11-17): The Rust game moved to its own repo rotland-rust; the Godot repo is frozen as the reference.
Godot paths (scripts/, scenes/, project.godot) refer to the frozen Godot repo (github.com/aymenhayderhtml-pixel/rotland, on the owner's machine at ../zombi servival).

The owner is asleep. Claude keeps working in a loop: when a round's workers finish, merge, build and test once, fix small breaks, wire into the game, check screenshots, commit and push, log worker mistakes in docs/JESTER.md, then start the next round. Workers: Gemini 3.8 Flash (owner's choice for these rounds), write-only (no builds/tests), worktree each. Plan: docs/RUST_PLAN.md.

## Done
- R3-07 spawner, R4-01 egui + HUD, R4-02 items/inventory/hotbar: merged, tested once, wired (play.rs), pushed (cacbcf6).

- R3-08 shooting, R3-09 zombie extras (+ R3-09b fixes), R4-03 loot crates: merged, tested once, wired, pushed. Leftover: gun bash on empty, scream alerting other zombies.

- R4-04 menus + R4-05 saves: merged, tested once, wired (Esc pause, main menu with --menu, slot picker, settings: mouse sensitivity / invert Y / show FPS applied; save and quit; Godot slot import), pushed. Leftover: FOV / render scale / vsync / volume settings not applied yet; New game picks a new random world.

- R5 camp logic: R5-01 economy/crafting, R5-02 survival/skills/difficulty, R5-03 bounties merged and tested; hunger/thirst wired (HUD bars, eating, saves). Not wired yet: wallet, skills XP, difficulty multipliers, bounties progress.

- R5-04 camp life + R5-05 camp screens merged and wired: trader, banker, guards and survivors stand at their posts, residents walk their routine (wave, sit at the fire, chop, carry, chat); E on the trader opens the shop, the banker the bank, the bounty board the bounties; Tab crafting, J bounties. Leftover for camp: guards shooting zombies, chat bubbles on screen, campfires (cooking recipes), skills XP from kills, hints, settlements save.

## Round now running
- Sound (kira) done: sounds.json library, distance fade and pan, day/night ambience, game sounds, footsteps.
- R6-02 farming merged and wired: four camp beds, E plants/waters/harvests/clears, a hoe tills new beds, crops grow with the clock, zombies trample field crops, farm saves. Leftover: dead crops are not drawn brown yet, no watering-can refill at water yet.
- R6-01 animals (worker cancelled part way, finished by Claude) merged and wired: deer, rabbits, boars spawn by biome, flee, boars charge; melee and guns hit them; hunting XP and hunt bounties; E butchers a corpse for meat, hide and antler.

- R7 (5 Gemini workers) merged and wired: player hub (Tab bag, L skills, J bounties, P profile, M map), XP toasts and level-up banner; ladders (E grab, W/S, Space); campfires (camp fire, campfire kit, cooking at a lit fire, relight with wood, glowing flame); dogs (breeder sells puppies, follow/stay with G, bites, E dog menu, pack); the car (camp car outside the gate, E in/out, WASD, Space, H, L, F refuel, chase camera, runs zombies over, gauge); map (explored fog, waypoint); tips for new players. Leftovers: car wheels do not spin/steer (part of the body mesh), parked cars are not solid, trunk screen not wired, stray dogs at POIs not spawned, map buildings not noted, profile stats (headshots, deaths, distance) not counted, hints for npc_talked/melee_swung/item_used/board_used events not sent.

- R8 (5 Gemini workers) merged and wired: axe fells trees (4 hits, 3-5 wood on the ground, the tree tips over, felled trees saved in worldstate), camp guards shoot zombies within 40 m (their kills are not the player's), speech bubbles over chatting camp people, profile counts kills by type / headshots / deaths / distance (save id "stats"), Settings > Controls rebinds every action (saved in settings.json; movement, E, R, G, Tab/L/J/P/M and 1-6 follow it), parked cars are solid (each car's own rays skip its box), stray dogs can wait at houses and cabins (5 % each), POI buildings are noted on the map. docs/RUST_PARITY.md lists what is still missing (top 15 at the end).

- R9 (7 Gemini workers, two restarted after Antigravity errors) merged and wired: first-person weapon in view (sway, kick, wall pull), gun bash when out of ammo, car trunk (E at the back), zombies waiting in buildings (wake at 35 m, cap 16, cleared buildings saved as visited_pois, clear bounties), zombie groans and snarls (25 m), Settings FOV / render scale / V-Sync applied live, car headlights (renderer has 3 spot lights). Leftover: headlights light only a short patch (low lamps, flat beam, Godot numbers); gun bash and trunk not yet tried in a real run.

## Round now running
- (none: waiting for the owner)

## Next rounds (in order)
1. ~~Guns in hand~~ (running as R3-08): wire game::guns into play (fire on left click when a gun is selected, reload R, ammo from the inventory, hit zombies, noise, muzzle flash light later), gun models in hand from the pack, aim (right mouse) with the rifle scope zoom. Claude can wire; or Gemini writes rotland/src/guns_play.rs.
2. ~~Zombie extras~~ (running as R3-09): scream, eat corpses, bang on doors/walls, herd, night horde, crawler (leg hits), ZombieSteering feelers + stuck escape (scripts/Enemies/ZombieAI.Actions.cs, ZombieSteering.cs, ZombieHerd.cs, ZombieHorde.cs, ZombieCrawl.cs, ZombieCorpses.cs).
3. ~~Loot crates~~ (running as R4-03): crates at world::chunk_crates spots (model crate_loot), loot tables (data/loot_tables.json, LootRoller), open with E, pick up items, dropped items.
4. ~~Night + flashlight~~ DONE by Claude (22b6ab1): moon, flashlight spot light, running clock. Leftover: night sky stars, the night horde timing: moon light and night sky in the renderer, flashlight spot light (engine needs one spot light in the shader), time of day running with world::time::Clock.
5. **Menus** (R4-04): main menu, pause (Esc), settings, 3 save slots; then saves (serde) + import of Godot slot files (never write the owner's real slot files in tests).
6. **Camp** (R5): NPCs, trader buy/sell, bank, bounties, crafting, campfire, hints, difficulty.
7. **Life** (R6): animals + hunting, farming, dog, car, map, ladders, sound (kira).

## Standing rules
- One windowed run at a time, always through `flock /tmp/rotland-heavy.lock`. Never `pkill -f`.
- Don't push broken builds: build + all tests + clippy clean before every push.
- Speed numbers only when the laptop is plugged in and quiet; say so when it is not.
- Every worker mistake (silly, made up, lazy, untested claims) goes in docs/JESTER.md as a plain table row.
