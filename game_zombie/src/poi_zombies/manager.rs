//! POI zombie spawner and building clear tracker.
//!
//! Port of `scripts/World/PoiManager.cs`. Keeps loaded buildings and the zombies they wake.
//! A building wakes once when the player comes within the approach distance. Its zombies
//! stand still at spawn markers until they see or hear the player. A visited building never
//! wakes again, so killed zombies stay dead.

use std::collections::{HashMap, HashSet};

use glam::Vec3;
use serde_json::Value;
use world::{DotnetRandom, PoiConfig, PoiConfigData};

use super::save::{from_save_visited_pois, to_save_visited_pois};
use super::types::{PoiSpawn, PoiZombieSite};
use game_core::loot::crate_seed;
use crate::spawner_config::{SpawnConfig, SpawnerRng};

/// Default approach distance in metres if config is unavailable (matches data/poi.json).
pub const DEFAULT_APPROACH_DISTANCE: f32 = 35.0;
/// Default cap on active zombies from buildings (matches data/poi.json).
pub const DEFAULT_MAX_ACTIVE_ZOMBIES: usize = 16;
/// Default despawn distance in metres (matches data/spawning.json / PoiManager.cs).
pub const DEFAULT_DESPAWN_DISTANCE: f32 = 120.0;

fn load_default_poi_config() -> PoiConfigData {
    let path = paths::repo_path("data/poi.json");
    PoiConfig::load_from_file(&path).unwrap_or_else(|_| {
        let fallback_json = include_str!("../../../data/poi.json");
        PoiConfig::from_json(fallback_json).unwrap_or_default()
    })
}

/// Pure logic manager for zombies waiting inside and around POI buildings.
// Godot: PoiManager.cs
#[derive(Clone, Debug)]
pub struct PoiZombies {
    pub world_seed: i32,
    pub approach_distance: f32,
    pub max_active_zombies: usize,
    pub despawn_dist: f32,
    pub is_night: bool,
    pub visited: HashSet<String>,
    pub remaining: HashMap<String, usize>,
    pub pending_spawns: HashMap<String, usize>,
    pub zombie_to_poi: HashMap<u64, String>,
    pub awake: HashSet<u64>,
    pub config: PoiConfigData,
    pub spawn_config: Option<SpawnConfig>,
    pub spawner_rng: SpawnerRng,
}

impl PoiZombies {
    /// Creates a manager loading configuration from `data/poi.json` and `data/spawning.json`.
    pub fn new(world_seed: i32) -> Self {
        let config = load_default_poi_config();
        Self::with_config(world_seed, config)
    }

    /// Creates a manager with custom POI configuration.
    pub fn with_config(world_seed: i32, config: PoiConfigData) -> Self {
        let approach_distance = if config.approach_distance > 0.0 {
            config.approach_distance
        } else {
            DEFAULT_APPROACH_DISTANCE
        };
        let max_active_zombies = if config.max_active_zombies > 0 {
            config.max_active_zombies as usize
        } else {
            DEFAULT_MAX_ACTIVE_ZOMBIES
        };
        let spawn_config = SpawnConfig::load_default().ok();
        let despawn_dist = spawn_config
            .as_ref()
            .map_or(DEFAULT_DESPAWN_DISTANCE, |c| c.despawn_dist);

        Self {
            world_seed,
            approach_distance,
            max_active_zombies,
            despawn_dist,
            is_night: false,
            visited: HashSet::new(),
            remaining: HashMap::new(),
            pending_spawns: HashMap::new(),
            zombie_to_poi: HashMap::new(),
            awake: HashSet::new(),
            config,
            spawn_config,
            spawner_rng: SpawnerRng::new(world_seed as u64),
        }
    }

    /// True if the building has woken before or was loaded from a visit save.
    // Godot: PoiManager.cs IsVisited
    pub fn is_visited(&self, poi_id: &str) -> bool {
        self.visited.contains(poi_id)
    }

    /// True once the building has woken and none of its zombies are left alive.
    // Godot: PoiManager.cs IsCleared
    pub fn is_cleared(&self, poi_id: &str) -> bool {
        if !self.visited.contains(poi_id) {
            return false;
        }
        let rem = self.remaining.get(poi_id).copied().unwrap_or(0);
        let pending = self.pending_spawns.get(poi_id).copied().unwrap_or(0);
        rem + pending == 0
    }

    /// Returns sorted list of all visited POI ids.
    // Godot: PoiManager.cs VisitedIds
    pub fn visited_ids(&self) -> Vec<String> {
        let mut list: Vec<String> = self.visited.iter().cloned().collect();
        list.sort();
        list
    }

    /// Total alive or pending zombies for a specific POI.
    pub fn remaining_zombies(&self, poi_id: &str) -> usize {
        let rem = self.remaining.get(poi_id).copied().unwrap_or(0);
        let pending = self.pending_spawns.get(poi_id).copied().unwrap_or(0);
        rem + pending
    }

    /// Number of awake zombies currently tracked.
    pub fn awake_count(&self) -> usize {
        self.awake.len()
    }

    /// Test hook: marks a building woken with this many zombies, without spawning any.
    // Godot: PoiManager.cs MarkWokenForTest
    pub fn mark_woken_for_test(&mut self, poi_id: &str, zombies: usize) {
        self.visited.insert(poi_id.to_string());
        self.remaining.insert(poi_id.to_string(), zombies);
        self.pending_spawns.insert(poi_id.to_string(), 0);
    }

    /// Wakes buildings within wake range that are not yet visited.
    /// Returns spawn requests with world positions computed from POI position/yaw and marker offsets.
    /// Respects the active zombie cap: buildings wait when cap would be exceeded, remaining unvisited.
    // Godot: PoiManager.cs Tick / Wake
    pub fn update(
        &mut self,
        player: Vec3,
        sites: &[PoiZombieSite],
        alive_zombies: usize,
    ) -> Vec<PoiSpawn> {
        let mut spawns = Vec::new();
        let mut current_active = alive_zombies.max(self.awake.len());

        for site in sites {
            if self.visited.contains(&site.poi_id) {
                continue;
            }

            let dx = player.x - site.position.x;
            let dz = player.z - site.position.z;
            let flat_dist = (dx * dx + dz * dz).sqrt();
            if flat_dist > self.approach_distance {
                continue;
            }

            let def = self.config.get(&site.building_id);
            let wake_height_m = def.map_or(0.0, |d| d.wake_height_m);
            if wake_height_m > 0.0 && (player.y - site.position.y).abs() > wake_height_m {
                continue;
            }

            let (min_z, max_z) = if let Some(def) = def {
                (def.zombies[0], def.zombies[1])
            } else {
                let n = site.spawn_points.len() as i32;
                (n, n)
            };

            let count = if min_z <= max_z && max_z > 0 {
                let seed = crate_seed(self.world_seed, &site.poi_id);
                let mut rng = DotnetRandom::new(seed);
                rng.next_range(min_z, max_z + 1).max(0) as usize
            } else {
                0
            };

            if count > 0 && current_active + count > self.max_active_zombies {
                continue;
            }

            self.visited.insert(site.poi_id.clone());

            if count == 0 || site.spawn_points.is_empty() {
                self.remaining.insert(site.poi_id.clone(), 0);
                continue;
            }

            *self.pending_spawns.entry(site.poi_id.clone()).or_default() += count;
            current_active += count;

            // A building may name the looks its zombies draw (R42 places do: police station, hospital...).
            let models = self
                .config
                .get(&site.building_id)
                .map(|def| def.zombie_models.clone())
                .unwrap_or_default();

            let rot = glam::Quat::from_rotation_y(site.yaw);
            for i in 0..count {
                let marker = site.spawn_points[i % site.spawn_points.len()];
                let pos = site.position + rot * marker;
                let kind = self.pick_kind_for(&site.building_id);
                let model = self.pick_model(&models);
                spawns.push(PoiSpawn::new(&site.poi_id, pos, kind).with_model(model));
            }
        }

        spawns
    }

    /// Registers a newly spawned zombie for a building lot.
    // Godot: PoiManager.cs NoteLeft / Wake
    pub fn on_zombie_spawned(&mut self, poi_id: &str, zombie_id: u64) {
        self.visited.insert(poi_id.to_string());
        self.zombie_to_poi.insert(zombie_id, poi_id.to_string());
        self.awake.insert(zombie_id);
        if let Some(pending) = self.pending_spawns.get_mut(poi_id)
            && *pending > 0
        {
            *pending -= 1;
        }
        *self.remaining.entry(poi_id.to_string()).or_default() += 1;
    }

    /// Handles a zombie death. If this was the last zombie of a building, returns `Some(cleared_poi_id)`.
    // Godot: PoiManager.cs NoteHomeGone -> NoteLeft
    pub fn on_zombie_died(&mut self, zombie_id: u64) -> Option<String> {
        let poi_id = self.zombie_to_poi.remove(&zombie_id)?;
        self.awake.remove(&zombie_id);

        let rem = self.remaining.entry(poi_id.clone()).or_insert(0);
        *rem = rem.saturating_sub(1);

        let pending = self.pending_spawns.get(&poi_id).copied().unwrap_or(0);
        if *rem == 0 && pending == 0 {
            Some(poi_id)
        } else {
            None
        }
    }

    /// Despawns a building zombie that fell too far behind. Returns `Some(cleared_poi_id)` if all are gone.
    // Godot: PoiManager.cs NoteHomeGone (despawned zombies are gone for good, building does not wait for them)
    pub fn on_zombie_despawned(&mut self, zombie_id: u64) -> Option<String> {
        self.on_zombie_died(zombie_id)
    }

    /// Identifies tracked zombies that are farther than `despawn_dist` from `player`.
    // Godot: PoiManager.cs Tick (Flat(zombie.GlobalPosition, playerPos) > far)
    pub fn check_despawns(&self, player: Vec3, zombie_positions: &[(u64, Vec3)]) -> Vec<u64> {
        let mut to_despawn = Vec::new();
        for &(id, pos) in zombie_positions {
            if self.zombie_to_poi.contains_key(&id) {
                let dx = pos.x - player.x;
                let dz = pos.z - player.z;
                if (dx * dx + dz * dz).sqrt() > self.despawn_dist {
                    to_despawn.push(id);
                }
            }
        }
        to_despawn
    }

    /// Picks a zombie type id using SpawnerRng and SpawnConfig.
    // Godot: PoiManager.cs PickTypeId
    pub fn pick_kind(&mut self) -> String {
        if let Some(cfg) = &self.spawn_config {
            cfg.pick_type_id(&mut self.spawner_rng, self.is_night).to_string()
        } else {
            "walker".to_string()
        }
    }

    /// Picks a zombie kind for a specific building (stairwells get crawlers/walkers by day, no runners).
    fn pick_kind_for(&mut self, building_id: &str) -> String {
        if building_id == "tw_stair_flight_z" && !self.is_night {
            // new: DECISIONS 2026-11-27 (stairwell landings: mostly crawlers/walkers, no runners by day)
            if self.spawner_rng.next_f32() < 0.6 {
                return "crawler".to_string();
            }
            return "walker".to_string();
        }
        self.pick_kind()
    }

    /// Picks one look from a building's `zombieModels`, or None if it names none.
    /// Deterministic: it draws from the manager's own rng, like `pick_kind`.
    fn pick_model(&mut self, models: &[String]) -> Option<String> {
        if models.is_empty() {
            return None;
        }
        let pick = (self.spawner_rng.next_f32() * models.len() as f32) as usize;
        models.get(pick).cloned()
    }

    /// Serializes visited POIs matching Godot WorldState save format.
    // Godot: WorldState.cs Save() -> ["visited_pois"]
    pub fn to_save(&self) -> Value {
        to_save_visited_pois(&self.visited)
    }

    /// Loads visited POIs matching Godot WorldState load format.
    // Godot: WorldState.cs Load() -> SaveData.GetList(data, "visited_pois")
    pub fn from_save(&mut self, data: &Value) {
        self.visited = from_save_visited_pois(data);
        self.remaining.clear();
        self.pending_spawns.clear();
        self.zombie_to_poi.clear();
        self.awake.clear();
    }
}

impl Default for PoiZombies {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny host config with one building that names its zombie looks (like the R42 places).
    fn config_with_models() -> PoiConfigData {
        PoiConfig::from_json(
            r#"{ "buildings": [
                { "id": "police_house", "lot": [40.0, 40.0], "chancePerChunk": 0.0,
                  "minFlatness": 0.9, "minDistanceFromSpawn": 0.0, "zombies": [6, 6],
                  "zombieModels": ["zombie_police", "zombie_soldier"] },
                { "id": "plain_house", "lot": [20.0, 20.0], "chancePerChunk": 0.0,
                  "minFlatness": 0.9, "minDistanceFromSpawn": 0.0, "zombies": [4, 4] }
            ] }"#,
        )
        .expect("host config with zombieModels must parse")
    }

    fn site(poi_id: &str, building_id: &str, at: Vec3) -> PoiZombieSite {
        PoiZombieSite::new(
            poi_id,
            building_id,
            at,
            0.0,
            vec![Vec3::new(1.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0)],
        )
    }

    #[test]
    fn spawn_models_come_from_the_building_list() {
        let mut manager = PoiZombies::with_config(7, config_with_models());
        let spawns = manager.update(Vec3::new(0.0, 0.0, 5.0), &[site("p1", "police_house", Vec3::ZERO)], 0);
        assert_eq!(spawns.len(), 6);
        let allowed = ["zombie_police", "zombie_soldier"];
        for spawn in &spawns {
            let model = spawn.model.as_deref().expect("a building with zombieModels always names a look");
            assert!(allowed.contains(&model), "{model} is not one of the building's looks");
        }
    }

    #[test]
    fn same_seed_picks_the_same_models() {
        let run = || {
            let mut manager = PoiZombies::with_config(7, config_with_models());
            manager
                .update(Vec3::new(0.0, 0.0, 5.0), &[site("p1", "police_house", Vec3::ZERO)], 0)
                .into_iter()
                .map(|s| (s.kind, s.model))
                .collect::<Vec<_>>()
        };
        assert_eq!(run(), run(), "the manager rng makes the looks deterministic");
    }

    #[test]
    fn building_without_zombie_models_keeps_the_kind_look() {
        let mut manager = PoiZombies::with_config(7, config_with_models());
        let spawns = manager.update(Vec3::new(0.0, 0.0, 5.0), &[site("p2", "plain_house", Vec3::ZERO)], 0);
        assert_eq!(spawns.len(), 4);
        assert!(spawns.iter().all(|s| s.model.is_none()), "no zombieModels means None (the kind's own look)");
    }

    fn config_with_wake_height() -> PoiConfigData {
        PoiConfig::from_json(
            r#"{ "buildings": [
                { "id": "floor_20", "lot": [24.0, 24.0], "chancePerChunk": 0.0,
                  "minFlatness": 0.9, "minDistanceFromSpawn": 0.0, "zombies": [3, 3],
                  "wakeHeightM": 3.0 },
                { "id": "flat_only", "lot": [24.0, 24.0], "chancePerChunk": 0.0,
                  "minFlatness": 0.9, "minDistanceFromSpawn": 0.0, "zombies": [2, 2] },
                { "id": "tw_stair_flight_z", "lot": [5.0, 7.0], "chancePerChunk": 0.0,
                  "minFlatness": 0.9, "minDistanceFromSpawn": 0.0, "zombies": [2, 2],
                  "wakeHeightM": 3.0 }
            ] }"#,
        )
        .expect("wakeHeightM test config must parse")
    }

    #[test]
    fn wake_height_blocks_player_below_and_wakes_on_same_floor() {
        let mut manager = PoiZombies::with_config(11, config_with_wake_height());
        let floor = site("tower_f20", "floor_20", Vec3::new(0.0, 30.0, 0.0));

        let below = manager.update(Vec3::new(0.0, 0.0, 5.0), std::slice::from_ref(&floor), 0);
        assert!(below.is_empty(), "player 30 m below must not wake a floor with wakeHeightM 3.0");
        assert!(!manager.is_visited("tower_f20"));

        let on_floor = manager.update(Vec3::new(0.0, 30.0, 5.0), &[floor], 0);
        assert_eq!(on_floor.len(), 3, "player on the same floor must wake it");
        assert!(manager.is_visited("tower_f20"));
    }

    #[test]
    fn zero_wake_height_keeps_flat_only_behaviour() {
        let mut manager = PoiZombies::with_config(11, config_with_wake_height());
        let flat_site = site("old_poi", "flat_only", Vec3::new(0.0, 30.0, 0.0));

        let spawns = manager.update(Vec3::new(0.0, 0.0, 5.0), &[flat_site], 0);
        assert_eq!(spawns.len(), 2, "wake_height_m == 0.0 must keep flat-only wake behaviour");
    }

    #[test]
    fn floor_20_zombies_do_not_despawn_from_lobby_and_killed_stay_dead() {
        let mut manager = PoiZombies::with_config(11, config_with_wake_height());
        let floor = site("tower_f20", "floor_20", Vec3::new(0.0, 80.0, 0.0));

        let spawns = manager.update(Vec3::new(0.0, 80.0, 5.0), std::slice::from_ref(&floor), 0);
        assert_eq!(spawns.len(), 3);
        for (i, _) in spawns.iter().enumerate() {
            manager.on_zombie_spawned("tower_f20", 100 + i as u64);
        }

        // Player returns to the lobby 80 m below: flat distance is < despawn_dist, so none despawn.
        let tracked = vec![
            (100, Vec3::new(1.0, 80.0, 0.0)),
            (101, Vec3::new(-1.0, 80.0, 0.0)),
            (102, Vec3::new(0.0, 80.0, 1.0)),
        ];
        assert!(
            manager.check_despawns(Vec3::new(0.0, 0.0, 5.0), &tracked).is_empty(),
            "zombies on floor 20 must not despawn when player is in the lobby below"
        );

        // Kill one zombie and re-check wake: building stays visited and does not respawn.
        manager.on_zombie_died(100);
        assert_eq!(manager.remaining_zombies("tower_f20"), 2);
        assert!(manager.update(Vec3::new(0.0, 80.0, 5.0), &[floor], 2).is_empty());
    }

    #[test]
    fn stairwell_spawns_only_crawlers_or_walkers_by_day() {
        let mut manager = PoiZombies::with_config(19, config_with_wake_height());
        manager.is_night = false;
        let stair = site("stair_z", "tw_stair_flight_z", Vec3::new(0.0, 10.0, 0.0));

        let spawns = manager.update(Vec3::new(0.0, 10.0, 2.0), &[stair], 0);
        assert_eq!(spawns.len(), 2);
        for s in &spawns {
            assert!(
                s.kind == "crawler" || s.kind == "walker",
                "stairwell daytime spawn must be crawler or walker, got {}",
                s.kind
            );
        }
    }
}
