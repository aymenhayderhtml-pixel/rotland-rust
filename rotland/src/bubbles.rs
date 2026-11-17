//! In-world speech bubble rendering over the 3D view using egui (port of scripts/Settlement/CampChatBubble.cs).

use engine::egui::{
    self, pos2, vec2, Color32, CornerRadius, FontId, Order, Pos2, Rect, Stroke, StrokeKind,
};
use glam::{Mat4, Vec3, Vec4};

/// Maximum bubble width in screen points, matching Godot CampChatBubble.cs Width = 360f.
pub const MAX_BUBBLE_WIDTH: f32 = 320.0;
/// Default font size for speech bubble text.
pub const BUBBLE_FONT_SIZE: f32 = 15.0;
/// Padding inside the bubble frame.
pub const BUBBLE_PADDING: egui::Vec2 = vec2(10.0, 6.0);
/// Offset above the projected world point in screen space.
pub const BUBBLE_Y_OFFSET: f32 = 12.0;

/// Projects a 3D world position to 2D screen coordinates using the camera's view-projection matrix.
/// Returns None if the point is behind the camera (w <= 0.0) or out of visible NDC range.
pub fn world_to_screen(world_pos: Vec3, view_proj: Mat4, screen: (f32, f32)) -> Option<Pos2> {
    if screen.0 <= 0.0 || screen.1 <= 0.0 {
        return None;
    }

    let clip = view_proj * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);
    // Point behind the camera or on near plane
    if clip.w <= 0.0001 {
        return None;
    }

    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;

    // Generous margin around viewport (-1.5 to 1.5) so bubbles partially visible are not clipped early
    if ndc_x < -1.5 || ndc_x > 1.5 || ndc_y < -1.5 || ndc_y > 1.5 {
        return None;
    }

    // Convert NDC [-1, 1] to screen [0, width] x [0, height] (egui Y is down)
    let screen_x = (ndc_x * 0.5 + 0.5) * screen.0;
    let screen_y = (1.0 - (ndc_y * 0.5 + 0.5)) * screen.1;

    Some(pos2(screen_x, screen_y))
}

/// Draws floating speech bubbles above 3D world points (Godot: CampChatBubble.cs).
///
/// Bubbles behind the camera or off screen are skipped.
/// Each bubble's opacity is scaled by its `alpha` value.
/// Text is wrapped and width is capped at `MAX_BUBBLE_WIDTH`.
pub fn draw_bubbles(
    ctx: &egui::Context,
    view_proj: Mat4,
    screen: (f32, f32),
    bubbles: &[(Vec3, &str, f32)],
) {
    if bubbles.is_empty() || screen.0 <= 0.0 || screen.1 <= 0.0 {
        return;
    }

    egui::Area::new(egui::Id::unique("camp_chat_bubbles"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();

            for &(world_pos, text, alpha) in bubbles {
                let a = alpha.clamp(0.0, 1.0);
                if a <= 0.001 || text.trim().is_empty() {
                    continue;
                }

                let Some(screen_pos) = world_to_screen(world_pos, view_proj, screen) else {
                    continue;
                };

                // Off-screen check with padding for bubble dimensions
                if screen_pos.x < -200.0
                    || screen_pos.x > screen.0 + 200.0
                    || screen_pos.y < -200.0
                    || screen_pos.y > screen.1 + 200.0
                {
                    continue;
                }

                let text_alpha = (a * 255.0) as u8;
                let bg_alpha = (a * 220.0) as u8;
                let stroke_alpha = (a * 180.0) as u8;

                let text_color = Color32::from_rgba_unmultiplied(240, 240, 240, text_alpha);
                let bg_color = Color32::from_rgba_unmultiplied(22, 26, 32, bg_alpha);
                let stroke_color = Color32::from_rgba_unmultiplied(90, 100, 115, stroke_alpha);

                let font_id = FontId::proportional(BUBBLE_FONT_SIZE);
                let galley = painter.layout(
                    text.to_string(),
                    font_id,
                    text_color,
                    MAX_BUBBLE_WIDTH,
                );

                if galley.is_empty() {
                    continue;
                }

                let text_size = galley.size();
                let box_size = vec2(
                    text_size.x + BUBBLE_PADDING.x * 2.0,
                    text_size.y + BUBBLE_PADDING.y * 2.0,
                );

                // Center horizontally above screen_pos, with bottom edge above screen_pos.y - BUBBLE_Y_OFFSET
                let min_x = screen_pos.x - box_size.x * 0.5;
                let min_y = screen_pos.y - BUBBLE_Y_OFFSET - box_size.y;
                let bubble_rect = Rect::from_min_size(pos2(min_x, min_y), box_size);

                // Draw bubble rounded background and border
                painter.rect(
                    bubble_rect,
                    CornerRadius::same(6),
                    bg_color,
                    Stroke::new(1.0, stroke_color),
                    StrokeKind::Outside,
                );

                // Small triangle tail pointing toward the speaker's head
                let tip = pos2(screen_pos.x, screen_pos.y - 4.0);
                let tail_left = pos2(screen_pos.x - 5.0, bubble_rect.max.y);
                let tail_right = pos2(screen_pos.x + 5.0, bubble_rect.max.y);
                painter.add(egui::Shape::convex_polygon(
                    vec![tip, tail_right, tail_left],
                    bg_color,
                    Stroke::NONE,
                ));

                // Paint wrapped text inside the bubble
                let text_pos = pos2(min_x + BUBBLE_PADDING.x, min_y + BUBBLE_PADDING.y);
                painter.galley(text_pos, galley, text_color);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine::render::Camera;

    #[test]
    fn world_to_screen_projects_front_and_skips_behind() {
        let mut cam = Camera {
            position: Vec3::new(0.0, 0.0, 5.0),
            fov_y_deg: 60.0,
            near: 0.1,
            ..Default::default()
        };
        cam.look_at(Vec3::ZERO);
        let view_proj = cam.view_proj(800.0 / 600.0);
        let screen = (800.0, 600.0);

        // Point at origin (0, 0, 0) is in front of the camera (distance 5 along -Z)
        let projected = world_to_screen(Vec3::ZERO, view_proj, screen);
        assert!(projected.is_some());
        let pos = projected.unwrap();
        assert!((pos.x - 400.0).abs() < 2.0);
        assert!((pos.y - 300.0).abs() < 2.0);

        // Point at (0, 0, 10) is behind the camera (cam is at z=5 looking towards -Z)
        let behind = world_to_screen(Vec3::new(0.0, 0.0, 10.0), view_proj, screen);
        assert!(behind.is_none());
    }
}
