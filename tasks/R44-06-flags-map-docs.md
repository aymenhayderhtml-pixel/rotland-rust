# R44-06: Test flags, big-map labels, docs (after R44-05)

Runs AFTER R44-05 is merged. Read first: `tasks/R44-common.md` (whole) and the Worker notes of R44-05. Style: `tasks/R43-09-flags-map.md`.

Owns:
- `rotland/src/app/metro_flags_r44.rs` (NEW): the pose functions for the five new spots.
- `rotland/src/app/metro_flags_r44_tests.rs` (NEW, `#[cfg(test)]`; the lead adds the `mod` line in `metro_flags.rs` as R43 did).
- `rotland/src/app/metro_flags.rs` (ONLY: 5 new `MetroSpot` variants, their `parse` arms, and the dispatch arm that calls `r44_pose`).
- `rotland/src/app/test_flags/safehouse.rs` (NEW): the start-of-game effects of the flags (claim the safe house for `safehouse_claimed`, open the rooftop shop for `rooftop_trader`). Call it from the same place `test_flags/metro_tp.rs` is called (read how it is called).
- `rotland_ui/src/metro_ui/draw_map.rs` (ONLY: the two new labels, next to the `Radio Mast` / `Stairs` labels at lines ~157-170).
- `docs/WORLD.md` (ONLY: a short "R44 rooftop life" section), `tools/README_TOOLS.md` (ONLY: the five new flags, one line each).

Not owned: anything else. Do not change R43 poses or their tests.

Read:
- `rotland/src/app/metro_flags_r43.rs` (pose function shape, the `MetroPose` return, `tower_local_xz`, `FLOOR_IN_DIST_M`, `r43_pose`).
- `rotland/src/app/metro_flags.rs` (the `MetroSpot` enum around line 46, `parse` around lines 117-148, the dispatch around lines 460-480).
- `rotland/src/app/test_flags/metro_tp.rs` (how a test flag changes play state at start: `play.hour = 10.0`).
- `rotland_ui/src/metro_ui/draw_map.rs` lines 150-172 (the label loop with `draw_small_label`).

## Flags (exact names, all `--metro=<name>`)
| Flag | Where the player is | Start state |
|---|---|---|
| `safehouse` | On the safe house roof at `travel_point` (local), facing the shack sign. Hour 10. | Unclaimed |
| `safehouse_claimed` | On the safe house roof, standing at the roof edge by `ladder.deck` (so the ladder prompt shows). Hour 10. | Claimed, ladder down |
| `rooftop_camp` | On the rooftop camp roof, 6 m in front of the trader stall, facing the tents. Hour 10. | (any) |
| `rooftop_trader` | Beside the rooftop trader; the rooftop Shop screen is already open. | Claimed |
| `rescue_floor` | On the stop floor of `rescue_floor_seeds[0]`, 3.0 m in from the seed position toward the door (use the R43 `FLOOR_IN_DIST_M` idea: the pose is the seed position, facing the door). | (any) |

Rules:
- Each flag uses the R44-05 functions: `SafeHousePlay::claim()`, `open_rooftop_shop()`, `safe_house_layout()`, `safe_house_spots`, and `rescue_floor_seeds`. Do not re-derive geometry; if a number is missing, say so in Worker notes.
- The pose uses the same `MetroPose` shape as R43 (position, yaw, eye height).
- Automated runs are muted and use `--save-dir=<scratch>`.

## Map labels
- `rotland_ui/src/metro_ui/draw_map.rs`: label the safe house tower `"Safe House"` (from `safe_house.json` `mapLabel`), and the rooftop camp tower `"Rooftop Camp"` (from `rooftopCamp.mapLabel`). Pass the two labels in as data (a small struct or two `&str` params), not as literals in the drawing code. The function already has many params: if it breaks clippy `too_many_arguments`, add `#[allow(clippy::too_many_arguments)]` with a comment like the one R43 added in `draw_map.rs`, or bundle the labels in a struct.
- Each label is drawn only when its tower is inside the map rect (the existing `rect.contains(p)` check).

## Tests (`metro_flags_r44_tests.rs`)
- Every new flag parses (`MetroSpot::parse("safehouse") == Some(...)`, and the 4 others), and an unknown name still returns `None`.
- Each pose's position is inside the city (use the R43 test helpers in `metro_flags_tests.rs`, read them and copy the pattern).
- `safehouse_claimed` pose is at the roof edge by `ladder.deck` (within 0.5 m flat).
- `rescue_floor` pose is on the floor of `rescue_floor_seeds[0]` (the floor y from the seed).
- The map label is drawn only for the right towers: a pure helper `fn label_for(...)` returns the text for a tower index (test it; do not test egui).

## Docs
- `docs/WORLD.md`: a short "R44 rooftop life" section: the safe house tower (nearest fire-escape tower to the centre), the rope ladder on the back face, the bed = save + clock to `restHour`, the chest = the bank, fast travel rules (free, `travelHours`, blocked by chasing zombies), the rooftop camp (Core climb tower farthest from the safe house), the trader (city stock list), the 3 floor rescues. Keep it under 30 lines.
- `tools/README_TOOLS.md`: 5 lines, one per new `--metro=` flag.
- Do NOT edit `docs/STATUS.md`, `docs/PLAN_1.1.md` or `docs/DECISIONS.md` (the lead does those).

## Number tags
Flags use only the numbers already in the common file (or data). The 0.5 m test tolerance is a test constant: tag it `// new: DECISIONS 2026-11-28 (test tolerance)`.

## Done when
The flags parse and pose as above; the labels draw; the docs are short; the gate passes (`tools/gate.sh --task tasks/R44-06-flags-map-docs.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` lines for the lead (`metro_flags.rs`, `test_flags/mod.rs`, `draw_map.rs` if needed).
- Any pose where the spot is not on the right surface, and what you did.

## Worker notes
