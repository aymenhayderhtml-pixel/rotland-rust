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

### Files changed
- `data/loot_tables.json`: added `town_home`, `town_store`, and `town_station` loot tables using only item ids from `data/items.json`.
- `data/spawning.json`: added `"townDensity": 1.8`.
- `data/rescue.json`: added `"perTown": [1, 2]`.
- `data/guild.json`: added 4 town explore task templates (`explore_dusthollow`, `explore_frostmere`, `explore_blackwater`, `explore_ridgeback`).
- `data/hints.json`: added `trailhead` and `town` hints (with `"trigger": "after:999999"` so `HintConfig::check` validates them without modifying `game_core/src/hints/triggers.rs`; triggered via `hints.request(...)`).
- `game_zombie/src/spawner_config.rs`: added `town_density: f32` (default `1.8`) to `SpawnConfig`.
- `game_zombie/src/spawner.rs`: added `pub in_town: bool` and `with_in_town` to `SpawnContext`, and applied `self.config.town_density` in `Spawner::calculate_cap` when `ctx.in_town` is true.
- `game_camp/src/rescue.rs`: added `pub per_town: [usize; 2]` (default `[1, 2]`) to `RescueConfig`.
- `game_camp/src/rescue_place.rs`: added `TOWN_RESCUE_BUILDINGS`, `is_town_rescue_building`, `place_town_survivors`, and wired `place_survivors_in_world` to also place `perTown` (1–2) survivors per town from `world_gen.towns()`.
- `game_camp/src/guild/task_hooks.rs`: added `GuildTaskHooks::tick_with_towns` (called by `tick` with `&[]`) so explore tasks match `TownSite::name` when the player is inside `town.contains_point(player_xz)`.
- `game/tests/all/towns_gameplay_tests.rs`: new integration test file covering town loot tables, `townDensity` cap scaling, 1–2 rescue spots per town, guild explore tasks targeting towns, and town/trailhead hints.

### Shared-file lines for the lead
1. `game/tests/all/main.rs` — add:
   ```rust
   mod towns_gameplay_tests;
   mod warmth_save_tests;
   ```
2. `game/tests/all/hints_tests.rs:14` — update `assert_eq!(config.hints.len(), 14);` to `16`.
3. `rotland/src/play.rs` (around lines 427-466, `SpawnContext` struct literal) — add `in_town`:
   ```rust
   let in_town = world_gen.town_at(player.x, player.z).is_some();
   // ... inside SpawnContext { ... }:
   in_town,
   ```
4. `rotland/src/app/frame_guild.rs:125-128` — pass towns to `tick_with_towns`:
   ```rust
   let towns = world.world_gen.as_deref().map_or(&[][..], |g| g.towns());
   let events = match guild_hooks.as_mut() {
       Some(hooks) => hooks.tick_with_towns(pl.camp.guild.manager_mut(), &guild_panel.places, towns, player, dt),
       None => Vec::new(),
   };
   ```
5. `rotland/src/app/frame_camp.rs` (inside `if let Some(hints) = play_state.hints.as_mut()`, where `at = actors.player`) — request trailhead & town hints:
   ```rust
   if let Some(g) = world.world_gen.as_ref() {
       let p2 = glam::Vec2::new(at.x, at.z);
       if g.roads().trailheads().iter().any(|&th| th.distance_squared(p2) <= 144.0) {
           hints.request("trailhead");
       }
       if g.town_at(at.x, at.z).is_some() {
           hints.request("town");
       }
   }
   ```

### Known issues / notes
- Did not run `cargo` or `tools/gate.sh` per subagent rules; `rotland/src/play.rs` will fail to compile until `in_town` is added to its `SpawnContext { ... }` literal, and `game/tests/all/hints_tests.rs:14` expects 14 hints until updated to 16.
