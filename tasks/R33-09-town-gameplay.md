# R33-09: towns are worth the trip (loot, zombies, trapped survivors, guild tasks)

Owns: data/loot_tables.json, data/spawning.json, data/rescue.json, data/guild.json, data/hints.json, data/poi.json, game_camp/src/rescue_place.rs, game_camp/src/guild/* (only the explore-target code), game_zombie/src/spawner_config.rs, game/tests/all/towns_gameplay_tests.rs (new)
Files: data/loot_tables.json, data/spawning.json, data/rescue.json, data/guild.json, data/hints.json, data/poi.json, game_camp/src/rescue_place.rs, game_camp/src/guild/task_hooks.rs, game_zombie/src/spawner_config.rs, game/tests/all/towns_gameplay_tests.rs, game/tests/all/main.rs
Read: tasks/R33-round.md, tasks/LEAD.md, docs/WORLD.md, docs/DECISIONS.md, game_camp/src/rescue_place.rs, game_camp/src/guild/task_hooks.rs, game_camp/src/guild/tasks.rs, game_camp/src/guild/roller.rs, game_zombie/src/spawner_config.rs, game_zombie/src/spawner.rs, rotland/src/app/frame_guild.rs, game/tests/all/main.rs
API: game_camp/src/lib.rs, game_zombie/src/lib.rs, world/src/lib.rs
Data: data/loot_tables.json, data/items.json, data/spawning.json, data/rescue.json, data/guild.json, data/hints.json, data/poi.json
Overlap: `game/tests/all/main.rs` is shared. This is the lowest-numbered R33 task that needs a `mod` line there, so you write BOTH `mod towns_gameplay_tests;` and `mod warmth_save_tests;` (the latter is R33-10's test file; it is not yet in the repo, add the line anyway). Rotland wiring (`rotland/src/app/frame_guild.rs` passing the town places to the guild, and the `in_town` spawn flag in the rotland spawner) is not in your Files: report those lines to the lead.
Rules: `tasks/LEAD.md` subagent rules. Use the "R33 shared names" in `tasks/R33-round.md` EXACTLY. Read how Ashford does each of these (`docs/DECISIONS.md` "2x plains zombies" for the city, the city loot tables, and how rescue spots are placed in houses: `game_camp/src/rescue_place.rs`) and copy it for towns. Report shared-file lines (`game/tests/all/main.rs` `mod` line, any rotland wiring) to the lead.

1. **Loot tables:** `town_home` (food, water, bandages, cloth, some ammo), `town_store` (more food/drink/tools, rare gun parts if they exist), `town_station` (ammo, tools, medkits, a rare weapon). Use only item ids that exist in `data/items.json` (read it). The danger tier bonus already adds rolls (R31).
2. **Zombies:** a town density factor in `data/spawning.json` (e.g. `townDensity: 1.8`). The spawner applies it when the spawn point is inside a town radius. That needs `town_at`: pass a plain `in_town: bool` (or the factor) through the spawn context, as the tier multiplier was passed in R31 (read `spawner_config.rs`). Report the rotland line that sets it.
3. **Trapped survivors in towns:** `rescue_place.rs` picks houses for survivors today. Let it also pick from town lots (say 1-2 per town, from data `rescue.json` `perTown`).
4. **Guild explore tasks:** the guild's "explore" tasks (R27) should also offer "Explore Dusthollow" etc. (target = the town centre, reached = inside the radius). Find the explore target list in `game_camp/src/guild/` and add towns as targets when the generator has them (pass the list in, don't depend on rotland).
5. **Hints:** first time near a trailhead: "Roads lead out to the far lands. The farther you go, the more dangerous it gets." First time in a town: "Towns have good loot but many zombies. Watch your back."
6. **Tests:** the loot tables use only existing items; town density > 1; 1-2 rescue spots per town; an explore task can target a town.

## Worker notes
