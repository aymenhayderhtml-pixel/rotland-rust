//! Notification queue widget for events: "Upgrade finished", "Horde incoming", "Bounty complete".

use engine::egui::{self, pos2, vec2, Align2, FontId, Rect, Stroke};
use serde::{Deserialize, Serialize};
use super::config::{color_with_alpha, parse_hex_color, HudConfig};
use super::text::draw_outline_text;

/// Kind of notification event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum NotificationKind {
    #[default]
    Info,
    Warning,
    Danger,
}

/// A single active event notification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HudNotification {
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub kind: NotificationKind,
    #[serde(default)]
    pub spawn_time: f64,
    #[serde(default)]
    pub duration: f32,
}

impl HudNotification {
    pub fn new(text: impl Into<String>, kind: NotificationKind, spawn_time: f64, duration: f32) -> Self {
        Self {
            text: text.into(),
            kind,
            spawn_time,
            duration,
        }
    }

    /// Remaining alpha fraction [0.0, 1.0].
    pub fn alpha(&self, now: f64) -> f32 {
        let elapsed = (now - self.spawn_time).max(0.0) as f32;
        game::hud_math::damage_alpha(elapsed, self.duration)
    }

    /// Whether the notification has expired.
    pub fn is_expired(&self, now: f64) -> bool {
        (now - self.spawn_time) as f32 >= self.duration
    }
}

/// Small queue of notifications displayed at top center under the compass.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HudNotifications {
    #[serde(default)]
    pub items: Vec<HudNotification>,
}

impl HudNotifications {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Pushes a new notification with default duration (4.0s).
    #[allow(dead_code)]
    pub fn push(&mut self, text: impl Into<String>, kind: NotificationKind) {
        self.push_with_duration(text, kind, 0.0, 4.0);
    }

    /// Pushes a new notification with timestamp and duration.
    #[allow(dead_code)]
    pub fn push_at(&mut self, text: impl Into<String>, kind: NotificationKind, now: f64) {
        self.push_with_duration(text, kind, now, 4.0);
    }

    /// Pushes a new notification with explicit duration.
    #[allow(dead_code)]
    pub fn push_with_duration(
        &mut self,
        text: impl Into<String>,
        kind: NotificationKind,
        now: f64,
        duration: f32,
    ) {
        self.items.push(HudNotification::new(text, kind, now, duration));
        if self.items.len() > 3 {
            self.items.remove(0);
        }
    }

    /// Removes expired notifications.
    #[allow(dead_code)]
    pub fn update(&mut self, now: f64) {
        self.items.retain(|n| !n.is_expired(now));
    }

    /// Returns the number of currently active notifications.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if there are no active notifications.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

/// Returns the primary color for a notification kind from config.
#[allow(dead_code)]
pub fn notification_color(kind: NotificationKind, config: &HudConfig) -> engine::egui::Color32 {
    let cfg = &config.notifications;
    match kind {
        NotificationKind::Info => parse_hex_color(&cfg.info_color),
        NotificationKind::Warning => parse_hex_color(&cfg.warning_color),
        NotificationKind::Danger => parse_hex_color(&cfg.danger_color),
    }
}

/// Draws active notifications in a column top-center under the compass.
pub fn draw_notifications(
    painter: &egui::Painter,
    screen: &Rect,
    notifications: &[HudNotification],
    now: f64,
    scale: f32,
    config: &HudConfig,
) {
    if notifications.is_empty() {
        return;
    }
    let cfg = &config.notifications;
    let base_y = 65.0 * scale;

    let mut drawn_idx = 0;
    for notif in notifications {
        let alpha = notif.alpha(now);
        if alpha <= 0.0 {
            continue;
        }

        let font = FontId::proportional(cfg.font_size_px * scale);
        let text_color = color_with_alpha(notification_color(notif.kind, config), alpha);
        let bg_color = color_with_alpha(parse_hex_color(&cfg.bg_color), alpha);

        // Approximate pill width based on text length + padding
        let char_w = cfg.font_size_px * 0.58 * scale;
        let text_w = notif.text.len() as f32 * char_w;
        let pill_w = (text_w + 36.0 * scale).max(140.0 * scale);
        let pill_h = cfg.height_px * scale;
        let y = base_y + drawn_idx as f32 * (pill_h + 4.0 * scale);

        let rect = Rect::from_center_size(pos2(screen.center().x, y + pill_h * 0.5), vec2(pill_w, pill_h));

        // Background and accent border
        painter.rect_filled(rect, cfg.corner_radius_px * scale, bg_color);
        painter.rect_stroke(
            rect,
            cfg.corner_radius_px * scale,
            Stroke::new(1.0 * scale, text_color),
            engine::egui::StrokeKind::Inside,
        );

        // Kind indicator dot on the left of pill
        let dot_pos = pos2(rect.min.x + 12.0 * scale, rect.center().y);
        painter.circle_filled(dot_pos, 4.0 * scale, text_color);

        // Text centered
        draw_outline_text(
            painter,
            pos2(rect.min.x + 22.0 * scale, rect.center().y),
            Align2::LEFT_CENTER,
            &notif.text,
            font,
            text_color,
        );

        drawn_idx += 1;
        if drawn_idx >= cfg.max_queue {
            break;
        }
    }
}
