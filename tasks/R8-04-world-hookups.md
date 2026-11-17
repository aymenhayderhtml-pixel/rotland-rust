# R8-04: Parked cars solid, stray dog sites, map buildings

Read `tasks/R8-common.md` first. This lane writes small rotland helper files plus game-crate glue.

1. `rust/rotland/src/car_solid.rs`: parked (not driven) cars must block the player and zombies. `ParkedCarSolids::sync(&mut self, physics: &mut PhysicsWorld, cars: &[(String id, Mat4 world, bool driven)])` adds a static box collider per parked car (size from the car model collider: `pack` model `car_hatchback_red` has 1 collider; or from `data/vehicles.json` body size), moves it when the car moved (remove + add), removes it while driven or when the car is gone. Open `rust/physics/src/world.rs` for `add_static(colliders, position, yaw, scale)` and `remove`. The car's own wheel rays must not hit its own box: say in your report how you avoided it (for example, remove while driven, which is the only time it moves).
2. `rust/rotland/src/poi_links.rs`: build the `game::DogPoiSite` list (read `rust/game/src/dogs/types.rs`) from the POIs the world places (`rust/world/src/poi.rs`, and how `rust/rotland/src/streaming.rs` loads them). Godot reference: `scripts/Dogs/` stray spawning and `scripts/World/PoiManager.cs`.
3. Same file: map building notes. Port `scripts/World/MapBuilding.cs` and the building part of `scripts/UI/MapMarkers.cs`: when a POI building is within reveal range, note it in `game::MapState` (read `rust/game/src/map/`; add a method there if it has none, in a new file `rust/game/src/map/buildings.rs`), so the map tab shows it. Save keys as Godot.
Tests: `rust/game/tests/map_buildings_tests.rs` (noted once, kept after save round trip), and `#[cfg(test)]` tests in car_solid.rs with a flat `PhysicsWorld` (a ray hits the parked car box, no hit after `sync` with the car driven).

## Worker notes

### Files changed
- `rust/rotland/src/car_solid.rs` (new): Manages static box colliders for parked (non-driven) cars in `PhysicsWorld`. Adds static collider when parked, removes when driven (wheel rays avoid own box), updates when car moves, removes when car leaves. Includes `#[cfg(test)]` tests with flat `PhysicsWorld`.
- `rust/rotland/src/poi_links.rs` (new): Converts world POIs to `game::DogPoiSite` list for stray dog spawning checks, tracks loaded POIs per chunk, and synchronizes building discoveries into `game::MapState`. Includes `#[cfg(test)]` tests.
- `rust/game/src/map/buildings.rs` (new): `PoiSiteInfo` struct and `MapState::note_nearby_building` / `MapState::note_nearby_poi_sites` methods for distance checking (within 60 m `FOUND_RADIUS`), deduplication, and looted crate tracking.
- `rust/game/src/map/mod.rs` (modified): Registered `mod buildings;` and re-exported `pub use buildings::*;`.
- `rust/game/tests/map_buildings_tests.rs` (new): Integration tests verifying building discovery radius, duplicate prevention, looted crate updates, map marker collection, and save/load round trips.

### Public API
- `rotland::car_solid`:
  - `pub fn default_car_box() -> Vec<[f32; 3]>`: 8 model-space box vertices (half_width 0.8, sill 0.2, roof 1.4, half_length 1.9).
  - `pub struct ParkedCarSolids`:
    - `pub fn new() -> Self`: Uses default car box.
    - `pub fn with_colliders(colliders: Vec<Vec<[f32; 3]>>) -> Self`: Uses custom convex colliders.
    - `pub fn from_pack(pack: &pack::ModelPack) -> Self`: Reads colliders from model `car_hatchback_red` / `car_hatchback_blue`, falling back to `default_car_box()`.
    - `pub fn sync(&mut self, physics: &mut PhysicsWorld, cars: &[(String, Mat4, bool)])`: Adds static colliders for parked cars, removes while driven or gone, moves when transformed.
    - `pub fn count(&self) -> usize`
    - `pub fn contains(&self, id: &str) -> bool`
    - `pub fn clear(&mut self, physics: &mut PhysicsWorld)`
- `rotland::poi_links`:
  - `pub fn build_dog_poi_site(poi: &world::PoiInstance, poi_config: &world::PoiConfigData) -> game::DogPoiSite`
  - `pub fn build_dog_poi_sites(pois: &[world::PoiInstance], poi_config: &world::PoiConfigData) -> Vec<game::DogPoiSite>`
  - `pub struct ActivePoiSite { pub chunk: IVec2, pub poi: PoiInstance, pub half_depth: f32, pub crate_ids: Vec<String> }`
  - `pub struct PoiLinks`:
    - `pub fn new() -> Self`
    - `pub fn on_chunk_loaded(&mut self, chunk: IVec2, pois: &[PoiInstance], poi_config: &PoiConfigData, crates: &[LootSpot])`
    - `pub fn on_chunk_unloaded(&mut self, chunk: IVec2)`
    - `pub fn dog_poi_sites(&self) -> Vec<DogPoiSite>`
    - `pub fn update_map_buildings(&self, map_state: &mut MapState, player_pos: Vec3, is_crate_looted: impl Fn(&str) -> bool)`
    - `pub fn len(&self) -> usize`
    - `pub fn is_empty(&self) -> bool`
    - `pub fn get(&self, poi_id: &str) -> Option<&ActivePoiSite>`
  - `pub fn note_nearby_pois(map_state: &mut MapState, player_pos: Vec3, pois: &[PoiInstance], crates: &[LootSpot], poi_config: &PoiConfigData, is_crate_looted: impl Fn(&str) -> bool)`
- `game::map` (`rust/game/src/map/buildings.rs`):
  - `pub struct PoiSiteInfo { pub poi_id: String, pub building_type: String, pub position: Vec2, pub crate_ids: Vec<String> }`:
    - `pub fn new(poi_id, building_type, position, crate_ids) -> Self`
    - `pub fn is_looted(&self, is_crate_looted: impl Fn(&str) -> bool) -> bool`
  - `MapState` methods:
    - `pub fn note_nearby_building(&mut self, player_xz: Vec2, poi_id: &str, building_type: &str, building_xz: Vec2, looted: bool) -> bool`
    - `pub fn note_nearby_poi_sites(&mut self, player_xz: Vec2, sites: &[PoiSiteInfo], is_crate_looted: impl Fn(&str) -> bool) -> usize`

### Save keys
Follows exact Godot save structure for system id `"map"`:
- `"explored"`: serialized byte array of explored grid cells.
- `"found"`: JSON array of discovered buildings:
  - `"id"`: string (POI building id)
  - `"type"`: string (building def id: "cabin", "gas_station", etc.)
  - `"x"`: f64 (ground world X)
  - `"z"`: f64 (ground world Z)
  - `"looted"`: bool (true when all crates on this lot are searched)
- `"death"`: `{ "x": f32, "z": f32 }` (player death position, if set)
- `"waypoint"`: `{ "x": f32, "z": f32 }` (custom waypoint, if set)

### Traced test numbers
- Car box geometry: half_width = 0.8 m, min_y = 0.2 m (ground clearance), max_y = 1.4 m (roof height), half_length = 1.9 m.
- Car vertical raycast: ray starts at (10, 5, 20), shoots `Vec3::NEG_Y`. Top of car box is y = 1.4 m. Time of impact / distance = 5.0 - 1.4 = 3.6 m. Hit point = (10, 1.4, 20).
- Car horizontal raycast: ray starts at (-5, 0.8, 0), shoots `Vec3::X`. Side of car box is x = -0.8 m. Distance = -0.8 - (-5.0) = 4.2 m. Hit point = (-0.8, 0.8, 0).
- `MapState::FOUND_RADIUS = 60.0 m` (`FOUND_RADIUS^2 = 3600.0`).
- Discovery test 1: Player (0, 0), building (100, 0): dist = 100 m > 60 m -> unrevealed.
- Discovery test 2: Player (50, 0), building (100, 0): dist = 50 m <= 60 m -> revealed.
- Discovery test 3: Player (0, 0), building (30, 40): dist = sqrt(30^2 + 40^2) = 50 m <= 60 m -> revealed.
- Dog lot depth: Lot [10.0, 12.0] -> half_depth = 12.0 / 2.0 = 6.0 m.

### Exact wiring steps for the orchestrator (Claude)
1. In `rust/rotland/src/main.rs`:
   - Declare modules:
     ```rust
     mod car_solid;
     mod poi_links;
     ```
   - In struct `R0`:
     ```rust
     parked_car_solids: car_solid::ParkedCarSolids,
     poi_links: poi_links::PoiLinks,
     ```
   - In `R0` initialization (after pack loaded):
     ```rust
     let parked_car_solids = pack.as_ref().map(car_solid::ParkedCarSolids::from_pack).unwrap_or_default();
     let poi_links = poi_links::PoiLinks::new();
     ```
   - When chunks load / unload in `Streamer`:
     - When chunk `c` unloads: `self.poi_links.on_chunk_unloaded(c);`
     - When chunk `built` uploads: `self.poi_links.on_chunk_loaded(built.coord, &built.pois, world_gen.poi_config(), &built.crates);`
   - Each frame / simulation step:
     - Pass `DogPoiSite` list to dogs context:
       ```rust
       let dog_poi_sites = self.poi_links.dog_poi_sites();
       dog_ctx = dog_ctx.with_pois(world_seed, &dog_poi_sites);
       ```
     - Update map building notes:
       ```rust
       self.poi_links.update_map_buildings(&mut play.map_state, player_pos, |crate_id| play.loot_state.is_looted(crate_id));
       ```
     - Sync parked car solids:
       ```rust
       if let Some(cars) = &self.cars {
           let cars_list: Vec<(String, Mat4, bool)> = cars
               .live_cars()
               .map(|c| (c.id.clone(), c.draw_state().world_matrix, c.is_driven()))
               .collect();
           self.parked_car_solids.sync(&mut self.physics, &cars_list);
       }
       ```
2. Wheel ray avoidance:
   - When a car is driven (`c.is_driven() == true`), `ParkedCarSolids::sync` removes the static collider for that car from `PhysicsWorld`. Therefore, its suspension wheel raycasts (which cast downward from 1 m above hubs) only hit the ground/terrain, never its own collision box.
   - When the driver exits, `c.is_driven()` becomes false, and the next `sync` call adds the static box collider at the car's parked transform, solid against player and zombies.
