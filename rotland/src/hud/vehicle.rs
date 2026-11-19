//! Dedicated driving dashboard HUD (bottom-right card) shown while the player drives a 4x4.
//!
//! Bold digital speed, a framed gear badge (R/P/N/1-4), a 10-segment colour-coded RPM bar,
//! pulsing fuel and hull bars, three status pills ([L] LIGHTS / [SPC] BRAKE / [V] CAM), and
//! pulsing alert banners for overturned / wrecked / no-fuel states.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Stroke};

use super::text::draw_outline_text;

/// Live driving state for the dashboard.
#[derive(Debug, Clone, PartialEq)]
pub struct VehicleHudInfo {
    pub speed_kmh: f32,
    pub forward_speed: f32,
    pub rpm: f32,
    pub fuel_litres: f32,
    pub fuel_fraction: f32,
    pub health: f32,
    pub max_health: f32,
    pub lights_on: bool,
    pub handbrake: bool,
    pub is_wrecked: bool,
    pub is_empty: bool,
    pub is_flipped: bool,
    pub camera_label: &'static str,
}

/// The gear badge text: R/P/N/1-4.
pub fn gear_label(info: &VehicleHudInfo) -> &'static str {
    if info.is_wrecked || info.is_empty {
        return "N";
    }
    if info.forward_speed < -0.35 {
        return "R";
    }
    if info.handbrake && info.speed_kmh < 1.5 {
        return "P";
    }
    if info.speed_kmh < 2.0 {
        "N"
    } else if info.speed_kmh < 22.0 {
        "1"
    } else if info.speed_kmh < 46.0 {
        "2"
    } else if info.speed_kmh < 70.0 {
        "3"
    } else {
        "4"
    }
}

fn gear_colour(gear: &str) -> Color32 {
    match gear {
        "R" => Color32::from_rgb(240, 178, 60),
        "P" => Color32::from_rgb(214, 58, 52),
        "N" => Color32::from_rgb(150, 154, 158),
        _ => Color32::WHITE,
    }
}

/// A subtle pulse in 0.0..1.0 for pulsing alert elements.
fn pulse(time: f64) -> f32 {
    ((time * 3.2).sin() * 0.5 + 0.5) as f32
}

/// Draws the driving dashboard card at the bottom-right, above the key help bar.
pub fn draw_vehicle_hud(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    info: &VehicleHudInfo,
    scale: f32,
) {
    let card_w = 248.0 * scale;
    let card_h = 126.0 * scale;
    let min = pos2(screen.max.x - 264.0 * scale, screen.max.y - 164.0 * scale);
    let card = Rect::from_min_size(min, vec2(card_w, card_h));

    painter.rect_filled(card, 6.0 * scale, Color32::from_rgba_unmultiplied(20, 22, 24, 215));
    painter.rect_stroke(
        card,
        6.0 * scale,
        Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 40)),
        egui::StrokeKind::Inside,
    );

    let pad = 12.0 * scale;

    // Top row: bold speed + KM/H on the left, gear badge on the right
    let speed = (info.speed_kmh.abs().round() as i32).to_string();
    draw_outline_text(
        painter,
        pos2(card.min.x + pad, card.min.y + pad),
        Align2::LEFT_TOP,
        &speed,
        FontId::proportional(28.0 * scale),
        Color32::WHITE,
    );
    let speed_w = painter
        .layout_no_wrap(
            speed.clone(),
            FontId::proportional(28.0 * scale),
            Color32::WHITE,
        )
        .size()
        .x;
    draw_outline_text(
        painter,
        pos2(card.min.x + pad + speed_w + 4.0 * scale, card.min.y + pad + 13.0 * scale),
        Align2::LEFT_TOP,
        "KM/H",
        FontId::proportional(11.0 * scale),
        Color32::from_rgba_unmultiplied(190, 194, 198, 255),
    );

    // Gear badge (framed)
    let gear = gear_label(info);
    let badge_w = 40.0 * scale;
    let badge_h = 34.0 * scale;
    let badge = Rect::from_min_size(
        pos2(card.max.x - pad - badge_w, card.min.y + pad),
        vec2(badge_w, badge_h),
    );
    painter.rect_filled(badge, 4.0 * scale, Color32::from_rgba_unmultiplied(38, 41, 44, 255));
    painter.rect_stroke(
        badge,
        4.0 * scale,
        Stroke::new(1.5, gear_colour(gear)),
        egui::StrokeKind::Inside,
    );
    draw_outline_text(
        painter,
        badge.center(),
        Align2::CENTER_CENTER,
        gear,
        FontId::proportional(22.0 * scale),
        gear_colour(gear),
    );

    // 10-segment RPM bar
    let bar_y = card.min.y + pad + 42.0 * scale;
    let bar_x = card.min.x + pad;
    let bar_w = card_w - pad * 2.0;
    let seg_gap = 2.0 * scale;
    let seg_w = (bar_w - seg_gap * 9.0) / 10.0;
    let seg_h = 8.0 * scale;
    let lit = (info.rpm.clamp(0.0, 1.0) * 10.0).ceil() as i32;
    for i in 0..10 {
        let colour_on = match i {
            0..=5 => Color32::from_rgb(70, 200, 84),
            6..=7 => Color32::from_rgb(240, 186, 52),
            _ => Color32::from_rgb(224, 60, 48),
        };
        let seg = Rect::from_min_size(
            pos2(bar_x + i as f32 * (seg_w + seg_gap), bar_y),
            vec2(seg_w, seg_h),
        );
        let colour = if i < lit {
            colour_on
        } else {
            Color32::from_rgba_unmultiplied(60, 63, 66, 255)
        };
        painter.rect_filled(seg, 1.5 * scale, colour);
    }

    // Fuel bar + litres
    let fuel_y = bar_y + 16.0 * scale;
    let bar_h = 7.0 * scale;
    let fuel_low = info.fuel_fraction < 0.15 || info.is_empty;
    let fuel_colour = if fuel_low {
        let a = (150.0 + pulse(time) * 105.0) as u8;
        Color32::from_rgba_unmultiplied(230, 70, 50, a)
    } else {
        Color32::from_rgb(224, 176, 58)
    };
    draw_progress_bar(
        painter,
        Rect::from_min_size(pos2(bar_x, fuel_y), vec2(bar_w, bar_h)),
        info.fuel_fraction,
        fuel_colour,
    );
    draw_outline_text(
        painter,
        pos2(bar_x + bar_w, fuel_y - 9.0 * scale),
        Align2::RIGHT_BOTTOM,
        &format!("{:.1} L", info.fuel_litres),
        FontId::proportional(10.0 * scale),
        Color32::from_rgba_unmultiplied(205, 208, 212, 255),
    );

    // Hull bar + health numbers
    let hull_y = fuel_y + 14.0 * scale;
    let hull_frac = (info.health / info.max_health.max(1.0)).clamp(0.0, 1.0);
    let hull_colour = if info.is_wrecked || hull_frac < 0.28 {
        let a = (150.0 + pulse(time) * 105.0) as u8;
        Color32::from_rgba_unmultiplied(224, 60, 48, a)
    } else if hull_frac < 0.55 {
        Color32::from_rgb(240, 186, 52)
    } else {
        Color32::from_rgb(70, 200, 84)
    };
    draw_progress_bar(
        painter,
        Rect::from_min_size(pos2(bar_x, hull_y), vec2(bar_w, bar_h)),
        hull_frac,
        hull_colour,
    );
    draw_outline_text(
        painter,
        pos2(bar_x + bar_w, hull_y - 9.0 * scale),
        Align2::RIGHT_BOTTOM,
        &format!("{:.0}/{:.0}", info.health.ceil(), info.max_health.ceil()),
        FontId::proportional(10.0 * scale),
        Color32::from_rgba_unmultiplied(205, 208, 212, 255),
    );

    // Bottom row: three status pills
    let pill_y = card.max.y - pad - 18.0 * scale;
    let mut pill_x = card.min.x + pad;
    pill_x += draw_status_pill(
        painter,
        pill_x,
        pill_y,
        "[L] LIGHTS",
        info.lights_on,
        Color32::from_rgb(240, 214, 96),
        scale,
    );
    pill_x += 6.0 * scale;
    pill_x += draw_status_pill(
        painter,
        pill_x,
        pill_y,
        "[SPC] BRAKE",
        info.handbrake,
        Color32::from_rgb(206, 60, 60),
        scale,
    );
    pill_x += 6.0 * scale;
    let cam_label = format!("[V] {}", info.camera_label);
    draw_status_pill(
        painter,
        pill_x,
        pill_y,
        &cam_label,
        true,
        Color32::from_rgb(96, 150, 200),
        scale,
    );

    // Alert strip above the card
    let alert = if info.is_flipped {
        Some("[R] RIGHT OVERTURNED 4x4")
    } else if info.is_wrecked {
        Some("WRECKED \u{2014} [E] EXIT & [T] REPAIR")
    } else if info.is_empty {
        Some("NO FUEL \u{2014} [E] EXIT & [F] REFUEL")
    } else {
        None
    };
    if let Some(text) = alert {
        let a = (170.0 + pulse(time) * 85.0) as u8;
        draw_outline_text(
            painter,
            pos2(card.center().x, card.min.y - 10.0 * scale),
            Align2::CENTER_BOTTOM,
            text,
            FontId::proportional(12.0 * scale),
            Color32::from_rgba_unmultiplied(240, 90, 70, a),
        );
    }
}

fn draw_progress_bar(
    painter: &egui::Painter,
    track: Rect,
    frac: f32,
    colour: Color32,
) {
    painter.rect_filled(track, 2.0, Color32::from_rgba_unmultiplied(52, 55, 58, 255));
    let frac = frac.clamp(0.0, 1.0);
    if frac > 0.0 {
        let fill = Rect::from_min_size(track.min, vec2(track.width() * frac, track.height()));
        painter.rect_filled(fill, 2.0, colour);
    }
}

fn draw_status_pill(
    painter: &egui::Painter,
    x: f32,
    y: f32,
    text: &str,
    on: bool,
    accent: Color32,
    scale: f32,
) -> f32 {
    let font = FontId::proportional(9.5 * scale);
    let text_w = painter.layout_no_wrap(text.to_string(), font.clone(), Color32::WHITE).size().x;
    let w = text_w + 12.0 * scale;
    let h = 18.0 * scale;
    let rect = Rect::from_min_size(pos2(x, y), vec2(w, h));
    let bg = if on {
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 60)
    } else {
        Color32::from_rgba_unmultiplied(50, 53, 56, 180)
    };
    painter.rect_filled(rect, 3.0 * scale, bg);
    let border = if on { accent } else { Color32::from_rgba_unmultiplied(90, 94, 98, 255) };
    painter.rect_stroke(rect, 3.0 * scale, Stroke::new(1.0, border), egui::StrokeKind::Inside);
    draw_outline_text(
        painter,
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        font,
        if on { Color32::WHITE } else { Color32::from_rgba_unmultiplied(170, 174, 178, 255) },
    );
    w
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> VehicleHudInfo {
        VehicleHudInfo {
            speed_kmh: 0.0,
            forward_speed: 0.0,
            rpm: 0.0,
            fuel_litres: 40.0,
            fuel_fraction: 1.0,
            health: 300.0,
            max_health: 300.0,
            lights_on: false,
            handbrake: false,
            is_wrecked: false,
            is_empty: false,
            is_flipped: false,
            camera_label: "CHASE",
        }
    }

    #[test]
    fn gear_label_covers_every_state() {
        let mut info = base();
        assert_eq!(gear_label(&info), "N", "stopped, no handbrake -> N");

        info.forward_speed = -1.0;
        assert_eq!(gear_label(&info), "R", "rolling backward -> R");

        info.forward_speed = 0.0;
        info.handbrake = true;
        info.speed_kmh = 0.4;
        assert_eq!(gear_label(&info), "P", "handbrake while stopped -> P");

        info.handbrake = false;
        info.speed_kmh = 10.0;
        assert_eq!(gear_label(&info), "1");
        info.speed_kmh = 30.0;
        assert_eq!(gear_label(&info), "2");
        info.speed_kmh = 55.0;
        assert_eq!(gear_label(&info), "3");
        info.speed_kmh = 80.0;
        assert_eq!(gear_label(&info), "4");

        info.is_wrecked = true;
        assert_eq!(gear_label(&info), "N", "wrecked -> N");
        info.is_wrecked = false;
        info.is_empty = true;
        assert_eq!(gear_label(&info), "N", "out of fuel -> N");
    }
}
