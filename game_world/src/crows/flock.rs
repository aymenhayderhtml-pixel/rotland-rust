//! Crow flock state machine (`Perched -> Startled -> Flying -> Gone -> Returning`) and high circlers.

use std::f32::consts::TAU;

use glam::Vec3;

use crate::animals::AnimalRng;
use crate::crows::{CrowsConfig, flap_frame};

const PERCH_SPREAD_RADIUS: f32 = 1.2; // new: DECISIONS 2026-11-27
const PERCH_YAW_JITTER: f32 = 0.8; // new: DECISIONS 2026-11-27
const PECK_INTERVAL_MIN: f32 = 1.5; // new: DECISIONS 2026-11-27
const PECK_INTERVAL_MAX: f32 = 4.5; // new: DECISIONS 2026-11-27
const PECK_DURATION_MIN: f32 = 0.4; // new: DECISIONS 2026-11-27
const PECK_DURATION_MAX: f32 = 0.9; // new: DECISIONS 2026-11-27
const STARTLE_DELAY_MAX: f32 = 0.4; // new: DECISIONS 2026-11-27
const FLY_WOBBLE_FREQ: f32 = 2.6; // new: DECISIONS 2026-11-27
const FLY_WOBBLE_AMP: f32 = 1.4; // new: DECISIONS 2026-11-27
const GLIDE_CYCLE_RATE: f32 = 0.5; // new: DECISIONS 2026-11-27
const RETURN_SNAP_DIST: f32 = 0.25; // new: DECISIONS 2026-11-27
const RETURN_SAFE_MULT: f32 = 2.0; // new: DECISIONS 2026-11-27

/// Which crow model variant to draw for a single crow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrowModel {
    Perched,
    Peck,
    Fly(u8),
}

/// World pose and model variant for one crow instance.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CrowPose {
    pub pos: Vec3,
    pub yaw: f32,
    pub model: CrowModel,
}

/// Lifecycle state of a perched crow flock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlockState {
    Perched,
    Startled,
    Flying,
    Gone,
    Returning,
}

#[derive(Clone, Debug)]
struct CrowMember {
    home_pos: Vec3,
    home_yaw: f32,
    pos: Vec3,
    yaw: f32,
    phase: f32,
    peck_timer: f32,
    pecking: bool,
    takeoff_delay: f32,
    airborne: bool,
    wing_time: f32,
    fly_dir: Vec3,
}

/// A flock of crows associated with a single perch.
#[derive(Clone, Debug)]
pub struct CrowFlock {
    perch_pos: Vec3,
    state: FlockState,
    members: Vec<CrowMember>,
    return_timer: f32,
    rng: AnimalRng,
}

impl CrowFlock {
    /// Creates a perched flock of `count` crows scattered within 1.2 m of `perch_pos`.
    pub fn new(perch_pos: Vec3, yaw: f32, count: u8, seed: u64) -> Self {
        let mut rng = AnimalRng::new(seed);
        let mut members = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let angle = rng.gen_range_f32(0.0, TAU);
            let dist = PERCH_SPREAD_RADIUS * rng.next_f32().sqrt();
            let home_pos = perch_pos + Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist);
            let home_yaw = yaw + rng.gen_range_f32(-PERCH_YAW_JITTER, PERCH_YAW_JITTER);
            let phase = rng.next_f32();
            let peck_timer = rng.gen_range_f32(PECK_INTERVAL_MIN, PECK_INTERVAL_MAX);
            members.push(CrowMember {
                home_pos,
                home_yaw,
                pos: home_pos,
                yaw: home_yaw,
                phase,
                peck_timer,
                pecking: false,
                takeoff_delay: 0.0,
                airborne: false,
                wing_time: phase,
                fly_dir: yaw_to_dir(home_yaw),
            });
        }
        Self {
            perch_pos,
            state: FlockState::Perched,
            members,
            return_timer: 0.0,
            rng,
        }
    }

    /// Creates a flock that is already `Gone` with `return_timer` seconds remaining.
    pub fn new_gone(perch_pos: Vec3, yaw: f32, count: u8, seed: u64, return_timer: f32) -> Self {
        let mut flock = Self::new(perch_pos, yaw, count, seed);
        flock.state = FlockState::Gone;
        flock.return_timer = return_timer.max(0.0);
        flock
    }

    /// Current flock state.
    pub fn state(&self) -> FlockState {
        self.state
    }

    /// Remaining seconds until return if the flock is currently `Gone`.
    pub fn gone_remaining(&self) -> Option<f32> {
        (self.state == FlockState::Gone).then_some(self.return_timer)
    }

    /// Advances the flock simulation by `dt` seconds.
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_running: bool,
        gunshot_at: Option<Vec3>,
        cfg: &CrowsConfig,
    ) {
        if dt <= 0.0 || self.members.is_empty() {
            return;
        }
        let startle_r = if player_running {
            cfg.startle_radius_running
        } else {
            cfg.startle_radius
        };
        let startled_now = self.is_startled(player_pos, startle_r, gunshot_at, cfg.gunshot_startle_radius);

        match self.state {
            FlockState::Perched => {
                if startled_now {
                    self.begin_startle(player_pos);
                } else {
                    self.tick_pecking(dt);
                }
            }
            FlockState::Startled => {
                let mut all_airborne = true;
                for m in &mut self.members {
                    if !m.airborne {
                        m.takeoff_delay -= dt;
                        if m.takeoff_delay <= 0.0 {
                            m.airborne = true;
                        } else {
                            all_airborne = false;
                        }
                    }
                    if m.airborne {
                        step_flying_member(m, dt, cfg, true);
                    }
                }
                if all_airborne {
                    self.state = FlockState::Flying;
                }
            }
            FlockState::Flying => {
                let gone_y = self.perch_pos.y + cfg.gone_height;
                let mut all_gone = true;
                for m in &mut self.members {
                    step_flying_member(m, dt, cfg, false);
                    if m.pos.y < gone_y {
                        all_gone = false;
                    }
                }
                if all_gone {
                    self.state = FlockState::Gone;
                    let lo = cfg.return_after[0].min(cfg.return_after[1]);
                    let hi = cfg.return_after[0].max(cfg.return_after[1]);
                    self.return_timer = self.rng.gen_range_f32(lo, hi);
                }
            }
            FlockState::Gone => {
                self.return_timer = (self.return_timer - dt).max(0.0);
                let safe_r = startle_r * RETURN_SAFE_MULT;
                let player_away = self.perch_pos.distance_squared(player_pos) > safe_r * safe_r;
                if self.return_timer <= 0.0 && player_away && !startled_now {
                    self.state = FlockState::Returning;
                    for m in &mut self.members {
                        m.pos = Vec3::new(
                            m.home_pos.x - m.fly_dir.x * cfg.gone_height,
                            self.perch_pos.y + cfg.gone_height,
                            m.home_pos.z - m.fly_dir.z * cfg.gone_height,
                        );
                        m.airborne = true;
                    }
                }
            }
            FlockState::Returning => {
                if startled_now {
                    self.begin_startle(player_pos);
                    return;
                }
                let mut all_landed = true;
                for m in &mut self.members {
                    let to_home = m.home_pos - m.pos;
                    let dist = to_home.length();
                    if dist <= RETURN_SNAP_DIST {
                        m.pos = m.home_pos;
                        m.yaw = m.home_yaw;
                        m.airborne = false;
                        m.pecking = false;
                    } else {
                        all_landed = false;
                        let step = (cfg.fly_speed * dt).min(dist);
                        let dir = to_home / dist;
                        m.pos += dir * step;
                        let flat = Vec3::new(dir.x, 0.0, dir.z);
                        if flat.length_squared() > 1e-4 {
                            m.yaw = dir_to_yaw(flat.normalize());
                        }
                        m.wing_time += dt;
                    }
                }
                if all_landed {
                    self.state = FlockState::Perched;
                }
            }
        }
    }

    /// Iterates over the visible crow poses in this flock. Empty when `Gone`.
    pub fn instances<'a>(&'a self, cfg: &'a CrowsConfig) -> impl Iterator<Item = CrowPose> + 'a {
        let visible = self.state != FlockState::Gone;
        self.members.iter().filter(move |_| visible).map(move |m| {
            let model = if !m.airborne {
                if m.pecking { CrowModel::Peck } else { CrowModel::Perched }
            } else {
                flying_model(m.phase, m.wing_time, cfg)
            };
            CrowPose { pos: m.pos, yaw: m.yaw, model }
        })
    }

    fn is_startled(
        &self,
        player_pos: Vec3,
        startle_r: f32,
        gunshot_at: Option<Vec3>,
        gunshot_r: f32,
    ) -> bool {
        if self.perch_pos.distance_squared(player_pos) <= startle_r * startle_r {
            return true;
        }
        gunshot_at.is_some_and(|g| self.perch_pos.distance_squared(g) <= gunshot_r * gunshot_r)
    }

    fn begin_startle(&mut self, player_pos: Vec3) {
        self.state = FlockState::Startled;
        let away = Vec3::new(self.perch_pos.x - player_pos.x, 0.0, self.perch_pos.z - player_pos.z);
        let base_dir = if away.length_squared() > 1e-4 {
            away.normalize()
        } else {
            Vec3::X
        };
        for m in &mut self.members {
            m.takeoff_delay = self.rng.gen_range_f32(0.0, STARTLE_DELAY_MAX);
            m.airborne = false;
            m.pecking = false;
            let spread = self.rng.gen_range_f32(-0.5, 0.5); // new: DECISIONS 2026-11-27
            let (s, c) = spread.sin_cos();
            m.fly_dir = Vec3::new(base_dir.x * c - base_dir.z * s, 0.0, base_dir.x * s + base_dir.z * c)
                .normalize_or_zero();
        }
    }

    fn tick_pecking(&mut self, dt: f32) {
        for m in &mut self.members {
            m.peck_timer -= dt;
            if m.peck_timer <= 0.0 {
                m.pecking = !m.pecking;
                m.peck_timer = if m.pecking {
                    self.rng.gen_range_f32(PECK_DURATION_MIN, PECK_DURATION_MAX)
                } else {
                    self.rng.gen_range_f32(PECK_INTERVAL_MIN, PECK_INTERVAL_MAX)
                };
            }
        }
    }
}

fn step_flying_member(m: &mut CrowMember, dt: f32, cfg: &CrowsConfig, takeoff: bool) {
    m.wing_time += dt;
    let climb = if takeoff { cfg.takeoff_speed } else { cfg.climb_rate };
    let wobble = (m.wing_time * FLY_WOBBLE_FREQ + m.phase * TAU).sin() * FLY_WOBBLE_AMP;
    let side = Vec3::new(-m.fly_dir.z, 0.0, m.fly_dir.x);
    let vel = m.fly_dir * cfg.fly_speed + side * wobble + Vec3::Y * climb;
    m.pos += vel * dt;
    let flat = Vec3::new(vel.x, 0.0, vel.z);
    if flat.length_squared() > 1e-4 {
        m.yaw = dir_to_yaw(flat.normalize());
    }
}

fn flying_model(phase: f32, wing_time: f32, cfg: &CrowsConfig) -> CrowModel {
    let cycle = (wing_time * GLIDE_CYCLE_RATE + phase).fract();
    if cycle < cfg.glide_share {
        CrowModel::Fly(1)
    } else {
        CrowModel::Fly(flap_frame(phase, wing_time, cfg.flap_rate) as u8)
    }
}

/// Godot forward = `(-sin yaw, 0, -cos yaw)`, so `yaw = (-dir.x).atan2(-dir.z)`.
fn dir_to_yaw(dir: Vec3) -> f32 {
    (-dir.x).atan2(-dir.z)
}

fn yaw_to_dir(yaw: f32) -> Vec3 {
    Vec3::new(-yaw.sin(), 0.0, -yaw.cos())
}

#[derive(Clone, Debug)]
struct Circler {
    radius: f32,
    height: f32,
    angle: f32,
    angular_speed: f32,
    phase: f32,
    wing_time: f32,
}

/// High-altitude crows circling slowly over the city.
#[derive(Clone, Debug)]
pub struct Circlers {
    center: Vec3,
    items: Vec<Circler>,
}

impl Circlers {
    /// Spawns `cfg.circlers` circling crows above `city_center`.
    pub fn new(city_center: Vec3, seed: u64, cfg: &CrowsConfig) -> Self {
        let mut rng = AnimalRng::new(seed);
        let mut items = Vec::with_capacity(cfg.circlers);
        let r_lo = cfg.circle_radius[0].min(cfg.circle_radius[1]);
        let r_hi = cfg.circle_radius[0].max(cfg.circle_radius[1]);
        let h_lo = cfg.circle_height[0].min(cfg.circle_height[1]);
        let h_hi = cfg.circle_height[0].max(cfg.circle_height[1]);
        for _ in 0..cfg.circlers {
            let radius = rng.gen_range_f32(r_lo, r_hi).max(1.0);
            let height = rng.gen_range_f32(h_lo, h_hi);
            let angle = rng.gen_range_f32(0.0, TAU);
            let sign = if rng.next_f32() < 0.5 { 1.0 } else { -1.0 }; // new: DECISIONS 2026-11-27
            let angular_speed = sign * (cfg.fly_speed / radius);
            let phase = rng.next_f32();
            items.push(Circler {
                radius,
                height,
                angle,
                angular_speed,
                phase,
                wing_time: phase,
            });
        }
        Self { center: city_center, items }
    }

    /// Advances all circling crows by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        for c in &mut self.items {
            c.angle = (c.angle + c.angular_speed * dt).rem_euclid(TAU);
            c.wing_time += dt;
        }
    }

    /// Iterates over the poses of all circling crows.
    pub fn instances<'a>(&'a self, cfg: &'a CrowsConfig) -> impl Iterator<Item = CrowPose> + 'a {
        self.items.iter().map(move |c| {
            let (s, cs) = c.angle.sin_cos();
            let pos = self.center + Vec3::new(cs * c.radius, c.height, s * c.radius);
            let dir = Vec3::new(-s * c.angular_speed.signum(), 0.0, cs * c.angular_speed.signum());
            CrowPose {
                pos,
                yaw: dir_to_yaw(dir),
                model: flying_model(c.phase, c.wing_time, cfg),
            }
        })
    }
}
