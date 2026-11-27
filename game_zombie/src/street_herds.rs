//! Runtime state for New Halden avenue zombie herds. Pure logic, no physics.

use std::collections::HashMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};
use world::DotnetRandom;
use world::metro::herds::{HerdRoute, HerdsConfig, route_pos};

/// Default in-game day length in real seconds (24 minutes at 1.0 min/game-hour).
const SECONDS_PER_GAME_DAY: f64 = 1440.0; // data: Clock default 24 min/day
/// Lookahead time in seconds for `leader_target` along the herd's route.
const LEADER_LOOKAHEAD_S: f64 = 2.0; // new: DECISIONS 2026-11-27
const SPREAD_CLAMP_SHARE: f32 = 0.95; // new: DECISIONS 2026-11-27 (keep blob strictly within spreadM)

/// Orders emitted by `StreetHerds::update` for `rotland/` to spawn or despawn horde zombies.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StreetHerdOrders {
    pub spawn: Vec<(u32, Vec<Vec3>)>,
    pub despawn_routes: Vec<u32>,
}

/// Serializable state for killed herds (stored in save `extra`).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StreetHerdsSaveState {
    #[serde(default)]
    pub killed_on_day: Vec<(u32, i32)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HerdState {
    Dormant,
    Active { chasing: bool, noticed_count: u32 },
    KilledOnDay(i32),
}

/// Manages dormant/active/killed states for all avenue zombie herds in New Halden.
#[derive(Debug, Clone)]
pub struct StreetHerds {
    routes: Vec<HerdRoute>,
    by_id: HashMap<u32, usize>,
    states: Vec<HerdState>,
    cfg: HerdsConfig,
}

impl StreetHerds {
    /// Creates a new `StreetHerds` manager with all routes starting dormant.
    pub fn new(routes: Vec<HerdRoute>, cfg: HerdsConfig) -> Self {
        let by_id = routes.iter().enumerate().map(|(i, r)| (r.id, i)).collect();
        let states = vec![HerdState::Dormant; routes.len()];
        Self { routes, by_id, states, cfg }
    }

    /// Updates all herds for `world_time` (real seconds) and `player` world position.
    pub fn update(
        &mut self,
        world_time: f64,
        player: Vec3,
        alive: &dyn Fn(u32) -> usize,
        ground: &dyn Fn(f32, f32) -> f32,
    ) -> StreetHerdOrders {
        let day = (world_time.max(0.0) / SECONDS_PER_GAME_DAY).floor() as i32 + 1;
        self.update_with_day(world_time, day, player, alive, ground)
    }

    /// Same as `update`, using an explicit in-game day number from the clock.
    pub fn update_with_day(
        &mut self,
        world_time: f64,
        day: i32,
        player: Vec3,
        alive: &dyn Fn(u32) -> usize,
        ground: &dyn Fn(f32, f32) -> f32,
    ) -> StreetHerdOrders {
        let player_xz = Vec2::new(player.x, player.z);
        let mut orders = StreetHerdOrders::default();

        for (idx, route) in self.routes.iter().enumerate() {
            if let HerdState::KilledOnDay(killed_day) = self.states[idx] {
                if day > killed_day {
                    self.states[idx] = HerdState::Dormant;
                } else {
                    continue;
                }
            }

            let (pos, yaw) = route_pos(route, world_time);
            let dist = player_xz.distance(pos);

            match self.states[idx] {
                HerdState::Active { chasing, .. } => {
                    if alive(route.id) == 0 {
                        self.states[idx] = HerdState::KilledOnDay(day);
                    } else if !chasing && dist > self.cfg.despawn_radius {
                        orders.despawn_routes.push(route.id);
                        self.states[idx] = HerdState::Dormant;
                    }
                }
                HerdState::Dormant => {
                    if dist <= self.cfg.active_radius {
                        let positions = spawn_blob(route, pos, yaw, self.cfg.spread_m, ground);
                        orders.spawn.push((route.id, positions));
                        self.states[idx] = HerdState::Active { chasing: false, noticed_count: 0 };
                    }
                }
                HerdState::KilledOnDay(_) => {}
            }
        }

        orders
    }

    /// Next world position a walking herd's leader should head toward at `world_time`.
    pub fn leader_target(&self, route: u32, world_time: f64) -> Vec3 {
        let Some(&idx) = self.by_id.get(&route) else {
            return Vec3::ZERO;
        };
        let (pos, _) = route_pos(&self.routes[idx], world_time + LEADER_LOOKAHEAD_S);
        Vec3::new(pos.x, 0.0, pos.y)
    }

    /// Marks that a member of `route` noticed the player, putting the herd into chasing state.
    pub fn on_noticed(&mut self, route: u32) {
        let Some(&idx) = self.by_id.get(&route) else { return };
        if let HerdState::Active { chasing, noticed_count } = &mut self.states[idx] {
            *chasing = true;
            *noticed_count = noticed_count.saturating_add(1);
        }
    }

    /// True if at least one member of `route` has noticed the player while active.
    pub fn is_chasing(&self, route: u32) -> bool {
        let Some(&idx) = self.by_id.get(&route) else { return false };
        matches!(self.states[idx], HerdState::Active { chasing: true, .. })
    }

    /// True once at least `aggroShare` of the herd's members have noticed the player.
    pub fn is_whole_herd_aggro(&self, route: u32) -> bool {
        let Some(&idx) = self.by_id.get(&route) else { return false };
        if let HerdState::Active { noticed_count, .. } = self.states[idx] {
            let need = (self.routes[idx].size as f32 * self.cfg.aggro_share).ceil() as u32;
            return noticed_count >= need.max(1);
        }
        false
    }

    /// Exports only the per-route killed-on-day list for saving.
    pub fn to_save_state(&self) -> StreetHerdsSaveState {
        let killed_on_day = self
            .routes
            .iter()
            .zip(&self.states)
            .filter_map(|(r, s)| match s {
                HerdState::KilledOnDay(day) => Some((r.id, *day)),
                _ => None,
            })
            .collect();
        StreetHerdsSaveState { killed_on_day }
    }

    /// Restores killed-on-day states from a save (other routes become dormant).
    pub fn from_save_state(&mut self, state: &StreetHerdsSaveState) {
        self.states.fill(HerdState::Dormant);
        for &(route_id, day) in &state.killed_on_day {
            if let Some(&idx) = self.by_id.get(&route_id) {
                self.states[idx] = HerdState::KilledOnDay(day);
            }
        }
    }
}

fn spawn_blob(
    route: &HerdRoute,
    leader: Vec2,
    yaw: f32,
    spread_m: f32,
    ground: &dyn Fn(f32, f32) -> f32,
) -> Vec<Vec3> {
    let fwd = Vec2::new(-yaw.sin(), -yaw.cos());
    let right = Vec2::new(-fwd.y, fwd.x);
    let mut rng = DotnetRandom::new(route.id as i32);
    let max_offset = spread_m * SPREAD_CLAMP_SHARE;
    let mut out = Vec::with_capacity(route.size as usize);

    for i in 0..route.size {
        let offset = if i == 0 {
            Vec2::ZERO
        } else {
            let back = rng.next_double() as f32 * max_offset;
            let lat = (rng.next_double() as f32 - 0.5) * spread_m;
            let v = -fwd * back + right * lat;
            if v.length() > max_offset { v.normalize() * max_offset } else { v }
        };
        let p = leader + offset;
        out.push(Vec3::new(p.x, ground(p.x, p.y), p.y));
    }
    out
}
