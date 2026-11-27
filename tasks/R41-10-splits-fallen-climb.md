# R41-10: R40 leftovers: split long files, fallen tower climbable

Owns:
- `world/src/metro/traces.rs` (+ new split files `traces_*.rs` next to it)
- `world/src/metro/damage.rs` (+ new split files `damage_*.rs` next to it, but NOT `damage_burnt.rs`'s public API)
- `rotland/src/streaming.rs` (+ new split files `streaming_*.rs` next to it)
- `tools/blender/damage/fallen_section.py`, `tools/blender/damage/rubble.py`
- their .glb files and previews (`tw_fallen_section*`, `rubble_field_*`)
- `world/src/metro/damage_tests.rs` (only to add the climb test)

Lines for the lead: new `mod` lines in `world/src/metro/mod.rs` and `rotland/src/main.rs`.
Not owned: anything another R41 task owns. R41-03 and R41-08 READ traces/damage. Keep every `pub` name and signature they use the same (only move code; re-export from the old module so callers don't change).
Read: tasks/R41-common.md, tasks/R40-round.md ("Claude review fixes", leftovers), the three files, `tools/blender/damage/fallen_section.py`, `world/src/metro/damage.rs` (fallen-site props + colliders), `rotland/src/metro_render/hulls.rs` (how set-piece colliders are made).

## 1. Splits (no behaviour change)
- `traces.rs` (434 lines) -> under ~300 per file. For example: `traces.rs` (entry + chunk function), `traces_walls.rs` (car walls), `traces_decals.rs` (graffiti + posters), `traces_bodies.rs`. `traces_camps.rs` already exists.
- `damage.rs` (375) -> `damage.rs` (sites + config) + `damage_props.rs` (chunk props).
- `rotland/src/streaming.rs` (492) -> under ~300 + `streaming_metro.rs` (all metro prop/tower loading).
- Pure move: the same tests must pass unedited (except `use` paths if needed).

## 2. Fallen tower top is climbable
- Today the rubble ramps reach only about 2.4 m, so you can't get onto the fallen section.
- Make a walkable way up:
  - a rubble ramp (`rubble_field_l` or a new `rubble_ramp` model in `rubble.py`) from the street up to the top face of the lowest broken segment, slope at most 35 degrees
  - broken floor slabs as steps from segment to segment
- The collider must follow the walkable surface: use the existing set-piece collider path; slopes as sloped boxes or a trimesh, like R40.
- Budget: the fallen section stays under 16k tris.
- Blender: `flock /tmp/rotland-blender.lock blender -b --python tools/blender/build_r40_models.py` is too wide. Run ONLY your two scripts the way `build_r40_models.py` calls them (write a tiny guard `if __name__ == "__main__"` runner or call the functions from a one-line `--python-expr`). No `Traceback`. Look at the previews.
- Test in `damage_tests.rs`: the chain of walkable collider pieces at the fallen site goes from street height to the top of the highest segment with no step taller than 0.5 m and no slope over 35 degrees (check the data the collider is built from).

## Done when
Every touched file is under ~300 lines, the moved code keeps the same public names, the climb test is written, and the models are rebuilt and look right. Fill `## Worker notes
- **Splits (all files <= ~300 lines, all `pub` names and signatures re-exported from original modules):**
  - `world/src/metro/traces.rs` (162 lines) + `world/src/metro/traces_config.rs` (186 lines) + `world/src/metro/traces_walls.rs` (110 lines).
  - `world/src/metro/damage.rs` (273 lines) + `world/src/metro/damage_props.rs` (149 lines).
  - `rotland/src/streaming.rs` (244 lines) + `rotland/src/streaming_metro.rs` (273 lines).
  - **Shared-file `mod` lines:** None needed in `world/src/metro/mod.rs` or `rotland/src/main.rs` because `#[path = "..."] mod ...;` is used inside `traces.rs`, `damage.rs`, and `streaming.rs` (matching the existing `traces_camps.rs` pattern).
- **Fallen tower climbable:**
  - `tools/blender/damage/rubble.py`: added `rubble_ramp` (1564 tris <= 8000) along `-Y` of `seg0` from street (`x=-38, z=0`) to `seg0` top edge (`x=-12, z=16.6`, slope `32.56 deg <= 35 deg`) with a landing (`x=-12..-9.5, z=16.6`) and convex colliders `ramp_slope` and `ramp_landing`. Also added `if __name__ == "__main__"` guard.
  - `tools/blender/damage/fallen_section.py`: lowered `BUDGET_LOD0` to `16000` (`tw_fallen_section` built at 13264 tris, `tw_fallen_section_lod1` at 68 tris); added full-length `top_deck` colliders on all three segments (covering the 3.5 m broken ends) and `_gap_steps` broken floor slab steps + colliders across both gaps (`gap0_step_0..3` from `18.0 m` to `17.0 m`, `gap1_step_0..4` from `17.0 m` to `18.5 m`, each step `<= 0.30 m`). Also added `if __name__ == "__main__"` guard.
  - `world/src/metro/damage_props.rs`: emits `rubble_ramp` at `fallen.pos` with `fallen.yaw`.
  - `world/src/metro/damage_tests.rs`: added `fallen_site_collider_chain_is_climbable_from_street_to_top`, reading `COL_*` vertices directly from `rubble_ramp.glb` and `tw_fallen_section.glb` to verify street-to-top continuity, step height `<= 0.5 m`, and slope `<= 35 deg`.
- **Not run (per subagent rules):** `cargo` / `tools/gate.sh` / `baker` / game were not run; lead should run `cargo run --release -p baker` (or bake models) and `tools/gate.sh` to verify Rust compilation, clippy, and tests. Previews `rubble_ramp.png`, `tw_fallen_section.png`, and `tw_fallen_section_top.png` were inspected.`.

## Worker notes
- **Splits (all files <= ~300 lines, all `pub` names and signatures re-exported from original modules):**
  - `world/src/metro/traces.rs` (162 lines) + `world/src/metro/traces_config.rs` (186 lines) + `world/src/metro/traces_walls.rs` (110 lines).
  - `world/src/metro/damage.rs` (273 lines) + `world/src/metro/damage_props.rs` (149 lines).
  - `rotland/src/streaming.rs` (244 lines) + `rotland/src/streaming_metro.rs` (273 lines).
  - **Shared-file `mod` lines:** None needed in `world/src/metro/mod.rs` or `rotland/src/main.rs` because `#[path = "..."] mod ...;` is used inside `traces.rs`, `damage.rs`, and `streaming.rs` (matching the existing `traces_camps.rs` pattern).
- **Fallen tower climbable:**
  - `tools/blender/damage/rubble.py`: added `rubble_ramp` (1564 tris <= 8000) along `-Y` of `seg0` from street (`x=-38, z=0`) to `seg0` top edge (`x=-12, z=16.6`, slope `32.56 deg <= 35 deg`) with a landing (`x=-12..-9.5, z=16.6`) and convex colliders `ramp_slope` and `ramp_landing`. Also added `if __name__ == "__main__"` guard.
  - `tools/blender/damage/fallen_section.py`: lowered `BUDGET_LOD0` to `16000` (`tw_fallen_section` built at 13264 tris, `tw_fallen_section_lod1` at 68 tris); added full-length `top_deck` colliders on all three segments (covering the 3.5 m broken ends) and `_gap_steps` broken floor slab steps + colliders across both gaps (`gap0_step_0..3` from `18.0 m` to `17.0 m`, `gap1_step_0..4` from `17.0 m` to `18.5 m`, each step `<= 0.30 m`). Also added `if __name__ == "__main__"` guard.
  - `world/src/metro/damage_props.rs`: emits `rubble_ramp` at `fallen.pos` with `fallen.yaw`.
  - `world/src/metro/damage_tests.rs`: added `fallen_site_collider_chain_is_climbable_from_street_to_top`, reading `COL_*` vertices directly from `rubble_ramp.glb` and `tw_fallen_section.glb` to verify street-to-top continuity, step height `<= 0.5 m`, and slope `<= 35 deg`.
- **Not run (per subagent rules):** `cargo` / `tools/gate.sh` / `baker` / game were not run; lead should run `cargo run --release -p baker` (or bake models) and `tools/gate.sh` to verify Rust compilation, clippy, and tests. Previews `rubble_ramp.png`, `tw_fallen_section.png`, and `tw_fallen_section_top.png` were inspected.
