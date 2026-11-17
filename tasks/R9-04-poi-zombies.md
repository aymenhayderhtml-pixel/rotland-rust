# R9-04: Zombies waiting inside buildings

Read `tasks/R9-common.md` first. Reference: `scripts/World/PoiManager.cs`, `PoiInstance.cs`, `PoiSite.cs`, `data/pois.json` (or the data the C# reads). Building models carry `spawn_*` markers (see `rust/pack/examples/list.rs`; pack models have `markers`).

Write `rust/game/src/poi_zombies/` (logic only):
- `PoiZombies` manager: when the player comes within Godot's wake range of a building that is not cleared, it returns spawn requests at that building's spawn markers (world positions computed from the POI position/yaw and the marker), respecting Godot's zombie cap and "wait when the cap would be exceeded"; tracks which zombies belong to which building; a building is cleared when all its zombies are dead (as Godot); sleep/despawn rules when the player leaves, as Godot.
- API: `update(player: Vec3, sites: &[PoiZombieSite {poi_id, building_id, position, yaw, spawn_points: Vec<Vec3>}], alive_zombies: usize) -> Vec<PoiSpawn {poi_id, position, kind}>`, `on_zombie_spawned(poi_id, zombie_id)`, `on_zombie_died(zombie_id) -> Option<cleared poi_id>`, `is_cleared(poi_id)`, `to_save()/from_save()` with Godot's exact keys and save id (find them; WorldState "visited_pois" may be it).
- Clear events also go to bounties if Godot does that (say which call in your notes).
Tests (`rust/game/tests/poi_zombies_tests.rs`): no spawn far away; spawn count = markers near; cap respected; cleared after all die and never respawns; save round trip.

## Worker notes

### Files changed / created
- `rust/game/src/poi_zombies/mod.rs` (new): submodules and public re-exports (`PoiZombies`, `PoiZombieSite`, `PoiSpawn`, save helpers and default constants).
- `rust/game/src/poi_zombies/types.rs` (new): `PoiZombieSite` with marker rotation helper `world_spawn_point`, and `PoiSpawn`.
- `rust/game/src/poi_zombies/save.rs` (new): Godot save/load functions for visited POIs matching `WorldState.cs`.
- `rust/game/src/poi_zombies/manager.rs` (new): `PoiZombies` manager implementing wake approach checks, zombie cap wait logic, yaw marker transforms, zombie tracking, death/despawn counting, clear detection, and save persistence.
- `rust/game/src/lib.rs`: added `pub mod poi_zombies;` and `pub use poi_zombies::*;`.
- `rust/game/tests/poi_zombies_tests.rs` (new): 7 tests covering no spawn far away, spawn count = markers near (including yaw rotation), cap respected, cleared after all die and never respawns, save round-trip, sleep/despawn rules on leave, and bounty integration.
- `tasks/R9-04-poi-zombies.md`: task report and worker notes.

### Public API
- `game::poi_zombies::PoiZombies`:
  - `pub fn new(world_seed: i32) -> Self`: loads config from `data/poi.json` and `data/spawning.json` (fallback embedded so never starts empty).
  - `pub fn with_config(world_seed: i32, config: PoiConfigData) -> Self`: custom config constructor.
  - `pub fn update(&mut self, player: Vec3, sites: &[PoiZombieSite], alive_zombies: usize) -> Vec<PoiSpawn>`: wake check within approach distance (35 m); respects active cap (16); returns spawn requests with world positions.
  - `pub fn on_zombie_spawned(&mut self, poi_id: &str, zombie_id: u64)`: maps zombie to building and tracks alive count.
  - `pub fn on_zombie_died(&mut self, zombie_id: u64) -> Option<String>`: decrements alive count; returns `Some(cleared_poi_id)` if all building zombies are dead.
  - `pub fn on_zombie_despawned(&mut self, zombie_id: u64) -> Option<String>`: despawn rules matching Godot (`NoteHomeGone` -> `NoteLeft`).
  - `pub fn check_despawns(&self, player: Vec3, zombie_positions: &[(u64, Vec3)]) -> Vec<u64>`: returns IDs of tracked zombies farther than `despawn_dist` (120 m).
  - `pub fn is_cleared(&self, poi_id: &str) -> bool`: true if building has woken and 0 zombies remain.
  - `pub fn is_visited(&self, poi_id: &str) -> bool`: true if building has ever woken or was loaded from save.
  - `pub fn visited_ids(&self) -> Vec<String>`: list of visited POI ids.
  - `pub fn remaining_zombies(&self, poi_id: &str) -> usize`: active + pending zombies for a building.
  - `pub fn awake_count(&self) -> usize`: total tracked awake zombies.
  - `pub fn mark_woken_for_test(&mut self, poi_id: &str, zombies: usize)`: test hook matching Godot `MarkWokenForTest`.
  - `pub fn to_save(&self) -> serde_json::Value`: returns `{"visited_pois": [...]}`.
  - `pub fn from_save(&mut self, data: &serde_json::Value)`: loads `visited_pois`.
- `game::poi_zombies::PoiZombieSite`:
  - `{ poi_id: String, building_id: String, position: Vec3, yaw: f32, spawn_points: Vec<Vec3> }`
  - `pub fn new(...) -> Self`
  - `pub fn world_spawn_point(&self, index: usize) -> Option<Vec3>`
- `game::poi_zombies::PoiSpawn`:
  - `{ poi_id: String, position: Vec3, kind: String }`
  - `pub fn new(...) -> Self`
- Save identifiers:
  - `pub const POI_ZOMBIES_SAVE_ID: &str = "worldstate";`
  - `pub const POI_ZOMBIES_SAVE_KEY: &str = "visited_pois";`

### Godot Save Keys and Save ID
- **Save ID**: `"worldstate"` (`scripts/World/WorldState.cs` line 21: `public string SaveId => "worldstate"`).
- **Save Key**: `"visited_pois"` (`scripts/World/WorldState.cs` line 123: `["visited_pois"] = _pois.VisitedIds().Cast<object>().ToList()`, line 194: `_pois.LoadVisited(SaveData.GetList(data, "visited_pois") ?? new List<object>())`).
- **Data format**: JSON array of string IDs `["cabin_1", "barn_2"]`. Loaded visited buildings count as cleared because building zombies are not saved (`PoiManager.cs` lines 22, 41-44).

### Bounty Integration Call
- In Godot `BountyManager.cs` lines 68-76 and 274-277: `bounty.Type == BountyConfig.ClearPoi` queries `pois.IsCleared(bounty.PoiId)`.
- In Rust `rust/game/src/bounties/progress.rs` line 94:
  `bounty_manager.on_poi_cleared(poi_id: &str) -> usize`
  When `poi_zombies.on_zombie_died(zombie_id)` or `poi_zombies.on_zombie_despawned(zombie_id)` returns `Some(cleared_poi_id)`, Claude/orchestrator calls:
  `bounty_manager.on_poi_cleared(&cleared_poi_id);`

### Traced Test Numbers and Sources
1. `35.0 m`: wake approach distance from `data/poi.json` (`"approachDistance": 35`) and `PoiConfigData.cs` line 15 (`public float ApproachDistance { get; set; } = 35f;`).
2. `141.42 m`: test player at `(0, 0, 0)` and building at `(100, 0, 100)`: horizontal distance `sqrt(100^2 + 100^2) = 141.42 m > 35.0 m`, asserts 0 spawns.
3. `1 zombie`: watchtower building zombie count from `data/poi.json` line 11 (`"watchtower": { "zombies": [1, 1] }`), asserts `spawns.len() == 1`.
4. `(12.4, 5.0, 20.0)`: site position `(10.0, 5.0, 20.0)` + marker 0 `(2.4, 0.0, 0.0)` from `tools/blender/buildings/watchtower.py:55` (`k.empty("spawn_1", (2.4, 0.0, 0.0))`), asserts spawn position matches.
5. `(10.0, 0.0, 8.0)`: site position `(10.0, 0.0, 10.0)` with yaw `FRAC_PI_2` (90 deg) rotating local marker `(2.0, 0.0, 0.0)` by +Y into `(0.0, 0.0, -2.0)`, resulting in `(10.0, 0.0, 8.0)`.
6. `16 zombies`: building zombie cap from `data/poi.json` line 5 (`"maxActiveZombies": 16`) and `PoiConfigData.cs` line 18 (`public int MaxActiveZombies { get; set; } = 16;`). Tested at 16 active zombies (building waits, remains unvisited) and at 15 active zombies (1 zombie wakes, cap reaches 16, second building waits).
7. `120.0 m`: despawn distance from `data/spawning.json` line 5 (`"despawnDist": 120`) and `PoiManager.cs:93` (`float far = SpawnConfig.Get().DespawnDist;`). Tested with player 150 m away, asserting despawn and building clear.

### Exact Wiring Steps for Claude / Orchestrator
1. **Creation / Initialization**:
   In `main.rs` (or `play.rs`), initialize `PoiZombies`:
   ```rust
   let mut poi_zombies = game::poi_zombies::PoiZombies::new(world_seed);
   ```
2. **POI Sites Collection**:
   When chunks are loaded or streaming, convert loaded POIs into `PoiZombieSite`:
   For each building, extract `spawn_*` marker translations from model pack markers (`m.markers.iter().filter(|k| k.name.starts_with("spawn_")).map(|k| Vec3::from(k.translation)).collect()`).
   ```rust
   let site = PoiZombieSite::new(&poi.id, &poi.building_id, poi.position, poi.yaw, spawn_markers);
   ```
3. **Each Frame (Physics / Gameplay tick)**:
   ```rust
   let spawns = poi_zombies.update(player.position, &poi_sites, zombies.list.len());
   for spawn in spawns {
       let zombie_id = zombies.spawn_at(&spawn.kind, spawn.position);
       poi_zombies.on_zombie_spawned(&spawn.poi_id, zombie_id);
   }

   // Despawn building zombies that fell behind:
   let live_positions: Vec<(u64, Vec3)> = zombies.list.iter().map(|z| (z.id, z.position)).collect();
   for despawn_id in poi_zombies.check_despawns(player.position, &live_positions) {
       zombies.despawn(despawn_id);
       if let Some(cleared_poi) = poi_zombies.on_zombie_despawned(despawn_id) {
           bounties.on_poi_cleared(&cleared_poi);
       }
   }
   ```
4. **On Zombie Death**:
   When a zombie dies in combat or hits:
   ```rust
   if let Some(cleared_poi) = poi_zombies.on_zombie_died(zombie_id) {
       bounties.on_poi_cleared(&cleared_poi);
   }
   ```
5. **Save / Load**:
   - On Save: merge `poi_zombies.to_save()` into the `"worldstate"` section of save JSON.
   - On Load: pass the `"worldstate"` section (or `worldstate["visited_pois"]`) to `poi_zombies.from_save(&val)`.
