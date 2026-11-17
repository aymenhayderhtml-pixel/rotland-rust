# R5-03: Bounties — Rust port

Read `tasks/R5-common.md` first.

Port to `rust/game/src/bounties/`:
- `scripts/Bounties/*.cs` except `BountyBoardVisuals.cs`: BountyConfig/Data and BountyTemplateData from `data/bounties.json`, BountyDef, BountyRoller (seeded rolls: must match Godot's for the same seed; use `world::DotnetRandom` (the C# System.Random port in the `world` crate) exactly like the C# code uses `System.Random`), BountyManager (accept, progress on events such as zombie kills by type / items delivered, complete, reward, expiry over days), BountyBoard logic (which bounties are offered, refresh per day).
- Progress comes in through plain calls (e.g. `on_zombie_killed(kind)`, `on_item_delivered(id, count)`), no event bus.
- Godot save id: "bounties" (check `BountyManager.cs`).
Tests: rolls for 3 fixed seeds are stable and valid against the data (count them), accept → progress → complete → reward, expiry, board refresh per day, save round trip.
