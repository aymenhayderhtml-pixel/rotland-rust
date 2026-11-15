//! Extra test hooks and state transitions for ZombieBrain (R3-09 extras).

use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};

use glam::Vec3;

use super::brain::ZombieBrain;
use super::herd::ZombieHorde;
use super::state::ZombieState;

pub(super) const IDLE_MIN: f32 = 1.0;
pub(super) const IDLE_MAX: f32 = 3.0;
pub(super) const WANDER_MIN: f32 = 2.0;
pub(super) const WANDER_MAX: f32 = 8.0;

impl ZombieBrain {
    pub fn is_crawler(&self) -> bool {
        self.crawler
    }

    pub fn set_crawler(&mut self, crawler: bool) {
        self.crawler = crawler;
    }

    pub fn set_scream_chance(&mut self, chance: f32) {
        self.scream_chance = chance;
    }

    pub fn set_hideout_override(&mut self, val: Option<bool>) {
        self.hideout_override = val;
    }

    pub fn set_eat_chance_override(&mut self, chance: Option<f32>) {
        self.eat_chance_override = chance;
    }

    pub fn set_horde(&mut self, horde: Option<Arc<Mutex<ZombieHorde>>>) {
        self.horde = horde;
    }

    pub fn is_escaping(&self) -> bool {
        self.steering.is_escaping()
    }

    /// Test hook: the zombie loses sight of the player long ago, last seen at `last_seen`.
    pub fn lose_sight_for_test(&mut self, last_seen: Vec3) {
        if self.state == ZombieState::Dead {
            return;
        }
        self.enter_chase();
        self.senses.forget_for_test(last_seen);
    }

    /// Test hook: where the zombie is walking to.
    pub fn goal(&self) -> Vec3 {
        self.goal
    }

    /// Test hook: sets where the zombie is walking to.
    pub fn set_goal_for_test(&mut self, goal: Vec3) {
        self.goal = goal;
    }

    /// Test hook: forces the zombie straight into a state.
    pub fn force_state_for_test(&mut self, state: ZombieState) {
        match state {
            ZombieState::Scream => self.enter_scream(None),
            ZombieState::Eat => self.enter_eat(),
            ZombieState::Bang => self.enter_bang(),
            ZombieState::Chase => self.enter_chase(),
            ZombieState::Investigate => self.enter_investigate(self.home, false),
            ZombieState::Wander => self.enter_wander(self.home),
            _ => self.enter_idle(),
        }
    }

    pub(super) fn enter(&mut self, state: ZombieState) {
        self.state = state;
        self.time_in_state = 0.0;
        self.looking_around = false;
        self.look_point = None;
    }

    pub(super) fn enter_idle(&mut self) {
        self.enter(ZombieState::Idle);
        self.timer = self.rng.range(IDLE_MIN, IDLE_MAX);
    }

    pub(super) fn enter_wander(&mut self, from: Vec3) {
        self.enter(ZombieState::Wander);
        let angle = self.rng.range(0.0, TAU);
        let dist = self.rng.range(WANDER_MIN, WANDER_MAX);
        let random_spot = from + Vec3::new(angle.cos(), 0.0, angle.sin()) * dist;

        if let Some(horde_arc) = &self.horde {
            if let Ok(mut horde) = horde_arc.lock() {
                let shared = horde.wander_target.unwrap_or(random_spot);
                horde.wander_target = Some(shared);
                let spread_x = self.rng.range(-1.5, 1.5);
                let spread_z = self.rng.range(-1.5, 1.5);
                self.goal = shared + Vec3::new(spread_x, 0.0, spread_z);
                return;
            }
        }
        self.goal = random_spot;
    }

    pub(super) fn enter_investigate(&mut self, spot: Vec3, bang_on_arrive: bool) {
        self.enter(ZombieState::Investigate);
        self.goal = spot;
        self.bang_on_arrive = bang_on_arrive;
    }

    pub(super) fn enter_chase(&mut self) {
        self.enter(ZombieState::Chase);
    }

    pub(super) fn enter_attack(&mut self) {
        self.enter(ZombieState::Attack);
        self.attack.start();
    }

    pub(super) fn enter_scream(&mut self, target: Option<Vec3>) {
        self.enter(ZombieState::Scream);
        self.timer = self.numbers.scream_seconds;
        self.look_point = target;
        self.just_screamed = true;
    }

    pub(super) fn enter_eat(&mut self) {
        self.enter(ZombieState::Eat);
        self.timer = self.rng.range(self.numbers.eat_min_seconds, self.numbers.eat_max_seconds);
    }

    pub(super) fn enter_bang(&mut self) {
        self.enter(ZombieState::Bang);
        self.timer = self.numbers.bang_seconds;
    }

    pub(super) fn enter_dead(&mut self) {
        self.enter(ZombieState::Dead);
        self.walk_velocity = Vec3::ZERO;
    }
}
