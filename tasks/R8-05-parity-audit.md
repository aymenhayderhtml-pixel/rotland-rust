# R8-05: Parity checklist (Godot vs Rust)

Read `tasks/R8-common.md` first. This task writes **one doc only**: `docs/RUST_PARITY.md` (you may edit `docs/` for this task). No code.

Go through `docs/CODEMAP.md` and every folder under `scripts/` (skip `scripts/Debug/`). For each Godot system/feature (a feature is something the player can see or do, e.g. "tree felling", "car trunk", "scope zoom", "night horde", "settings: FOV"), find whether the Rust code in `rust/` has it, with evidence (file path + function name, found by searching). Mark each row one of:
- **done** (logic and it is reached from `rust/rotland/src/main.rs` / `play.rs` / `camp.rs`; follow the calls),
- **logic only** (code exists in the game crate but nothing in rotland calls it),
- **missing**.
Table columns: Area | Feature | Godot file(s) | Status | Rust evidence | Notes. Group by area (Player, Combat, Zombies, World, Camp, Economy, Animals, Farming, Dogs, Vehicles, UI, Audio, Saves, Settings).
At the end: a list of the 15 most important missing/logic-only features for a player, ordered by how much a player would notice, each with a one-line size guess (small/medium/large).
Be exact: never mark "done" without having found the call in rotland. If unsure, mark "check" and say why.

## Worker notes

- **Files changed / created**:
  - `docs/RUST_PARITY.md`: Complete Godot vs Rust parity matrix across all 14 requested areas (Player, Combat, Zombies, World, Camp, Economy, Animals, Farming, Dogs, Vehicles, UI, Audio, Saves, Settings), with exact file paths, function references, and status tags (`done`, `logic only`, `missing`). Also includes ranked Top 15 missing / logic-only features with player impact analysis and size estimates.
  - `tasks/R8-05-parity-audit.md`: Filled in worker notes.
- **Verification methodology**:
  - No code changes, no builds performed (per task rules).
  - Traced every system from Godot source files in `scripts/` against Rust workspace crates (`rust/game`, `rust/world`, `rust/audio`, `rust/save`, `rust/engine`, `rust/humanoid`, `rust/physics`, `rust/rotland`).
  - Verified call sites in `rust/rotland/src/main.rs`, `rust/rotland/src/play.rs`, and `rust/rotland/src/camp.rs` to distinguish between `done` (actively reached in gameplay runtime) vs `logic only` (implemented in library crates but uncalled or disabled by stubbed arguments).
- **Known issues / key observations**:
  - Tree felling, POI building zombies, first-person weapon view model, and controls key rebinding are prominent missing features.
  - Several systems (such as zombie special states like scream, corpse eating, crawling, herd join; car trunk screen; guard shooting; FOV setting) exist in logic but await hookup into the main application.

