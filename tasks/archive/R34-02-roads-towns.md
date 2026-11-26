# R34-02: roads reach the home land, Blackwater walkways

Owns: world/src/roads/build.rs, world/src/towns/lots.rs, data/roads.json, data/towns.json
Files: world/src/roads/build.rs, world/src/towns/lots.rs, data/roads.json, data/towns.json
Read only: these 4 files, plus `world/src/towns/mod.rs` and `world/src/generator_contents.rs` (`keep_scattered_prop`, `append_town_contents`) to see how lots are placed.

1. **Roads:** the radial roads start ~2.6 km from camp, so from the home land they can't be seen. Make each trailhead sit at `core_radius + trailhead_margin` (set `trailhead_margin` to 30 in data) so the road runs through the blend band up to the home land edge. The road must NOT enter `core_radius` (test it). Golden test must still pass: the road paint/carving must not reach inside `core_radius`.
2. **Blackwater boardwalks:** today `append_boardwalk_lots` places many `boardwalk_section` pieces in rows on both sides of the street, at right angles, so they look like benches. Instead: one straight line of sections end-to-end (each 4 m long along its local Z, check the yaw) running parallel to the street, in front of each building's door side, from the street edge to the door. At most ~3 sections per building. Keep them off the street itself.
3. Tests: no trailhead closer than `core_radius + 20` or farther than `core_radius + 60`; every boardwalk lot is parallel to the street (yaw matches `street_dir` within 1°) and outside the street's 4 m half width.

## Worker notes
- **What changed:**
  - `data/roads.json`: set `trailhead_margin` from `60.0` to `30.0`.
  - `world/src/roads/build.rs`: updated `build_trailheads` to place trailheads at `core_radius + trailhead_margin` (removing `+ blend_band`), updated `build_network` so radial highways use `core_radius + CORE_CLEARANCE_BUFFER_M` while ring trails keep `core_radius + blend_band + CORE_CLEARANCE_BUFFER_M`, and added unit test `trailheads_sit_at_core_edge_and_roads_stay_outside_core_radius`.
  - `data/towns.json`: fixed `footprints.boardwalk_section` from `[2.0, 1.0]` to `[1.0, 2.0]` to match the 2 m (local X) x 4 m (local Z) model.
  - `world/src/towns/lots.rs`: updated `append_boardwalk_lots` to place a single straight line of up to 3 end-to-end `boardwalk_section` lots parallel to `street_dir` (`yaw_from_forward(street_dir)`) in front of each building's door side at `boardwalk_offset_m` (5.5 m, keeping all corners >= 4.5 m from street center), and added unit test `blackwater_boardwalks_are_parallel_walkways_outside_street`.
- **Unowned files the lead must update (existing hardcoded expectations):**
  - `world/src/generator_world.rs` (line 404, `default_roads_data`): `trailhead_margin: 60.0` -> `30.0`.
  - `world/tests/roads_tests.rs`:
    - `four_trailheads_sit_at_sector_angles_and_expected_radius` (lines 56-58): expects `core_radius + blend_band + trailhead_margin`; update to `core_radius + trailhead_margin`.
    - `no_road_point_or_segment_enters_core_plus_blend_band` (lines 116-143): highways now enter the blend band down to `core_radius + trailhead_margin` (staying outside `core_radius`), while ring trails stay outside `core_radius + blend_band`.
  - `world/tests/towns_tests.rs`:
    - `no_lot_overlaps_and_lots_face_street` (lines 144-152): checks `lot.forward().dot(to_street) > 0.999` for every lot; skip or check `lot.forward().dot(site.street_dir) > 0.999` when `lot.building == BOARDWALK_MODEL`.
- **How to see it:** Run `worldmap` example to verify radial highways touch the home land circle (`core_radius + 30 m`), and take `--trailhead=0` and `--town=blackwater` shots.
- **Note:** Per subagent rules, `cargo` / `tools/gate.sh` were not run.
