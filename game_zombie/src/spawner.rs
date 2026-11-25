//! Zombie spawning logic in the open world, ported from scripts/Enemies/ZombieSpawner.cs.
//! Spawns zombies around the player out of sight and on walkable ground, and despawns
//! those that fall too far behind. Numbers match data/spawning.json.

use glam::Vec3;
use physics::PhysicsWorld;
pub use world::{SettlementConfigData, SettlementSite};

pub use crate::spawner_config::*;

pub const MAX_SPAWN_SLOPE_DEG: f32 = 35.0;
pub const SLOPE_PROBE_DIST: f32 = 1.0;
#[allow(dead_code)]
const CLEARANCE_RADIUS: f32 = 0.35;
pub const CLEARANCE_HEIGHT: f32 = 1.0;
pub const FLOOR_OFFSET: f32 = 0.1;
pub const SPAWN_ATTEMPTS_PER_TICK: usize = 12;
/// A chasing zombie is kept this much farther than the normal despawn distance.
pub const CHASE_KEEP_FACTOR: f32 = 1.5;

/// Snapshot of the game state needed to place or despawn zombies this tick.
pub struct SpawnContext<'a> {
    pub player_position: Vec3,
    pub player_forward: Vec3,
    pub is_night: bool,
    pub alive_zombies: &'a [ActiveZombie],
    pub settlements: &'a [SettlementSite],
    pub settlement_config: Option<&'a SettlementConfigData>,
    pub physics: &'a PhysicsWorld,
    pub ground_check: Option<&'a dyn Fn(Vec3) -> Option<Vec3>>,
    pub zombie_amount: f32,
    pub biome_name: Option<&'a str>,
    pub in_city: bool,
    pub in_city_core: bool,
    pub spawn_rate_multiplier: f32,
    pub max_active_multiplier: f32,
    pub special_chance: f32,
}

impl<'a> SpawnContext<'a> {
    pub fn new(
        player_position: Vec3,
        physics: &'a PhysicsWorld,
        alive_zombies: &'a [ActiveZombie],
        settlements: &'a [SettlementSite],
    ) -> Self {
        Self {
            player_position,
            player_forward: Vec3::NEG_Z,
            is_night: false,
            alive_zombies,
            settlements,
            settlement_config: None,
            physics,
            ground_check: None,
            zombie_amount: 1.0,
            biome_name: None,
            in_city: false,
            in_city_core: false,
            spawn_rate_multiplier: 1.0,
            max_active_multiplier: 1.0,
            special_chance: 0.0,
        }
    }

    pub fn with_multipliers(mut self, rate: f32, max_active: f32, special: f32) -> Self {
        self.spawn_rate_multiplier = rate;
        self.max_active_multiplier = max_active;
        self.special_chance = special;
        self
    }

    pub fn with_in_city(mut self, in_city: bool) -> Self {
        self.in_city = in_city;
        self
    }

    pub fn with_in_city_core(mut self, in_city_core: bool) -> Self {
        self.in_city_core = in_city_core;
        self
    }

    pub fn with_night(mut self, is_night: bool) -> Self {
        self.is_night = is_night;
        self
    }

    pub fn with_forward(mut self, forward: Vec3) -> Self {
        self.player_forward = forward;
        self
    }

    pub fn with_zombie_amount(mut self, amount: f32) -> Self {
        self.zombie_amount = amount;
        self
    }

    #[allow(dead_code)]
    fn with_settlement_config(mut self, cfg: &'a SettlementConfigData) -> Self {
        self.settlement_config = Some(cfg);
        self
    }

    #[allow(dead_code)]
    fn with_ground_check(mut self, check: &'a dyn Fn(Vec3) -> Option<Vec3>) -> Self {
        self.ground_check = Some(check);
        self
    }

    pub fn with_biome(mut self, biome: &'a str) -> Self {
        self.biome_name = Some(biome);
        self
    }
}

/// Distance in metres ignoring height from the point to the nearest camp centre.
pub fn nearest_camp_distance(point: Vec3, settlements: &[SettlementSite]) -> f32 {
    let mut best = f32::INFINITY;
    for site in settlements {
        let dx = site.center.x - point.x;
        let dz = site.center.z - point.z;
        let dist = (dx * dx + dz * dz).sqrt();
        if dist < best {
            best = dist;
        }
    }
    best
}

/// True when the point is within the safe zone plus the spawn padding.
pub fn in_no_spawn_zone(
    point: Vec3,
    settlements: &[SettlementSite],
    cfg: Option<&SettlementConfigData>,
) -> bool {
    for site in settlements {
        let inside = match cfg {
            Some(c) => site.contains(point, c.safe_zone_padding, c),
            None => {
                let limit = 17.5 + 40.0;
                let dx = (point.x - site.center.x).abs();
                let dz = (point.z - site.center.z).abs();
                dx <= limit && dz <= limit
            }
        };
        if inside {
            return true;
        }
    }
    false
}

/// Queries PhysicsWorld: finds ground height, checks slope limit and body clearance.
pub fn try_ground_physics(physics: &PhysicsWorld, point: Vec3) -> Option<Vec3> {
    let ray_start = Vec3::new(point.x, point.y + 500.0, point.z);
    let hit = physics.raycast(ray_start, Vec3::NEG_Y, 1000.0)?;

    let slope_deg = hit.normal.y.clamp(-1.0, 1.0).acos().to_degrees();
    if slope_deg > MAX_SPAWN_SLOPE_DEG {
        return None;
    }

    let body_start = Vec3::new(point.x, hit.point.y + FLOOR_OFFSET, point.z);
    if physics.raycast(body_start, Vec3::Y, CLEARANCE_HEIGHT).is_some() {
        return None;
    }

    Some(Vec3::new(point.x, hit.point.y + FLOOR_OFFSET, point.z))
}

/// Open world zombie spawner pure logic state machine.
pub struct Spawner {
    pub config: SpawnConfig,
    pub rng: SpawnerRng,
    pub play_time: f32,
    pub spawn_timer: f32,
    pub start_delay: f32,
    pub spawn_interval: f32,
    pub max_active: u32,
    pub ignore_biome_density: bool,
}

impl Spawner {
    pub fn new(config: SpawnConfig, seed: u64) -> Self {
        let start_delay = config.start_delay;
        let spawn_interval = config.spawn_interval;
        let max_active = config.max_active;
        Self {
            config,
            rng: SpawnerRng::new(seed),
            play_time: 0.0,
            spawn_timer: 0.0,
            start_delay,
            spawn_interval,
            max_active,
            ignore_biome_density: false,
        }
    }

    /// Active cap scaled by the difficulty's zombie multiplier.
    pub fn scaled_max_active(&self, zombie_amount: f32) -> u32 {
        let amount = if zombie_amount <= 0.0 { 1.0 } else { zombie_amount };
        (self.max_active as f32 * amount).floor() as u32
    }

    /// Full spawn cap taking difficulty, biome, night factor, camp proximity and city into account.
    pub fn calculate_cap(&self, ctx: &SpawnContext) -> usize {
        let scaled = self.scaled_max_active(ctx.zombie_amount) as f32;
        let density = if self.ignore_biome_density {
            1.0
        } else if ctx.in_city && let Some(city) = &self.config.city {
            city.density
        } else if let Some(biome) = ctx.biome_name {
            self.config.density_for(biome)
        } else {
            1.0
        };
        let camp_factor = if self.ignore_biome_density {
            1.0
        } else {
            let camp_dist = nearest_camp_distance(ctx.player_position, ctx.settlements);
            self.config.camp_factor_for(camp_dist, ctx.is_night)
        };
        let mut day_cap = scaled * density * camp_factor;
        if ctx.in_city && let Some(city) = &self.config.city {
            day_cap = day_cap.min(city.max_active as f32);
        }
        let night_factor = if ctx.is_night {
            self.config.night_max_active_multiplier
        } else {
            1.0
        };

        let base_cap = (day_cap * night_factor).floor().max(0.0);
        let mult = if ctx.max_active_multiplier <= 0.0 { 1.0 } else { ctx.max_active_multiplier };
        let cap = (base_cap * mult).floor().max(0.0) as usize;
        let hard = if self.config.hard_cap > 0 { self.config.hard_cap } else { 60 }; // data: spawning.json hardCap
        cap.min(hard)
    }

    /// Advances time, issues despawns for out-of-range zombies, and attempts spawning up to the cap.
    pub fn update(&mut self, dt: f32, ctx: &SpawnContext) -> SpawnOrders {
        let mut orders = SpawnOrders::default();

        let despawn_dist = self.config.despawn_dist;
        let chase_despawn_dist = despawn_dist * CHASE_KEEP_FACTOR;
        for zombie in ctx.alive_zombies {
            let dx = zombie.position.x - ctx.player_position.x;
            let dz = zombie.position.z - ctx.player_position.z;
            let flat_dist = (dx * dx + dz * dz).sqrt();
            let limit = if zombie.is_chasing {
                chase_despawn_dist
            } else {
                despawn_dist
            };
            if flat_dist > limit {
                orders.despawns.push(zombie.id);
            }
        }

        self.play_time += dt;
        if self.play_time < self.start_delay {
            return orders;
        }

        self.spawn_timer -= dt * ctx.spawn_rate_multiplier.max(0.001);
        if self.spawn_timer > 0.0 {
            return orders;
        }
        self.spawn_timer = self.spawn_interval;

        let cap = self.calculate_cap(ctx);
        let active_count = ctx.alive_zombies.len().saturating_sub(orders.despawns.len());
        if active_count >= cap {
            return orders;
        }

        if let Some(order) = self.try_spawn_one(ctx) {
            orders.spawns.push(order);
        }

        orders
    }

    fn try_spawn_one(&mut self, ctx: &SpawnContext) -> Option<SpawnOrder> {
        let mut forward = Vec3::new(ctx.player_forward.x, 0.0, ctx.player_forward.z);
        if forward.length_squared() > 0.0001 {
            forward = forward.normalize();
        } else {
            forward = Vec3::NEG_Z;
        }

        for _ in 0..SPAWN_ATTEMPTS_PER_TICK {
            let angle_mag = self.rng.range(self.config.out_of_view_angle_deg, 180.0);
            let sign = if self.rng.next_bool() { 1.0 } else { -1.0 };
            let angle_deg = angle_mag * sign;
            let distance = self.rng.range(self.config.min_spawn_dist, self.config.max_spawn_dist);

            let rot = glam::Quat::from_rotation_y(angle_deg.to_radians());
            let dir = rot * forward;
            let point = ctx.player_position + dir * distance;

            if in_no_spawn_zone(point, ctx.settlements, ctx.settlement_config) {
                continue;
            }

            let feet = match ctx.ground_check {
                Some(cb) => match cb(point) {
                    Some(f) => f,
                    None => continue,
                },
                None => match try_ground_physics(ctx.physics, point) {
                    Some(f) => f,
                    None => continue,
                },
            };

            let type_id = if (ctx.is_night || ctx.in_city)
                && ctx.special_chance > 0.0
                && self.rng.range(0.0, 1.0) < ctx.special_chance
            {
                let specials = ["brute", "screamer", "crawler"];
                let idx = (self.rng.next_f32() * specials.len() as f32).min(specials.len() as f32 - 1.0) as usize;
                specials[idx]
            } else {
                self.config.pick_type_id_core(&mut self.rng, ctx.is_night, ctx.in_city_core)
            };
            return Some(SpawnOrder {
                type_id: type_id.to_string(),
                position: feet,
            });
        }

        None
    }
}
