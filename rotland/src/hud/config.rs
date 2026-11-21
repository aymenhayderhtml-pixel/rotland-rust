//! Configuration loaded from `data/hud.json`.
#![allow(dead_code)]

use std::path::Path;
use engine::egui::Color32;
use serde::{Deserialize, Serialize};

/// Parses a hex color string (#rrggbb or #rrggbbaa) into an egui Color32.
pub fn parse_hex_color(hex: &str) -> Color32 {
    let s = hex.trim_start_matches('#');
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        Color32::from_rgb(r, g, b)
    } else if s.len() == 8 {
        let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
        let a = u8::from_str_radix(&s[6..8], 16).unwrap_or(255);
        Color32::from_rgba_unmultiplied(r, g, b, a)
    } else {
        Color32::WHITE
    }
}

/// Applies an alpha multiplier [0.0, 1.0] to a Color32.
pub fn color_with_alpha(color: Color32, alpha_mult: f32) -> Color32 {
    let a = ((color.a() as f32) * alpha_mult.clamp(0.0, 1.0)) as u8;
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DamageDirectionConfig {
    #[serde(default = "default_damage_fade")]
    pub fade_duration_s: f32,
    #[serde(default = "default_arc_radius_fraction")]
    pub arc_radius_fraction: f32,
    #[serde(default = "default_arc_span_deg")]
    pub arc_span_deg: f32,
    #[serde(default = "default_arc_thickness_px")]
    pub arc_thickness_px: f32,
    #[serde(default = "default_damage_color")]
    pub color: String,
    #[serde(default = "default_damage_alpha")]
    pub alpha: f32,
}

fn default_damage_fade() -> f32 { 1.2 }
fn default_arc_radius_fraction() -> f32 { 0.85 }
fn default_arc_span_deg() -> f32 { 40.0 }
fn default_arc_thickness_px() -> f32 { 5.0 }
fn default_damage_color() -> String { "#e62828".to_string() }
fn default_damage_alpha() -> f32 { 0.85 }

impl Default for DamageDirectionConfig {
    fn default() -> Self {
        Self {
            fade_duration_s: default_damage_fade(),
            arc_radius_fraction: default_arc_radius_fraction(),
            arc_span_deg: default_arc_span_deg(),
            arc_thickness_px: default_arc_thickness_px(),
            color: default_damage_color(),
            alpha: default_damage_alpha(),
        }
    }
}

impl DamageDirectionConfig {
    pub fn color32(&self, alpha_mult: f32) -> Color32 {
        let base = parse_hex_color(&self.color);
        color_with_alpha(base, self.alpha * alpha_mult)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PickupPopupConfig {
    #[serde(default = "default_pickup_fade")]
    pub fade_duration_s: f32,
    #[serde(default = "default_pickup_max_visible")]
    pub max_visible: usize,
    #[serde(default = "default_pickup_width")]
    pub width_px: f32,
    #[serde(default = "default_pickup_height")]
    pub height_px: f32,
    #[serde(default = "default_pickup_font_size")]
    pub font_size_px: f32,
    #[serde(default = "default_pickup_bg")]
    pub bg_color: String,
    #[serde(default = "default_pickup_text")]
    pub text_color: String,
    #[serde(default = "default_pickup_corner_radius")]
    pub corner_radius_px: f32,
}

fn default_pickup_fade() -> f32 { 2.5 }
fn default_pickup_max_visible() -> usize { 5 }
fn default_pickup_width() -> f32 { 180.0 }
fn default_pickup_height() -> f32 { 28.0 }
fn default_pickup_font_size() -> f32 { 14.0 }
fn default_pickup_bg() -> String { "#121214e6".to_string() }
fn default_pickup_text() -> String { "#ffffff".to_string() }
fn default_pickup_corner_radius() -> f32 { 6.0 }

impl Default for PickupPopupConfig {
    fn default() -> Self {
        Self {
            fade_duration_s: default_pickup_fade(),
            max_visible: default_pickup_max_visible(),
            width_px: default_pickup_width(),
            height_px: default_pickup_height(),
            font_size_px: default_pickup_font_size(),
            bg_color: default_pickup_bg(),
            text_color: default_pickup_text(),
            corner_radius_px: default_pickup_corner_radius(),
        }
    }
}

impl PickupPopupConfig {
    pub fn bg_color32(&self, alpha_mult: f32) -> Color32 {
        color_with_alpha(parse_hex_color(&self.bg_color), alpha_mult)
    }

    pub fn text_color32(&self, alpha_mult: f32) -> Color32 {
        color_with_alpha(parse_hex_color(&self.text_color), alpha_mult)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_notif_duration")]
    pub duration_s: f32,
    #[serde(default = "default_notif_max_queue")]
    pub max_queue: usize,
    #[serde(default = "default_notif_height")]
    pub height_px: f32,
    #[serde(default = "default_notif_font_size")]
    pub font_size_px: f32,
    #[serde(default = "default_info_color")]
    pub info_color: String,
    #[serde(default = "default_warning_color")]
    pub warning_color: String,
    #[serde(default = "default_danger_color")]
    pub danger_color: String,
    #[serde(default = "default_notif_bg")]
    pub bg_color: String,
    #[serde(default = "default_notif_corner_radius")]
    pub corner_radius_px: f32,
}

fn default_notif_duration() -> f32 { 4.0 }
fn default_notif_max_queue() -> usize { 3 }
fn default_notif_height() -> f32 { 30.0 }
fn default_notif_font_size() -> f32 { 14.0 }
fn default_info_color() -> String { "#38bdf8".to_string() }
fn default_warning_color() -> String { "#fbbf24".to_string() }
fn default_danger_color() -> String { "#ef4444".to_string() }
fn default_notif_bg() -> String { "#121214f0".to_string() }
fn default_notif_corner_radius() -> f32 { 15.0 }

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            duration_s: default_notif_duration(),
            max_queue: default_notif_max_queue(),
            height_px: default_notif_height(),
            font_size_px: default_notif_font_size(),
            info_color: default_info_color(),
            warning_color: default_warning_color(),
            danger_color: default_danger_color(),
            bg_color: default_notif_bg(),
            corner_radius_px: default_notif_corner_radius(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudBarsConfig {
    #[serde(default = "default_bar_corner_radius")]
    pub corner_radius_px: f32,
    #[serde(default = "default_bar_bg")]
    pub bg_color: String,
    #[serde(default = "default_trail_color")]
    pub trail_color: String,
    #[serde(default = "default_trail_shrink")]
    pub trail_shrink_rate: f32,
    #[serde(default = "default_trail_delay")]
    pub trail_delay_s: f32,
    #[serde(default = "default_low_health")]
    pub low_health_threshold: f32,
    #[serde(default = "default_low_stamina")]
    pub low_stamina_threshold: f32,
    #[serde(default = "default_low_food")]
    pub low_food_threshold: f32,
    #[serde(default = "default_low_water")]
    pub low_water_threshold: f32,
    #[serde(default = "default_pulse_speed")]
    pub pulse_speed: f32,
    #[serde(default = "default_health_color")]
    pub health_color: String,
    #[serde(default = "default_stamina_color")]
    pub stamina_color: String,
    #[serde(default = "default_food_color")]
    pub food_color: String,
    #[serde(default = "default_water_color")]
    pub water_color: String,
}

fn default_bar_corner_radius() -> f32 { 6.0 }
fn default_bar_bg() -> String { "#0c0c0ee6".to_string() }
fn default_trail_color() -> String { "#ffffffc8".to_string() }
fn default_trail_shrink() -> f32 { 0.35 }
fn default_trail_delay() -> f32 { 0.4 }
fn default_low_health() -> f32 { 0.25 }
fn default_low_stamina() -> f32 { 0.20 }
fn default_low_food() -> f32 { 0.20 }
fn default_low_water() -> f32 { 0.20 }
fn default_pulse_speed() -> f32 { 6.0 }
fn default_health_color() -> String { "#cc2626".to_string() }
fn default_stamina_color() -> String { "#40bf4d".to_string() }
fn default_food_color() -> String { "#f28c26".to_string() }
fn default_water_color() -> String { "#408cf2".to_string() }

impl Default for HudBarsConfig {
    fn default() -> Self {
        Self {
            corner_radius_px: default_bar_corner_radius(),
            bg_color: default_bar_bg(),
            trail_color: default_trail_color(),
            trail_shrink_rate: default_trail_shrink(),
            trail_delay_s: default_trail_delay(),
            low_health_threshold: default_low_health(),
            low_stamina_threshold: default_low_stamina(),
            low_food_threshold: default_low_food(),
            low_water_threshold: default_low_water(),
            pulse_speed: default_pulse_speed(),
            health_color: default_health_color(),
            stamina_color: default_stamina_color(),
            food_color: default_food_color(),
            water_color: default_water_color(),
        }
    }
}

impl HudBarsConfig {
    pub fn bg_color32(&self) -> Color32 {
        parse_hex_color(&self.bg_color)
    }

    pub fn trail_color32(&self) -> Color32 {
        parse_hex_color(&self.trail_color)
    }

    pub fn health_color32(&self) -> Color32 {
        parse_hex_color(&self.health_color)
    }

    pub fn stamina_color32(&self) -> Color32 {
        parse_hex_color(&self.stamina_color)
    }

    pub fn food_color32(&self) -> Color32 {
        parse_hex_color(&self.food_color)
    }

    pub fn water_color32(&self) -> Color32 {
        parse_hex_color(&self.water_color)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HudConfig {
    #[serde(default)]
    pub damage_direction: DamageDirectionConfig,
    #[serde(default)]
    pub pickup_popup: PickupPopupConfig,
    #[serde(default)]
    pub notifications: NotificationConfig,
    #[serde(default)]
    pub bars: HudBarsConfig,
}

impl HudConfig {
    pub fn load_default() -> Self {
        Self::load(&paths::repo_path("data/hud.json"))
    }

    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str::<Self>(&text).unwrap_or_default()
    }
}
