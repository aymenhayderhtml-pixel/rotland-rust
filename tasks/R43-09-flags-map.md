# R43-09: Flags, map labels, docs — after 01-08

Owns:
- `rotland/src/app/metro_flags_r43.rs` (NEW: the poses; keep `metro_flags.rs` from growing, adding only the enum entries + dispatch lines there), `rotland/src/app/metro_flags_tests.rs` (pose tests)
- `rotland_ui/src/metro_ui/draw_map.rs` (labels only)
- `docs/WORLD.md` (a short R43 section), `tools/README_TOOLS.md` (the new flags)

Read: tasks/R43-common.md (the flag list), the Worker notes of R43-01..08, `rotland/src/app/metro_flags.rs` + `metro_flags_r42.rs` (how poses are made: `MetroPose { pos, yaw, top, y }`, indoor `y`), `rotland_ui/src/metro_ui/draw_map.rs` (the R42 place labels).

## Poses (`--metro=<spot>`; the first climb tower in plan order unless said otherwise)
- **`climb`:** in the lobby, 6 m in front of the open stair door, facing it, so the "ROOF" sign is visible.
- **`climb_stairs`:** on flight A of a mid-height storey (about half the floors), facing up the flight (local +Z).
- **`climb_floor`:** on the first stop floor, 3 m in from the shaft door, facing into the floor.
- **`climb_roof`:** on the roof deck of the tallest NON-landmark climb tower, at the edge, looking out over the city toward the landmarks (THE rooftop view shot).
- **`roof_planks`:** on a roof with a plank link, 4 m back from the plank, facing along it.
- **`fire_escape`:** on the street 14 m from a working fire escape, facing it.
- **`radio`:** on the mast platform, 2 m from the radio console, facing the hut.

Each pose has a test like the R42 ones (it resolves; it stands on the right floor height).

## Map
- The big map shows "Radio Mast" at the tallest landmark and a small "Stairs" label at each other climb tower, in the R42 label style.
- The minimap needs no labels.

## Done when
Flags + labels + docs; the R43-round checks done by the lead. Fill `## Worker notes

### Files changed
- `rotland/src/app/metro_flags_r43.rs` (NEW): `r43_pose` for `climb`, `climb_stairs`, `climb_floor`, `climb_roof`, `roof_planks`, `fire_escape`, `radio` with exact floor/deck/platform `y` heights and Godot-convention `yaw` (`yaw_facing`).
- `rotland/src/app/metro_flags.rs`: added the 7 `MetroSpot` variants, `parse` arms, `use super::metro_flags_r43::r43_pose;`, and `metro_pose` dispatch arms.
- `rotland/src/app/metro_flags_tests.rs`: added parsing assertions for all 7 R43 flags and `r43_poses_stand_at_right_height_and_face_targets` verifying exact `y` and target-facing `yaw` on a `MetroWorld` fixture.
- `rotland_ui/src/metro_ui/draw_map.rs`: added `metro: Option<&MetroWorld>` parameter to `draw_metro_map` and drew `"Radio Mast"` at the tallest landmark climb tower and `"Stairs"` at each other climb tower using `draw_small_label`.
- `docs/WORLD.md`: added the `### R43: Climbing, Rooftops, Fire Escapes & Radio` section.
- `tools/README_TOOLS.md`: documented `--metro=climb|climb_stairs|climb_floor|climb_roof|roof_planks|fire_escape|radio`.

### Exact shared-file lines for the lead
1. **`rotland/src/app/mod.rs`** (next to `mod metro_flags_r42;`):
```rust
mod metro_flags_r43;
```
2. **`rotland_ui/src/map_ui/draw.rs`** (~line 144, inside `if let Some(layout) = view.generator.and_then(|g| g.metro_layout())`):
```rust
            draw_metro_map(
                &painter,
                state,
                &view.map_state.explore,
                layout,
                view.generator.and_then(|g| g.metro_world()),
                &state.metro_cfg,
                rect,
                view_size,
            );
```

### How to test
- `cargo test metro_flags` (runs `spot_names_parse_any_case_and_reject_unknown` and `r43_poses_stand_at_right_height_and_face_targets`).
- `--metro=climb`, `--metro=climb_stairs`, `--metro=climb_floor`, `--metro=climb_roof`, `--metro=roof_planks`, `--metro=fire_escape`, `--metro=radio` with `--shot=... --save-dir=/tmp/r43-saves`.
- `--map` (after discovering New Halden) to verify `"Radio Mast"` and `"Stairs"` labels on the big map.

### Notes
- Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.`.

## Worker notes

### Files changed
- `rotland/src/app/metro_flags_r43.rs` (NEW): `r43_pose` for `climb`, `climb_stairs`, `climb_floor`, `climb_roof`, `roof_planks`, `fire_escape`, `radio` with exact floor/deck/platform `y` heights and Godot-convention `yaw` (`yaw_facing`).
- `rotland/src/app/metro_flags.rs`: added the 7 `MetroSpot` variants, `parse` arms, `use super::metro_flags_r43::r43_pose;`, and `metro_pose` dispatch arms.
- `rotland/src/app/metro_flags_tests.rs`: added parsing assertions for all 7 R43 flags and `r43_poses_stand_at_right_height_and_face_targets` verifying exact `y` and target-facing `yaw` on a `MetroWorld` fixture.
- `rotland_ui/src/metro_ui/draw_map.rs`: added `metro: Option<&MetroWorld>` parameter to `draw_metro_map` and drew `"Radio Mast"` at the tallest landmark climb tower and `"Stairs"` at each other climb tower using `draw_small_label`.
- `docs/WORLD.md`: added the `### R43: Climbing, Rooftops, Fire Escapes & Radio` section.
- `tools/README_TOOLS.md`: documented `--metro=climb|climb_stairs|climb_floor|climb_roof|roof_planks|fire_escape|radio`.

### Exact shared-file lines for the lead
1. **`rotland/src/app/mod.rs`** (next to `mod metro_flags_r42;`):
```rust
mod metro_flags_r43;
```
2. **`rotland_ui/src/map_ui/draw.rs`** (~line 144, inside `if let Some(layout) = view.generator.and_then(|g| g.metro_layout())`):
```rust
            draw_metro_map(
                &painter,
                state,
                &view.map_state.explore,
                layout,
                view.generator.and_then(|g| g.metro_world()),
                &state.metro_cfg,
                rect,
                view_size,
            );
```

### How to test
- `cargo test metro_flags` (runs `spot_names_parse_any_case_and_reject_unknown` and `r43_poses_stand_at_right_height_and_face_targets`).
- `--metro=climb`, `--metro=climb_stairs`, `--metro=climb_floor`, `--metro=climb_roof`, `--metro=roof_planks`, `--metro=fire_escape`, `--metro=radio` with `--shot=... --save-dir=/tmp/r43-saves`.
- `--map` (after discovering New Halden) to verify `"Radio Mast"` and `"Stairs"` labels on the big map.

### Notes
- Per subagent rules, did not run `cargo`, `tools/gate.sh`, `blender`, or the game.
