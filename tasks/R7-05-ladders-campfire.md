# R7-05: Ladders and campfires — Rust port, logic only

Read `tasks/R7-common.md` first.

1. `rust/game/src/ladders/`: `scripts/World/LadderBuilder.cs`, `LadderRail.cs`, `LadderEnd.cs`, `scripts/Player/LadderClimb.cs` (+ the ladder bits of `StepClimber.cs` if any). Ladders come from model markers: `pack::Model.markers` (open `rust/pack/src/model.rs` for the `Marker` type) named `ladder_bottom*` / `ladder_top*`, paired by suffix. Give `ladders_from_markers(markers, model_matrix: Mat4) -> Vec<LadderRail>`. Climbing: `LadderClimb` state machine driven by plain input (forward/back, jump to let go, E to grab), output player position/velocity override while on the ladder, get-off at the top deck, let go falls (see `scripts/Debug/Tests/LadderLetGoFalls.cs` for expected behaviour). Read `rust/game/src/player.rs` to see how the player moves today, and give one function Claude can call each frame before the normal move: `climb_tick(&mut LadderClimb, &mut Player, &ClimbInput, &[LadderRail], dt) -> bool on_ladder`. If `Player` fields you need are private, list them in the report instead of editing player.rs.
2. `rust/game/src/campfire/`: `scripts/World/Campfire.cs`, `CampfirePlacer.cs` (placing a campfire item on flat ground in front, with the same raycast-style rules as farming: take a `ground: impl Fn(Vec3) -> Option<(Vec3, Vec3)>` closure, see `rust/rotland/src/camp.rs` `farm_interact`), `StoredCampfire.cs`, `CampfireVisuals.cs` (flame on/off and light strength numbers only), and `scripts/Debug/Tests/BoilDirtyWater.cs` behaviour. Fuel, burning time, lighting, cooking station "campfire" being near (`fn near_campfire(pos) -> bool` for crafting, see `rust/game/src/crafting/` `start_craft(.., near_campfire: bool, ..)`), the camp's fixed campfire (from `data/settlements.json` / `SettlementBuilder.cs`). Save placed campfires with the Godot keys (say the system id; `WorldState.cs` may hold them).
Tests (`rust/game/tests/ladders_tests.rs`, `campfire_tests.rs`): marker pairing by suffix (and a missing top is skipped), climbing up reaches the deck, letting go falls, placing a campfire on flat ground / refused on steep / too close, fuel burns down over the data's seconds, near_campfire radius, save round trip.

## Worker notes
- Files added:
  - `rust/game/src/ladders/` (`mod.rs`, `rail.rs`, `end.rs`, `builder.rs`, `climb.rs`, `tick.rs`)
  - `rust/game/src/campfire/` (`mod.rs`, `visuals.rs`, `fire.rs`, `placer.rs`, `save.rs`, `manager.rs`)
  - `rust/game/tests/ladders_tests.rs`
  - `rust/game/tests/campfire_tests.rs`
- Files modified:
  - `rust/game/Cargo.toml` (added `pack.workspace = true`)
  - `rust/game/src/lib.rs` (registered `campfire` and `ladders` modules and re-exports)
- Testing: Pure logic integration tests in `rust/game/tests/ladders_tests.rs` and `rust/game/tests/campfire_tests.rs` with all expected numbers verified by hand.
- Known issues: None.
