use std::collections::HashMap;
use glam::{Quat, Vec3};

use crate::damage::DamageInfo;
use crate::vehicles::config::VehicleConfigData;

const COOLDOWN_FRAMES: u64 = 60;

/// Hits targets in the car's path (port of scripts/Vehicles/CarRunOver.cs).
#[derive(Clone, Debug, Default)]
pub struct CarRunOver {
    pub(crate) hits: u32,
    frame: u64,
    last_hit_frame: HashMap<u64, u64>,
}

impl CarRunOver {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn hits(&self) -> u32 {
        self.hits
    }

    /// Checks targets against the car's path, returning hits (entity_id, damage_info, knockback).
    pub fn step(
        &mut self,
        car_pos: Vec3,
        car_rot: Quat,
        speed_kmh: f32,
        is_active: bool,
        is_wrecked: bool,
        cfg: &VehicleConfigData,
        targets: &[(u64, Vec3)],
    ) -> Vec<(u64, DamageInfo, Vec3)> {
        self.frame += 1;
        if !is_active || is_wrecked || speed_kmh < cfg.run_over_min_kmh {
            return Vec::new();
        }

        let inv_rot = car_rot.inverse();
        let mut results = Vec::new();

        for &(id, target_pos) in targets {
            let local = inv_rot * (target_pos - car_pos);
            let in_path = local.x.abs() <= cfg.hit_half_width
                && local.z >= -cfg.hit_front
                && local.z <= cfg.hit_back;

            if !in_path || self.recent(id) {
                continue;
            }

            self.last_hit_frame.insert(id, self.frame);
            self.hits += 1;

            let mut away = target_pos - car_pos;
            away.y = 0.0;
            let away = if away.length_squared() < 1e-4 {
                car_rot * -Vec3::Z
            } else {
                away.normalize()
            };
            let knockback = away * cfg.run_over_knockback_mps + Vec3::Y * 1.5;
            let damage_amount = speed_kmh * cfg.run_over_damage_per_kmh;

            let info = DamageInfo {
                amount: damage_amount,
                source: None,
                hit_point: target_pos,
                direction: away,
                damage_type: "car".to_string(),
                headshot: false,
                leg_hit: false,
            };

            results.push((id, info, knockback));
        }

        self.prune();
        results
    }

    fn recent(&self, id: u64) -> bool {
        self.last_hit_frame
            .get(&id)
            .is_some_and(|&f| self.frame - f < COOLDOWN_FRAMES)
    }

    fn prune(&mut self) {
        if self.last_hit_frame.len() < 32 {
            return;
        }
        let current_frame = self.frame;
        self.last_hit_frame
            .retain(|_, &mut f| current_frame - f < COOLDOWN_FRAMES);
    }
}
