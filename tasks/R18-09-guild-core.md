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

