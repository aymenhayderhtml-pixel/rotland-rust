//! Vital status bars: health, stamina, food, water, and pet dog HUD.
//!
//! Port of Godot C# `scripts/UI/HUD.cs` and `scripts/Dogs/DogHud.cs`.

use engine::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect};
use serde::{Deserialize, Serialize};
use super::config::{parse_hex_color, HudBarsConfig, HudConfig};
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

/// Persistent state for smooth "recent damage" trails on vital bars.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VitalsTrailState {
    #[serde(default = "default_100")]
    pub health_trail: f32,
    #[serde(default = "default_100")]
    pub stamina_trail: f32,
    #[serde(default = "default_1")]
    pub food_trail: f32,
    #[serde(default = "default_1")]
    pub water_trail: f32,
    #[serde(default)]
    pub initialized: bool,
}

fn default_100() -> f32 { 100.0 }
fn default_1() -> f32 { 1.0 }

impl Default for VitalsTrailState {
    fn default() -> Self {
        Self {
            health_trail: 100.0,
            stamina_trail: 100.0,
            food_trail: 1.0,
            water_trail: 1.0,
            initialized: false,
        }
    }
}

impl VitalsTrailState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates vital bar trails toward current values using delta time.
    pub fn update(
        &mut self,
        health: f32,
        stamina: f32,
        food_frac: Option<f32>,
        water_frac: Option<f32>,
        dt: f32,
        config: &HudBarsConfig,
    ) {
        if !self.initialized {
            self.health_trail = health;
            self.stamina_trail = stamina;
            self.food_trail = food_frac.unwrap_or(1.0);
            self.water_trail = water_frac.unwrap_or(1.0);
            self.initialized = true;
            return;
        }

        // Health trail
        if health > self.health_trail {
            self.health_trail = health;
        } else {
            self.health_trail = game::hud_math::vitals_trail_update(
                self.health_trail,
                health,
                dt,
                config.trail_shrink_rate * 100.0,
            );
        }

        // Stamina trail
        if stamina > self.stamina_trail {
            self.stamina_trail = stamina;
        } else {
            self.stamina_trail = game::hud_math::vitals_trail_update(
                self.stamina_trail,
                stamina,
                dt,
                config.trail_shrink_rate * 100.0,
            );
        }

        // Food trail
        if let Some(f) = food_frac {
            if f > self.food_trail {
                self.food_trail = f;
            } else {
                self.food_trail = game::hud_math::vitals_trail_update(
                    self.food_trail,
                    f,
                    dt,
                    config.trail_shrink_rate,
                );
            }
        }

        // Water trail
        if let Some(w) = water_frac {
            if w > self.water_trail {
                self.water_trail = w;
            } else {
                self.water_trail = game::hud_math::vitals_trail_update(
                    self.water_trail,
                    w,
                    dt,
                    config.trail_shrink_rate,
                );
            }
        }
    }
}

/// Draws player health and stamina bars in the bottom-left corner with default styling.
#[allow(dead_code)]
pub fn draw_vital_bars(
    painter: &egui::Painter,
    screen: &Rect,
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    scale: f32,
) {
    draw_vital_bars_polished(
        painter,
        screen,
        health,
        max_health,
        stamina,
        max_stamina,
        None,
        0.0,
        scale,
        &HudConfig::default(),
    );
}

/// Draws polished player health and stamina bars with rounded ends, darker back, recent damage trail, and low pulse.
#[allow(clippy::too_many_arguments)]
pub fn draw_vital_bars_polished(
    painter: &egui::Painter,
    screen: &Rect,
    health: f32,
    max_health: f32,
    stamina: f32,
    max_stamina: f32,
    trail: Option<&VitalsTrailState>,
    time: f64,
    scale: f32,
    config: &HudConfig,
) {
    let cfg = &config.bars;
    let bar_bg = cfg.bg_color32();
    let trail_col = cfg.trail_color32();
    let health_base = cfg.health_color32();
    let stamina_base = cfg.stamina_color32();

    let hp_frac = (health / max_health.max(1.0)).clamp(0.0, 1.0);
    let stam_frac = (stamina / max_stamina.max(1.0)).clamp(0.0, 1.0);

    // Pulse when low
    let hp_low = hp_frac <= cfg.low_health_threshold;
    let stam_low = stam_frac <= cfg.low_stamina_threshold;
    let pulse = game::hud_math::bar_pulse_factor(time, cfg.pulse_speed);

    let health_fill = if hp_low {
        let factor = 0.65 + 0.35 * pulse;
        Color32::from_rgb(
            (health_base.r() as f32 * factor) as u8,
            (health_base.g() as f32 * pulse) as u8,
            (health_base.b() as f32 * pulse) as u8,
        )
    } else {
        health_base
    };

    let stamina_fill = if stam_low {
        let factor = 0.65 + 0.35 * pulse;
        Color32::from_rgb(
            (stamina_base.r() as f32 * pulse) as u8,
            (stamina_base.g() as f32 * factor) as u8,
            (stamina_base.b() as f32 * pulse) as u8,
        )
    } else {
        stamina_base
    };

    let corner_r = cfg.corner_radius_px * scale;

    // Health bar: width 260, height 22
    let hp_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0 * scale, screen.max.y - 38.0 * scale),
        vec2(260.0 * scale, 22.0 * scale),
    );
    // Darker back with rounded ends
    painter.rect_filled(hp_rect, corner_r, bar_bg);

    // Smooth recent damage trail (white part that shrinks after hit)
    if let Some(tr) = trail {
        let tr_frac = (tr.health_trail / max_health.max(1.0)).clamp(0.0, 1.0);
        if tr_frac > hp_frac {
            let trail_rect = Rect::from_min_size(
                hp_rect.min,
                vec2(hp_rect.width() * tr_frac, hp_rect.height()),
            );
            painter.rect_filled(trail_rect, corner_r, trail_col);
        }
    }

    // Health fill
    if hp_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            hp_rect.min,
            vec2(hp_rect.width() * hp_frac, hp_rect.height()),
        );
        painter.rect_filled(fill_rect, corner_r, health_fill);
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
    let stam_corner_r = (corner_r * 0.5).max(2.0 * scale);
    painter.rect_filled(stam_rect, stam_corner_r, bar_bg);

    // Stamina trail
    if let Some(tr) = trail {
        let tr_frac = (tr.stamina_trail / max_stamina.max(1.0)).clamp(0.0, 1.0);
        if tr_frac > stam_frac {
            let trail_rect = Rect::from_min_size(
                stam_rect.min,
                vec2(stam_rect.width() * tr_frac, stam_rect.height()),
            );
            painter.rect_filled(trail_rect, stam_corner_r, trail_col);
        }
    }

    // Stamina fill
    if stam_frac > 0.0 {
        let fill_rect = Rect::from_min_size(
            stam_rect.min,
            vec2(stam_rect.width() * stam_frac, stam_rect.height()),
        );
        painter.rect_filled(fill_rect, stam_corner_r, stamina_fill);
    }
}

/// Draws food and water need bars above the health bar.
#[allow(dead_code)]
pub fn draw_need_bars(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    food: Option<(f32, bool)>,
    water: Option<(f32, bool)>,
    scale: f32,
) {
    draw_need_bars_polished(painter, screen, time, food, water, None, scale, &HudConfig::default());
}

/// Draws polished food and water need bars with rounded ends, darker back, recent damage trail, and low pulse.
#[allow(clippy::too_many_arguments)]
pub fn draw_need_bars_polished(
    painter: &egui::Painter,
    screen: &Rect,
    time: f64,
    food: Option<(f32, bool)>,
    water: Option<(f32, bool)>,
    trail: Option<&VitalsTrailState>,
    scale: f32,
    config: &HudConfig,
) {
    let cfg = &config.bars;
    let bar_bg = cfg.bg_color32();
    let trail_col = cfg.trail_color32();
    let corner_r = (cfg.corner_radius_px * 0.6).max(2.0 * scale);

    let rows = [
        ("Food", food, cfg.food_color32(), 68.0, trail.map(|t| t.food_trail)),
        ("Water", water, cfg.water_color32(), 54.0, trail.map(|t| t.water_trail)),
    ];
    for (label, value, fill, top, trail_val) in rows {
        let Some((frac, low)) = value else { continue };
        let pulse = if low {
            game::hud_math::bar_pulse_factor(time, cfg.pulse_speed)
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
        painter.rect_filled(rect, corner_r, bar_bg);

        // Trail
        if let Some(tr) = trail_val {
            let tr_clamped = tr.clamp(0.0, 1.0);
            if tr_clamped > frac {
                painter.rect_filled(
                    Rect::from_min_size(rect.min, vec2(rect.width() * tr_clamped, rect.height())),
                    corner_r,
                    trail_col,
                );
            }
        }

        let frac = frac.clamp(0.0, 1.0);
        if frac > 0.0 {
            painter.rect_filled(
                Rect::from_min_size(rect.min, vec2(rect.width() * frac, rect.height())),
                corner_r,
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
        game::hud_math::bar_pulse_factor(time, 6.0)
    } else {
        1.0
    };

    let healthy_col = Color32::from_rgb(115, 191, 77);
    let low_col = Color32::from_rgb(217, 38, 38);
    let fill_color = if is_low {
        Color32::from_rgb(
            low_col.r(),
            (low_col.g() as f32 * pulse) as u8,
            (low_col.b() as f32 * pulse) as u8,
        )
    } else {
        healthy_col
    };

    let bar_bg = parse_hex_color("#0c0c0ee6");
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

    // Dog health bar with rounded ends
    let bar_rect = Rect::from_min_size(
        pos2(screen.min.x + 16.0 * scale, base_y + 18.0 * scale),
        vec2(200.0 * scale, 10.0 * scale),
    );
    painter.rect_filled(bar_rect, 4.0 * scale, bar_bg);
    if frac > 0.0 {
        painter.rect_filled(
            Rect::from_min_size(bar_rect.min, vec2(bar_rect.width() * frac, bar_rect.height())),
            4.0 * scale,
            fill_color,
        );
    }
}
