# R16-02: player-built base, core (pure logic + data)

Read first: `tasks/R16-common.md`, `game/src/lib.rs`, `world/src/poi.rs` (`PoiGeometry` footprint helpers, reuse ideas), `data/items.json` (material ids only).
Files: game/src/building/*, game/src/lib.rs, game/tests/all/building_tests.rs, game/tests/all/main.rs, data/build_parts.json, tasks/R16-02-build-mode-core.md
Test: building

The player places base parts anywhere (owner question about "only near camp" comes on day 2; make the rule a config flag `onlyNearCamp`, default false, with `campRadius`).
- Grid: 2 m cells (x/z), 90-degree turns, heights in 3 m floors (max 2 floors).
- Parts in `data/build_parts.json`: `floor` (2x2), `wall`, `wall_window`, `wall_door` (door opens/closes), `half_wall`, `roof` (flat 2x2), `stairs`, `bed` (sets respawn later), `storage_box` (holds items later), `workbench`, `light` (lantern), `spikes` (hurts zombies), `barricade`. Each: footprint cells, which edge/cell it snaps to (cell parts vs edge parts: walls sit on cell edges), height, cost (existing item ids), HP, `blocksZombies`.
- `BuildPlan` placement check: `check(&state, &cfg, part, cell, floor, rot, ground: impl Fn(f32,f32)->f32, blocked: impl Fn(Vec2, Vec2)->bool) -> Result<Placement, Invalid>` with reasons: overlapping part, needs floor below (upper floors, roofs), slope too steep (ground height difference across the footprint over `maxSlope`), blocked (inside a POI or city lot, via the closure), too far from camp when `onlyNearCamp`.
- `BaseState` (save struct): placed parts (id, part, cell, floor, rot, hp, door open), `place`, `remove` (refund `refundShare` 0.5), `damage(part_id, amount) -> destroyed`, `repair`, `toggle_door`, `parts_in_radius(p, r)`, and `world_transform(part)` -> (position Vec3, yaw) for drawing and colliders.
- Supports: removing a floor removes or drops what stands on it (keep it simple: parts with no support are destroyed, test it).

Tests in `game/tests/all/building_tests.rs` (names start `building_`): every placement error; wall on an edge between two cells; door toggles; support collapse; refund; save round-trip; transforms for each rotation (yaw uses AGENTS.md rule 7).

## Worker notes

### Files changed
- `data/build_parts.json`: Data configuration for player-built base parts (grid 2m, floor height 3m, max 2 floors, max slope 0.8, refund share 0.5, camp radius 60m, and definitions for all 13 parts: `floor`, `wall`, `wall_window`, `wall_door`, `half_wall`, `roof`, `stairs`, `bed`, `storage_box`, `workbench`, `light`, `spikes`, `barricade`).
- `game/src/building/mod.rs`: Module declaration and public re-exports for base building.
- `game/src/building/def.rs`: `BuildConfig`, `BuildPartDef`, `SnapTarget`, and data loading helpers (`load_default`, `load_from_file`, `from_json`).
- `game/src/building/plan.rs`: `BuildPlan::check` placement validation logic, `Placement`, `Invalid` / `PlacementError`, rotation helpers (`rot_to_yaw`, `neighbor_cell`, `canonical_edge`, `CanonicalEdge`).
- `game/src/building/state.rs`: `BaseState` save struct, `PlacedPart`, mutation methods (`place`, `place_part`, `remove`, `remove_simple`, `damage`, `repair`, `repair_amount`, `toggle_door`, `is_door_open`, `parts_in_radius`, `parts_in_radius_ids`, `world_transform`, `world_transform_id`), query helpers (`has_floor_at`, `has_roof_at`, `has_object_at`, `has_edge_part_at`, `blocks_zombies`), and recursive support collapse (`collapse_unsupported`, `is_supported`).
- `game/src/lib.rs`: Registered `pub mod building;` and exported `pub use building::*;`.
- `game/tests/all/main.rs`: Added `mod building_tests;`.
- `game/tests/all/building_tests.rs`: Comprehensive unit test suite (23 tests) covering all placement errors, edge parts & shared edges, door toggling, support collapse cascades, refund calculations, serde save/load round-trip, and Godot coordinate transform parity across rotations.

### How to test
```bash
tools/gate.sh --task tasks/R16-02-build-mode-core.md
```
Or run the building integration tests directly:
```bash
cargo test building
```

### Wiring notes for Claude (Day 2)
1. **Loading Config:**
   ```rust
   use game::building::{BuildConfig, BaseState, BuildPlan, Placement, Invalid};

   let cfg = BuildConfig::load_default()?;
   ```
2. **Validating & Placing a Part (Build Mode Preview / Ghost):**
   ```rust
   let ground_fn = |x, z| terrain.sample_height(x, z);
   let blocked_fn = |min_xz, max_xz| poi_manager.is_blocked_box(min_xz, max_xz);

   match BuildPlan::check(&base_state, &cfg, part_id, cell, floor, rot, ground_fn, blocked_fn) {
       Ok(placement) => {
           // Green ghost preview at placement.position and placement.yaw
           if player_clicked_place {
               // Deduct placement.cost from player inventory
               let part_id = base_state.place(placement);
           }
       }
       Err(invalid) => {
           // Red ghost preview with error message: invalid.to_string()
       }
   }
   ```
3. **Removing a Part:**
   ```rust
   if let Some(refund_map) = base_state.remove(&cfg, part_id) {
       for (item_id, count) in refund_map {
           player_inventory.add_item(&item_id, count);
       }
   }
   ```
4. **Interacting with Doors:**
   ```rust
   let new_state = base_state.toggle_door(part_id); // true = open, false = closed
   ```
5. **Combat / Damage / Repairs:**
   ```rust
   let destroyed = base_state.damage(part_id, damage_amount); // collapses unsupported parts if floor was destroyed
   base_state.repair(&cfg, part_id); // restores to max HP
   ```
6. **World Transforms for Rendering & Physics Colliders:**
   ```rust
   for part in &base_state.parts {
       let (pos, yaw) = base_state.world_transform(part);
       // draw mesh or instance collider at pos with yaw rotation
   }
   ```
7. **Saving & Loading:**
   `BaseState` implements `Serialize` and `Deserialize` with `#[serde(default)]` on all fields. Embed `pub base: BaseState` in the game save struct.

### Known issues
None. All 27 tests pass, clippy is clean with zero warnings, number checks and unused checks pass 100%.

## Gate report
- date: 2026-11-21
- base: main
- files changed (9): data/build_parts.json, game/src/building/def.rs, game/src/building/mod.rs, game/src/building/plan.rs, game/src/building/state.rs, game/src/lib.rs, game/tests/all/building_tests.rs, game/tests/all/main.rs, tasks/R16-02-build-mode-core.md
- tests passed: 27
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
