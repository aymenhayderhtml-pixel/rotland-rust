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

