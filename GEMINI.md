# GEMINI.md — Rules & Lessons for Gemini Workers

Read and follow **`AGENTS.md`** first. It contains all mandatory project rules.
Next, read your assigned task file in `tasks/`.

You are a **worker agent**. You receive one task file (e.g. `tasks/R<N>-0X.md`) and must implement only that task.
- **Do NOT run `cargo build`, `cargo test`, `cargo clippy`, or the game.** (Claude runs a single build/test pass per round).
- In a git worktree, commit your finished work to your branch in one commit (never push, never touch `main`).
- Fill in your report under `## Worker notes` in your task file: files changed, how to test, wiring notes for Claude, and known issues.
- Never edit `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`, or docs files unless your task specifies.

---

## Top Real Past Mistakes to Avoid (from `docs/JESTER.md`)

Before writing code, study these frequent failure patterns logged from previous rounds:

1. **Inventing nonexistent APIs or function signatures:**
   - *Past mistakes:* Calling `Vec3::distance_to` (glam has `distance`), calling a fictitious `BountyDef::new(..)` constructor, calling `health.revive()` without the required `Option<f32>`, or using deprecated egui methods.
   - *Rule:* Never guess an API. Open the target type's file, inspect `pub fn`, and copy the exact signature.

2. **Writing unused helper functions (clippy `dead_code` warnings):**
   - *Past mistakes:* Adding helpers nobody calls (`FpSway::new`, `cancel_swing`, `default_headlight_spots`, `from_settings`).
   - *Rule:* Add only the exact functions and types requested by the task. Helpers needed only by tests must be marked `#[cfg(test)]`.

3. **Inventing numbers or altering Godot behavior:**
   - *Past mistakes:* Tilted car headlights down 5.7 deg "to hit the ground" when Godot's beam was straight ahead; guessed item weights instead of reading `data/items.json`.
   - *Rule:* Port exact numbers and logic from the Godot C# file named in the task or from `data/*.json`. If you think a value should change, propose it in `Worker notes`, never in code.

4. **Renaming shared things or clashing public exports:**
   - *Past mistakes:* Renamed `Lighting.spot` to `spots` without updating callers; exported `roll` and `DEFAULT_ZOOM` which clashed with existing public exports.
   - *Rule:* Grep the crate before renaming any public field/method. Avoid wildcard glob re-exports (`pub use x::*`) that pollute crate namespaces.

5. **Borrow-checker slips with `&mut self`:**
   - *Past mistakes:* Holding `&mut` to an element in `self.guard_states` or a farm plot while calling `&mut self` methods or passing closures borrowing `self`.
   - *Rule:* Complete all read-only `&self` lookups first, store the values in local variables, and only then borrow `&mut`.

6. **Visibility errors across files and integration tests:**
   - *Past mistakes:* Defining private fields accessed from sibling files in the same crate; trying to access non-`pub` fields from integration tests in `tests/`.
   - *Rule:* Multi-file structs need `pub(super)` or `pub(crate)` on shared fields. Integration tests in `tests/` can ONLY access items and fields marked `pub`.

7. **Writing nested `if`s instead of let chains:**
   - *Past mistakes:* Triggering clippy `collapsible_if` warnings across newly added logic.
   - *Rule:* Use idiomatic `if let Some(x) = ... && condition` chains from the start.

8. **Hand-writing math approximations:**
   - *Past mistakes:* Writing `0.7071` by hand in tests or logic, which fails clippy.
   - *Rule:* Use `std::f32::consts::FRAC_1_SQRT_2` or construct the vector and call `.normalize()`.

9. **Deriving traits not supported by all struct fields:**
   - *Past mistakes:* Deriving `PartialEq` or `Debug` on structs containing fields that do not implement them (e.g., closures, unannotated structs).
   - *Rule:* Open the definition of every field's type before adding `#[derive(...)]`.

10. **Coordinate orientation in Godot:**
    - *Past mistakes:* Assuming yaw +90 deg faces east.
    - *Rule:* Godot convention: forward = `(-sin yaw, 0, -cos yaw)`. Facing east (+X) is yaw `-PI/2`.
