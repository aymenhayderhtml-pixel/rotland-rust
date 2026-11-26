# R18-09: guild, core (pure logic + data): task board, recruits, guild shop

Read first: `tasks/R16-common.md` (foundation rules), `game/src/bounties/` (today's bounty system: reuse ideas, don't duplicate), `game/src/economy/` (prices), `data/bounties.json`, `data/items.json` (ids only), GitHub issue text: "Guild building to see tasks, recruit teammates, and buy items".
Files: game/src/guild/*, game/src/lib.rs, game/tests/all/guild_tests.rs, game/tests/all/main.rs, data/guild.json, tasks/R18-09-guild-core.md
Test: guild

- **Task board:** generated tasks refreshed every in-game morning: kill (N zombies of a type, optionally in the city), fetch (bring N of an item), explore (reach a named place: "Ashford core", "the lake", a POI), clear (clear a building). Rewards: cash + guild reputation + sometimes an item. Max 5 open on the board, accept up to 3. Progress updates from game events (`on_kill(type, in_city)`, `on_item_delivered`, `on_place_reached(name)`, `on_poi_cleared`). Deterministic from seed + day.
- **Reputation ranks** (Rookie -> Veteran -> Elite...) unlock recruit slots and better shop stock.
- **Recruits:** hire teammates (name, role: fighter/scavenger/guard, level, daily wage, health). States: idle at camp, following the player, guarding the camp, dead. API to hire, dismiss, set order (follow/guard/stay), pay wages each morning (unpaid = they leave). (Their 3D actors and AI come later: here only data and state.)
- **Guild shop:** stock tier by rank; buy only (sell stays at the trader).
- One save struct `GuildState` (rules in R16-common).
Tests (`guild_` names): board refresh deterministic; accept limit; each task type progresses and completes and pays; rank thresholds; recruit hire/wage/leave; shop stock by rank; save round trip.

## Worker notes
- **Files added/changed:**
  - `data/guild.json`: data definitions for reset hour, max offered/active tasks, reputation ranks, task templates (kill, fetch, explore, clear), recruit candidate pool, and tiered guild shop items.
  - `game/src/guild/mod.rs`: module root re-exporting guild core types and functions.
  - `game/src/guild/config.rs`: `GuildConfig`, rank/task/recruit/shop defs, and `from_repo()` / JSON loading.
  - `game/src/guild/rank.rs`: rank progression (Rookie -> Veteran -> Elite -> Master -> Legend), reputation thresholds, recruit slot and shop tier lookups.
  - `game/src/guild/tasks.rs`: `GuildTask`, `GuildTaskKind` (Kill, Fetch, Explore, Clear), `GuildReward`, progress ratio and event matching methods.
  - `game/src/guild/roller.rs`: `GuildTaskRoller` for deterministic task board generation from world seed and in-game day.
  - `game/src/guild/board.rs`: `GuildBoard` managing offered tasks (max 5) and active tasks (max 3), accept/abandon/claim and event dispatch.
  - `game/src/guild/recruits.rs`: `GuildRecruit`, `GuildRecruits`, roles (`Fighter`, `Scavenger`, `Guard`), states (`IdleAtCamp`, `FollowingPlayer`, `GuardingCamp`, `Dead`), orders (`Stay`, `Follow`, `Guard`), hiring within rank limit, dismiss, and morning wage payment (`pay_wages`: unpaid leave).
  - `game/src/guild/shop.rs`: `GuildShop`, `GuildShopItem`, buy-only purchasing gated by player's unlocked rank shop tier, inventory tracking, and daily restocking.
  - `game/src/guild/save.rs`: single save struct `GuildState` with `#[serde(default)]` on all fields.
  - `game/src/guild/manager.rs`: `GuildManager` tying together board, recruits, shop, and `refresh_morning(day, hour, funds)`.
  - `game/src/lib.rs`: registered and exported `pub mod guild; pub use guild::*;`.
  - `game/tests/all/main.rs`: registered `mod guild_tests;`.
  - `game/tests/all/guild_tests.rs`: 14 comprehensive tests covering deterministic roll, accept limits, task progress/completion/claim for each task type, rank unlocks, recruit orders/wages/leaving, shop buying by tier/stock/funds, morning refresh, and save round-trip.

- **How to test:**
  `tools/gate.sh --task tasks/R18-09-guild-core.md` or `cargo test guild`.

- **Wiring notes for Claude:**
  - Initialize manager on world load:
    `let mut guild = GuildManager::from_repo(world_seed)?;`
    Or restore from save:
    `guild.from_state(save_data.guild);`
  - In morning game loop (each hour tick or day transition):
    `if let Some(report) = guild.refresh_morning(day, hour, Some(&mut player_wallet.dollars_mut())) { ... }`
  - When game events happen:
    - Zombie killed: `guild.on_kill(zombie_type, in_city);`
    - Items brought/collected: `guild.on_item_delivered(item_id, count);`
    - Exploration / location reached: `guild.on_place_reached(location_name);`
    - POI cleared: `guild.on_poi_cleared(poi_id);`
  - UI interactions at Guild building:
    - Inspect board: `guild.board().offered()`
    - Accept task: `guild.accept_task(task_id)`
    - Abandon task: `guild.abandon_task(task_id)`
    - Claim completed task: `let reward = guild.claim_task(task_id)?;` (adds dollars to wallet, items to inventory)
    - View/hire recruits: `guild.recruits().available()`, `guild.hire_recruit(recruit_id, &mut player_wallet.dollars_mut())`
    - Command teammate: `guild.set_recruit_order(recruit_id, RecruitOrder::Follow / Guard / Stay)`
    - Buy guild shop items: `guild.shop().available_items(guild.shop_tier())`, `guild.buy_shop_item(item_id, count, &mut player_wallet.dollars_mut())`
  - Save serialization:
    `let save_state: GuildState = guild.to_state();`

- **Known issues:**
  None. All 14 tests pass and clippy is clean.

## Gate report
- date: 2026-11-21
- base: main
- files changed (15): data/guild.json, game/src/guild/board.rs, game/src/guild/config.rs, game/src/guild/manager.rs, game/src/guild/mod.rs, game/src/guild/rank.rs, game/src/guild/recruits.rs, game/src/guild/roller.rs, game/src/guild/save.rs, game/src/guild/shop.rs, game/src/guild/tasks.rs, game/src/lib.rs, game/tests/all/guild_tests.rs, game/tests/all/main.rs, tasks/R18-09-guild-core.md
- tests passed: 14
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
