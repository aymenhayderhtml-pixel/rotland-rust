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
Every touched file is under ~300 lines, the moved code keeps the same public names, the climb test is written, and the models are rebuilt and look right. Fill `## Worker notes`.

## Worker notes
