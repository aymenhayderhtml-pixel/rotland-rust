//! Hotbar widget with item picture icons, stack counts, condition bars, and slot selection borders.
//!
//! Port of Godot C# `scripts/UI/HotbarBar.cs` and GitHub issue #2:
//! Shows item icon pictures from `assets/icons/<item_id>.png` with fallback to text.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, StrokeKind};
use super::IconCache;
use super::text::draw_outline_text;
use super::HotbarSlot;

/// Color gradient for weapon condition bars: green (>0.6) -> yellow (0.3..0.6) -> red (<0.3).
fn condition_color(cond: f32) -> Color32 {
    let c = cond.clamp(0.0, 1.0);
    if c >= 0.6 {
        let t = ((c - 0.6) / 0.4).clamp(0.0, 1.0);
        Color32::from_rgb(
            (215.0 + (76.0 - 215.0) * t) as u8,
            (204.0 + (212.0 - 204.0) * t) as u8,
            (58.0 + (92.0 - 58.0) * t) as u8,
        )
    } else if c >= 0.3 {
        let t = ((c - 0.3) / 0.3).clamp(0.0, 1.0);
        Color32::from_rgb(
            (235.0 + (215.0 - 235.0) * t) as u8,
            (130.0 + (204.0 - 130.0) * t) as u8,
            (54.0 + (58.0 - 54.0) * t) as u8,
        )
    } else {
        let t = (c / 0.3).clamp(0.0, 1.0);
        Color32::from_rgb(
            (218.0 + (235.0 - 218.0) * t) as u8,
            (52.0 + (130.0 - 52.0) * t) as u8,
            (52.0 + (54.0 - 52.0) * t) as u8,
        )
    }
}

/// Draws the player hotbar at the bottom-centre of the screen.
#[allow(clippy::too_many_arguments)]
pub fn draw_hotbar(
    painter: &egui::Painter,
    ctx: &egui::Context,
    screen: &Rect,
    slots: &[HotbarSlot],
    item_ids: Option<&[Option<String>]>,
    conditions: Option<&[Option<f32>]>,
    icons: &mut IconCache,
    scale: f32,
) {
    if slots.is_empty() {
        return;
    }

    let slot_size = 56.0 * scale;
    let slot_gap = 6.0 * scale;
    let num_slots = slots.len();
    let total_w = num_slots as f32 * slot_size + (num_slots - 1) as f32 * slot_gap;
    let start_x = screen.center().x - total_w * 0.5;
    let y = screen.max.y - 72.0 * scale;

    let empty_bg = Color32::from_rgba_unmultiplied(26, 26, 26, 153);
    let border_color = Color32::from_rgb(102, 102, 102);
    let highlight_color = Color32::from_rgb(255, 217, 51);

    for (i, slot) in slots.iter().enumerate() {
        let slot_rect = Rect::from_min_size(
            pos2(start_x + i as f32 * (slot_size + slot_gap), y),
            vec2(slot_size, slot_size),
        );
        let border = if slot.selected {
            highlight_color
        } else {
            border_color
        };

        // Slot background and border
        painter.rect(
            slot_rect,
            4.0 * scale,
            empty_bg,
            Stroke::new(2.0 * scale, border),
            StrokeKind::Inside,
        );

        // Key number (1..6) top-left
        let key_text = (i + 1).to_string();
        draw_outline_text(
            painter,
            pos2(slot_rect.min.x + 4.0 * scale, slot_rect.min.y + 2.0 * scale),
            Align2::LEFT_TOP,
            &key_text,
            FontId::proportional(12.0 * scale),
            Color32::WHITE,
        );

        // Determine item ID from explicit list or by resolving slot label
        let explicit_id = item_ids.and_then(|ids| ids.get(i).and_then(|id_opt| id_opt.as_deref()));
        let texture_id = icons.icon_or_label(ctx, explicit_id, &slot.label);
        let cond = conditions.and_then(|c| c.get(i).copied().flatten());

        if let Some(tid) = texture_id {
            // Draw item icon picture: Godot size 44x34 at offset (6, 4)
            let icon_rect = Rect::from_min_size(
                pos2(slot_rect.min.x + 6.0 * scale, slot_rect.min.y + 4.0 * scale),
                vec2(44.0 * scale, 34.0 * scale),
            );
            painter.image(
                tid,
                icon_rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else if !slot.label.is_empty() {
            // Fallback when PNG is missing: draw label in the middle of slot
            draw_outline_text(
                painter,
                pos2(slot_rect.center().x, slot_rect.center().y - 2.0 * scale),
                Align2::CENTER_CENTER,
                &slot.label,
                FontId::proportional(11.0 * scale),
                Color32::WHITE,
            );
        }

        // Stack count bottom-right if > 1 (issue #2: stack count bottom-right)
        if slot.count > 1 {
            let count_text = slot.count.to_string();
            draw_outline_text(
                painter,
                pos2(slot_rect.max.x - 4.0 * scale, slot_rect.max.y - 14.0 * scale),
                Align2::RIGHT_BOTTOM,
                &count_text,
                FontId::proportional(12.0 * scale),
                Color32::WHITE,
            );
        }

        // Short item label at bottom-centre if icon was drawn
        if texture_id.is_some() && !slot.label.is_empty() {
            let label_y_offset = if cond.is_some() { 6.5 } else { 3.0 };
            draw_outline_text(
                painter,
                pos2(slot_rect.center().x, slot_rect.max.y - label_y_offset * scale),
                Align2::CENTER_BOTTOM,
                &slot.label,
                FontId::proportional(10.0 * scale),
                Color32::WHITE,
            );
        }

        // Thin weapon condition bar along bottom edge of weapon slots
        if let Some(c) = cond {
            let clamped = c.clamp(0.0, 1.0);
            let bar_h = (3.0 * scale).max(2.0);
            let pad_x = 4.0 * scale;
            let bar_rect = Rect::from_min_size(
                pos2(slot_rect.min.x + pad_x, slot_rect.max.y - bar_h - 2.5 * scale),
                vec2((slot_rect.width() - pad_x * 2.0).max(4.0), bar_h),
            );
            let color = condition_color(clamped);
            painter.rect_filled(bar_rect, 1.0, Color32::from_rgba_unmultiplied(12, 14, 18, 220));
            if clamped > 0.0 {
                let fill = Rect::from_min_size(
                    bar_rect.min,
                    vec2((bar_rect.width() * clamped).max(2.0), bar_rect.height()),
                );
                painter.rect_filled(fill, 1.0, color);
            } else {
                painter.rect_stroke(
                    bar_rect,
                    1.0,
                    Stroke::new(1.0, color),
                    StrokeKind::Inside,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_color_transitions_green_yellow_red() {
        let healthy = condition_color(0.95);
        assert!(healthy.g() > healthy.r() && healthy.g() > 180);

        let mid = condition_color(0.45);
        assert!(mid.r() > 180 && mid.g() > 150 && mid.b() < 100);

        let low = condition_color(0.1);
        assert!(low.r() > 180 && low.g() < 100);
    }
}
