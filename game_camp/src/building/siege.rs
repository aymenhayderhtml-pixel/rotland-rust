//! Pure base-siege simulation: zombies chasing the player damage blocking building parts.

use std::collections::HashMap;

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use super::def::BuildConfig;
use super::state::BaseState;

/// Minimum horizontal cosine between zombie->part and zombie->player for a part to count as blocking.
const MIN_FORWARD_DOT: f32 = 0.25; // new: DECISIONS 2026-11-23
/// Squared horizontal distance threshold below which direction is treated as degenerate.
const DIR_EPSILON_SQ: f32 = 1e-6; // new: DECISIONS 2026-11-23

fn default_active_range() -> f32 {
    // data: siege.json activeRange
    60.0
}

fn default_reach() -> f32 {
    // data: siege.json reach
    1.6
}

fn default_hit_interval() -> f32 {
    // data: siege.json hitInterval
    1.2
}

fn default_door_multiplier() -> f32 {
    // data: siege.json doorMultiplier
    1.5
}

fn default_alert_seconds() -> f32 {
    // data: siege.json alertSeconds
    5.0
}

fn default_max_attackers() -> usize {
    // data: siege.json maxAttackers
    24
}

fn default_damage_value() -> f32 {
    // data: siege.json defaultDamage
    1.5
}

fn default_damage_map() -> HashMap<String, f32> {
    HashMap::from([
        ("walker".to_string(), 2.0),   // data: siege.json damage.walker
        ("runner".to_string(), 1.5),   // data: siege.json damage.runner
        ("crawler".to_string(), 1.0),  // data: siege.json damage.crawler
        ("screamer".to_string(), 0.8), // data: siege.json damage.screamer
        ("brute".to_string(), 18.0),   // data: siege.json damage.brute
    ])
}

/// Configuration for zombie attacks on player-built base parts, loaded from `data/siege.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SiegeConfig {
    #[serde(default = "default_active_range")]
    pub active_range: f32,
    #[serde(default = "default_reach")]
    pub reach: f32,
    #[serde(default = "default_hit_interval")]
    pub hit_interval: f32,
    #[serde(default = "default_door_multiplier")]
    pub door_multiplier: f32,
    #[serde(default = "default_alert_seconds")]
    pub alert_seconds: f32,
    #[serde(default = "default_max_attackers")]
    pub max_attackers: usize,
    #[serde(default = "default_damage_map")]
    pub damage: HashMap<String, f32>,
    #[serde(default = "default_damage_value")]
    pub default_damage: f32,
}

impl Default for SiegeConfig {
    fn default() -> Self {
        Self {
            active_range: default_active_range(),
            reach: default_reach(),
            hit_interval: default_hit_interval(),
            door_multiplier: default_door_multiplier(),
            alert_seconds: default_alert_seconds(),
            max_attackers: default_max_attackers(),
            damage: default_damage_map(),
            default_damage: default_damage_value(),
        }
    }
}

impl SiegeConfig {
    /// Parses a siege configuration from JSON text.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Loads `data/siege.json` from the repository root via `paths::repo_path`.
    pub fn from_repo() -> Result<Self, std::io::Error> {
        let path = paths::repo_path("data/siege.json");
        let text = std::fs::read_to_string(path)?;
        Self::from_json(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    /// Returns the base hit damage for a zombie type, falling back to `default_damage`.
    pub fn damage_for(&self, kind: &str) -> f32 {
        self.damage
            .get(kind)
            .copied()
            .unwrap_or(self.default_damage)
            .max(0.0)
    }
}

/// Snapshot of a single zombie that may attack the player's base during a tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SiegeAttacker<'a> {
    pub id: u32,
    pub kind: &'a str,
    pub position: Vec3,
    pub chasing: bool,
}

/// Event emitted during a base-siege simulation tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SiegeEvent {
    Hit {
        attacker: u32,
        part_id: u32,
        amount: f32,
    },
    Broken {
        part_id: u32,
        position: Vec3,
    },
    Collapsed {
        part_id: u32,
    },
}

/// Result of one [`SiegeSystem::tick`] call.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SiegeFrame {
    pub events: Vec<SiegeEvent>,
    pub geometry_changed: bool,
}

/// Tracks per-zombie hit cadence and the player's base-under-attack alert timer.
#[derive(Debug, Clone)]
pub struct SiegeSystem {
    cfg: SiegeConfig,
    /// Per-attacker `(elapsed_seconds, last_seen_tick_stamp)`.
    timers: HashMap<u32, (f32, u64)>,
    /// Reusable scratch list of `(attacker_index, dist_sq_to_player)` to avoid per-tick allocations.
    candidates: Vec<(usize, f32)>,
    alert_remaining: f32,
    tick_stamp: u64,
}

impl SiegeSystem {
    /// Creates a new siege simulation system with the given configuration.
    pub fn new(cfg: SiegeConfig) -> Self {
        Self {
            cfg,
            timers: HashMap::new(),
            candidates: Vec::new(),
            alert_remaining: 0.0,
            tick_stamp: 0,
        }
    }

    /// Advances base-siege timers, applies hits to blocking parts, and returns frame events.
    pub fn tick(
        &mut self,
        base: &mut BaseState,
        build: &BuildConfig,
        attackers: &[SiegeAttacker<'_>],
        player: Vec3,
        dt: f32,
    ) -> SiegeFrame {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        self.alert_remaining = (self.alert_remaining - dt).max(0.0);

        if base.is_empty() {
            self.timers.clear();
            return SiegeFrame::default();
        }

        let mut frame = SiegeFrame::default();
        if self.cfg.max_attackers == 0 || self.cfg.active_range < 0.0 || attackers.is_empty() {
            self.timers.clear();
            return frame;
        }

        let active_range_sq = self.cfg.active_range * self.cfg.active_range;
        self.candidates.clear();
        for (idx, attacker) in attackers.iter().enumerate() {
            if !attacker.chasing {
                continue;
            }
            let dist_sq = attacker.position.distance_squared(player);
            if dist_sq <= active_range_sq {
                self.candidates.push((idx, dist_sq));
            }
        }

        if self.candidates.is_empty() {
            self.timers.clear();
            return frame;
        }

        let cmp = |a: &(usize, f32), b: &(usize, f32)| {
            a.1.total_cmp(&b.1)
                .then_with(|| attackers[a.0].id.cmp(&attackers[b.0].id))
        };
        let max = self.cfg.max_attackers;
        if self.candidates.len() > max {
            self.candidates.select_nth_unstable_by(max - 1, cmp);
            self.candidates.truncate(max);
        }
        self.candidates.sort_unstable_by(cmp);

        self.tick_stamp = self.tick_stamp.wrapping_add(1);
        let stamp = self.tick_stamp;
        let interval = self.cfg.hit_interval;

        let SiegeSystem {
            cfg,
            timers,
            candidates,
            alert_remaining,
            ..
        } = self;

        for &(idx, _) in candidates.iter() {
            if base.is_empty() {
                break;
            }
            let attacker = &attackers[idx];

            let Some(target_id) =
                find_target_part(base, build, attacker.position, player, cfg.reach)
            else {
                continue;
            };

            let mut should_hit = false;
            if let Some((elapsed, last_stamp)) = timers.get_mut(&attacker.id) {
                if *last_stamp == stamp {
                    continue;
                }
                *last_stamp = stamp;
                *elapsed += dt;
                if interval <= 0.0 {
                    *elapsed = 0.0;
                    should_hit = true;
                } else if *elapsed >= interval {
                    *elapsed = (*elapsed - interval).clamp(0.0, interval);
                    should_hit = true;
                }
            } else {
                let mut elapsed = dt;
                if interval <= 0.0 {
                    elapsed = 0.0;
                    should_hit = true;
                } else if elapsed >= interval {
                    elapsed = (elapsed - interval).clamp(0.0, interval);
                    should_hit = true;
                }
                timers.insert(attacker.id, (elapsed, stamp));
            }

            if !should_hit {
                continue;
            }

            let is_door = base
                .get(target_id)
                .and_then(|p| build.get(&p.part))
                .is_some_and(|d| d.is_door);
            let base_damage = cfg.damage_for(attacker.kind);
            let amount = if is_door {
                base_damage * cfg.door_multiplier.max(0.0)
            } else {
                base_damage
            };
            if amount <= 0.0 {
                continue;
            }

            let Some(part) = base.get_mut(target_id) else {
                continue;
            };
            let part_pos = part.position;
            let broken = part.hp <= amount;
            if !broken {
                part.hp -= amount;
            }

            frame.events.push(SiegeEvent::Hit {
                attacker: attacker.id,
                part_id: target_id,
                amount,
            });
            *alert_remaining = cfg.alert_seconds.max(0.0);

            if broken {
                base.parts.retain(|p| p.id != target_id);
                frame.events.push(SiegeEvent::Broken {
                    part_id: target_id,
                    position: part_pos,
                });
                frame.geometry_changed = true;

                for collapsed_id in base.collapse_unsupported() {
                    frame.events.push(SiegeEvent::Collapsed {
                        part_id: collapsed_id,
                    });
                }
            }
        }

        timers.retain(|_, (_, last_stamp)| *last_stamp == stamp);
        frame
    }

    /// True while the base has been hit within the last `alertSeconds`.
    pub fn under_attack(&self) -> bool {
        self.alert_remaining > 0.0
    }

    /// Alert fade factor: `1.0` immediately after a hit, fading linearly to `0.0`.
    pub fn alert_fraction(&self) -> f32 {
        if self.cfg.alert_seconds <= 0.0 || self.alert_remaining <= 0.0 {
            0.0
        } else {
            (self.alert_remaining / self.cfg.alert_seconds).clamp(0.0, 1.0)
        }
    }

    /// Number of attackers currently tracked in the internal timer map.
    pub fn tracked_attackers(&self) -> usize {
        self.timers.len()
    }

    /// Resets all per-attacker timers and clears the alert (call on player death or save load).
    pub fn clear(&mut self) {
        self.timers.clear();
        self.candidates.clear();
        self.alert_remaining = 0.0;
    }
}

/// Finds the nearest blocking part within `reach` that lies roughly between the attacker and player.
fn find_target_part(
    base: &BaseState,
    build: &BuildConfig,
    attacker_pos: Vec3,
    player: Vec3,
    reach: f32,
) -> Option<u32> {
    if reach <= 0.0 {
        return None;
    }

    let to_player_xz = Vec2::new(player.x - attacker_pos.x, player.z - attacker_pos.z);
    let player_dist_sq = to_player_xz.length_squared();
    if player_dist_sq <= DIR_EPSILON_SQ {
        return None;
    }
    let player_dist = player_dist_sq.sqrt();
    let dir_to_player = to_player_xz / player_dist;
    let reach_sq = reach * reach;

    let mut best: Option<(u32, f32)> = None;

    for part in &base.parts {
        if part.door_open {
            continue;
        }

        let to_part_xz = Vec2::new(
            part.position.x - attacker_pos.x,
            part.position.z - attacker_pos.z,
        );
        let horiz_sq = to_part_xz.length_squared();
        if horiz_sq > reach_sq {
            continue;
        }

        if horiz_sq > DIR_EPSILON_SQ {
            let proj = to_part_xz.dot(dir_to_player);
            if proj <= 0.0 || proj > player_dist {
                continue;
            }
            let horiz_dist = horiz_sq.sqrt();
            if proj / horiz_dist < MIN_FORWARD_DOT {
                continue;
            }
        }

        let Some(def) = build.get(&part.part) else {
            continue;
        };
        let min_y = part.position.y;
        let max_y = part.position.y + def.height.max(0.0);
        let dy = if attacker_pos.y < min_y {
            min_y - attacker_pos.y
        } else if attacker_pos.y > max_y {
            attacker_pos.y - max_y
        } else {
            0.0
        };
        let dist_sq = horiz_sq + dy * dy;
        if dist_sq > reach_sq {
            continue;
        }

        if !base.blocks_zombies(build, part.id) {
            continue;
        }

        let is_better = match best {
            None => true,
            Some((best_id, best_dist_sq)) => {
                dist_sq < best_dist_sq || (dist_sq == best_dist_sq && part.id < best_id)
            }
        };
        if is_better {
            best = Some((part.id, dist_sq));
        }
    }

    best.map(|(id, _)| id)
}
