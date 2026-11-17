# Rules for every R5 (camp) worker task

- Read `AGENTS.md`, `rust/README.md` and `docs/JESTER.md` first (past worker mistakes: do not repeat them).
- Write-only: do NOT run cargo build, test or clippy. Claude builds and tests once after merging. So trace every expected number in your tests by hand from the code and the data file, and never report test output you did not see.
- Pure logic in the `game` crate (`rust/game/src/<module>/`), no rendering, no egui, no winit. Claude wires it into the game and draws the screens.
- Data comes from `data/*.json` through `paths::repo_path("data/...")`; the default loader must load the real file (never quietly empty). Keep Godot's JSON key names; use serde `rename`/`alias` where Rust names differ.
- Port behaviour exactly, including rounding (C# `Math.Round` is banker's rounding: use `f64::round_ties_even`), clamps and default values. Add a `// Godot: File.cs Method` comment where a rule is ported.
- Save data: provide `to_save() -> serde_json::Value` and `from_save(&serde_json::Value)` using the exact Godot save keys from the `.cs` `Save()`/`Load()` code (Claude stores them under the Godot system id in the save file).
- Randomness: take a seed or a `&mut impl FnMut() -> f32`; never global randomness. Tests switch randomness off or fix the seed.
- `mod x;` inside `a.rs` means file `a/x.rs`. Prefer `a/mod.rs` with submodules in `a/`.
- Tests in `rust/game/tests/<module>_*.rs` (integration tests use only temp folders; `cfg(test)` is off there). Comparison tests count what they compared and fail on zero.
- Clippy-clean style: no index loops where an iterator works, collapse nested `if let` with `&&` (Rust 2024 let-chains), no `3.14`-like literals.
- Files under ~300 lines. Commit to your branch in one commit at the end; never push, never touch main.
- Final report: files, public API, Godot save keys, anything left out or unsure.
