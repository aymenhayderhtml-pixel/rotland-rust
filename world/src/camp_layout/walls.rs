//! Wall ring and gate placement logic for camp upgrades.

use std::f32::consts::{FRAC_PI_2, PI};
use crate::scatter::PropInstance;
use crate::settlement::SettlementSite;
use super::types::CampLayoutConfigData;

/// Computes the number of wall sections in the closed ring (excluding the gate gap).
pub fn wall_section_count(wall_radius: f32, section_length: f32, gate_width: f32) -> usize {
    let sections_per_side = (2.0 * wall_radius / section_length).round() as usize;
    let gate_sections = (gate_width / section_length).round() as usize;
    4 * sections_per_side - gate_sections
}

pub(crate) fn wall_models(level: u8) -> Option<(&'static str, &'static str)> {
    match level {
        1 => Some(("camp_wall_wood", "camp_gate_wood")),
        2 => Some(("camp_wall_metal", "camp_gate_metal")),
        3.. => Some(("camp_wall_concrete", "camp_gate_concrete")),
        0 => None,
    }
}

pub(crate) fn place_wall_ring(
    site: &SettlementSite,
    wall_level: u8,
    cfg: &CampLayoutConfigData,
    out: &mut Vec<PropInstance>,
) {
    let Some((wall_model, gate_model)) = wall_models(wall_level) else {
        return;
    };

    let radius = cfg.wall_radius;
    let sec_len = cfg.wall_section_length;
    let gate_w = cfg.gate_width;
    let half_gate = gate_w * 0.5;

    let count = (2.0 * radius / sec_len).round() as usize;
    let start = -radius + sec_len * 0.5;

    // 1. North side (z = -radius): facing entrance with gate in the center
    let gate_pos = site.center + site.local(0.0, 0.0, -radius);
    out.push(PropInstance::new(gate_model, gate_pos, site.yaw, 1.0));

    for i in 0..count {
        let coord = start + i as f32 * sec_len;
        if coord.abs() >= half_gate {
            let pos = site.center + site.local(coord, 0.0, -radius);
            out.push(PropInstance::new(wall_model, pos, site.yaw, 1.0));
        }
    }

    // 2. South side (z = +radius): outside faces South (+Z)
    let south_yaw = site.yaw + PI;
    for i in 0..count {
        let coord = start + i as f32 * sec_len;
        let pos = site.center + site.local(coord, 0.0, radius);
        out.push(PropInstance::new(wall_model, pos, south_yaw, 1.0));
    }

    // 3. East side (x = +radius): outside faces East (+X)
    let east_yaw = site.yaw - FRAC_PI_2;
    for i in 0..count {
        let coord = start + i as f32 * sec_len;
        let pos = site.center + site.local(radius, 0.0, coord);
        out.push(PropInstance::new(wall_model, pos, east_yaw, 1.0));
    }

    // 4. West side (x = -radius): outside faces West (-X)
    let west_yaw = site.yaw + FRAC_PI_2;
    for i in 0..count {
        let coord = start + i as f32 * sec_len;
        let pos = site.center + site.local(-radius, 0.0, coord);
        out.push(PropInstance::new(wall_model, pos, west_yaw, 1.0));
    }
}
