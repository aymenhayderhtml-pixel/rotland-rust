//! New Halden runtime wiring: metro zombie spawn profiles and slow avenue herds (R41-09).

use std::collections::HashMap;

use game::{
    MetroSpawnConfig, MetroSpawnProfile, StreetHerds, StreetHerdsSaveState,
    metro_profile_near_place,
};
use glam::{Vec2, Vec3};
use physics::PhysicsWorld;
use world::WorldGenerator;
use world::metro::{
    army_line_blocked_points, army_line_site, car_wall_sites, herd_routes_avoiding, place_sites,
};

use crate::zombies::Zombies;

/// A herd member farther than this from the player has not noticed them, whatever its state.
const HERD_NOTICE_M: f32 = 35.0; // new: DECISIONS 2026-11-27

/// Normal horde chase speed when a street herd aggros on the player.
const CHASE_HORDE_SPEED: f32 = 2.2; // data: director.json speed

/// Returns the metro spawn profile at `player` if inside New Halden, else `None`.
pub fn metro_profile_at(
    world_gen: &WorldGenerator,
    player: Vec3,
    night: bool,
    cfg: &MetroSpawnConfig,
) -> Option<MetroSpawnProfile> {
    let layout = world_gen.metro_layout()?;
    let p = Vec2::new(player.x, player.z);
    let district = layout.district_at(p)?;
    let near_place = cfg.place_danger.as_ref().is_some_and(|d| {
        let r2 = d.radius * d.radius;
        place_sites(layout).iter().any(|s| s.pos.distance_squared(p) <= r2)
            || army_line_site(layout).is_some_and(|s| s.origin.distance_squared(p) <= r2)
    });
    Some(metro_profile_near_place(cfg, district.into(), night, near_place))
}

/// Spawn ground check inside New Halden: never on a tower (the physics ray from above hits the
/// tower roof, so zombies used to spawn on roofs and never reached the player), only near the
/// street level. Outside the city it is the normal physics check.
pub fn metro_spawn_ground(world_gen: &WorldGenerator, physics: &PhysicsWorld, point: Vec3) -> Option<Vec3> {
    let feet = game::spawner::try_ground_physics(physics, point)?;
    let Some(metro) = world_gen.metro_world() else { return Some(feet) };
    if metro.in_tower_footprint(Vec2::new(point.x, point.z)) {
        return None;
    }
    let ground = world_gen.height(point.x, point.z);
    (feet.y <= ground + MAX_SPAWN_ABOVE_GROUND_M).then_some(feet)
}

/// A spawn more than this above the terrain is on a roof, a car or a set piece: skipped.
const MAX_SPAWN_ABOVE_GROUND_M: f32 = 1.5; // new: DECISIONS 2026-11-27

/// Manages New Halden's avenue zombie herds, syncing them with `Zombies` and `HordeSet`.
#[derive(Default)]
pub struct MetroHerdRuntime {
    pub herds: Option<StreetHerds>,
    pub speed: f32,
    route_leader: HashMap<u32, u64>,
    pending_save: Option<StreetHerdsSaveState>,
}

impl MetroHerdRuntime {
    /// Initializes `StreetHerds` once from `world_gen.metro_world()`.
    pub fn ensure_init(&mut self, world_gen: &WorldGenerator, seed: u64) {
        if self.herds.is_some() {
            return;
        }
        let Some(metro) = world_gen.metro_world() else {
            return;
        };
        let mut blocked: Vec<Vec2> = vec![metro.damage.fallen.pos];
        for (cw_pos, _) in car_wall_sites(metro) {
            blocked.push(cw_pos);
        }
        blocked.extend(army_line_blocked_points(metro));
        if let Some(end) = metro.highway.points.last() {
            let end_local = metro.layout.site.to_local(*end);
            if let Some(ave) = metro
                .layout
                .streets
                .iter()
                .filter(|s| s.avenue)
                .min_by(|a, b| {
                    a.closest_point(end_local)
                        .distance_squared(end_local)
                        .total_cmp(&b.closest_point(end_local).distance_squared(end_local))
                })
            {
                let (entrance, far) =
                    if ave.a.distance_squared(end_local) <= ave.b.distance_squared(end_local) {
                        (ave.a, ave.b)
                    } else {
                        (ave.b, ave.a)
                    };
                let inward = (far - entrance).normalize_or_zero();
                let first_block_mid = metro.layout.site.to_world(entrance + inward * 32.0); // new: DECISIONS 2026-11-27
                blocked.push(first_block_mid);
            }
        }
        let cfg = &metro.config.herds;
        let routes = herd_routes_avoiding(&metro.layout, cfg, &blocked, seed);
        let mut herds = StreetHerds::new(routes, cfg.clone());
        if let Some(saved) = self.pending_save.take() {
            herds.from_save_state(&saved);
        }
        self.speed = cfg.speed;
        self.herds = Some(herds);
    }

    /// Updates street herds: spawns/despawns walkers, drives walking targets, and handles aggro.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        world_time: f64,
        day: i32,
        player: Vec3,
        hard_cap: usize,
        zombies: &mut Zombies,
        physics: &PhysicsWorld,
        world_gen: &WorldGenerator,
        seed: u64,
    ) {
        self.ensure_init(world_gen, seed);
        let Some(herds) = self.herds.as_mut() else {
            return;
        };

        let orders = herds.update_with_day(
            world_time,
            day,
            player,
            &|route| {
                zombies
                    .list
                    .iter()
                    .filter(|z| z.herd_route == Some(route) && !z.health.is_dead())
                    .count()
            },
            &|x, z| physics.ground_height(x, z).unwrap_or_else(|| world_gen.height(x, z)),
        );

        for route in orders.despawn_routes {
            let ids: Vec<u64> = zombies
                .list
                .iter()
                .filter(|z| z.herd_route == Some(route))
                .map(|z| z.id)
                .collect();
            if !ids.is_empty() {
                zombies.despawn(&ids);
            }
            self.route_leader.remove(&route);
        }

        for (route, positions) in orders.spawn {
            let room = hard_cap.saturating_sub(zombies.alive());
            if room == 0 || positions.is_empty() {
                continue;
            }
            let take = positions.len().min(room);
            let mut spawned_ids = Vec::with_capacity(take);
            for pos in positions.into_iter().take(take) {
                if let Some(id) = zombies.spawn_kind("walker", pos) {
                    if let Some(z) = zombies.list.iter_mut().find(|z| z.id == id) {
                        z.herd_route = Some(route);
                        z.brain.set_wander_speed(self.speed);
                    }
                    spawned_ids.push(id);
                }
            }
            if let Some((&leader, members)) = spawned_ids.split_first() {
                let leader_pos = zombies
                    .list
                    .iter()
                    .find(|z| z.id == leader)
                    .map_or(player, |z| z.position);
                let h_idx = zombies
                    .hordes
                    .create_horde(leader, members.to_vec(), leader_pos);
                let target = herds.leader_target(route, world_time);
                if let Some(h) = zombies.hordes.hordes_mut().get_mut(h_idx) {
                    h.speed = self.speed;
                    h.set_target_pos(Some(target));
                }
                self.route_leader.insert(route, leader);
            }
        }

        // Detect noticed members and update each active herd's target & speed.
        let active_routes: Vec<(u32, u64)> =
            self.route_leader.iter().map(|(&r, &l)| (r, l)).collect();
        for (route, leader) in active_routes {
            // A member noticed the player only when it is close and hunting (horde followers can
            // be in the chase state just from following the leader).
            let any_noticed = zombies.list.iter().any(|z| {
                z.herd_route == Some(route)
                    && !z.health.is_dead()
                    && z.position.distance(player) <= HERD_NOTICE_M
                    && !matches!(
                        z.brain.state(),
                        game::zombie::ZombieState::Idle | game::zombie::ZombieState::Wander
                    )
            });
            if any_noticed {
                herds.on_noticed(route);
            }

            let chasing = herds.is_chasing(route);
            let whole_aggro = herds.is_whole_herd_aggro(route);
            if whole_aggro {
                for z in &mut zombies.list {
                    if z.herd_route == Some(route) && !z.health.is_dead() {
                        z.brain.alert_chase(player);
                    }
                }
            }

            let target = if chasing {
                player
            } else {
                herds.leader_target(route, world_time)
            };
            let speed = if chasing { CHASE_HORDE_SPEED } else { self.speed };
            if let Some(h) = zombies
                .hordes
                .hordes_mut()
                .iter_mut()
                .find(|h| h.leader == leader)
            {
                h.speed = speed;
                h.set_target_pos(Some(target));
            }
        }
    }

    /// Exports killed-on-day state for saving in `systems.extra["street_herds"]`.
    pub fn to_save(&self) -> Option<serde_json::Value> {
        let herds = self.herds.as_ref()?;
        serde_json::to_value(herds.to_save_state()).ok()
    }

    /// Restores killed-on-day state from `systems.extra["street_herds"]`.
    pub fn restore_save(&mut self, value: &serde_json::Value) {
        if let Ok(state) = serde_json::from_value::<StreetHerdsSaveState>(value.clone()) {
            if let Some(herds) = self.herds.as_mut() {
                herds.from_save_state(&state);
            } else {
                self.pending_save = Some(state);
            }
        }
    }
}

/// Timed use state for the 6-second surgery kit (R41-09).
#[derive(Default, Clone, Copy, Debug)]
pub struct SurgeryUse {
    pub remaining: f32,
    pub active: bool,
}

impl SurgeryUse {
    /// Starts the timed surgery kit action if not already active.
    pub fn start(&mut self, seconds: f32) -> bool {
        if self.active || seconds <= 0.0 {
            return false;
        }
        self.remaining = seconds;
        self.active = true;
        true
    }

    /// Advances the timer; returns `true` on the frame it completes.
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.active {
            return false;
        }
        self.remaining = (self.remaining - dt).max(0.0);
        if self.remaining <= 0.0 {
            self.active = false;
            true
        } else {
            false
        }
    }
}
