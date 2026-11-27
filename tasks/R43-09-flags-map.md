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
Flags + labels + docs; the R43-round checks done by the lead. Fill `## Worker notes`.

## Worker notes
