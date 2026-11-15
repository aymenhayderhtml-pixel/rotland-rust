//! Player HUD: health, stamina, crosshair, hotbar, compass, clock, FPS, ammo, and death overlay.
//! Layout, sizes, and colours ported from Godot C# HUD (scripts/UI/HUD.cs, Compass.cs, Crosshair.cs).

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke, StrokeKind};

/// One slot in the player hotbar (1..6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotbarSlot {
    pub label: String,
    pub count: u32,
    pub selected: bool,
}

/// The entire HUD state passed in from the game rules each frame.
#[derive(Debug, Clone)]
pub struct HudState {
    pub health: f32,
    pub max_health: f32,
    pub stamina: f32,
    pub max_stamina: f32,
    pub hour: f32,
    pub day: u32,
    pub heading_yaw: f32,
    pub hotbar: Vec<HotbarSlot>,
    pub ammo: Option<(u32, u32)>,
    pub fps: f32,
    pub message: Option<String>,
    pub dead: bool,
}

/// Draws the HUD overlay using egui painter.
pub fn draw(ctx: &egui::Context, s: &HudState) {
    egui::Area::new(egui::Id::new("rotland_hud"))
        .fixed_pos(pos2(0.0, 0.0))
        .interactable(false)
        .order(egui::Order::Foreground)
        .show(ctx, |ui| {
            let painter = ui.painter();
            let screen = ctx.screen_rect();

            // 1. FPS top left
            let fps_text = format!("{:.0} FPS", s.fps);
            draw_outline_text(
                painter,
                pos2(screen.min.x + 16.0, screen.min.y + 16.0),
                Align2::LEFT_TOP,
                &fps_text,
                FontId::proportional(16.0),
                Color32::from_rgb(220, 220, 220),
            );

            // 2. Clock "Day N · HH:MM" top right (ClockLabel.cs)
            let hours = (s.hour.floor() as u32) % 24;
            let minutes = ((s.hour.fract() * 60.0).floor() as u32).min(59);
            let clock_text = format!("Day {} · {:02}:{:02}", s.day, hours, minutes);
            draw_outline_text(
                painter,
                pos2(screen.max.x - 16.0, screen.min.y + 16.0),
                Align2::RIGHT_TOP,
                &clock_text,
                FontId::proportional(18.0),
                Color32::WHITE,
            );

            // 3. Compass strip top centre (Compass.cs, CompassMath.cs)
            draw_compass(painter, &screen, s.heading_yaw);

            // 4. Crosshair in the centre (Crosshair.cs)
            if !s.dead {
                draw_crosshair(painter, screen.center());
            }

            // 5. Health and stamina bars bottom left (HUD.cs, HUD.tscn)
            draw_vital_bars(painter, &screen, s.health, s.max_health, s.stamina, s.max_stamina);

            // 6. Hotbar bottom centre (HotbarBar.cs)
            draw_hotbar(painter, &screen, &s.hotbar);

            // 7. Ammo readout bottom right (HUD.cs, HUD.tscn)
            if let Some((mag, spare)) = s.ammo {
                let ammo_text = if mag == 0 && spare == 0 {
                    "Out of ammo: bash [LMB]".to_string()
                } else {
                    format!("{mag} / {spare}")
                };
                draw_outline_text(
                    painter,
                    pos2(screen.max.x - 24.0, screen.max.y - 36.0),
                    Align2::RIGHT_BOTTOM,
                    &ammo_text,
                    FontId::proportional(32.0),
                    Color32::WHITE,
                );
            }

            // 8. Centered prompt/message if active
            if let Some(msg) = &s.message {
                if !s.dead {
                    draw_outline_text(
                        painter,
                        pos2(screen.center().x, screen.center().y + 50.0),
                        Align2::CENTER_CENTER,
                        msg,
                        FontId::proportional(18.0),
                        Color32::WHITE,
                    );
                }
            }

            // 9. Death overlay when dead (HUD.cs, HUD.tscn)
            if s.dead {
                draw_death_overlay(painter, &screen, s.message.as_deref());
            }
        });
}

fn draw_compass(painter: &egui::Painter, screen: &Rect, yaw_rad: f32) {
    let strip_w = 600.0f32.min(screen.width() - 32.0);
    let strip_h = 40.0;
    let center_x = screen.center().x;
    let strip_rect = Rect::from_min_size(
        pos2(center_x - strip_w * 0.5, screen.min.y + 10.0),
        vec2(strip_w, strip_h),
    );

    // Strip background: Color(0, 0, 0, 0.62)
    painter.rect_filled(strip_rect, 0.0, Color32::from_rgba_unmultiplied(0, 0, 0, 158));

    // Bottom line: Color(1, 1, 1, 0.3)
    painter.line_segment(
        [
            pos2(strip_rect.min.x, strip_rect.max.y),
            pos2(strip_rect.max.x, strip_rect.max.y),
        ],
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 76)),
    );

    // Center view tick: Color(1, 1, 1, 0.7)
    painter.line_segment(
        [
            pos2(center_x, strip_rect.min.y),
            pos2(center_x, strip_rect.min.y + 6.0),
        ],
        Stroke::new(2.0, Color32::from_rgba_unmultiplied(255, 255, 255, 178)),
    );

    let text_color = Color32::from_rgb(245, 245, 245);
    let north_color = Color32::from_rgb(255, 209, 76);

    for bearing in (0..360).step_by(15) {
        let raw = bearing as f32 + yaw_rad.to_degrees();
        let rel = ((raw + 180.0).rem_euclid(360.0)) - 180.0;
        if rel.abs() > 90.0 {
            continue;
        }
        let t = (rel / 90.0 + 1.0) * 0.5;
        let x = strip_rect.min.x + t * strip_w;
        let major = bearing % 45 == 0;
        let tick_len = if major { 12.0 } else { 6.0 };
        painter.line_segment(
            [
                pos2(x, strip_rect.max.y),
                pos2(x, strip_rect.max.y - tick_len),
            ],
            Stroke::new(1.0, text_color),
        );
        if major {
            let name = match bearing {
                0 => "N",
                45 => "NE",
                90 => "E",
                135 => "SE",
                180 => "S",
                225 => "SW",
                270 => "W",
                315 => "NW",
                _ => "",
            };
            let color = if bearing == 0 { north_color } else { text_color };
            draw_outline_text(
                painter,
                pos2(x, strip_rect.min.y + 14.0),
                Align2::CENTER_CENTER,
                name,
                FontId::proportional(15.0),
                color,
            );
        }
    }
}

fn draw_crosshair(painter: &egui::Painter, center: Pos2) {
    let line_color = Color32::from_rgba_unmultiplied(255, 255, 255, 230);
    let hip_gap = 12.0;
    let line_len = 8.0;
    let line_width = 2.0;

    // Center dot
    painter.circle_filled(center, 2.0, line_color);

    // 4 ticks: Up, Down, Left, Right
    painter.line_segment(
        [
            pos2(center.x, center.y - hip_gap),
            pos2(center.x, center.y - hip_gap - line_len),
        ],
        Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [
            pos2(center.x, center.y + hip_gap),
            pos2(center.x, center.y + hip_gap + line_len),
        ],
        Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [
            pos2(center.x - hip_gap, center.y),
            pos2(center.x - hip_gap - line_len, center.y),
        ],
        Stroke::new(line_width, line_color),
    );
    painter.line_segment(
        [
            pos2(center.x + hip_gap, center.y),
            pos2(center.x + hip_gap + line_len, center.y),
        ],
        Stroke::new(line_width, line_color),
    );
}

fn draw_vital_bars(
    painter: &egui::Painter,
    screen: &Rect,
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
) {
    let bar_bg = Color32::from_rgba_unmultiplied(31, 31, 31, 217);
    let health_fill = Color32::from_rgb(204, 38, 38);
    let stamina_fill = Color32::from_rgb(64, 191, 77);

    // Health bar: width 260, height 22. offset_top -38, offset_bottom -16
    let hp_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0, screen.max.y - 38.0),
        vec2(260.0, 22.0),
    );
    painter.rect_filled(hp_rect, 4.0, bar_bg);
    let hp_frac = (health / max_health.max(1.0)).clamp(0.0, 1.0);
    if hp_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            hp_rect.min,
            vec2(hp_rect.width() * hp_frac, hp_rect.height()),
        );
        painter.rect_filled(fill_rect, 4.0, health_fill);
    }
    let hp_label = format!("{} / {}", health.ceil() as i32, max_health.ceil() as i32);
    draw_outline_text(
        painter,
        hp_rect.center(),
        Align2::CENTER_CENTER,
        &hp_label,
        FontId::proportional(14.0),
        Color32::WHITE,
    );

    // Stamina bar: width 260, height 8. offset_top -13, offset_bottom -5
    let stam_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0, screen.max.y - 13.0),
        vec2(260.0, 8.0),
    );
    painter.rect_filled(stam_rect, 2.0, bar_bg);
    let stam_frac = (stamina / max_stamina.max(1.0)).clamp(0.0, 1.0);
    if stam_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            stam_rect.min,
            vec2(stam_rect.width() * stam_frac, stam_rect.height()),
        );
        painter.rect_filled(fill_rect, 2.0, stamina_fill);
    }
}

fn draw_hotbar(painter: &egui::Painter, screen: &Rect, slots: &[HotbarSlot]) {
    if slots.is_empty() {
        return;
    }
    let slot_size = 56.0;
    let slot_gap = 6.0;
    let num_slots = slots.len();
    let total_w = num_slots as f32 * slot_size + (num_slots - 1) as f32 * slot_gap;
    let start_x = screen.center().x - total_w * 0.5;
    let y = screen.max.y - 72.0;

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
        painter.rect(
            slot_rect,
            4.0,
            empty_bg,
            Stroke::new(2.0, border),
            StrokeKind::Inside,
        );

        // Key number (1..6) top-left
        let key_text = (i + 1).to_string();
        draw_outline_text(
            painter,
            pos2(slot_rect.min.x + 4.0, slot_rect.min.y + 2.0),
            Align2::LEFT_TOP,
            &key_text,
            FontId::proportional(12.0),
            Color32::WHITE,
        );

        // Count top-right if > 1
        if slot.count > 1 {
            let count_text = slot.count.to_string();
            draw_outline_text(
                painter,
                pos2(slot_rect.max.x - 4.0, slot_rect.min.y + 2.0),
                Align2::RIGHT_TOP,
                &count_text,
                FontId::proportional(12.0),
                Color32::WHITE,
            );
        }

        // Label bottom-center
        if !slot.label.is_empty() {
            draw_outline_text(
                painter,
                pos2(slot_rect.center().x, slot_rect.max.y - 4.0),
                Align2::CENTER_BOTTOM,
                &slot.label,
                FontId::proportional(11.0),
                Color32::WHITE,
            );
        }
    }
}

fn draw_death_overlay(painter: &egui::Painter, screen: &Rect, msg: Option<&str>) {
    // 70% opacity black background
    painter.rect_filled(
        *screen,
        0.0,
        Color32::from_rgba_unmultiplied(0, 0, 0, 178),
    );

    let center_y = screen.center().y;
    // YOU DIED
    draw_outline_text(
        painter,
        pos2(screen.center().x, center_y - 60.0),
        Align2::CENTER_CENTER,
        "YOU DIED",
        FontId::proportional(72.0),
        Color32::from_rgb(220, 38, 38),
    );

    let lost_text = msg.unwrap_or("You had nothing to lose");
    draw_outline_text(
        painter,
        pos2(screen.center().x, center_y + 10.0),
        Align2::CENTER_CENTER,
        lost_text,
        FontId::proportional(26.0),
        Color32::WHITE,
    );

    draw_outline_text(
        painter,
        pos2(screen.center().x, center_y + 50.0),
        Align2::CENTER_CENTER,
        "Your skills are kept.",
        FontId::proportional(18.0),
        Color32::from_rgb(180, 180, 180),
    );

    draw_outline_text(
        painter,
        pos2(screen.center().x, center_y + 90.0),
        Align2::CENTER_CENTER,
        "Respawning…",
        FontId::proportional(22.0),
        Color32::WHITE,
    );
}

/// Draws text with a 1px dark drop-shadow outline for high contrast against any game background.
fn draw_outline_text(
    painter: &egui::Painter,
    pos: Pos2,
    anchor: Align2,
    text: &str,
    font_id: FontId,
    color: Color32,
) {
    let outline_color = Color32::from_rgba_unmultiplied(0, 0, 0, 230);
    let offsets = [
        vec2(-1.0, 0.0),
        vec2(1.0, 0.0),
        vec2(0.0, -1.0),
        vec2(0.0, 1.0),
        vec2(-1.0, -1.0),
        vec2(1.0, -1.0),
        vec2(-1.0, 1.0),
        vec2(1.0, 1.0),
    ];
    for offset in offsets {
        painter.text(pos + offset, anchor, text, font_id.clone(), outline_color);
    }
    painter.text(pos, anchor, text, font_id, color);
}
