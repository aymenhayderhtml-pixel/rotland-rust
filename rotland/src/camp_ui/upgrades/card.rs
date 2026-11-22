//! Individual upgrade card rendering for one upgrade line on the Camp Board.

use engine::egui::{
    self, Align2, Color32, FontFamily, FontId, Frame, Margin, Rect, RichText, ScrollArea,
    Sense, Stroke, Vec2, pos2, vec2,
};
use game::ItemRegistry;
use game::camp_upgrades::UpgradeLine;

use crate::camp_ui::action::CampAction;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::{CampUpgradesView, CampView, NextUpgradeView, UpgradeLineView};
use crate::menus::theme::{self, ButtonTone};

use super::colors::{
    cash_cost_state, current_summary_text, effective_cant_start_reason, format_cash_cost,
    format_hours, format_item_cost, format_wall_hp_label, item_cost_state, line_display_name,
    wall_hp_color, wall_hp_fraction,
};
use super::detail::{draw_bar, draw_cost_row, draw_level_pips, draw_line_icon, find_item_color};
use super::in_progress::draw_in_progress_body;
use super::layout::CardLayout;

const CASH_COLOR: Color32 = Color32::from_rgb(143, 194, 74); // new: DECISIONS 2026-11-22

pub fn draw_upgrade_card(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    view: &CampView,
    state: &mut CampUiState,
    layout: &CardLayout,
    scale: f32,
) -> Option<CampAction> {
    let upgrades = view.upgrades.as_ref()?;
    let item_registry = view.item_registry;
    let size = layout.card_size();
    let font_size = layout.font_size;

    let is_building = upgrades
        .in_progress
        .as_ref()
        .is_some_and(|p| p.line == line.line);
    let is_maxed = line.is_max_level();

    let border_color = if is_building {
        theme::ACCENT
    } else if is_maxed {
        theme::SAFE
    } else {
        theme::BORDER
    };
    let icon_color = if is_maxed { theme::SAFE } else { theme::ACCENT };

    let pad = (8.0 * scale).round().clamp(4.0, 14.0) as i8; // new: DECISIONS 2026-11-22
    let inner_w = (size.x - 2.0 * f32::from(pad)).max(80.0);
    let inner_h = (size.y - 2.0 * f32::from(pad)).max(120.0);

    let mut action = None;

    Frame::NONE
        .fill(theme::CARD_BG)
        .inner_margin(Margin::same(pad))
        .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
        .stroke(Stroke::new(1.0, border_color))
        .show(ui, |ui| {
            ui.set_width(inner_w);
            ui.set_min_height(inner_h);
            ui.set_max_height(inner_h);
            ui.spacing_mut().item_spacing = vec2(0.0, 4.0 * scale);

            // 1. Header: Icon, line title, level pips
            draw_card_header(ui, line, icon_color, inner_w, font_size, scale);

            // 2. Current summary & Wall HP (if walls)
            draw_current_summary(ui, line, upgrades, inner_w, scale);

            // 3. Body: in-progress, completed, or next level
            if is_building {
                if let Some(prog) = &upgrades.in_progress {
                    action = draw_in_progress_body(ui, prog, upgrades, state, inner_w, scale);
                }
            } else if is_maxed {
                draw_complete_body(ui, scale);
            } else if let Some(next) = &line.next {
                action = draw_next_level_body(
                    ui,
                    line,
                    next,
                    upgrades,
                    item_registry,
                    inner_w,
                    scale,
                );
            }
        });

    action
}

fn draw_card_header(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    icon_color: Color32,
    width: f32,
    font_size: f32,
    scale: f32,
) {
    let header_h = 28.0 * scale; // new: DECISIONS 2026-11-22
    let (rect, _) = ui.allocate_exact_size(vec2(width, header_h), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));

    let icon_sz = 26.0 * scale; // new: DECISIONS 2026-11-22
    let icon_rect = Rect::from_min_max(rect.min, pos2(rect.min.x + icon_sz, rect.min.y + icon_sz));
    draw_line_icon(&painter, icon_rect, line.line, icon_color, scale);

    let title_x = icon_rect.right() + 7.0 * scale; // new: DECISIONS 2026-11-22
    let title_font = FontId::new(font_size.max(12.0) * scale, FontFamily::Proportional);
    painter.text(
        pos2(title_x, rect.center().y),
        Align2::LEFT_CENTER,
        line_display_name(line.line),
        title_font,
        theme::TEXT_PRIMARY,
    );

    let pips_w = (80.0 * scale).min(width * 0.35); // new: DECISIONS 2026-11-22
    let pips_rect = Rect::from_min_max(pos2(rect.right() - pips_w, rect.min.y), rect.max);
    let mut pips_ui = ui.new_child(egui::UiBuilder::new().max_rect(pips_rect));
    draw_level_pips(&mut pips_ui, line, None, pips_w, scale);
}

fn draw_current_summary(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    upgrades: &CampUpgradesView,
    width: f32,
    scale: f32,
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Current:")
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_DISABLED)
                .strong(),
        );
        ui.label(
            RichText::new(current_summary_text(line))
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_MUTED),
        );
    });

    if line.line == UpgradeLine::Walls {
        let frac = wall_hp_fraction(upgrades.wall_hp, upgrades.wall_hp_max);
        let bar_color = wall_hp_color(frac);
        let hp_label = format_wall_hp_label(upgrades.wall_hp, upgrades.wall_hp_max);

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Wall HP: {hp_label}"))
                    .size(11.0 * scale) // new: DECISIONS 2026-11-22
                    .color(bar_color),
            );
        });
        draw_bar(ui, vec2(width, 6.0 * scale), frac, bar_color, scale);
    }
}

fn draw_complete_body(ui: &mut egui::Ui, scale: f32) {
    ui.add_space(10.0 * scale);
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        theme::status_badge(ui, "COMPLETE", theme::SAFE, scale);
        ui.add_space(6.0 * scale);
        ui.label(
            RichText::new("Maximum level reached")
                .size(theme::FONT_BODY * scale)
                .color(theme::TEXT_PRIMARY)
                .strong(),
        );
        ui.label(
            RichText::new("All upgrades in this line are built.")
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_MUTED),
        );
    });
}

fn draw_next_level_body(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    next: &NextUpgradeView,
    upgrades: &CampUpgradesView,
    item_registry: Option<&ItemRegistry>,
    width: f32,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Next: {}", next.name))
                .size(theme::FONT_BODY * scale)
                .color(theme::ACCENT_HOVER)
                .strong(),
        );
    });

    if !next.description.is_empty() {
        ui.add(
            egui::Label::new(
                RichText::new(&next.description)
                    .size(theme::FONT_HELP * scale)
                    .color(theme::TEXT_MUTED),
            )
            .wrap_mode(egui::TextWrapMode::Wrap),
        );
    }

    ui.add_space(2.0 * scale);
    theme::paint_divider(ui, width, scale);
    ui.add_space(2.0 * scale);

    let costs_h = (58.0 * scale).min(ui.available_height() - 44.0 * scale).max(24.0 * scale); // new: DECISIONS 2026-11-22
    ScrollArea::vertical()
        .id_salt(format!("card_costs_{}", line.line.as_str()))
        .max_height(costs_h)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 2.0 * scale);
            if next.cash_cost > 0 {
                draw_cost_row(
                    ui,
                    "Cash",
                    &format_cash_cost(upgrades.player_cash, next.cash_cost),
                    cash_cost_state(upgrades.player_cash, next.cash_cost),
                    Some(CASH_COLOR),
                    width,
                    scale,
                );
            }
            for item in &next.items {
                let icon_c = find_item_color(item_registry, &item.name);
                draw_cost_row(
                    ui,
                    &item.name,
                    &format_item_cost(item.have, item.need),
                    item_cost_state(item.have, item.need),
                    icon_c,
                    width,
                    scale,
                );
            }
            if next.cash_cost <= 0 && next.items.is_empty() {
                ui.label(
                    RichText::new("No material cost")
                        .size(theme::FONT_HELP * scale)
                        .color(theme::SAFE_HOVER),
                );
            }
        });

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("Build time: {}", format_hours(next.build_hours)))
                .size(theme::FONT_HELP * scale)
                .color(theme::TEXT_MUTED),
        );
    });

    ui.add_space(3.0 * scale);

    let blocker = effective_cant_start_reason(next, upgrades.in_progress.as_ref(), line.line);
    let can_start = blocker.is_none();
    let btn_h = 32.0 * scale; // new: DECISIONS 2026-11-22
    let mut btn = theme::menu_button(ui, "Upgrade", vec2(width, btn_h), scale, ButtonTone::Primary, can_start);
    if let Some(reason) = blocker {
        btn = btn.on_hover_text(reason);
        ui.label(
            RichText::new(reason)
                .size(11.0 * scale) // new: DECISIONS 2026-11-22
                .color(theme::DANGER_HOVER),
        );
    }
    if can_start && btn.clicked() {
        action = Some(CampAction::StartUpgrade(line.line));
    }

    action
}
