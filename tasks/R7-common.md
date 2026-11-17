# Rules for every R7 worker task

Everything in `tasks/R5-common.md` applies (write-only: no cargo build/test/clippy; game crate logic; real data files with Godot keys; seeded randomness; private submodules + `pub use`; `mod.rs` layout; tests in `rust/game/tests/`; one commit on your branch; never push).

Extra rules from the last rounds' mistakes (`docs/JESTER.md` #13-#19), because nobody compiles your code until Claude merges:
1. **Open every file you call into** and copy the real signature. Examples that broke last time: `Health::revive(Option<f32>)` (not `revive()`), `Inventory::add(id, count) -> u32 leftover`, `SkillSystem::add_xp(&str, f32)`, `SkillSystem::get_bonus(&str) -> f32`, `SkillSystem::get_level(&str) -> u32`, `PhysicsWorld::raycast(origin, dir, max) -> Option<Hit { point, normal, distance }>`.
2. **Fields used by a second file of the same type must be `pub(super)`** (or `pub(crate)`). Check every `self.field` in each file against where the struct is defined.
3. **Borrows:** no `self.x` use while another borrow of `self` is alive; closures that capture `self` cannot be passed to `&mut self` methods; lend RNGs out with `std::mem::replace` if needed.
4. **Derives:** only derive a trait every field has (`&dyn Fn` has no `Debug`/`Clone`; `Inventory` has no serde).
5. **No unused imports or variables** (clippy runs with warnings as failures). No `map_or(false, ..)` (use `is_some_and`), collapse nested `if`s, no manual range checks (`(a..b).contains(&x)`).
6. **Tests:** every expected number traced by hand from the data file and the first frame (timers that start at 0 fire at once). State in your report which numbers you traced.
7. Max ~300 lines per file. If the Godot file is longer, split by topic (`foo/mod.rs`, `foo/ai.rs`, ...).
8. Your lane owns only the files listed in your task. Do not edit `rust/rotland/src/main.rs`, `play.rs`, other lanes' folders, or `docs/`. `rust/game/src/lib.rs`: add only your own `pub mod x;` + `pub use x::*;` lines (Claude resolves the merge).
9. Before `pub use x::*` from `lib.rs`, make sure your public names do not clash with existing ones in the game crate (search `rust/game/src` for each type name). Prefix if needed (e.g. `DogConfigData`, `CarState`).
