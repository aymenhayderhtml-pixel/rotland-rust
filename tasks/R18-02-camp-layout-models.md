# R18-02: where the camp upgrade models stand (pure layout, world crate)

Read first: `tasks/R18-common.md`, `world/src/settlement.rs` (camp site, lot size, safe zone, how the camp model is placed), `tools/blender/buildings/settlement.py` (the camp model's layout: where its fence, towers, tents are, so new parts don't overlap), `game/src/camp_upgrades/types.rs` (`CampEffects`: levels per line), the model ids made today: `camp_wall_wood|metal|concrete`, `camp_gate_wood|metal|concrete` (4 m wall sections, 6 m gate), `camp_tower_wood|metal`, `camp_workshop_1|2|3`, `camp_doctor`, `camp_mechanic`.
Files: world/src/camp_layout/*, world/src/lib.rs, data/camp_layout.json, tasks/R18-02-camp-layout-models.md
Test: camp_layout

A pure function the game will call when the camp's levels change: `camp_upgrade_props(site: &SettlementSite, levels: CampLevels) -> Vec<PropInstance>` (world crate; define `CampLevels { walls, guards, workshop, traders: u8 }` here so world doesn't depend on game).
- Walls level 1-3: a closed ring of wall sections around the camp lot at `wallRadius`/half-size from `data/camp_layout.json` (square ring following the camp's yaw), sections every 4 m, one gate gap facing the camp's existing entrance (find it in the settlement model / site yaw) filled by the matching gate model. Level 0 = nothing (today's fence stays).
- Guards level 1: 1 wood tower at a corner; 2: same tower; 3: 2 metal towers at opposite corners (replace wood). Tower positions in the JSON as corner indexes.
- Workshop 1-3: the matching `camp_workshop_N` at a fixed spot inside the camp (JSON offset in camp-local coords).
- Traders 2+: `camp_doctor`; 3+: `camp_mechanic`, fixed spots (JSON).
- All spots must be inside the wall ring, not overlap each other or the existing camp buildings (offsets chosen by reading settlement.py; add a test with footprints).
Tests (`camp_layout_` names): level 0 empty; wall ring closed except the gate gap; section count matches the perimeter; towers at the right corners per level; nothing overlaps; yaw rotates everything.

## Worker notes
- **Files created / changed**:
  - `data/camp_layout.json`: Configuration for wall radius, gate width, tower corners, and offsets for workshop, doctor, and mechanic.
  - `world/src/camp_layout/types.rs`: `CampLevels`, `TowerCorner`, `CampLayoutConfigData` loader and data structures with source tags.
  - `world/src/camp_layout/walls.rs`: Placement of 4 m wall sections and 6 m/8 m gate along the perimeter.
  - `world/src/camp_layout/buildings.rs`: Placement of guard towers, workshop (tiers 1-3), doctor, and mechanic.
  - `world/src/camp_layout/mod.rs`: `camp_upgrade_props` and `camp_upgrade_props_with_config` entry points.
  - `world/src/camp_layout/tests.rs`: Tests covering level 0 empty, wall ring continuity, perimeter section count, tower corner placement, 2D footprint non-overlap vs all settlement fixtures, and yaw rotation.
  - `world/src/lib.rs`: Module declaration and public re-exports.
- **How to test**:
  - Run `tools/gate.sh --task tasks/R18-02-camp-layout-models.md`
  - All 9 unit tests pass under `camp_layout` filter.
- **Wiring notes for Claude**:
  - Game code can call `world::camp_upgrade_props(&site, CampLevels { walls, guards, workshop, traders })` to obtain `Vec<PropInstance>` for streaming / scene rendering.
  - When `levels.walls == 0`, no replacement wall sections are returned, leaving the baseline fence.
  - All positions are derived via `site.center + site.local(x, 0.0, z)` and `site.yaw + local_yaw`.
- **Known issues**: None.

## Gate report
- date: 2026-11-21
- base: main
- files changed (8): data/camp_layout.json, tasks/R18-02-camp-layout-models.md, world/src/camp_layout/buildings.rs, world/src/camp_layout/mod.rs, world/src/camp_layout/tests.rs, world/src/camp_layout/types.rs, world/src/camp_layout/walls.rs, world/src/lib.rs
- tests passed: 9
- clippy: PASS
- test: PASS
- checks: PASS
- gate: PASS
