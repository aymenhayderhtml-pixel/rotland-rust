//! Graphical widgets and drawing helpers for upgrade icons, pips, bars, and costs.

use engine::egui::{
    self, Align2, Color32, CornerRadius, FontFamily, FontId, Rect, Sense, Stroke, StrokeKind,
    Vec2, pos2, vec2,
};
use game::camp_upgrades::UpgradeLine;

use crate::menus::theme;

use super::colors::{CostRowState, cost_row_color};
use super::super::view::UpgradeLineView;

const ICON_CUTOUT: Color32 = Color32::from_rgb(16, 18, 21); // moved: rotland/src/camp_ui/draw_upgrades.rs

pub fn find_item_color(registry: Option<&game::ItemRegistry>, name: &str) -> Option<Color32> {
    let reg = registry?;
    let def = reg.get(name).or_else(|| {
        reg.all().find(|d| d.name.eq_ignore_ascii_case(name) || d.id.eq_ignore_ascii_case(name))
    })?;
    Some(crate::camp_ui::draw::parse_hex_color(&def.color))
}

pub fn draw_line_icon(
    painter: &egui::Painter,
    rect: Rect,
    line: UpgradeLine,
    color: Color32,
    scale: f32,
) {
    painter.rect(
        rect,
        theme::radius(theme::CORNER_CONTROL, scale),
        theme::CARD_EMPTY_BG,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );

    let w = rect.width();
    let h = rect.height();
    let pt = |nx: f32, ny: f32| pos2(rect.left() + w * nx, rect.top() + h * ny);
    let sub = |x0: f32, y0: f32, x1: f32, y1: f32| Rect::from_min_max(pt(x0, y0), pt(x1, y1));

    match line {
        UpgradeLine::Walls => {
            painter.rect_filled(sub(0.14, 0.20, 0.32, 0.38), 1.0, color);
            painter.rect_filled(sub(0.41, 0.20, 0.59, 0.38), 1.0, color);
            painter.rect_filled(sub(0.68, 0.20, 0.86, 0.38), 1.0, color);
            painter.rect_filled(sub(0.14, 0.36, 0.86, 0.84), 1.0, color);
            let mortar = Stroke::new((1.2 * scale).max(1.0), ICON_CUTOUT);
            painter.line_segment([pt(0.15, 0.56), pt(0.85, 0.56)], mortar);
            painter.line_segment([pt(0.38, 0.37), pt(0.38, 0.56)], mortar);
            painter.line_segment([pt(0.62, 0.37), pt(0.62, 0.56)], mortar);
            painter.rect_filled(
                sub(0.40, 0.58, 0.60, 0.84),
                theme::radius(3.0, scale),
                ICON_CUTOUT,
            );
        }
        UpgradeLine::Guards => {
            painter.rect_filled(sub(0.18, 0.14, 0.82, 0.22), 1.0, color);
            painter.rect_filled(sub(0.24, 0.20, 0.76, 0.44), 1.0, color);
            painter.rect_filled(sub(0.33, 0.27, 0.67, 0.36), 1.0, ICON_CUTOUT);
            let leg = Stroke::new((2.4 * scale).max(1.5), color);
            let brace = Stroke::new((1.6 * scale).max(1.0), color);
            painter.line_segment([pt(0.33, 0.44), pt(0.23, 0.82)], leg);
            painter.line_segment([pt(0.67, 0.44), pt(0.77, 0.82)], leg);
            painter.line_segment([pt(0.31, 0.50), pt(0.73, 0.78)], brace);
            painter.line_segment([pt(0.69, 0.50), pt(0.27, 0.78)], brace);
            painter.rect_filled(sub(0.16, 0.80, 0.84, 0.87), 1.0, color);
        }
        UpgradeLine::Workshop => {
            let shaft = Stroke::new((4.2 * scale).max(2.5), color);
            painter.line_segment([pt(0.26, 0.74), pt(0.64, 0.36)], shaft);
            painter.circle_filled(pt(0.24, 0.76), w * 0.09, color);
            painter.circle_filled(pt(0.24, 0.76), w * 0.035, ICON_CUTOUT);
            painter.circle_filled(pt(0.68, 0.32), w * 0.18, color);
            let jaw_cut = Stroke::new((4.8 * scale).max(2.5), ICON_CUTOUT);
            painter.line_segment([pt(0.67, 0.33), pt(0.84, 0.16)], jaw_cut);
            painter.circle_filled(pt(0.79, 0.21), w * 0.09, ICON_CUTOUT);
        }
        UpgradeLine::Traders => {
            let center = rect.center();
            painter.circle_filled(center, w * 0.35, color);
            painter.circle_stroke(
                center,
                w * 0.27,
                Stroke::new((1.2 * scale).max(1.0), ICON_CUTOUT),
            );
            painter.text(
                center,
                Align2::CENTER_CENTER,
                "$",
                FontId::new(15.0 * scale, FontFamily::Proportional), // moved: rotland/src/camp_ui/draw_upgrades.rs
                ICON_CUTOUT,
            );
        }
        UpgradeLine::Gunsmith => {
            let center = rect.center();
            let ring = Stroke::new((2.0 * scale).max(1.5), color);
            painter.circle_stroke(center, w * 0.28, ring);
            let tick = Stroke::new((1.6 * scale).max(1.0), color);
            painter.line_segment([pt(0.16, 0.50), pt(0.36, 0.50)], tick);
            painter.line_segment([pt(0.64, 0.50), pt(0.84, 0.50)], tick);
            painter.line_segment([pt(0.50, 0.16), pt(0.50, 0.36)], tick);
            painter.line_segment([pt(0.50, 0.64), pt(0.50, 0.84)], tick);
            painter.circle_filled(center, w * 0.06, color);
        }
    }
}

pub fn draw_level_pips(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    building_target: Option<u8>,
    width: f32,
    scale: f32,
) {
    let h = 18.0 * scale; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let (rect, _) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    let label_text = format!("Lv {}/{}", line.current_level, line.max_level);
    let label_w = 48.0 * scale; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let pips_count = usize::from(line.max_level.max(1));
    let pip_gap = 4.0 * scale; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let pips_area_w = (width - label_w - 6.0 * scale).max(24.0 * scale);
    let pip_w = ((pips_area_w - pip_gap * (pips_count.saturating_sub(1) as f32))
        / (pips_count as f32))
        .max(6.0 * scale);
    let pip_h = 7.0 * scale; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let pip_y = rect.center().y - pip_h * 0.5;
    let r = theme::radius(2.0, scale);
    let maxed = line.current_level >= line.max_level && line.max_level > 0;

    for idx in 0..pips_count {
        let lvl = (idx as u8).saturating_add(1);
        let x0 = rect.left() + (idx as f32) * (pip_w + pip_gap);
        let pip_rect = Rect::from_min_max(pos2(x0, pip_y), pos2(x0 + pip_w, pip_y + pip_h));
        let (fill, stroke_color) = if lvl <= line.current_level {
            let c = if maxed { theme::SAFE } else { theme::ACCENT };
            (c, c)
        } else if building_target == Some(lvl) {
            (theme::ACCENT_SUBTLE, theme::ACCENT_HOVER)
        } else {
            (theme::CARD_EMPTY_BG, theme::BORDER)
        };
        painter.rect(
            pip_rect,
            r,
            fill,
            Stroke::new(1.0, stroke_color),
            StrokeKind::Inside,
        );
    }

    painter.text(
        pos2(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        label_text,
        FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional),
        if maxed {
            theme::SAFE_HOVER
        } else {
            theme::TEXT_MUTED
        },
    );
}

pub fn draw_bar(
    ui: &mut egui::Ui,
    size: Vec2,
    fraction: f32,
    fill_color: Color32,
    scale: f32,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    painter.rect(
        rect,
        theme::radius(3.0, scale),
        theme::CARD_EMPTY_BG,
        Stroke::new(1.0, theme::BORDER),
        StrokeKind::Inside,
    );
    let clamped = fraction.clamp(0.0, 1.0);
    if clamped > 0.0 {
        let inner = rect.shrink(1.0);
        let fill_w = (inner.width() * clamped)
            .max(2.0 * scale)
            .min(inner.width());
        let fill_rect = Rect::from_min_max(inner.min, pos2(inner.min.x + fill_w, inner.max.y));
        painter.rect_filled(fill_rect, theme::radius(2.0, scale), fill_color);
    }
}

pub fn draw_cost_row(
    ui: &mut egui::Ui,
    label: &str,
    amount: &str,
    state: CostRowState,
    icon_color: Option<Color32>,
    width: f32,
    scale: f32,
) {
    let row_h = 18.0 * scale; // moved: rotland/src/camp_ui/draw_upgrades.rs
    let (rect, _) = ui.allocate_exact_size(vec2(width, row_h), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let text_color = cost_row_color(state);

    let offset_x = if let Some(c) = icon_color {
        let icon_sz = (12.0 * scale).max(8.0); // new: DECISIONS 2026-11-22
        let icon_rect = Rect::from_min_max(
            pos2(rect.left() + 2.0, rect.center().y - icon_sz * 0.5),
            pos2(rect.left() + 2.0 + icon_sz, rect.center().y + icon_sz * 0.5),
        );
        painter.rect_filled(icon_rect, CornerRadius::same(2), c);
        icon_sz + 6.0 * scale // new: DECISIONS 2026-11-22
    } else {
        let dot_r = (3.0 * scale).max(2.0); // moved: rotland/src/camp_ui/draw_upgrades.rs
        painter.circle_filled(
            pos2(rect.left() + dot_r + 1.0, rect.center().y),
            dot_r,
            text_color,
        );
        dot_r * 2.0 + 7.0 * scale // moved: rotland/src/camp_ui/draw_upgrades.rs
    };

    let font = FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional);
    painter.text(
        pos2(rect.left() + offset_x, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        font.clone(),
        theme::TEXT_PRIMARY,
    );
    painter.text(
        pos2(rect.right(), rect.center().y),
        Align2::RIGHT_CENTER,
        amount,
        font,
        text_color,
    );
}
