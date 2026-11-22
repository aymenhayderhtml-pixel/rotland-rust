# R19-04: guild screen at the camp board (task board, recruits, guild shop)

Read first: `tasks/R19-common.md`, `tasks/R18-09-guild-core.md` (the API and its wiring notes), `game/src/guild/*`, `rotland/src/camp_ui/*` (the camp board screens: tabs in `screen.rs`/`state.rs`, actions in `action.rs`; `draw_shop.rs` is a good style example), `rotland/src/menus/theme.rs`.
Files: rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/state.rs, rotland/src/camp_ui/view.rs, rotland/src/camp_ui/action.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/guild/*, rotland/src/camp_ui/tests.rs, rotland/src/guild_play.rs, tasks/R19-04-guild-screen.md
Test: camp_ui

The guild logic exists (`game::guild::GuildManager`) but nothing in the game uses it.
1. `rotland/src/guild_play.rs`: owns a `GuildManager` for the game: create from the world seed, `refresh_morning` when the in-game day changes, hooks `on_kill(kind, in_city)`, `on_place_reached(name)`, `on_item_delivered`, and `to_state`/`from_state` for the save. Pure functions + tests.
2. A new camp board tab "Guild" (`rotland/src/camp_ui/guild/`): three sections:
   - Task board: offered tasks (name, goal, reward) with Accept; active tasks with progress bars, Abandon, and Claim when done.
   - Recruits: available recruits (name, role, wage) with Hire; hired recruits with their order (Stay / Follow / Guard) buttons and Dismiss.
   - Guild shop: items for the player's rank tier with price and Buy.
   Show the player's rank and reputation at the top. New `CampAction` variants for each button. Must fit 1280x720 and 1920x986.
3. Test flag `--camp-screen=guild` must open it (follow how `--camp-screen=upgrades` works; if the parser is in a file you don't own, put the line in Wiring notes).
4. Wiring notes for Claude: exactly where to create the `GuildPlay` in app state, where to call the kill / day-change hooks (`rotland/src/app/frame_sim.rs`, `frame_camp.rs`), how to apply each new `CampAction`, and the save field (`save` crate) with `#[serde(default)]`.
Recruits following the player in the 3D world is NOT in this task (next round).

## Worker notes
