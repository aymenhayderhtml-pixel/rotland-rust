//! Damage direction indicators: red arc on the screen edge pointing toward the attacker.

use engine::egui::{self, pos2, Rect, Stroke};
use glam::Vec3;
use super::config::HudConfig;

/// A single active damage direction indicator.
#[derive(Debug, Clone, PartialEq)]
pub struct DamageIndicator {
    pub attacker_pos: Vec3,
    pub spawn_time: f64,
    pub duration: f32,
}

impl DamageIndicator {
    pub fn new(attacker_pos: Vec3, spawn_time: f64, duration: f32) -> Self {
        Self {
            attacker_pos,
            spawn_time,
            duration,
        }
    }

    /// Whether this indicator has finished its fade duration.
    pub fn is_expired(&self, now: f64) -> bool {
        (now - self.spawn_time) as f32 >= self.duration
    }

    /// Remaining alpha fraction [0.0, 1.0].
    pub fn alpha(&self, now: f64) -> f32 {
        let elapsed = (now - self.spawn_time).max(0.0) as f32;
        game::hud_math::damage_alpha(elapsed, self.duration)
    }
}

/// Draws all active damage direction indicators around the screen edges.
#[allow(clippy::too_many_arguments)]
pub fn draw_damage_indicators(
    painter: &egui::Painter,
    screen: &Rect,
    indicators: &[DamageIndicator],
    player_pos: Vec3,
    camera_yaw: f32,
    now: f64,
    scale: f32,
    config: &HudConfig,
) {
    let cfg = &config.damage_direction;
    let center = screen.center();
    let margin = 24.0 * scale;
    let radius_x = (center.x - margin).max(10.0) * cfg.arc_radius_fraction;
    let radius_y = (center.y - margin).max(10.0) * cfg.arc_radius_fraction;
    let span_rad = cfg.arc_span_deg.to_radians();
    let thickness = cfg.arc_thickness_px * scale;
    let segments = 16;

    for ind in indicators {
        let alpha = ind.alpha(now);
        if alpha <= 0.0 {
            continue;
        }

        let angle = game::hud_math::damage_direction_angle_rad(
            player_pos.x,
            player_pos.z,
            ind.attacker_pos.x,
            ind.attacker_pos.z,
            camera_yaw,
        );

        let pts = game::hud_math::damage_arc_points(
            center.x,
            center.y,
            radius_x,
            radius_y,
            angle,
            span_rad,
            segments,
        );

        let color = cfg.color32(alpha);
        let stroke = Stroke::new(thickness, color);

        // Draw curved arc as a single polyline so semi-transparent segments don't overlap at joints.
        let arc_pts: Vec<egui::Pos2> = pts.iter().map(|&(x, y)| pos2(x, y)).collect();
        painter.add(egui::Shape::line(arc_pts, stroke));

        // Outward pointer chevron at arc midpoint
        let mid_idx = pts.len() / 2;
        let mid = pts[mid_idx];
        let (dir_x, dir_y) = game::hud_math::damage_screen_direction(angle);
        let tip = pos2(mid.0 + dir_x * 8.0 * scale, mid.1 + dir_y * 8.0 * scale);
        let left_wing = pos2(mid.0 - dir_y * 6.0 * scale, mid.1 + dir_x * 6.0 * scale);
        let right_wing = pos2(mid.0 + dir_y * 6.0 * scale, mid.1 - dir_x * 6.0 * scale);
        painter.add(egui::Shape::line(vec![left_wing, tip, right_wing], stroke));
    }
}
