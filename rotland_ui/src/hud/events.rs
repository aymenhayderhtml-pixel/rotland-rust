//! Event notifications, direction words, cooldown timers, and horde screen-edge arrows.

use engine::egui::{self, pos2, Rect, Stroke};
use glam::Vec3;
use super::config::{color_with_alpha, parse_hex_color, HudConfig};

/// Maps a bearing in degrees [0.0, 360.0) to an 8-direction name.
/// N = 0°, NE = 45°, E = 90°, SE = 135°, S = 180°, SW = 225°, W = 270°, NW = 315°.
pub fn direction_word_from_angle(bearing_deg: f32) -> &'static str {
    // 22.5 half-width window around each 45-degree sector
    let norm = (bearing_deg.rem_euclid(360.0) + 22.5).rem_euclid(360.0);
    let idx = (norm / 45.0) as usize % 8;
    match idx {
        0 => "north",
        1 => "north-east",
        2 => "east",
        3 => "south-east",
        4 => "south",
        5 => "south-west",
        6 => "west",
        _ => "north-west",
    }
}

/// Computes the 8-direction word from observer `from` to world target `to` on flat ground (X/Z).
/// Follows Godot coordinate convention: yaw 0 looks North (-Z), East is +X.
pub fn direction_word_to_target(from: Vec3, to: Vec3) -> &'static str {
    let dx = to.x - from.x;
    let dz = to.z - from.z;
    // North is -Z, so -dz is positive northward
    let bearing_deg = dx.atan2(-dz).to_degrees();
    direction_word_from_angle(bearing_deg)
}

/// Category of HUD event for cooldown rate limiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    HordeNear,
    SpecialSeen,
    CampAttack,
    WaveIncoming,
    PhaseChange,
}

/// Per-event-type cooldown timer tracker to avoid spamming notifications.
#[derive(Debug, Clone, PartialEq)]
pub struct EventCooldowns {
    pub horde_near: f64,
    pub special_seen: f64,
    pub camp_attack: f64,
    pub wave_incoming: f64,
    pub phase_change: f64,
}

impl Default for EventCooldowns {
    fn default() -> Self {
        // -1000.0 ensures immediate first trigger
        Self {
            horde_near: -1000.0,
            special_seen: -1000.0,
            camp_attack: -1000.0,
            wave_incoming: -1000.0,
            phase_change: -1000.0,
        }
    }
}

impl EventCooldowns {
    pub fn new() -> Self {
        Self::default()
    }

    /// Last trigger timestamp for the given event kind.
    pub fn last_trigger(&self, kind: EventKind) -> f64 {
        match kind {
            EventKind::HordeNear => self.horde_near,
            EventKind::SpecialSeen => self.special_seen,
            EventKind::CampAttack => self.camp_attack,
            EventKind::WaveIncoming => self.wave_incoming,
            EventKind::PhaseChange => self.phase_change,
        }
    }

    /// Whether enough time has elapsed since the last notification of this kind.
    pub fn can_trigger(&self, kind: EventKind, now: f64, cooldown_s: f32) -> bool {
        (now - self.last_trigger(kind)) >= cooldown_s as f64
    }

    /// Records a trigger timestamp for this event kind.
    pub fn record_trigger(&mut self, kind: EventKind, now: f64) {
        match kind {
            EventKind::HordeNear => self.horde_near = now,
            EventKind::SpecialSeen => self.special_seen = now,
            EventKind::CampAttack => self.camp_attack = now,
            EventKind::WaveIncoming => self.wave_incoming = now,
            EventKind::PhaseChange => self.phase_change = now,
        }
    }

    /// Attempts to trigger an event notification. If cooled down, updates timer and returns true.
    pub fn try_trigger(&mut self, kind: EventKind, now: f64, cooldown_s: f32) -> bool {
        if self.can_trigger(kind, now, cooldown_s) {
            self.record_trigger(kind, now);
            true
        } else {
            false
        }
    }
}

/// A screen-edge arrow indicator pointing toward an active nearby horde.
#[derive(Debug, Clone, PartialEq)]
pub struct HordeIndicator {
    pub horde_pos: Vec3,
    pub spawn_time: f64,
    pub duration: f32,
}

impl HordeIndicator {
    pub fn new(horde_pos: Vec3, spawn_time: f64, duration: f32) -> Self {
        Self {
            horde_pos,
            spawn_time,
            duration,
        }
    }

    pub fn is_expired(&self, now: f64) -> bool {
        (now - self.spawn_time) as f32 >= self.duration
    }

    pub fn alpha(&self, now: f64) -> f32 {
        let elapsed = (now - self.spawn_time).max(0.0) as f32;
        game::hud_math::damage_alpha(elapsed, self.duration)
    }
}

/// Draws small screen-edge indicator arrows pointing toward nearby hordes.
#[allow(clippy::too_many_arguments)]
pub fn draw_horde_arrows(
    painter: &egui::Painter,
    screen: &Rect,
    indicators: &[HordeIndicator],
    player_pos: Vec3,
    camera_yaw: f32,
    now: f64,
    scale: f32,
    config: &HudConfig,
) {
    if indicators.is_empty() {
        return;
    }
    let center = screen.center();
    let margin = 45.0 * scale; // new: DECISIONS 2026-11-21
    let radius_x = (center.x - margin).max(10.0);
    let radius_y = (center.y - margin).max(10.0);
    let arrow_size = 14.0 * scale; // new: DECISIONS 2026-11-21
    let horde_color = parse_hex_color(&config.notifications.horde_color);

    for ind in indicators {
        let alpha = ind.alpha(now);
        if alpha <= 0.0 {
            continue;
        }

        let angle = game::hud_math::damage_direction_angle_rad(
            player_pos.x,
            player_pos.z,
            ind.horde_pos.x,
            ind.horde_pos.z,
            camera_yaw,
        );

        let tip_x = center.x + radius_x * angle.sin();
        let tip_y = center.y - radius_y * angle.cos();
        let tip = pos2(tip_x, tip_y);

        let fwd_x = angle.sin();
        let fwd_y = -angle.cos();
        let right_x = -fwd_y;
        let right_y = fwd_x;

        let base_x = tip_x - fwd_x * arrow_size;
        let base_y = tip_y - fwd_y * arrow_size;
        let left_pt = pos2(base_x - right_x * arrow_size * 0.5, base_y - right_y * arrow_size * 0.5);
        let right_pt = pos2(base_x + right_x * arrow_size * 0.5, base_y + right_y * arrow_size * 0.5);

        let color = color_with_alpha(horde_color, alpha);
        let stroke = Stroke::new(2.5 * scale, color); // new: DECISIONS 2026-11-21
        painter.line_segment([left_pt, tip], stroke);
        painter.line_segment([right_pt, tip], stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hud_direction_words_from_angles() {
        assert_eq!(direction_word_from_angle(0.0), "north");
        assert_eq!(direction_word_from_angle(20.0), "north");
        assert_eq!(direction_word_from_angle(25.0), "north-east");
        assert_eq!(direction_word_from_angle(45.0), "north-east");
        assert_eq!(direction_word_from_angle(65.0), "north-east");
        assert_eq!(direction_word_from_angle(70.0), "east");
        assert_eq!(direction_word_from_angle(90.0), "east");
        assert_eq!(direction_word_from_angle(135.0), "south-east");
        assert_eq!(direction_word_from_angle(180.0), "south");
        assert_eq!(direction_word_from_angle(225.0), "south-west");
        assert_eq!(direction_word_from_angle(270.0), "west");
        assert_eq!(direction_word_from_angle(315.0), "north-west");
        assert_eq!(direction_word_from_angle(355.0), "north");
        assert_eq!(direction_word_from_angle(-45.0), "north-west");
    }

    #[test]
    fn test_hud_direction_words_to_targets() {
        let from = Vec3::ZERO;
        // North is -Z
        assert_eq!(direction_word_to_target(from, Vec3::new(0.0, 0.0, -10.0)), "north");
        // North-East is +X, -Z
        assert_eq!(direction_word_to_target(from, Vec3::new(10.0, 0.0, -10.0)), "north-east");
        // East is +X
        assert_eq!(direction_word_to_target(from, Vec3::new(10.0, 0.0, 0.0)), "east");
        // South-East is +X, +Z
        assert_eq!(direction_word_to_target(from, Vec3::new(10.0, 0.0, 10.0)), "south-east");
        // South is +Z
        assert_eq!(direction_word_to_target(from, Vec3::new(0.0, 0.0, 10.0)), "south");
        // South-West is -X, +Z
        assert_eq!(direction_word_to_target(from, Vec3::new(-10.0, 0.0, 10.0)), "south-west");
        // West is -X
        assert_eq!(direction_word_to_target(from, Vec3::new(-10.0, 0.0, 0.0)), "west");
        // North-West is -X, -Z
        assert_eq!(direction_word_to_target(from, Vec3::new(-10.0, 0.0, -10.0)), "north-west");
    }

    #[test]
    fn test_hud_event_cooldowns() {
        let mut cd = EventCooldowns::new();
        let kind = EventKind::HordeNear;

        // First trigger succeeds
        assert!(cd.try_trigger(kind, 10.0, 30.0));
        assert_eq!(cd.last_trigger(kind), 10.0);

        // Within cooldown window: fails
        assert!(!cd.can_trigger(kind, 20.0, 30.0));
        assert!(!cd.try_trigger(kind, 20.0, 30.0));
        assert_eq!(cd.last_trigger(kind), 10.0);

        // Just before cooldown: fails
        assert!(!cd.try_trigger(kind, 39.9, 30.0));

        // Exactly when cooldown expires: succeeds
        assert!(cd.can_trigger(kind, 40.0, 30.0));
        assert!(cd.try_trigger(kind, 40.0, 30.0));
        assert_eq!(cd.last_trigger(kind), 40.0);

        // Different event kinds are independent
        assert!(cd.try_trigger(EventKind::SpecialSeen, 40.0, 20.0));
        assert_eq!(cd.last_trigger(EventKind::SpecialSeen), 40.0);
    }
}
