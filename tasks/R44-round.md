# R44 round: New Halden rooftop life. Lead file for the gate

Files: world/*, game_zombie/*, game_camp/*, game_world/*, game/*, rotland/*, rotland_ui/*, data/*, tools/blender/*, tools/README_TOOLS.md, assets/models/*, assets/models.pack, docs/WORLD.md, game/tests/all/*, tasks/R44-*
Rules: `tasks/LEAD.md` + `tasks/R44-common.md` (what is true today, premise checks, owner decisions, "Shared names": follow them EXACTLY).

## Phase 0 (lead)
- Bake the current models once in the worktree: `cargo run --release -p baker` (under the flock). `assets/models.pack` is not in git.
- FPS before: `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r44/perf_before.txt` (metro ONLY).
- Shots before (hour 10): `--metro=top`, `--metro=core` -> `/tmp/r44/before_*.png`.

## Tasks (6)
| Task | What | Run |
|---|---|---|
| `R44-01-safehouse-logic.md` | pure safe-house state, travel rule, chasing count, `data/safe_house.json`, tests | 1st wave, at once |
| `R44-02-safehouse-models.md` | Blender: shack, bed, chest, travel sign, rope segment, tent, green smoke barrel + plume, trader stall; `build_r44_models.py` ends "R44 BUILD OK" | 1st wave, at once |
| `R44-03-safehouse-placement.md` | `world/src/metro/safehouse.rs`: safe house tower, rooftop camp tower, spots, chunk props, rooftop NPCs, 3 floor rescue seeds, `safe_house_layout()` loader, tests | 1st wave, at once |
| `R44-04-trader-rescue-data.md` | `data/rooftop_trader.json`, rooftop stock, `floor_survivors` (rescue.json), floor rescue spots, random rescue pass skips climb floors, tests | 1st wave, at once |
| `R44-05-wiring.md` | play glue: rope ladder runtime rail, claim / bed / chest / ladder / signs, fast travel, rooftop shop, rescues, save key, stats keys | 2nd wave, after 01-04 merged |
| `R44-06-flags-map-docs.md` | flags `safehouse`, `safehouse_claimed`, `rooftop_camp`, `rooftop_trader`, `rescue_floor`; map labels "Safe House" and "Rooftop Camp"; docs | 3rd wave, after 05 merged |

Start 01-04 at once (4 subagents). Each one writes its `mod` / `pub use` lines into Worker notes; the lead adds them to shared files (see `R44-common.md`, "Lines for the lead"). The gate of 01-04 runs in each worktree, so a file that uses another task's new name must wait for that task's merge: 05 and 06 do.
Start 05 when 01-04 are merged and the lead has run one build. Start 06 when 05 is merged.

## After each merge (lead)
- Read each task's Worker notes. Check the premises in `R44-common.md` still hold (there is no camp bed, no chase flag, no home-camp sign).
- Run ONE `cargo build`, `cargo test`, `cargo clippy --all-targets` under the flock after the last wave, not after each task.
- `flock /tmp/rotland-heavy.lock` for every game run below. Build the owner binary: `cargo build --profile play`.

## Checks after the gate (`target/play/rotland`, `--save-dir=/tmp/r44-saves`, pictures to `/tmp/r44/`, hour 10)
- **Each pose** (`--metro=safehouse`, `safehouse_claimed`, `rooftop_camp`, `rooftop_trader`, `rescue_floor`): look at every picture.
  - The shack reads as a tarp shack with a bed, a chest and a sign, on a roof.
  - `safehouse_claimed`: the rope ladder hangs down the BACK face to the street; it does not run over the fire escape.
  - `rooftop_camp`: two tents, the green smoke barrel with a green plume, the trader stall, survivors standing or sitting.
  - `rooftop_trader`: the rooftop Shop screen lists the city stock.
  - `rescue_floor`: a survivor on a stop floor.
- **Claim, then reload (proves saving):**
  - Run `--metro=safehouse --hold-e --frames=300 --save-at=3`: the shack claim prompt shows, `safeHouseClaimed` is true.
  - Then `--load=1` (same save dir): `safeHouseClaimed` true and `safeHouseLadderDown` true in the stats.
  - Unclaimed old saves still load (no `metro_safehouse` key = unclaimed).
- **Fast travel** (from the claimed save):
  - At the shack sign, `--hold-e`: the stats `playerPosition` moves to the camp (away from the roof), and `gameHour` rises by `travelHours` (3.0) from the start.
  - At the home camp sign (or the camp point Worker notes named), `--hold-e`: the player is on the safe house roof, `gameHour` rises by 3.0 again.
  - Blocked: with a zombie chasing (`--spawn-zombie=<kind>`, a kind listed in `tools/README_TOOLS.md`), travel shows "Zombies are chasing you" and `gameHour` does NOT move.
- **Ladder pull-up** (from `safehouse_claimed`): `--hold-e` at the roof edge: `safeHouseLadderDown` becomes false; the ladder rail is gone from the stats (`ladders` count drops by one). A second `--hold-e` brings it back down.
- **Bed:** `--hold-e` at the bed: `gameHour` goes to `restHour` (7.0), a save is written (`--save-at`), toast "Saved. Morning."
- **Trader:** `rooftop_trader` shot + one `--hold-e` at the trader: the Shop screen opens with the rooftop stock (picture); buy one `bandage`; the wallet goes down.
- **Rescue:** `rescue_floor` + `--hold-e --frames=200`: `rescuedCount` goes from 0 to 1 and the toast says "<name> joined your team". Then the camp (`--at` the camp) shows the new team member.
- **Random rescues stay off the climb floors:** `--metro=rescue_floor` with `--shot-frame=900`: no two survivors on one climb floor (the rescue seeds are the only climb-floor spots).
- **Floor zombies unchanged:** the R43 checks still pass (`--metro=climb_floor --shot-frame=900`).
- **The map** (`--map`): "Safe House" and "Rooftop Camp" labels show on the right towers.
- Nothing z-fights (signs, tent doors, the ladder rungs); no prop floats or sinks on the roof (`stats.placedModels` sane); the ground stays green.
- **Tests:** `cargo test` passes; the count is more than R43's 1527.
- **FPS after:** `tools/perf_scenes.sh --set metro --seconds 20 --bin target/play/rotland` -> `/tmp/r44/perf_after.txt`. Pass = **average at least 20**. The 1% low is for information. If the average is under 20, lower the rooftop prop counts first (the NPC count is data); never strip the city.

## Lead notes
Built `target/play/rotland` (R44-06 worktree, `--save-dir=/tmp/r44-saves-r06`). Pictures in `/tmp/r44/`, stats in `/tmp/r44/*.json`.
- **Each pose:** PASS, with notes. `safehouse` (`/tmp/r44/safehouse.png`): tarp shack on the roof, bed and sign visible. `safehouse_claimed` (`/tmp/r44/safehouse_claimed.png`, side view not separately checked): roof edge with the camera looking down the back face; the rope ladder is NOT clearly visible in the picture, so the back-face rope is UNCONFIRMED. `rooftop_camp` (`/tmp/r44/rooftop_camp.png`): two tents, green barrel, stall, survivors (one seated, one standing) on clips. Green plume not clearly seen. `rooftop_trader` (`/tmp/r44/rooftop_trader.png`): Shop screen open, lists city stock (Bandage, Water, Beans, Pistol Ammo, Antibiotics x2 etc.). `rescue_floor` (`/tmp/r44/rescue_floor.png`): survivor "Rescue Mark Vance" on the stop floor. Done, PASS.
- **Claim, then reload:** PASS. `--metro=safehouse --hold-e --save-at=3` (`/tmp/r44/claim.json`): `safeHouseClaimed` true, (the claim toast is not in the picture). `--load=1` (`/tmp/r44/load1.json`): claimed true, `safeHouseLadderDown` true.
- **Fast travel:** NOT PROVEN. Roof to camp via shack sign: no "Travel to camp" prompt at the sign even after the reach fix (`/tmp/r44/travel_prompt.png`, `travel_camp*.json`: position and gameHour unchanged). Camp to roof: `--load=1 --at=` ignores `--at` (the save restores the roof), so `travel_roof.json` is not a valid test. Blocked travel: not run (no flag makes a zombie chase).
- **Ladder pull:** NOT PROVEN. With `--hold-e` the press repeats every frame, so the rope toggles every frame (`ladder_pull.json`: down true, rails 4 to 2, unreliable). Needs one real press; the toggle logic is in game tests.
- **Bed:** PASS. `--load=1 --metro=safehouse_bed --hold-e` (`/tmp/r44/bed.json`): gameHour 7.0 (07:00 shown), toast "Saved. Morning." (`/tmp/r44/bed.png`), claim kept.
- **Trader:** shop opens with rooftop stock: PASS for the picture. Buying a bandage needs a click (not scriptable): NOT RUN, wallet not checked.
- **Rescue:** PASS. `--metro=rescue_floor --hold-e` (`/tmp/r44/rescue.json`): rescuedCount 0 to 1, Mark Vance on the team card (`/tmp/r44/rescue_done.png`). Camp team check not run.
- **Random rescues off the climb floors:** NOT RUN as a picture (900-frame run). Covered by R44-04 tests (`rescue_floor` tests).
- **Floor zombies unchanged:** PASS (`--metro=climb_floor --shot-frame=900`, `/tmp/r44/climb_floor.png`: office floor, zombies, normal).
- **Map:** PASS. "Safe House" and "Rooftop Camp" show (`/tmp/r44/map.png`). The Rooftop Camp label overlaps a "Stairs" label nearby.
- **Nothing floats, z-fight, ground green:** looked at in each picture above; no problem seen.
- **Tests:** `cargo test` (gate) 1577 passed (R43 was 1527). PASS.
- **FPS (metro only, `tools/perf_scenes.sh --set metro --seconds 20`, `/tmp/r44/perf_after.txt`):** average 56.0 FPS, 1% low 30.3. PASS (average at least 20).
- **Changes the lead must know:** `data/safe_house.json` shack sign moved to `[-2.6, -3.6]`; a sixth flag `safehouse_bed`; rooftop people are now animated camp figures (`camp_npcs.rs`), not props.
- **Open for the next round:** (1) the shack-sign prompt does not show from the travel point; check the reach/collider in game with a real key press; (2) hold-E repeats actions each frame, so the ladder and the bed cannot be proven by flags; a one-press flag would fix the proof; (3) a camp-to-roof travel test needs `--at` to work after `--load`.

## Gate report
(filled by the gate)
