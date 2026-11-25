//! Hotbar widget with item picture icons, stack counts, and slot selection borders.
//!
//! Port of Godot C# `scripts/UI/HotbarBar.cs` and GitHub issue #2:
//! Shows item icon pictures from `assets/icons/<item_id>.png` with fallback to text.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke, StrokeKind};
use super::IconCache;
use super::text::draw_outline_text;
use super::HotbarSlot;

/// Draws the player hotbar at the bottom-centre of the screen.
pub fn draw_hotbar(
    painter: &egui::Painter,
    ctx: &egui::Context,
    screen: &Rect,
    slots: &[HotbarSlot],
    item_ids: Option<&[Option<String>]>,
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
            draw_outline_text(
                painter,
                pos2(slot_rect.center().x, slot_rect.max.y - 3.0 * scale),
                Align2::CENTER_BOTTOM,
                &slot.label,
                FontId::proportional(10.0 * scale),
                Color32::WHITE,
            );
        }
    }
}
