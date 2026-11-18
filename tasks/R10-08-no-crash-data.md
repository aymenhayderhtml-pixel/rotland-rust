# R10-08: No crashes on missing or broken files; load item data once (GitHub #14, #19 RULE-003/CODE-002/CODE-004) — PHASE A

Read `tasks/R10-common.md` first. `gh issue view 14` has every finding with file:line.

## You own
- `game/src/items/**` (inventory, hotbar, start kit, registry)
- `game/src/economy/bank.rs`
- `game/src/animals/config.rs` (only the `Default` impl)
- `game/src/loot/roller.rs`
- `game/src/vehicles/manager.rs` (line ~91 unwrap only)
- `game/src/crafting/system.rs` (line ~142 unwrap only)
- `paths/src/lib.rs`
- `rotland/src/player_visual.rs`
- `save/tests/saves.rs`, `world/tests/terrain_unit_tests.rs` (RULE-003 only)
- `engine/src/render/camera.rs` (`ortho`), `save/src/slots.rs` (`SlotInfo::damaged`) — CODE-004 only: remove if truly unused (grep the whole repo first), else say where it's used.

## 1. One shared item registry (CRASH-003/004, PERF-005)
- `Inventory::new`, `with_capacity`, `Bank::new` re-read `data/items.json` from disk and panic. Add constructors that take `Arc<ItemRegistry>` (or whatever the registry type is called — open it).
- **Keep the old constructors** so call sites in main.rs / camp.rs / play.rs still compile this round, but make them load through one cached `OnceLock<Arc<ItemRegistry>>` and never panic (log + empty registry). Mark them `#[deprecated(note = "R11: pass Arc<ItemRegistry>")]` **only if** that doesn't create clippy warnings at call sites you don't own — otherwise use a doc comment.
- List every call site (grep) in your notes so Claude can switch them over.

## 2. Friendly errors instead of panics
- **CRASH-001** `player_visual.rs`: missing character/weapon model or skeleton in `models.pack` → log an error and use a fallback (a simple box mesh or skip drawing that weapon). The game must keep running.
- **CRASH-002** (main.rs:465, not yours): write a `pub fn` in the world/game crate that returns `Result<_, DataError>` with the **file name** in the message; Claude will call it from main.rs and show the message.
- **CRASH-005/006**: start kit and `AnimalConfigData::default` → no disk reads in `Default`; return `Result` from the loader.
- **CRASH-007** `paths`: started outside the project folder without `ROTLAND_REPO` → clear message: what it looked for and how to fix (`ROTLAND_REPO=/path/to/rotland-rust`). Return `Result` or exit with that message; no panic backtrace.
- **BUG-005** loot roller `.last().expect()` on an empty table → return no item. **BUG-006/007/008** → `if let` / entry API. **CODE-002** loot roller: entry API instead of 3 lookups.

## 3. RULE-003
Replace `env!("CARGO_MANIFEST_DIR")` in the two test files with `paths::repo_path`.

## Tests
- Empty loot table rolls nothing (no panic).
- Inventory built from a shared registry behaves the same as before (existing inventory tests still meaningful).
- `paths` with a bad dir returns the error message (pure function test).
- player_visual fallback: a pure helper that picks fallback when a mesh id is missing.

## Worker notes
