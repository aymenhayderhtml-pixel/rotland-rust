//! Body warmth HUD bar styled after the Food/Water need bars in `vitals.rs`.
//!
//! Hidden in the core home lands while comfortable; shows a blue -> green -> orange
//! bar labelled "Warmth" with a vector snowflake or sun marker drawn via egui shapes.

use std::f32::consts::{FRAC_PI_3, FRAC_PI_4};

use engine::egui::{self, Align2, Color32, FontId, Pos2, Rect, Stroke, pos2, vec2};

use super::config::HudConfig;
use super::text::draw_outline_text;

/// Temperature band mirrored for HUD rendering without coupling UI to simulation crates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmthHudState {
    Freezing,
    Cold,
    Comfy,
    Hot,
    Overheated,
}

/// Live body warmth snapshot passed from gameplay into the HUD.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarmthHudInfo {
    /// Body warmth on the 0..=100 scale (0 = freezing, 50 = comfy, 100 = overheated).
    pub value: f32,
    /// Current temperature state band.
    pub state: WarmthHudState,
    /// Current biome tier (`0` = core/home lands, `> 0` = outer biomes).
    pub biome_tier: u8,
}

impl WarmthHudInfo {
    pub const fn new(value: f32, state: WarmthHudState, biome_tier: u8) -> Self {
        Self {
            value,
            state,
            biome_tier,
        }
    }

    /// Shows the warmth bar whenever outside the core lands or not comfortable.
    pub const fn should_show(&self) -> bool {
        !matches!(self.state, WarmthHudState::Comfy) || self.biome_tier > 0
    }
}

/// Draws the body warmth bar in the vitals stack (matching Food/Water style).
pub fn draw_warmth_bar(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    warmth: Option<&WarmthHudInfo>,
    scale: f32,
    config: &HudConfig,
) {
    let Some(info) = warmth else { return };
    if !info.should_show() {
        return;
    }

    let cfg = &config.bars;
    let bar_bg = cfg.bg_color32();
    // new: DECISIONS 2026-11-25
    let corner_r = (cfg.corner_radius_px * 0.6).max(2.0 * scale);

    // Placed on the same 14 px row stride as Water (54) and Food (68) in vitals.rs,
    // sitting cleanly above Food and below the Dog HUD (118..90) without colliding
    // with the Health bar (38..16).
    // new: DECISIONS 2026-11-25
    let top_offset = 82.0 * scale;
    // new: DECISIONS 2026-11-25
    let rect = Rect::from_min_size(
        pos2(screen.min.x + 72.0 * scale, screen.max.y - top_offset),
        vec2(204.0 * scale, 10.0 * scale),
    );

    let frac = (info.value / 100.0).clamp(0.0, 1.0); // new: DECISIONS 2026-11-25
    let extreme = matches!(
        info.state,
        WarmthHudState::Freezing | WarmthHudState::Overheated
    );
    let pulse = if extreme {
        game::hud_math::bar_pulse_factor(time, cfg.pulse_speed)
    } else {
        1.0 // new: DECISIONS 2026-11-25
    };

    painter.rect_filled(rect, corner_r, bar_bg);

    // Subtle 3-zone blue -> green -> orange guide backdrop so the scale stays legible.
    draw_scale_backdrop(painter, rect, corner_r, scale);

    let fill_color = pulse_color(warmth_gradient_color(frac), extreme, pulse);
    if frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            rect.min,
            vec2(rect.width() * frac, rect.height()),
        );
        painter.rect_filled(fill_rect, corner_r, fill_color);
    }

    // Center tick marking comfy midpoint (50).
    let mid_x = rect.min.x + rect.width() * 0.5; // new: DECISIONS 2026-11-25
    painter.line_segment(
        [
            pos2(mid_x, rect.min.y + 1.0 * scale),
            pos2(mid_x, rect.max.y - 1.0 * scale),
        ],
        Stroke::new(
            (1.0 * scale).max(1.0), // new: DECISIONS 2026-11-25
            Color32::from_rgba_unmultiplied(255, 255, 255, 95), // new: DECISIONS 2026-11-25
        ),
    );

    // Bright position marker so Freezing (0) remains unmistakably visible on the left.
    let marker_w = (3.0 * scale).max(2.0); // new: DECISIONS 2026-11-25
    let marker_x = (rect.min.x + rect.width() * frac - marker_w * 0.5)
        .clamp(rect.min.x, rect.max.x - marker_w);
    let marker_rect = Rect::from_min_size(
        pos2(marker_x, rect.min.y),
        vec2(marker_w, rect.height()),
    );
    let marker_color = if extreme {
        pulse_color(Color32::WHITE, true, pulse)
    } else {
        Color32::from_rgba_unmultiplied(245, 248, 250, 230) // new: DECISIONS 2026-11-25
    };
    painter.rect_filled(marker_rect, (1.5 * scale).max(1.0), marker_color); // new: DECISIONS 2026-11-25

    // Label aligned with "Food" and "Water" in vitals.rs.
    let label_color = if extreme {
        fill_color
    } else {
        Color32::WHITE
    };
    draw_outline_text(
        painter,
        pos2(screen.min.x + 16.0 * scale, rect.center().y), // new: DECISIONS 2026-11-25
        Align2::LEFT_CENTER,
        "Warmth",
        FontId::proportional(12.0 * scale), // new: DECISIONS 2026-11-25
        label_color,
    );

    // Snowflake / sun vector marker to the right of the bar (no textures).
    let icon_center = pos2(rect.max.x + 12.0 * scale, rect.center().y); // new: DECISIONS 2026-11-25
    let icon_radius = 5.5 * scale; // new: DECISIONS 2026-11-25
    match info.state {
        WarmthHudState::Freezing | WarmthHudState::Cold => {
            let col = pulse_color(Color32::from_rgb(130, 212, 255), extreme, pulse); // new: DECISIONS 2026-11-25
            draw_snowflake(painter, icon_center, icon_radius, col, scale);
        }
        WarmthHudState::Hot | WarmthHudState::Overheated => {
            let col = pulse_color(Color32::from_rgb(255, 174, 66), extreme, pulse); // new: DECISIONS 2026-11-25
            draw_sun(painter, icon_center, icon_radius, col, scale);
        }
        WarmthHudState::Comfy => {
            if info.value < 50.0 {
                // new: DECISIONS 2026-11-25
                let col = Color32::from_rgba_unmultiplied(150, 210, 240, 150);
                draw_snowflake(painter, icon_center, icon_radius * 0.9, col, scale);
            } else {
                // new: DECISIONS 2026-11-25
                let col = Color32::from_rgba_unmultiplied(240, 195, 110, 150);
                draw_sun(painter, icon_center, icon_radius * 0.9, col, scale);
            }
        }
    }
}

fn draw_scale_backdrop(painter: &egui::Painter, rect: Rect, corner_r: f32, scale: f32) {
    let inset = (1.0 * scale).min(rect.height() * 0.25); // new: DECISIONS 2026-11-25
    let inner = rect.shrink(inset);
    let third = inner.width() / 3.0; // new: DECISIONS 2026-11-25
    let inner_r = (corner_r - inset).max(0.0);

    // new: DECISIONS 2026-11-25
    let cold_tint = Color32::from_rgba_unmultiplied(66, 154, 245, 48);
    // new: DECISIONS 2026-11-25
    let comfy_tint = Color32::from_rgba_unmultiplied(94, 196, 112, 42);
    // new: DECISIONS 2026-11-25
    let hot_tint = Color32::from_rgba_unmultiplied(242, 128, 44, 48);

    painter.rect_filled(
        Rect::from_min_size(inner.min, vec2(third, inner.height())),
        inner_r,
        cold_tint,
    );
    painter.rect_filled(
        Rect::from_min_size(pos2(inner.min.x + third, inner.min.y), vec2(third, inner.height())),
        0.0,
        comfy_tint,
    );
    painter.rect_filled(
        Rect::from_min_size(
            pos2(inner.min.x + third * 2.0, inner.min.y),
            vec2(third, inner.height()),
        ),
        inner_r,
        hot_tint,
    );
}

/// Maps warmth fraction (0..=1) smoothly across blue -> green -> orange.
fn warmth_gradient_color(frac: f32) -> Color32 {
    // new: DECISIONS 2026-11-25
    let blue = Color32::from_rgb(66, 154, 245);
    // new: DECISIONS 2026-11-25
    let green = Color32::from_rgb(94, 196, 112);
    // new: DECISIONS 2026-11-25
    let orange = Color32::from_rgb(242, 128, 44);

    if frac <= 0.5 {
        lerp_color(blue, green, frac * 2.0) // new: DECISIONS 2026-11-25
    } else {
        lerp_color(green, orange, (frac - 0.5) * 2.0) // new: DECISIONS 2026-11-25
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    Color32::from_rgb(
        lerp_u8(a.r(), b.r(), t),
        lerp_u8(a.g(), b.g(), t),
        lerp_u8(a.b(), b.b(), t),
    )
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let t = t.clamp(0.0, 1.0);
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t).round() as u8
}

fn pulse_color(color: Color32, active: bool, pulse: f32) -> Color32 {
    if !active {
        return color;
    }
    // new: DECISIONS 2026-11-25
    let factor = 0.65 + 0.35 * pulse;
    Color32::from_rgba_unmultiplied(
        (f32::from(color.r()) * factor).round() as u8,
        (f32::from(color.g()) * factor).round() as u8,
        (f32::from(color.b()) * factor).round() as u8,
        color.a(),
    )
}

/// Draws a 6-armed vector snowflake using egui line segments.
fn draw_snowflake(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, scale: f32) {
    let stroke = Stroke::new((1.3 * scale).max(1.0), color); // new: DECISIONS 2026-11-25
    let branch_start = radius * 0.58; // new: DECISIONS 2026-11-25
    let branch_len = radius * 0.34; // new: DECISIONS 2026-11-25

    for i in 0..3 {
        let angle = i as f32 * FRAC_PI_3;
        let dir = vec2(angle.cos(), angle.sin());
        painter.line_segment([center - dir * radius, center + dir * radius], stroke);

        let left = vec2((angle + FRAC_PI_3).cos(), (angle + FRAC_PI_3).sin());
        let right = vec2((angle - FRAC_PI_3).cos(), (angle - FRAC_PI_3).sin());

        for sign in [-1.0_f32, 1.0_f32] {
            let joint = center + dir * (sign * branch_start);
            painter.line_segment([joint, joint + left * (sign * branch_len)], stroke);
            painter.line_segment([joint, joint + right * (sign * branch_len)], stroke);
        }
    }
}

/// Draws a vector sun with central disc and 8 radial rays using egui shapes.
fn draw_sun(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, scale: f32) {
    let stroke = Stroke::new((1.3 * scale).max(1.0), color); // new: DECISIONS 2026-11-25
    painter.circle_filled(center, radius * 0.44, color); // new: DECISIONS 2026-11-25

    let ray_in = radius * 0.64; // new: DECISIONS 2026-11-25
    for i in 0..8 {
        let angle = i as f32 * FRAC_PI_4;
        let dir = vec2(angle.cos(), angle.sin());
        painter.line_segment([center + dir * ray_in, center + dir * radius], stroke);
    }
}
