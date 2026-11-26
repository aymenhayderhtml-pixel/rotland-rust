//! Guard shooting logic and state machine (port of scripts/Settlement/SettlementGuard.cs).

use std::collections::HashMap;
use glam::{Vec2, Vec3};
use world::DotnetRandom;

use super::config::CampGuardConfig;
use super::types::{GuardFrame, GuardPost, GuardRifle, GuardShot, GuardTarget};
use game_core::damage::DamageInfo;

/// Internal state per guard (timers and target tracking).
#[derive(Clone, Debug, PartialEq)]
pub struct GuardState {
    pub scan_timer: f32,
    pub fire_timer: f32,
    pub current_target_id: Option<u64>,
    pub shots_fired: u32,
    pub yaw: f32,
}

impl Default for GuardState {
    fn default() -> Self {
        Self {
            scan_timer: 0.0,
            fire_timer: 0.0,
            current_target_id: None,
            shots_fired: 0,
            yaw: 0.0,
        }
    }
}

/// Manages all camp guards shooting at approaching zombies (port of SettlementGuard.cs).
pub struct CampGuards {
    pub config: CampGuardConfig,
    pub camp_center: Option<Vec3>,
    pub guard_states: HashMap<String, GuardState>,
    pub rng: DotnetRandom,
}

impl Default for CampGuards {
    fn default() -> Self {
        Self::new(CampGuardConfig::default())
    }
}

impl CampGuards {
    /// Creates a new guard manager with the given configuration.
    pub fn new(config: CampGuardConfig) -> Self {
        Self {
            config,
            camp_center: None,
            guard_states: HashMap::new(),
            rng: DotnetRandom::new(12345),
        }
    }

    /// Loads configuration from the repository settlements.json and constructs CampGuards.
    pub fn from_repo() -> Result<Self, String> {
        let config = CampGuardConfig::from_repo()?;
        Ok(Self::new(config))
    }

    /// Sets the camp center point (targets beyond `guard_range` of this point are ignored).
    pub fn with_camp_center(mut self, center: Vec3) -> Self {
        self.camp_center = Some(center);
        self
    }

    /// Sets the random seed for reproducible tests.
    pub fn with_seed(mut self, seed: i32) -> Self {
        self.rng = DotnetRandom::new(seed);
        self
    }

    /// Sets the miss chance directly (0.0 means 100% hits, useful for fire rate tests).
    pub fn with_miss_chance(mut self, miss_chance: f32) -> Self {
        self.config.guard_miss_chance = miss_chance;
        self
    }

    /// Returns the number of shots fired by a specific guard.
    pub fn shots_fired(&self, guard_id: &str) -> u32 {
        self.guard_states.get(guard_id).map_or(0, |s| s.shots_fired)
    }

    /// Returns total shots fired across all guards.
    pub fn total_shots_fired(&self) -> u32 {
        self.guard_states.values().map(|s| s.shots_fired).sum()
    }

    /// Returns the target id currently acquired by a guard, if any.
    pub fn current_target(&self, guard_id: &str) -> Option<u64> {
        self.guard_states.get(guard_id).and_then(|s| s.current_target_id)
    }

    /// Updates guard senses, aims at targets, and fires rifle shots.
    // Godot: SettlementGuard.cs _PhysicsProcess
    pub fn update(
        &mut self,
        dt: f32,
        guards: &[GuardPost],
        targets: &[GuardTarget],
        los: &dyn Fn(Vec3, Vec3) -> bool,
    ) -> GuardFrame {
        let mut frame = GuardFrame::default();

        for guard in guards {
            // Look for a target before borrowing this guard's state (find_target reads self)
            let rescan = self.guard_states.get(&guard.id).is_none_or(|s| s.scan_timer - dt <= 0.0);
            let found = if rescan { Some(self.find_target(guard, targets, los)) } else { None };
            let state = self
                .guard_states
                .entry(guard.id.clone())
                .or_insert_with(|| GuardState {
                    yaw: guard.yaw,
                    ..Default::default()
                });

            // 1. Senses: sight interval scan timer countdown
            // Godot: SettlementGuard.cs lines 55-60
            state.scan_timer -= dt;
            if state.scan_timer <= 0.0 {
                state.scan_timer = self.config.guard_sight_interval;
                state.current_target_id = found.flatten();
            }

            // 2. Validate target: dead or absent target is dropped at once
            // Godot: SettlementGuard.cs lines 63-66
            if let Some(target_id) = state.current_target_id {
                let is_valid = targets
                    .iter()
                    .any(|t| t.id == target_id && t.alive);
                if !is_valid {
                    state.current_target_id = None;
                }
            }

            // 3. When there is no target, fire timer resets to 0.0
            // Godot: SettlementGuard.cs lines 68-72
            let Some(target_id) = state.current_target_id else {
                state.fire_timer = 0.0;
                frame.yaws.push((guard.id.clone(), state.yaw));
                continue;
            };

            let Some(target) = targets.iter().find(|t| t.id == target_id) else {
                state.fire_timer = 0.0;
                frame.yaws.push((guard.id.clone(), state.yaw));
                continue;
            };

            // 4. Aim at target: Godot yaw = atan2(-offset.x, -offset.z)
            // Godot: SettlementGuard.cs lines 74-75
            let offset = target.position - guard.position;
            let aim_yaw = (-offset.x).atan2(-offset.z);
            state.yaw = aim_yaw;
            frame.yaws.push((guard.id.clone(), aim_yaw));

            // 5. Fire rate timer countdown
            // Godot: SettlementGuard.cs lines 77-82
            state.fire_timer -= dt;
            if state.fire_timer <= 0.0 {
                state.fire_timer = self.config.guard_fire_interval;
                state.shots_fired += 1;

                let head = guard.position + Vec3::new(0.0, self.config.guard_eye_height, 0.0);
                frame.sounds.push((GuardRifle::SHOT_SOUND.to_string(), head));

                // Miss chance check (Godot: SettlementGuard.cs Fire lines 110-113)
                let miss = if self.config.guard_miss_chance <= 0.0 {
                    false
                } else {
                    (self.rng.next_double() as f32) < self.config.guard_miss_chance
                };

                if !miss {
                    let chest = target.position + Vec3::new(0.0, GuardRifle::CHEST_HEIGHT, 0.0);
                    let to_chest = chest - head;
                    let dir = to_chest.normalize_or_zero();

                    frame.shots.push(GuardShot {
                        guard_id: guard.id.clone(),
                        target_id: target.id,
                        from: head,
                        to: chest,
                        damage: DamageInfo {
                            amount: self.config.guard_damage,
                            source: None,
                            hit_point: chest,
                            direction: dir,
                            damage_type: GuardRifle::DAMAGE_TYPE.to_string(),
                            headshot: false,
                            leg_hit: false,
                        },
                    });
                }
            }
        }

        frame
    }

    /// Convenience wrapper that assumes clear line of sight for all rays.
    pub fn update_clear(
        &mut self,
        dt: f32,
        guards: &[GuardPost],
        targets: &[GuardTarget],
    ) -> GuardFrame {
        self.update(dt, guards, targets, &|_, _| true)
    }

    /// Finds the nearest eligible live target in range with clear line of sight.
    // Godot: SettlementGuard.cs FindTarget
    fn find_target(
        &self,
        guard: &GuardPost,
        targets: &[GuardTarget],
        los: &dyn Fn(Vec3, Vec3) -> bool,
    ) -> Option<u64> {
        let head = guard.position + Vec3::new(0.0, self.config.guard_eye_height, 0.0);
        let camp_center = self.camp_center.unwrap_or(guard.position);

        let mut best: Option<u64> = None;
        let mut best_distance = f32::MAX;

        for target in targets {
            if !target.alive {
                continue;
            }

            // Target must be within guardRange of the camp centre
            // Godot: FlatDistance(at, _site.Center) > _cfg.GuardRange
            let flat_dist = Vec2::new(
                target.position.x - camp_center.x,
                target.position.z - camp_center.z,
            )
            .length();
            if flat_dist > self.config.guard_range {
                continue;
            }

            // Nearest target by distance from head to target position
            // Godot: float distance = head.DistanceTo(at);
            let distance = head.distance(target.position);
            if distance >= best_distance {
                continue;
            }

            // Line of sight check to target chest
            // Godot: !HasLine(head, at + Vector3.Up * ChestHeight)
            let chest = target.position + Vec3::new(0.0, GuardRifle::CHEST_HEIGHT, 0.0);
            if !los(head, chest) {
                continue;
            }

            best = Some(target.id);
            best_distance = distance;
        }

        best
    }
}
