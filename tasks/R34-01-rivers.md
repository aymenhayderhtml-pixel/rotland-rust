# R34-01: long rivers

Owns: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json
Files: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json
Read only: world/src/rivers/path.rs, world/src/rivers/mod.rs, data/rivers.json

Problem: on the world map the 3 rivers are short, curly loops (~300-600 m) instead of rivers several km long. `trace_river` follows downhill and gets stuck circling in dips.

Fix:
1. The river must make progress toward `target_pos` every step: blend the downhill choice with the target direction (raise `target_pull` in data, e.g. 0.6) and never let the heading turn more than `max_turn_deg` (new data key, e.g. 12) per step.
2. Stop when within `step_m * 2` of the target, or when it would enter `safe_radius` (then slide along the circle, as today).
3. Water level must never go UP along the river: keep `water_levels[i] = min(previous, local)`; carve the bed so it is always below the water level.
4. Tests (in mod.rs tests or world/tests): each river with seed 42 is at least 2500 m long; the end point is within 100 m of its target; the path never self-crosses (check segment pairs, skip neighbours); water levels never rise.

## Worker notes
- Changed:
  - `data/rivers.json`: raised `target_pull` to `0.6`, added `max_turn_deg: 12.0`, raised `max_length_m` to `9000.0` (straight-line distance for `mire_creek` is ~6702 m, so `6000.0` would stop short of target).
  - `world/src/rivers/mod.rs`: added `#[serde(default = "default_max_turn_deg")] pub max_turn_deg: f32` to `RiversData` (validated `> 0.0` in `from_json`) and added unit test `seed_42_rivers_are_long_reach_target_never_self_cross_and_never_rise`.
  - `world/src/rivers/path.rs`: updated `trace_river` to stop when within `step_m * 2.0` of `target_pos`, blend `pick_downhill_heading` direction with `to_target` using `target_pull`, clamp per-step heading turn to `max_turn_deg` via `wrap_angle_rad`, and keep monotonic `build_water_levels` (`prev_level.min(candidate)`).
- Shared/unowned file line needed by lead:
  - In `world/src/generator_world.rs` inside `default_rivers_data()` (around line 420), add:
    `max_turn_deg: 12.0, // data: rivers.json max_turn_deg`
    (Also `target_pull` and `max_length_m` in `default_rivers_data()` can be updated to `0.6` and `9000.0` to match `data/rivers.json` if desired.)
- How to see it: run `cargo test` (especially `seed_42_rivers_are_long_reach_target_never_self_cross_and_never_rise` and `world/tests/rivers_tests.rs`) and generate `worldmap.png` to inspect the 3 long rivers.
- Unsure / skipped: did not run `cargo` or `tools/gate.sh` per subagent rules.
