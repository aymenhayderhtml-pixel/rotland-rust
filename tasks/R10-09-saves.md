# R10-09: Save fixes — dropped items, camp fire, safe loading, weapons, settlements (GitHub #15 + parity) — PHASE B

**Start only after Claude merged R10-01 into main** (pull main first: `git merge main` in your worktree). Read `tasks/R10-common.md`, then R10-01's "Where to wire things" table in `tasks/R10-01-split-main.md`.

Godot reference: `../zombi servival/scripts/Core/SaveSystem.cs`, `scripts/World/WorldState.cs`, `scripts/World/DroppedItem.cs`, `scripts/Combat/WeaponHolder.cs`, `scripts/Settlement/SettlementWatcher.cs`. Save format must stay compatible with Godot saves (same keys).

**Safety: tests never touch the owner's real saves.** Use temp dirs only.

## You own
- `rotland/src/session.rs`
- `rotland/src/camp.rs` — save/load parts only
- The save/load file R10-01 created under `rotland/src/app/` (say which)
- `save/src/**`, `save/tests/**`
- `game/src/campfire/manager.rs`, `game/src/loot/world_items.rs` (save hooks only)

## Fix
1. **SAVE-002 dropped items**: write and read `worldstate.dropped` (the slot exists in `save/src/systems/worldstate.rs`). Items on the ground survive save + reload, including in unloaded chunks.
2. **BUG-003 camp fire duplicates**: when loading a save that already has the permanent camp fire, don't add a second one (skip if a permanent fire is within 1 m). Test it.
3. **SAVE-001 / ARCH-002**: all world state (campfires, looted, fallen trees, visited POIs, dropped) is captured in **one** place: `session::capture` (and restored in one place). Remove the scattered writes.
4. **SEC-001**: on load, check health / position / yaw / pitch / time with `is_finite()` and clamp to valid ranges (health 0..max, hour 0..24). Bad values → safe default + log.
5. **SAVE-004**: damaged campfire/farm/bounty data → keep the reset, but also return a message the game shows as a toast ("Farm data was damaged and was reset"). Give Claude the hook.
6. **SAVE-003 / SEC-002**: `slots.rs` `path_for` accepts only slots 1-3; path check resolves symlinks (`canonicalize`) before comparing.
7. **Weapons save (parity: missing)**: save the equipped weapon and each gun's loaded magazine rounds (Godot `weapons` system keys). Today the bat is re-equipped on load.
8. **Settlements save (parity: logic only)**: save/load `SettlementWatcher` as system `settlements` (Godot keys); respawn uses its chosen camp.

## Tests (`save/tests/`, temp dirs)
Round-trip each: dropped items, weapons+mag, settlements; double camp fire not created; NaN health loads as a valid value; slot 4 rejected.

## Worker notes
