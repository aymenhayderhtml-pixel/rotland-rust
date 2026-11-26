# R37-07: gunsmith in camp, gun shops in towns

Owns: data/settlements.json (camp NPC/bench spot), data/camp_upgrades.json (a `gunsmith` tier track, like `traders`), data/towns.json (one `gun_shop` lot per town + footprint), world/src/towns/lots.rs (guarantee one gun_shop lot), game_camp/src/camp/npc.rs + resident.rs (the `npc_gunsmith` resident), data/camp_life.json (gunsmith lines/schedule if residents use it)
Read: tasks/R37-common.md, how the camp trader NPC and its stall are placed (grep `npc_trader` in data/ and game_camp/, world/src/camp_layout/), game_camp/src/camp_upgrades/, world/src/towns/ (mod.rs, lots.rs, interior.rs), data/camp_upgrades.json "traders".

- Camp: the gunsmith bench + NPC next to the trader area, inside the walls, not blocking paths (check the camp layout); golden test must still pass: if the camp layout is checked by `world/tests/golden.rs`, place the bench as an upgrade prop (like the R19 camp upgrade props), never move existing camp objects.
- Camp upgrade track `gunsmith` tiers 0-3 (costs like `traders`), tier unlocks stock in R37-03.
- Towns: each town gets exactly one `gun_shop` lot near its centre on the main street, with a `shopkeeper_<town_key>` NPC inside behind the counter (data spot in building-local space). Shopkeepers are friendly survivors (zombies don't target them inside; copy how rescue survivors/NPCs are kept safe if needed, else report).
- Tests: every town has one gun_shop; camp has the gunsmith spot; tiers load.
