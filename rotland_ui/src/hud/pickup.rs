//! Pickup popups: "+2 Canned Beans" with item icon stacking above hotbar on the right.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect};
use super::config::{color_with_alpha, HudConfig};
use super::icons::IconCache;
use super::text::draw_outline_text;

/// A single active pickup popup message.
#[derive(Debug, Clone, PartialEq)]
pub struct PickupPopup {
    pub item_id: String,
    pub label: String,
    pub count: u32,
    pub spawn_time: f64,
    pub duration: f32,
}

impl PickupPopup {
    pub fn new(item_id: impl Into<String>, label: impl Into<String>, count: u32, spawn_time: f64, duration: f32) -> Self {
        Self {
            item_id: item_id.into(),
            label: label.into(),
            count,
            spawn_time,
            duration,
        }
    }

    /// Returns remaining alpha fraction [0.0, 1.0].
    pub fn alpha(&self, now: f64) -> f32 {
        let elapsed = (now - self.spawn_time).max(0.0) as f32;
        game::hud_math::damage_alpha(elapsed, self.duration)
    }

    /// Whether the popup has faded out.
    pub fn is_expired(&self, now: f64) -> bool {
        (now - self.spawn_time) as f32 >= self.duration
    }
}

/// Helper function to convert snake_case or raw ID to Title Case words.
pub fn format_pickup_label(item_id: &str) -> String {
    item_id
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Adds or merges an item pickup into a queue of popups.
/// If `item_id` is already present, increments `count` and resets the timer to `now`.
pub fn push_or_merge_pickup(
    queue: &mut Vec<PickupPopup>,
    item_id: &str,
    label_opt: Option<&str>,
    count: u32,
    now: f64,
    duration: f32,
    max_visible: usize,
) {
    if let Some(existing) = queue.iter_mut().find(|p| p.item_id == item_id) {
        existing.count += count;
        existing.spawn_time = now;
        if let Some(lbl) = label_opt {
            existing.label = lbl.to_string();
        }
    } else {
        let label = label_opt
            .map(|s| s.to_string())
            .unwrap_or_else(|| format_pickup_label(item_id));
        queue.push(PickupPopup::new(item_id, label, count, now, duration));
        if queue.len() > max_visible {
            queue.remove(0);
        }
    }
}

/// Draws the pickup popups stacked on the right side above the hotbar.
#[allow(clippy::too_many_arguments)]
pub fn draw_pickup_popups(
    painter: &egui::Painter,
    ctx: &egui::Context,
    screen: &Rect,
    popups: &[PickupPopup],
    icons: &mut IconCache,
    now: f64,
    scale: f32,
    config: &HudConfig,
) {
    if popups.is_empty() {
        return;
    }
    let cfg = &config.pickup_popup;
    let hotbar_w = (6.0 * 56.0 + 5.0 * 6.0) * scale;
    // Right side above the hotbar
    let start_x = screen.center().x + hotbar_w * 0.5 - cfg.width_px * scale;
    let base_y = screen.max.y - 78.0 * scale;

    let mut drawn_idx = 0;
    // Render in reverse order so newest is at the bottom, closest to hotbar
    for popup in popups.iter().rev() {
        let alpha = popup.alpha(now);
        if alpha <= 0.0 {
            continue;
        }

        let y = base_y - (drawn_idx as f32 + 1.0) * (cfg.height_px + 4.0) * scale;
        let rect = Rect::from_min_size(
            pos2(start_x, y),
            vec2(cfg.width_px * scale, cfg.height_px * scale),
        );

        // Background pill
        painter.rect_filled(rect, cfg.corner_radius_px * scale, cfg.bg_color32(alpha));

        // Icon on the left
        let icon_size = (cfg.height_px - 6.0) * scale;
        let icon_rect = Rect::from_min_size(
            pos2(rect.min.x + 4.0 * scale, rect.min.y + 3.0 * scale),
            vec2(icon_size * 1.25, icon_size),
        );
        let tex_id = icons.icon_or_label(ctx, Some(&popup.item_id), &popup.label);
        if let Some(tid) = tex_id {
            painter.image(
                tid,
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                color_with_alpha(Color32::WHITE, alpha),
            );
        }

        // Text "+count Label"
        let text_x = if tex_id.is_some() {
            icon_rect.max.x + 6.0 * scale
        } else {
            rect.min.x + 8.0 * scale
        };
        let text = format!("+{} {}", popup.count, popup.label);
        draw_outline_text(
            painter,
            pos2(text_x, rect.center().y),
            Align2::LEFT_CENTER,
            &text,
            FontId::proportional(cfg.font_size_px * scale),
            cfg.text_color32(alpha),
        );

        drawn_idx += 1;
        if drawn_idx >= cfg.max_visible {
            break;
        }
    }
}
