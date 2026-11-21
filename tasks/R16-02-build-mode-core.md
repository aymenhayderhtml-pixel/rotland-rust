# R16-02: player-built base, core (pure logic + data)

Read first: `tasks/R16-common.md`, `game/src/lib.rs`, `world/src/poi.rs` (`PoiGeometry` footprint helpers, reuse ideas), `data/items.json` (material ids only).
Files: game/src/building/*, game/src/lib.rs, game/tests/all/building_tests.rs, game/tests/all/main.rs, data/build_parts.json, tasks/R16-02-build-mode-core.md
Test: building

The player places base parts anywhere (owner question about "only near camp" comes on day 2; make the rule a config flag `onlyNearCamp`, default false, with `campRadius`).
- Grid: 2 m cells (x/z), 90-degree turns, heights in 3 m floors (max 2 floors).
- Parts in `data/build_parts.json`: `floor` (2x2), `wall`, `wall_window`, `wall_door` (door opens/closes), `half_wall`, `roof` (flat 2x2), `stairs`, `bed` (sets respawn later), `storage_box` (holds items later), `workbench`, `light` (lantern), `spikes` (hurts zombies), `barricade`. Each: footprint cells, which edge/cell it snaps to (cell parts vs edge parts: walls sit on cell edges), height, cost (existing item ids), HP, `blocksZombies`.
- `BuildPlan` placement check: `check(&state, &cfg, part, cell, floor, rot, ground: impl Fn(f32,f32)->f32, blocked: impl Fn(Vec2, Vec2)->bool) -> Result<Placement, Invalid>` with reasons: overlapping part, needs floor below (upper floors, roofs), slope too steep (ground height difference across the footprint over `maxSlope`), blocked (inside a POI or city lot, via the closure), too far from camp when `onlyNearCamp`.
- `BaseState` (save struct): placed parts (id, part, cell, floor, rot, hp, door open), `place`, `remove` (refund `refundShare` 0.5), `damage(part_id, amount) -> destroyed`, `repair`, `toggle_door`, `parts_in_radius(p, r)`, and `world_transform(part)` -> (position Vec3, yaw) for drawing and colliders.
- Supports: removing a floor removes or drops what stands on it (keep it simple: parts with no support are destroyed, test it).

Tests in `game/tests/all/building_tests.rs` (names start `building_`): every placement error; wall on an edge between two cells; door toggles; support collapse; refund; save round-trip; transforms for each rotation (yaw uses AGENTS.md rule 7).

## Worker notes

