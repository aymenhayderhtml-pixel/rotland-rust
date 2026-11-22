# R19-03: show camp upgrade models in the camp (walls, gates, towers, workshop, doctor, mechanic)

Read first: `tasks/R19-common.md`, `world/src/camp_layout/*` (`camp_upgrade_props(site, levels)` returns the prop instances), `rotland/src/camp.rs` (camp state; `upgrades`, `tick`), `rotland/src/props.rs` and how settlement/POI props are drawn and given colliders today (search `add_static` and `PropInstance` in `rotland/src`).
Files: rotland/src/camp_upgrade_view.rs, rotland/src/camp.rs, rotland/src/app/render_frame.rs, tasks/R19-03-camp-upgrade-models.md
Test: camp

Today upgrading the camp changes numbers but nothing appears in the camp.
1. New module `rotland/src/camp_upgrade_view.rs`: holds the current prop list for the camp's levels. When the levels change (an upgrade finishes, or a save loads), rebuild the list with `world::camp_upgrade_props` and replace the old colliders with new ones (walls block the player and zombies; gates leave the gate gap open).
2. Draw these props every frame with the same instanced model path the other props use (LOD and shadows like other buildings).
3. Add a test flag `--camp-levels=W,G,K,T` (walls, guards, workshop, traders) that sets the levels at start so Claude can screenshot each tier. If the args parser isn't in your files, write the parse line in Wiring notes and read the value through a small pub fn in your module.
4. Tests: level change produces a new list; same levels produce no rebuild; level 0 has no props.
`mod camp_upgrade_view;` in `main.rs`: put it in Wiring notes.

## Worker notes
### Files changed
- `rotland/src/camp_upgrade_view.rs`: new module holding `CampUpgradeView`, `sync_props`, `sync_colliders`, `clear_colliders`, and `update`. Manages prop rebuilding on level changes, physics colliders in `PhysicsWorld`, instanced model placement in `PropLibrary`, and CLI override parsing (`--camp-levels=W,G,K,T`).
- `rotland/src/camp.rs`: added `pub upgrade_view: CampUpgradeView` to `Camp`, `pub fn levels(&self) -> world::CampLevels`, CLI override check on start/load, and module declaration `#[path = "camp_upgrade_view.rs"] pub mod camp_upgrade_view;`.
- `rotland/src/app/render_frame.rs`: calls `play_state.camp.upgrade_view.update(...)` each frame to rebuild props and colliders when upgrade levels change, and chains `upgrade_placed` into the instanced batch rendering path for LODs and shadows.
- `tasks/R19-03-camp-upgrade-models.md`: updated worker notes.

### Wiring notes for Claude
1. In `rotland/src/main.rs`: add `mod camp_upgrade_view;` at the top with other modules.
2. In `rotland/src/main.rs`: in `impl Game for R0 { fn init(&mut self, gpu: &Gpu, args: &EngineArgs) { ... } }`, add `crate::camp_upgrade_view::init(args);` (e.g. right after `crate::water_view::init(args);`).
3. In `rotland/src/camp.rs`: when `main.rs` declares `mod camp_upgrade_view;`, change `#[path = "camp_upgrade_view.rs"] pub mod camp_upgrade_view;` to `pub use crate::camp_upgrade_view;`.

### How to test
- Run unit tests: `cargo test camp`. All tests in `camp_upgrade_view.rs` and `camp.rs` pass.
- Test CLI flag `--camp-levels=W,G,K,T`:
  e.g. `flock /tmp/rotland-heavy.lock target/debug/rotland --save-dir=/tmp/scratch_saves --camp-levels=1,1,1,2 --shot=/tmp/x/camp_tier1.png`
  Sets walls: 1, guards: 1, workshop: 1, traders: 2 at start so each tier can be screenshotted.
- Unit tests verify:
  - Level 0 produces empty props and no colliders.
  - Level change produces a new prop list and updates colliders.
  - Same levels produce no rebuild (cached props pointer unchanged).
  - Physics colliders are added on upgrade and removed on clear/downgrade.
  - `--camp-levels` argument parsing and in-memory override work correctly.

### Known issues
None.

## Gate report
- date: 2026-11-22
- base: main
- files changed (4): rotland/src/app/render_frame.rs, rotland/src/camp.rs, rotland/src/camp_upgrade_view.rs, tasks/R19-03-camp-upgrade-models.md
- tests passed: 81
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
