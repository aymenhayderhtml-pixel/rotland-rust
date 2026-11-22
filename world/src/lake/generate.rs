use glam::Vec2;

use crate::lake::config::VillageConfig;
use crate::lake::types::{Dock, FishingSpot, FishingSpotKind, VillageLot};
use crate::noise::GodotNoise;

#[inline]
pub(crate) fn ellipse_radius(u: Vec2, radius: Vec2) -> f32 {
    let qx = u.x / radius.x;
    let qy = u.y / radius.y;
    1.0 / (qx * qx + qy * qy).max(1e-6).sqrt()
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_village_lots(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    village_cfg: &VillageConfig,
    beach_width: f32,
    noise_amp: f32,
    noise: &GodotNoise,
    water_level: f32,
    deck_height: f32,
    raw_height: &impl Fn(Vec2) -> f32,
) -> Vec<VillageLot> {
    let mut lots = Vec::with_capacity(village_cfg.count);
    let span = 1.0;
    for i in 0..village_cfg.count {
        let t = if village_cfg.count > 1 {
            (i as f32) / (village_cfg.count as f32 - 1.0) - 0.5
        } else {
            0.0
        };
        let angle = base_angle + t * span;
        let u = Vec2::new(angle.cos(), angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;

        let size_arr = village_cfg.sizes[i % village_cfg.sizes.len()];
        let size = Vec2::new(size_arr[0], size_arr[1]);
        let distance_inland = beach_width + size.y * 0.5 + 4.0;
        let lot_center = shore_pt + u * distance_inland;

        let forward = -u;
        let yaw = (-forward.x).atan2(-forward.y);

        let touches_water = {
            let right = Vec2::new(-forward.y, forward.x);
            let hx = right * (size.x * 0.5);
            let hy = forward * (size.y * 0.5);
            let corners = [
                lot_center - hx - hy,
                lot_center + hx - hy,
                lot_center + hx + hy,
                lot_center - hx + hy,
            ];
            corners.iter().any(|&c| {
                let d = c - center;
                let dist = d.length();
                let dir_u = if dist > 1e-5 { d / dist } else { Vec2::new(1.0, 0.0) };
                let r_b = ellipse_radius(dir_u, radius);
                let n_val = noise.get_noise_2d(center.x + dir_u.x * r_b, center.y + dir_u.y * r_b);
                dist <= r_b + n_val * noise_amp
            })
        };

        const INLAND_BUILDINGS: [&str; 4] = ["fish_shack", "house_small", "cabin", "shed"];
        let building_id = if touches_water {
            "boathouse".to_string()
        } else {
            INLAND_BUILDINGS[i % INLAND_BUILDINGS.len()].to_string()
        };

        let floor_y = if touches_water {
            water_level + deck_height
        } else {
            raw_height(lot_center)
        };

        lots.push(VillageLot {
            center: lot_center,
            size,
            yaw,
            building_id,
            floor_y,
        });
    }
    lots
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn generate_docks(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    count: usize,
    length: f32,
    noise_amp: f32,
    noise: &GodotNoise,
    water_level: f32,
    deck_height: f32,
) -> Vec<Dock> {
    let mut docks = Vec::with_capacity(count);
    for j in 0..count {
        let t = if count > 1 {
            (j as f32 / (count as f32 - 1.0) - 0.5) * 0.5
        } else {
            0.0
        };
        let dock_angle = base_angle + t;
        let u = Vec2::new(dock_angle.cos(), dock_angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;

        let forward = -u;
        let start = shore_pt + u * 2.0;
        let end = start + forward * length;
        let yaw = (-forward.x).atan2(-forward.y);
        let deck_y = water_level + deck_height;

        docks.push(Dock { start, end, yaw, deck_y });
    }
    docks
}

pub(crate) fn generate_fishing_spots(
    center: Vec2,
    radius: Vec2,
    base_angle: f32,
    docks: &[Dock],
    beach_width: f32,
    noise_amp: f32,
    noise: &GodotNoise,
) -> Vec<FishingSpot> {
    let mut spots = Vec::new();
    for dock in docks {
        spots.push(FishingSpot {
            position: dock.end,
            kind: FishingSpotKind::Dock,
        });
    }

    let beach_angles = [
        base_angle - 0.5,
        base_angle - 0.25,
        base_angle + 0.25,
        base_angle + 0.5,
    ];
    for angle in beach_angles {
        let u = Vec2::new(angle.cos(), angle.sin());
        let r_base = ellipse_radius(u, radius);
        let noise_val = noise.get_noise_2d(center.x + u.x * r_base, center.y + u.y * r_base);
        let r_shore = r_base + noise_val * noise_amp;
        let shore_pt = center + u * r_shore;
        let beach_pos = shore_pt + u * (beach_width * 0.5);
        spots.push(FishingSpot {
            position: beach_pos,
            kind: FishingSpotKind::Beach,
        });
    }

    spots
}
