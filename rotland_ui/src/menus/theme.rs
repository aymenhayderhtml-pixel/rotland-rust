//! Shared visual theme, typography scale, and custom widgets for Rotland's menus.
//!
//! Styled around weathered road-sign rust-orange accents on dark translucent charcoal panels,
//! scoped to menu rendering via [`with_menu_style`] so the HUD and other screens stay unchanged.

use std::sync::Arc;

use engine::egui::{
    self, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Frame, Margin, Mesh, Rect,
    Response, Role, Sense, Shape, Stroke, StrokeKind, Style, Vec2, Visuals, WidgetInfo, pos2, vec2,
};
use engine::egui::style::TextStyle;

// Palette — dark charcoal panels, weathered road-sign rust-orange accent, off-white text,
// muted survivor green ("safe"), and hazard red ("danger").
pub const PANEL_BG: Color32 = Color32::from_rgba_premultiplied(18, 20, 22, 234); // new: DECISIONS 2026-11-21
pub const CARD_BG: Color32 = Color32::from_rgba_premultiplied(26, 29, 33, 240); // new: DECISIONS 2026-11-21
pub const CARD_EMPTY_BG: Color32 = Color32::from_rgba_premultiplied(12, 13, 15, 196); // new: DECISIONS 2026-11-21
pub const BORDER: Color32 = Color32::from_rgb(60, 65, 71); // new: DECISIONS 2026-11-21
pub const DIVIDER: Color32 = Color32::from_rgba_premultiplied(35, 38, 41, 115); // new: DECISIONS 2026-11-21

pub const ACCENT: Color32 = Color32::from_rgb(214, 102, 42); // new: DECISIONS 2026-11-21
pub const ACCENT_HOVER: Color32 = Color32::from_rgb(234, 122, 58); // new: DECISIONS 2026-11-21
pub const ACCENT_SUBTLE: Color32 = Color32::from_rgba_premultiplied(37, 18, 7, 44); // new: DECISIONS 2026-11-21

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(238, 235, 227); // new: DECISIONS 2026-11-21
pub const TEXT_MUTED: Color32 = Color32::from_rgb(154, 152, 146); // new: DECISIONS 2026-11-21
pub const TEXT_DISABLED: Color32 = Color32::from_rgb(104, 104, 102); // new: DECISIONS 2026-11-21

pub const SAFE: Color32 = Color32::from_rgb(98, 156, 94); // new: DECISIONS 2026-11-21
pub const SAFE_HOVER: Color32 = Color32::from_rgb(120, 180, 114); // new: DECISIONS 2026-11-21
pub const DANGER: Color32 = Color32::from_rgb(186, 62, 54); // new: DECISIONS 2026-11-21
pub const DANGER_HOVER: Color32 = Color32::from_rgb(214, 80, 70); // new: DECISIONS 2026-11-21

pub const PAUSE_DIM: Color32 = Color32::from_rgba_premultiplied(6, 8, 9, 165); // new: DECISIONS 2026-11-21
pub const SETTINGS_DIM: Color32 = Color32::from_rgba_premultiplied(7, 9, 10, 188); // new: DECISIONS 2026-11-21
pub const MODAL_DIM: Color32 = Color32::from_rgba_premultiplied(4, 5, 6, 175); // new: DECISIONS 2026-11-21

const BTN_BG: Color32 = Color32::from_rgba_premultiplied(29, 32, 35, 232); // new: DECISIONS 2026-11-21
const BTN_HOVER: Color32 = Color32::from_rgba_premultiplied(42, 46, 52, 246); // new: DECISIONS 2026-11-21
const BTN_ACTIVE: Color32 = Color32::from_rgba_premultiplied(53, 49, 45, 250); // new: DECISIONS 2026-11-21
const BTN_DISABLED: Color32 = Color32::from_rgba_premultiplied(14, 15, 16, 150); // new: DECISIONS 2026-11-21
const CONTROL_BG: Color32 = Color32::from_rgb(36, 40, 45); // new: DECISIONS 2026-11-21
const EXTREME_BG: Color32 = Color32::from_rgb(14, 16, 18); // new: DECISIONS 2026-11-21

// Base sizing at 1280x720 (scaled linearly by screen height up to 2560x1440).
const BASE_HEIGHT: f32 = 720.0; // new: DECISIONS 2026-11-21
const MIN_SCALE: f32 = 0.85; // new: DECISIONS 2026-11-21
const MAX_SCALE: f32 = 1.85; // new: DECISIONS 2026-11-21

pub const CORNER_PANEL: f32 = 6.0; // new: DECISIONS 2026-11-21
pub const CORNER_CONTROL: f32 = 4.0; // new: DECISIONS 2026-11-21
pub const BTN_HEIGHT: f32 = 44.0; // new: DECISIONS 2026-11-21
pub const BTN_HEIGHT_COMPACT: f32 = 36.0; // new: DECISIONS 2026-11-21

pub const FONT_TITLE: f32 = 62.0; // new: DECISIONS 2026-11-21
pub const FONT_HEADING: f32 = 26.0; // new: DECISIONS 2026-11-21
pub const FONT_SUBHEADING: f32 = 18.0; // new: DECISIONS 2026-11-21
pub const FONT_BUTTON: f32 = 17.0; // new: DECISIONS 2026-11-21
pub const FONT_BODY: f32 = 15.0; // new: DECISIONS 2026-11-21
pub const FONT_HELP: f32 = 12.5; // new: DECISIONS 2026-11-21

/// Semantic color tone for [`menu_button`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonTone {
    Default,
    Primary,
    Safe,
    Danger,
}

/// Computes a responsive UI scale factor from screen height (1.0 at 720p, ~1.5 at 1080p, 1.85 at 1440p).
pub fn ui_scale(screen_height: f32) -> f32 {
    if screen_height <= 0.0 || !screen_height.is_finite() {
        return 1.0;
    }
    (screen_height / BASE_HEIGHT).clamp(MIN_SCALE, MAX_SCALE)
}

/// Scales a corner radius and clamps it to `u8`.
pub fn radius(base: f32, scale: f32) -> CornerRadius {
    CornerRadius::same((base * scale).round().clamp(0.0, f32::from(u8::MAX)) as u8)
}

/// Scales a uniform margin and clamps it to `i8`.
pub fn margin(base: f32, scale: f32) -> Margin {
    Margin::same((base * scale).round().clamp(0.0, f32::from(i8::MAX)) as i8)
}

struct StyleGuard<'a> {
    ctx: &'a egui::Context,
    previous: Arc<Style>,
}

impl Drop for StyleGuard<'_> {
    fn drop(&mut self) {
        self.ctx.set_global_style(Arc::clone(&self.previous));
    }
}

/// Applies the menu `Style` to `ctx` only for the duration of `f`, restoring the previous style afterward.
pub fn with_menu_style<R>(ctx: &egui::Context, scale: f32, f: impl FnOnce() -> R) -> R {
    let previous = ctx.global_style();
    let style = build_menu_style(&previous, scale);
    ctx.set_global_style(style);
    let _guard = StyleGuard { ctx, previous };
    f()
}

fn build_menu_style(base: &Style, scale: f32) -> Style {
    let mut style = base.clone();
    let mut visuals = Visuals::dark();

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.weak_text_color = Some(TEXT_MUTED);
    visuals.warn_fg_color = ACCENT;
    visuals.error_fg_color = DANGER_HOVER;
    visuals.hyperlink_color = ACCENT_HOVER;
    visuals.panel_fill = PANEL_BG;
    visuals.window_fill = PANEL_BG;
    visuals.window_stroke = Stroke::new(1.0, BORDER);
    visuals.window_corner_radius = radius(CORNER_PANEL, scale);
    visuals.menu_corner_radius = radius(CORNER_CONTROL, scale);
    visuals.extreme_bg_color = EXTREME_BG;
    visuals.faint_bg_color = CARD_BG;
    visuals.slider_trailing_fill = true;

    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.5, TEXT_PRIMARY);

    let ctrl_radius = radius(CORNER_CONTROL, scale);
    visuals.widgets.noninteractive.bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.weak_bg_fill = PANEL_BG;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    visuals.widgets.noninteractive.corner_radius = ctrl_radius;

    visuals.widgets.inactive.bg_fill = CONTROL_BG;
    visuals.widgets.inactive.weak_bg_fill = BTN_BG;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.2, TEXT_PRIMARY);
    visuals.widgets.inactive.corner_radius = ctrl_radius;

    visuals.widgets.hovered.bg_fill = BTN_HOVER;
    visuals.widgets.hovered.weak_bg_fill = BTN_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.2, ACCENT);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
    visuals.widgets.hovered.corner_radius = ctrl_radius;
    visuals.widgets.hovered.expansion = 0.0;

    visuals.widgets.active.bg_fill = BTN_ACTIVE;
    visuals.widgets.active.weak_bg_fill = BTN_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, ACCENT_HOVER);
    visuals.widgets.active.fg_stroke = Stroke::new(1.8, TEXT_PRIMARY);
    visuals.widgets.active.corner_radius = ctrl_radius;
    visuals.widgets.active.expansion = 0.0;

    visuals.widgets.open.bg_fill = BTN_HOVER;
    visuals.widgets.open.weak_bg_fill = BTN_HOVER;
    visuals.widgets.open.bg_stroke = Stroke::new(1.2, ACCENT);
    visuals.widgets.open.fg_stroke = Stroke::new(1.5, TEXT_PRIMARY);
    visuals.widgets.open.corner_radius = ctrl_radius;

    style.visuals = visuals;
    style.spacing.item_spacing = vec2(10.0 * scale, 8.0 * scale); // new: DECISIONS 2026-11-21
    style.spacing.button_padding = vec2(12.0 * scale, 6.0 * scale); // new: DECISIONS 2026-11-21
    style.spacing.slider_width = 185.0 * scale; // new: DECISIONS 2026-11-21
    style.spacing.slider_rail_height = 6.0 * scale; // new: DECISIONS 2026-11-21
    style.spacing.combo_width = 160.0 * scale; // new: DECISIONS 2026-11-21
    style.spacing.icon_width = 18.0 * scale; // new: DECISIONS 2026-11-21
    style.spacing.icon_width_inner = 10.0 * scale; // new: DECISIONS 2026-11-21
    style.spacing.interact_size = vec2(40.0 * scale, 24.0 * scale); // new: DECISIONS 2026-11-21

    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(FONT_HEADING * scale, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Body,
        FontId::new(FONT_BODY * scale, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(FONT_BUTTON * scale, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(FONT_HELP * scale, FontFamily::Proportional),
    );

    style
}

/// Standard dark charcoal panel frame with subtle border and scaled padding.
pub fn panel_frame(scale: f32, border_color: Color32) -> Frame {
    Frame::NONE
        .fill(PANEL_BG)
        .inner_margin(margin(24.0, scale)) // new: DECISIONS 2026-11-21
        .corner_radius(radius(CORNER_PANEL, scale))
        .stroke(Stroke::new(1.0, border_color))
}

/// Paints a crisp 3px accent stripe along the top edge of a panel rect.
pub fn paint_top_accent(ui: &egui::Ui, rect: Rect, color: Color32, scale: f32) {
    let h = (3.0 * scale).max(2.0); // new: DECISIONS 2026-11-21
    let bar = Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + h));
    ui.painter().rect_filled(bar, 0.0, color);
}

/// Draws a thin horizontal separator rule across `width`.
pub fn paint_divider(ui: &mut egui::Ui, width: f32, scale: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, (1.0 * scale).max(1.0)), Sense::hover());
    ui.painter()
        .hline(rect.x_range(), rect.center().y, Stroke::new(1.0, DIVIDER));
}

/// Paints a smooth dark charcoal gradient over the left half of `rect`, leaving the right half clear.
pub fn paint_left_gradient(painter: &egui::Painter, rect: Rect) {
    let w = rect.width();
    // 4 vertical stops from left edge fading smoothly to transparent past the screen midpoint.
    let stops = [
        (0.00_f32, Color32::from_rgba_premultiplied(11, 13, 15, 235)), // new: DECISIONS 2026-11-21
        (0.30_f32, Color32::from_rgba_premultiplied(10, 11, 13, 204)), // new: DECISIONS 2026-11-21
        (0.46_f32, Color32::from_rgba_premultiplied(4, 5, 6, 92)),  // new: DECISIONS 2026-11-21
        (0.58_f32, Color32::from_rgba_premultiplied(0, 0, 0, 0)),   // new: DECISIONS 2026-11-21
    ];

    let mut mesh = Mesh::default();
    for (frac, color) in stops {
        let x = rect.left() + w * frac;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: pos2(x, rect.top()),
            uv: egui::epaint::WHITE_UV,
            color,
        });
        mesh.vertices.push(egui::epaint::Vertex {
            pos: pos2(x, rect.bottom()),
            uv: egui::epaint::WHITE_UV,
            color,
        });
    }

    for seg in 0..(stops.len() as u32 - 1) {
        let tl = seg * 2;
        let bl = tl + 1;
        let tr = tl + 2;
        let br = tl + 3;
        mesh.indices.extend_from_slice(&[tl, tr, br, tl, br, bl]);
    }

    painter.add(Shape::mesh(mesh));
}

/// Wide, left-aligned menu button with a colored left accent bar on hover/focus.
pub fn menu_button(
    ui: &mut egui::Ui,
    label: &str,
    size: Vec2,
    scale: f32,
    tone: ButtonTone,
    enabled: bool,
) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, mut response) = ui.allocate_exact_size(size, sense);
    response.widget_info(|| WidgetInfo::labeled(Role::Button, enabled, label));
    if enabled {
        response = response.on_hover_cursor(CursorIcon::PointingHand);
    }

    if ui.is_rect_visible(rect) {
        let hovered = enabled && (response.hovered() || response.has_focus());
        let active = enabled && response.is_pointer_button_down_on();

        let (accent_rest, accent_hot) = match tone {
            ButtonTone::Default => (Color32::TRANSPARENT, ACCENT),
            ButtonTone::Primary => (ACCENT, ACCENT_HOVER),
            ButtonTone::Safe => (SAFE, SAFE_HOVER),
            ButtonTone::Danger => (DANGER, DANGER_HOVER),
        };

        let bg = if !enabled {
            BTN_DISABLED
        } else if active {
            BTN_ACTIVE
        } else if hovered {
            BTN_HOVER
        } else {
            BTN_BG
        };

        let border_color = if !enabled {
            Color32::from_rgba_premultiplied(21, 23, 25, 90) // new: DECISIONS 2026-11-21
        } else if hovered {
            accent_hot
        } else {
            BORDER
        };

        let text_color = if !enabled {
            TEXT_DISABLED
        } else if tone == ButtonTone::Danger && hovered {
            DANGER_HOVER
        } else {
            TEXT_PRIMARY
        };

        let r = radius(CORNER_CONTROL, scale);
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.rect(rect, r, bg, Stroke::new(1.0, border_color), StrokeKind::Inside);

        let bar_color = if !enabled {
            Color32::TRANSPARENT
        } else if hovered {
            accent_hot
        } else {
            accent_rest
        };
        if bar_color != Color32::TRANSPARENT {
            let bar_w = if hovered { 4.5 * scale } else { 3.0 * scale }; // new: DECISIONS 2026-11-21
            let bar_rect = Rect::from_min_max(rect.min, pos2(rect.min.x + bar_w, rect.max.y));
            painter.rect_filled(bar_rect, 0.0, bar_color);
        }

        let font_size = if size.y < 40.0 * scale {
            FONT_BODY * scale
        } else {
            FONT_BUTTON * scale
        };
        let pad_x = 18.0 * scale; // new: DECISIONS 2026-11-21
        painter.text(
            pos2(rect.left() + pad_x, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::new(font_size, FontFamily::Proportional),
            text_color,
        );
    }

    response
}

/// Top tab button for the Settings screen with a bottom rust-orange indicator when active.
pub fn tab_button(
    ui: &mut egui::Ui,
    label: &str,
    width: f32,
    scale: f32,
    selected: bool,
) -> Response {
    let height = 38.0 * scale; // new: DECISIONS 2026-11-21
    let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::click());
    response.widget_info(|| WidgetInfo::selected(Role::Button, true, selected, label));
    let response = response.on_hover_cursor(CursorIcon::PointingHand);

    if ui.is_rect_visible(rect) {
        let hovered = response.hovered();
        let bg = if selected {
            ACCENT_SUBTLE
        } else if hovered {
            BTN_HOVER
        } else {
            Color32::TRANSPARENT
        };
        let text_color = if selected || hovered { TEXT_PRIMARY } else { TEXT_MUTED };

        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.rect_filled(rect, radius(CORNER_CONTROL, scale), bg);

        if selected {
            let bar_h = (3.0 * scale).max(2.0); // new: DECISIONS 2026-11-21
            let bar = Rect::from_min_max(pos2(rect.left(), rect.bottom() - bar_h), rect.max);
            painter.rect_filled(bar, 0.0, ACCENT);
        }

        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            label,
            FontId::new(FONT_BODY * scale, FontFamily::Proportional),
            text_color,
        );
    }

    response
}

/// Compact pill status badge (e.g., ACTIVE / EMPTY / DAMAGED on save cards).
pub fn status_badge(ui: &mut egui::Ui, label: &str, color: Color32, scale: f32) {
    let bg = Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 38); // new: DECISIONS 2026-11-21
    Frame::NONE
        .fill(bg)
        .stroke(Stroke::new(1.0, color))
        .corner_radius(radius(3.0, scale)) // new: DECISIONS 2026-11-21
        .inner_margin(Margin::symmetric(
            (7.0 * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8, // new: DECISIONS 2026-11-21
            (2.0 * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8, // new: DECISIONS 2026-11-21
        ))
        .show(ui, |ui| {
            ui.add(
                egui::Label::new(
                    egui::RichText::new(label)
                        .size(11.0 * scale) // new: DECISIONS 2026-11-21
                        .color(color)
                        .strong(),
                )
                .wrap_mode(egui::TextWrapMode::Extend),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_scale_tracks_screen_height_and_clamps() {
        assert!((ui_scale(720.0) - 1.0).abs() < 1e-4);
        assert!((ui_scale(1080.0) - 1.5).abs() < 1e-4);
        assert!((ui_scale(1440.0) - MAX_SCALE).abs() < 1e-4);
        assert!((ui_scale(400.0) - MIN_SCALE).abs() < 1e-4);
        assert!((ui_scale(0.0) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn with_menu_style_restores_previous_style() {
        let ctx = egui::Context::default();
        let before = ctx.global_style();

        let inside_panel_fill = with_menu_style(&ctx, 1.0, || ctx.global_style().visuals.panel_fill);
        let after = ctx.global_style();

        assert_eq!(inside_panel_fill, PANEL_BG);
        assert_eq!(before.visuals.panel_fill, after.visuals.panel_fill);
        assert_eq!(before.spacing.item_spacing, after.spacing.item_spacing);
    }
}
