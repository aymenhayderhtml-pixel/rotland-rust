//! Remembers where zombies died recently, so an idle zombie nearby may stop to eat (port of ZombieCorpses.cs).

use glam::{Vec2, Vec3};

const MAX_SPOTS: usize = 16;

/// Remembers recent zombie death spots for eating behaviours.
#[derive(Clone, Debug, Default)]
pub struct ZombieCorpses {
    spots: Vec<(Vec3, f64)>,
}

impl ZombieCorpses {
    pub fn new() -> Self {
        Self {
            spots: Vec::with_capacity(MAX_SPOTS),
        }
    }

    /// Record a corpse position at the given game time in seconds.
    pub fn record(&mut self, position: Vec3, time: f64) {
        self.spots.push((position, time));
        if self.spots.len() > MAX_SPOTS {
            self.spots.remove(0);
        }
    }

    /// True when a zombie died within `radius` (flat distance) of `point` in the last `seconds`.
    pub fn near_recent(&self, point: Vec3, radius: f32, seconds: f32, now: f64) -> bool {
        for &(spot, time) in &self.spots {
            if now - time <= seconds as f64 {
                let d = Vec2::new(spot.x - point.x, spot.z - point.z).length();
                if d <= radius {
                    return true;
                }
            }
        }
        false
    }

    pub fn clear(&mut self) {
        self.spots.clear();
    }

    /// Fills `out` with all corpse positions recorded within `seconds` of `now`.
    pub fn recent_positions(&self, seconds: f32, now: f64, out: &mut Vec<Vec3>) {
        out.clear();
        for &(spot, time) in &self.spots {
            if now - time <= seconds as f64 {
                out.push(spot);
            }
        }
    }
}
