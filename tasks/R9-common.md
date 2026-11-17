# Rules for every R9 worker task

Everything in `tasks/R8-common.md` (and R7-common, R5-common) applies: write-only (no cargo build/test/clippy), Godot C# files are the reference, real data files with Godot keys, tests in `rust/game/tests/` (or `#[cfg(test)]` in a rotland file), one commit on your branch, never push, never touch main.

Seven lanes run at the same time. **Edit only the files your task lists.** Never edit `rust/rotland/src/main.rs` (Claude wires everything in). If you need a one-line hook in a file you don't own, write it under "Wiring for Claude" in your Worker notes instead.

Extra rules from R8's mistakes (`docs/JESTER.md` #25-#28):
1. When a loop holds `&mut` to an entry of `self.something`, finish every `&self` call first and keep its result in a local.
2. Before claiming how another system behaves ("only the driven car runs physics"), open that loop and quote the line in your notes.
3. Don't build a second copy of state that another system already keeps (search for it first).
4. Write `if let .. && let ..` chains; never nested `if`s that clippy collapses.
5. In your notes, list exactly: new public API, the calls Claude must add to main.rs and when (each frame / on key / on load), and every number your tests check and where it comes from.
