# R45-06: Test flags, big-map labels, docs, and the checks to run after the gate

Runs AFTER R45-05 is merged and the lead has run one build. Read first: `tasks/R45-common.md` (whole) and the Worker notes of R45-02..05. Style: `tasks/R44-06-flags-map-docs.md`.

Owns:
- `rotland/src/app/metro_flags_r45.rs` (NEW): the five pose functions and `spot_hour`.
- `rotland/src/app/metro_flags_r45_tests.rs` (NEW, `#[cfg(test)]`; the lead adds the `mod` line in `rotland/src/app/mod.rs` as R44 did).
- `rotland/src/app/metro_flags.rs` (ONLY: 5 new `MetroSpot` variants, their `parse` arms, and one dispatch arm that calls `r45_pose`).
- `rotland/src/app/test_flags/park.rs` (NEW): the start-of-game hour and the stadium horde state for the flags (only if a flag needs it; the hour is set the same way `metro_tp.rs` sets `play.hour`).
- `rotland_ui/src/metro_ui/draw_map.rs` (ONLY: two labels, "Park" and "Stadium", data from `park.json` / `stadium.json` `mapLabel`; the labels are drawn at the park and stadium centres).
- `docs/WORLD.md` (ONLY: a short "R45 park and stadium" section, under 30 lines).
- `tools/README_TOOLS.md` (ONLY: five lines, one per new `--metro=` flag).

Not owned: anything else. Do not change R43 or R44 poses or tests. Do NOT edit `docs/STATUS.md`, `docs/PLAN_1.1.md` or `docs/DECISIONS.md` (the lead does those).

Read:
- `rotland/src/app/metro_flags_r44.rs` (pose function shape, the `MetroPose` return, `TopView` camera use) and `rotland/src/app/metro_flags_r44_tests.rs` (the test helpers and the position-inside-city checks).
- `rotland/src/app/metro_flags.rs` (the `MetroSpot` enum, `parse`, the dispatch).
- `rotland/src/app/test_flags/safehouse.rs` and `metro_tp.rs` (how start state is set; `play.hour`).
- `rotland_ui/src/metro_ui/draw_map.rs` (the label loop, `draw_small_label`, the R44 `RooftopMapLabels` struct: add the two labels the same way, bundled in a struct).
- The R45-02/03 public functions: `park_site`, `park_contains`, `park_pond_world`, `stadium_site`, `stadium_contains`, `default_stadium()` (`gates`, `pitch_half`, `outer_half`, `map_label`), `default_park()` (`map_label`).

## Flags (exact names, all `--metro=<name>`)
| Flag | Where the player stands | Hour |
|---|---|---|
| `park` | Park centre, 10 m north of the centre (site -Z side), facing the pond. Eye height as the R44 pose. | 12 |
| `park_pond` | 3 m from the pond edge (the `reedRingM` ring), facing the pond, low camera. | 12 |
| `stadium` | On the avenue 25 m out from the north gate (`site.gates[0]`), facing the gate. This is also the "street" for the park danger check. | 12 |
| `stadium_in` | On the pitch, 15 m from the centre, facing the relief camp. The horde is inside (all 30 members on the pitch). | 12 |
| `stadium_night` | On the avenue 12 m out from the north gate, facing the gate. The horde is outside (all 30 members). | 23 |

Rules:
- Positions come from the R45-02/03 functions and the data (`park.json`, `stadium.json`). Do not re-derive geometry. If a number is missing, say so in Worker notes.
- `MetroPose` shape as in R44. Camera: a `TopView` only for `park` (so the park reads from above), the normal eye view for the others.
- `spot_hour(spot: MetroSpot) -> f32`: 12.0 for park/park_pond/stadium/stadium_in, 23.0 for stadium_night; the test flag sets `play.hour` to this value at start. Other spots keep their current hour.
- Automated runs are muted and use `--save-dir=<scratch>`.

## Map labels
- `rotland_ui/src/metro_ui/draw_map.rs`: label "Park" at the park centre and "Stadium" at the stadium centre, drawn only when the point is inside the map rect (the existing check). Pass the two texts in as data (a struct), not literals. If `too_many_arguments` appears, bundle the labels in a struct (R44 did so).
- A pure helper `fn r45_label_for(...)` returns the text (or None) for a point; test it (not egui).

## Tests (`metro_flags_r45_tests.rs`)
- Every new flag parses (`MetroSpot::parse("park")` ... `"stadium_night"`); an unknown name still returns `None`.
- `spot_hour`: 12 for the four day flags, 23 for `stadium_night`.
- `park` pose is inside `park_contains`; `park_pond` pose is within 3.0 m (+ tolerance 0.5 m, tagged) of the pond edge (`pond.radiusM` from `park.json`, read via `default_park()` if exposed, else the test reads the file) and outside the pond disc.
- `stadium` and `stadium_night` poses are outside `stadium_contains` and within 30 m of `gates[0]`; `stadium_in` pose is inside `pitch_half`.
- `r45_label_for` returns "Park" for the park centre and "Stadium" for the stadium centre, None for a point far away.

## Numbers
Pose distances (10, 3, 25, 15, 12 m) and the test tolerance: `// new: DECISIONS 2026-11-28`. Eye heights and camera values: as R44 (tag them the same way).

## Docs
- `docs/WORLD.md` "R45 park and stadium": the park (the reserved 3x3 blocks, the calm rule and `parkCalm` numbers, the pond, the trees, the loot), the stadium (the 2x2 group, the relief camp on the pitch, the 30-member horde and its timetable from `stadium_horde.json`: inside 07:00-20:00, leaves 20:00-22:00, outside 22:00-05:00, returns 05:00-07:00; real zombies only within 120 m, despawn past 150 m, counted in the zombie cap). Under 30 lines.
- `tools/README_TOOLS.md`: five lines, one per flag, like the R44 lines.

## Done when (worker part)
The flags parse and pose as above; the labels draw; the docs are short; the gate passes (`tools/gate.sh --task tasks/R45-06-flags-map-docs-checks.md`). Fill `## Worker notes` with:
- Files changed.
- The `mod` lines for the lead (`rotland/src/app/mod.rs`: `mod metro_flags_r45;` and the `#[cfg(test)]` test mod; `metro_flags.rs`; `test_flags/mod.rs` if you added `park.rs`).
- Any pose not on the right surface, and what you did.

## Checks after the gate (run by the LEAD, not by the worker; pictures to `/tmp/r45/`, `target/play/rotland`, `--save-dir=/tmp/r45-saves`, under `flock /tmp/rotland-heavy.lock`)
Build the owner binary once: `cargo build --profile play`. Before the round, the lead records the baseline: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland --out /tmp/r45/perf_before.txt` on main.
1. **Pictures, one per flag.** `--metro=<flag> --hour=<spot hour> --shot=/tmp/r45/<flag>.png`. Look at every picture:
   - `park`: trees and bushes dense, paths half covered, the bandstand, fountain, playground, statue, kiosk visible; no bare grey patches; the ground stays green.
   - `park_pond`: pond with reeds, dark water flat, no floating props.
   - `stadium`: the stands, floodlights, gate; the avenue in front.
   - `stadium_in`: the pitch with the relief camp (tents, buses, crates, medical tent); idle zombies on the pitch (the horde by day).
   - `stadium_night`: zombies at the gate and on the avenue (the horde outside).
   - The map (`--map --metro=core`): "Park" and "Stadium" labels on the right spots, no overlap that hides a label.
2. **Horde inside by day, outside at night (stats, from the flags).**
   - `--metro=stadium_in --hour=12 --frames=120 --stats=/tmp/r45/horde_noon.json`: `stadiumHordeInside` 30, `stadiumHordeOutside` 0.
   - `--metro=stadium_night --hour=23 --frames=120 --stats=/tmp/r45/horde_night.json`: `stadiumHordeInside` 0, `stadiumHordeOutside` 30; `stadiumHordeReal` between 1 and `hardCap`; `zombies` never above `hardCap` (60).
   - Dawn: `--hour=6` gives `stadiumHordeInside` between 0 and 30 (returning).
3. **Park danger lower than the street (stats).** `--metro=park --hour=12 --stats=/tmp/r45/park_stats.json` and `--metro=stadium --hour=12 --stats=/tmp/r45/street_stats.json`: `inPark` true for the park and false for the street; `metroDensity` in the park is lower than on the street (park = 0.4 x the street profile before the other boosts).
4. **FPS (the changed area only).** `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland --out /tmp/r45/perf_after.txt`. Pass = **average FPS at least 20**. Do NOT use or report the 1% low (the owner's rule: it is mostly background spikes). If the average is under 20, lower the park tree and bush counts in `park.json` first (data only); never strip the city.
5. **Tests.** `cargo test` passes (the gate ran it). Count more than R44's 1579.
6. **Nothing floats or sinks; the ground stays green** (look at the pictures).

## Lead notes
(The lead fills this after the checks: each check done or not, the numbers, the picture paths.)
