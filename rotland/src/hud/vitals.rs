//! Vital status bars: health, stamina, food, water, and pet dog HUD.
//!
//! Port of Godot C# `scripts/UI/HUD.cs` and `scripts/Dogs/DogHud.cs`.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect};
use super::text::draw_outline_text;

/// Live information for pet dog HUD strip.
#[derive(Debug, Clone, PartialEq)]
pub struct DogHudInfo {
    pub name: String,
    pub health: f32,
    pub max_health: f32,
}

impl DogHudInfo {
    pub const fn new(name: String, health: f32, max_health: f32) -> Self {
        Self {
            name,
            health,
            max_health,
        }
    }
}

/// Draws player health and stamina bars in the bottom-left corner.
pub fn draw_vital_bars(
    painter: &egui::Painter,
    screen: &Rect,
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    scale: f32,
) {
    let bar_bg = Color32::from_rgba_unmultiplied(31, 31, 31, 217);
    let health_fill = Color32::from_rgb(204, 38, 38);
    let stamina_fill = Color32::from_rgb(64, 191, 77);

    // Health bar: width 260, height 22
    let hp_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0 * scale, screen.max.y - 38.0 * scale),
        vec2(260.0 * scale, 22.0 * scale),
    );
    painter.rect_filled(hp_rect, 4.0 * scale, bar_bg);
    let hp_frac = (health / max_health.max(1.0)).clamp(0.0, 1.0);
    if hp_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            hp_rect.min,
            vec2(hp_rect.width() * hp_frac, hp_rect.height()),
        );
        painter.rect_filled(fill_rect, 4.0 * scale, health_fill);
    }
    let hp_label = format!("{} / {}", health.ceil() as i32, max_health.ceil() as i32);
    draw_outline_text(
        painter,
        hp_rect.center(),
        Align2::CENTER_CENTER,
        &hp_label,
        FontId::proportional(14.0 * scale),
        Color32::WHITE,
    );

    // Stamina bar: width 260, height 8
    let stam_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0 * scale, screen.max.y - 13.0 * scale),
        vec2(260.0 * scale, 8.0 * scale),
    );
    painter.rect_filled(stam_rect, 2.0 * scale, bar_bg);
    let stam_frac = (stamina / max_stamina.max(1.0)).clamp(0.0, 1.0);
    if stam_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            stam_rect.min,
            vec2(stam_rect.width() * stam_frac, stam_rect.height()),
        );
        painter.rect_filled(fill_rect, 2.0 * scale, stamina_fill);
    }
}

/// Draws food and water need bars above the health bar.
pub fn draw_need_bars(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    food: Option<(f32, bool)>,
    water: Option<(f32, bool)>,
    scale: f32,
) {
    let bar_bg = Color32::from_rgba_unmultiplied(31, 31, 31, 217);
    let rows = [
        ("Food", food, Color32::from_rgb(242, 140, 38), 68.0),
        ("Water", water, Color32::from_rgb(64, 140, 242), 54.0),
    ];
    for (label, value, fill, top) in rows {
        let Some((frac, low)) = value else { continue };
        let pulse = if low {
            0.5 + 0.5 * (time * 6.0).sin() as f32
        } else {
            1.0
        };
        let tint = |c: Color32| {
            if low {
                Color32::from_rgb(
                    c.r(),
                    (c.g() as f32 * pulse) as u8,
                    (c.b() as f32 * pulse) as u8,
                )
            } else {
                c
            }
        };
        let rect = Rect::from_min_size(
            pos2(screen.min.x + 72.0 * scale, screen.max.y - top * scale),
            vec2(204.0 * scale, 10.0 * scale),
        );
        painter.rect_filled(rect, 2.0 * scale, bar_bg);
        let frac = frac.clamp(0.0, 1.0);
        if frac > 0.0 {
            painter.rect_filled(
                Rect::from_min_size(rect.min, vec2(rect.width() * frac, rect.height())),
                2.0 * scale,
                tint(fill),
            );
        }
        draw_outline_text(
            painter,
            pos2(screen.min.x + 16.0 * scale, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(12.0 * scale),
            tint(Color32::WHITE),
        );
    }
}

/// Draws the pet dog's HUD strip above the needs bars, only when a dog is owned.
/// Port of Godot `scripts/Dogs/DogHud.cs`:
/// - Low threshold is 30% (< 0.3 fraction).
/// - Under 30% health gets a "!" warning prefix and pulses red.
pub fn draw_dog_hud(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    dog: Option<&DogHudInfo>,
    scale: f32,
) {
    let Some(d) = dog else { return };
    let frac = (d.health / d.max_health.max(1.0)).clamp(0.0, 1.0);
    let is_low = frac < 0.3;

    let pulse = if is_low {
        0.5 + 0.5 * (time * 6.0).sin() as f32
    } else {
        1.0
    };

    let healthy_col = Color32::from_rgb(115, 191, 77); // (0.45, 0.75, 0.3)
    let low_col = Color32::from_rgb(217, 38, 38);       // (0.85, 0.15, 0.15)
    let fill_color = if is_low {
        Color32::from_rgb(
            low_col.r(),
            (low_col.g() as f32 * pulse) as u8,
            (low_col.b() as f32 * pulse) as u8,
        )
    } else {
        healthy_col
    };

    let bar_bg = Color32::from_rgba_unmultiplied(0, 0, 0, 140);
    let base_y = screen.max.y - 118.0 * scale;

    // Dog name and HP text
    let label_text = format!(
        "{}{}  {:.0}/{:.0} HP",
        if is_low { "! " } else { "" },
        d.name,
        d.health,
        d.max_health
    );
    draw_outline_text(
        painter,
        pos2(screen.min.x + 16.0 * scale, base_y),
        Align2::LEFT_TOP,
        &label_text,
        FontId::proportional(14.0 * scale),
        if is_low { fill_color } else { Color32::WHITE },
    );

    // Dog health bar
    let bar_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0 * scale, base_y + 18.0 * scale),
        vec2(200.0 * scale, 10.0 * scale),
    );
    painter.rect_filled(bar_rect, 2.0 * scale, bar_bg);
    if frac > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(bar_rect.min, vec2(bar_rect.width() * frac, bar_rect.height())),
            2.0 * scale,
            fill_color,
        );
    }
}
