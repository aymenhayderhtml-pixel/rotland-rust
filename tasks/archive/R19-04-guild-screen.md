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
`rotland/src/camp_ui/mod.rs` belongs to lane R19-01: put your `mod guild;` / `pub use` lines in Wiring notes.
Recruits following the player in the 3D world is NOT in this task (next round).

## Worker notes
- **Files Changed**:
  - `rotland/src/guild_play.rs`: Created `GuildPlay` wrapper owning `game::guild::GuildManager`. Dispatches `refresh_morning`, event hooks (`on_kill`, `on_place_reached`, `on_item_delivered`, `on_poi_cleared`), and save state restoration (`to_state`, `from_state`, `from_seed_and_state`, `apply_state`). Fully covered by unit tests.
  - `rotland/src/camp_ui/screen.rs`: Added `CampScreen::Guild` enum variant with `title() -> "Guild"`, `tab_name() -> "Guild"`, and `as_str() -> "guild"`.
  - `rotland/src/camp_ui/action.rs`: Defined `GuildCampAction` with 7 variants (`AcceptTask`, `AbandonTask`, `ClaimTask`, `HireRecruit`, `DismissRecruit`, `SetRecruitOrder`, `BuyItem`).
  - `rotland/src/camp_ui/view.rs`: Added `CampView::guild(&self)` and thread-safe helper `set_guild_context` / `get_guild_context` so `CampView` provides the guild manager during rendering without breaking other lanes' instantiation of `CampView`.
  - `rotland/src/camp_ui/draw.rs`: Mounted `pub mod guild;`, added the Guild tab to the camp board tab bar, widened the panel dynamically to 1160px for the 3-column guild layout, and wired `draw_guild`.
  - `rotland/src/camp_ui/guild/layout.rs`: Added responsive 3-column layout calculation (`calc_guild_layout`) with support for 1920x986, 1280x720, and 1024x600 screen sizes.
  - `rotland/src/camp_ui/guild/tasks_ui.rs`: Task board section rendering active and offered tasks with progress bars and action buttons.
  - `rotland/src/camp_ui/guild/recruits_ui.rs`: Recruits section rendering hired teammates with order controls (Stay / Follow / Guard) and dismiss, alongside available candidate recruitment.
  - `rotland/src/camp_ui/guild/shop_ui.rs`: Guild shop section rendering available items by rank tier, buy buttons (1 / 5 / count stepper), and higher-tier locked previews.
  - `rotland/src/camp_ui/guild/mod.rs`: Top rank and reputation banner with progress bar to next rank threshold, shop tier, recruit slot badges, and 3-column layout dispatcher.
  - `rotland/src/camp_ui/guild/tests.rs` & `rotland/src/camp_ui/tests.rs`: Comprehensive unit tests covering string formatting, badges, actions, tab switching, and headless egui rendering.

- **Wiring Notes for Claude**:
  1. `rotland/src/main.rs`:
     - Add: `mod guild_play; pub use guild_play::GuildPlay;`
  2. `rotland/src/camp_ui/mod.rs`:
     - Add: `pub mod guild; pub use guild::*;`
     - Re-export `GuildCampAction`.
  3. Merging `GuildCampAction` into `CampAction` (`rotland/src/camp_ui/action.rs`):
     ```rust
     GuildAcceptTask { task_id: String },
     GuildAbandonTask { task_id: String },
     GuildClaimTask { task_id: String },
     GuildHireRecruit { recruit_id: String },
     GuildDismissRecruit { recruit_id: String },
     GuildSetRecruitOrder { recruit_id: String, order: RecruitOrder },
     GuildBuyItem { item_id: String, count: u32 },
     ```
  4. Handling actions in `rotland/src/camp.rs` (`apply`):
     - `GuildAcceptTask { task_id }`: `self.guild.accept_task(&task_id)`
     - `GuildAbandonTask { task_id }`: `self.guild.abandon_task(&task_id)`
     - `GuildClaimTask { task_id }`: `self.guild.claim_task(&task_id)` (award cash reward to wallet)
     - `GuildHireRecruit { recruit_id }`: `self.guild.hire_recruit(&recruit_id, &mut self.wallet)`
     - `GuildDismissRecruit { recruit_id }`: `self.guild.dismiss_recruit(&recruit_id)`
     - `GuildSetRecruitOrder { recruit_id, order }`: `self.guild.set_recruit_order(&recruit_id, order)`
     - `GuildBuyItem { item_id, count }`: `self.guild.buy_shop_item(&item_id, count, &mut self.wallet)`
  5. CLI test flag `--camp-screen=guild` (`rotland/src/app/startup.rs`):
     - In `--camp-screen` parser: `"guild" => Some(CampScreen::Guild)`
  6. App state ownership & frame UI (`rotland/src/app/state.rs` & `frame_ui.rs`):
     - Store `pub guild: GuildPlay` on camp/app state, initialized via `GuildPlay::new(world_seed)`.
     - In `frame_ui.rs`: set guild context via `camp_ui::view::set_guild_context(self.guild.manager().clone())` (or pass directly to `view.guild`).
  7. Save integration (`save` crate):
     - Add field `#[serde(default)] pub guild: game::guild::GuildState` to `SaveData`.
     - On save: `save.guild = app.guild.to_state()`
     - On load: `app.guild = GuildPlay::from_seed_and_state(world_seed, save.guild)`
  8. Gameplay event hooks:
     - Morning refresh (`frame_sim.rs` / `frame_camp.rs`): On day rollover, call `app.guild.refresh_morning(day, hour, Some(&mut player_wallet.dollars))`.
     - On kill: `app.guild.on_kill(&zombie_type, in_city)`
     - On place reached: `app.guild.on_place_reached(place_name)`
     - On item delivered: `app.guild.on_item_delivered(item_id, count)`
     - On POI cleared: `app.guild.on_poi_cleared(poi_id)`

## Gate report
- date: 2026-11-22
- base: main
- files changed (13): rotland/src/camp_ui/action.rs, rotland/src/camp_ui/draw.rs, rotland/src/camp_ui/guild/layout.rs, rotland/src/camp_ui/guild/mod.rs, rotland/src/camp_ui/guild/recruits_ui.rs, rotland/src/camp_ui/guild/shop_ui.rs, rotland/src/camp_ui/guild/tasks_ui.rs, rotland/src/camp_ui/guild/tests.rs, rotland/src/camp_ui/screen.rs, rotland/src/camp_ui/tests.rs, rotland/src/camp_ui/view.rs, rotland/src/guild_play.rs, tasks/R19-04-guild-screen.md
- tests passed: 30
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
