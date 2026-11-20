# Rules for every R8 worker task

Everything in `tasks/R5-common.md` and `tasks/R7-common.md` applies: write-only (no cargo build/test/clippy), real data files with Godot keys, seeded randomness, private submodules + `pub use`, tests in `rust/game/tests/`, one commit on your branch, never push.

Extra rules from R7's mistakes (`docs/JESTER.md` #19-#24):
1. Enum variants use `::` (`MapMarkerKind::Camp`), never `.`.
2. `gen` is a reserved word in Rust 2024. Don't name anything `gen`.
3. Never write `0.7071` for a unit vector: use `.normalize()` or `std::f32::consts::FRAC_1_SQRT_2`.
4. Check your public names against the whole game crate before `pub use x::*` (R7 clashed on `roll` and `DEFAULT_ZOOM`). Prefix them.
5. Godot yaw: forward = (-sin yaw, 0, -cos yaw). East (+X) is yaw -PI/2.
6. egui is 0.37: open `rust/rotland/src/hub_ui/` for working examples before using any egui call.
7. Fields read from integration tests (`rust/game/tests/`) must be `pub`.
8. Don't edit `rust/rotland/src/main.rs`, `play.rs`, `camp.rs` or other lanes' files. Claude wires your code into the game. In your report, write exactly the wiring Claude needs: which calls, when each frame, which inputs.
