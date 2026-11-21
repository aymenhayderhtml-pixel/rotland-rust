//! egui 0.37 rendering for the Camp Board (`CampScreen::Upgrades`) using the shared menu theme.

use engine::egui::{
    self, Align2, Color32, FontFamily, FontId, Frame, Id, Margin, Order, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Vec2, pos2, vec2,
};
use game::camp_upgrades::UpgradeLine;

use crate::menus::theme::{self, ButtonTone};

use super::action::CampAction;
use super::screen::CampScreen;
use super::state::CampUiState;
use super::view::{
    CampUpgradesView, CampView, NextUpgradeView, UpgradeInProgressView, UpgradeLineView,
};

const CANCEL_CONFIRM_KEY: &str = "confirm_cancel_upgrade";

const BASE_PANEL_W: f32 = 1160.0; // new: DECISIONS 2026-11-21
const BASE_PANEL_H: f32 = 668.0; // new: DECISIONS 2026-11-21
const PANEL_MARGIN_X: f32 = 32.0; // new: DECISIONS 2026-11-21
const PANEL_MARGIN_Y: f32 = 24.0; // new: DECISIONS 2026-11-21
const PANEL_PAD: f32 = 16.0; // new: DECISIONS 2026-11-21

const HEADER_H: f32 = 38.0; // new: DECISIONS 2026-11-21
const TAB_WIDTH: f32 = 96.0; // new: DECISIONS 2026-11-21
const BUILD_BAR_H: f32 = 52.0; // new: DECISIONS 2026-11-21
const CANCEL_ZONE_W: f32 = 232.0; // new: DECISIONS 2026-11-21
const FOOTER_H: f32 = 36.0; // new: DECISIONS 2026-11-21

const CARD_PAD: f32 = 12.0; // new: DECISIONS 2026-11-21
const ICON_SIZE: f32 = 32.0; // new: DECISIONS 2026-11-21
const PIPS_H: f32 = 18.0; // new: DECISIONS 2026-11-21
const CURRENT_H: f32 = 36.0; // new: DECISIONS 2026-11-21
const EXTRA_BAR_H: f32 = 32.0; // new: DECISIONS 2026-11-21
const NEXT_PAD: f32 = 10.0; // new: DECISIONS 2026-11-21
const DESC_H: f32 = 38.0; // new: DECISIONS 2026-11-21
const COSTS_H: f32 = 74.0; // new: DECISIONS 2026-11-21
const BUILD_BTN_H: f32 = 36.0; // new: DECISIONS 2026-11-21
const REASON_H: f32 = 26.0; // new: DECISIONS 2026-11-21

const WALL_HP_WARN: f32 = 0.5; // new: DECISIONS 2026-11-21
const WALL_HP_CRIT: f32 = 0.25; // new: DECISIONS 2026-11-21
const ICON_CUTOUT: Color32 = Color32::from_rgb(16, 18, 21); // new: DECISIONS 2026-11-21

/// Affordability state of a single cash or material cost row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CostRowState {
    Enough,
    Short,
}

pub(crate) fn cash_cost_state(have_cash: i32, need_cash: i32) -> CostRowState {
    if have_cash >= need_cash {
        CostRowState::Enough
    } else {
        CostRowState::Short
    }
}

pub(crate) fn item_cost_state(have: u32, need: u32) -> CostRowState {
    if have >= need {
        CostRowState::Enough
    } else {
        CostRowState::Short
    }
}

pub(crate) fn cost_row_color(state: CostRowState) -> Color32 {
    match state {
        CostRowState::Enough => theme::SAFE_HOVER,
        CostRowState::Short => theme::DANGER_HOVER,
    }
}

/// Formats in-game hours as e.g. `"3 h 20 min"`, `"2 h"`, `"30 min"`, or `"< 1 min"`.
pub(crate) fn format_hours(hours: f32) -> String {
    if !hours.is_finite() || hours <= 0.0 {
        return "0 min".to_string();
    }
    let total_mins = (hours * 60.0).round() as u32;
    if total_mins == 0 {
        return "< 1 min".to_string();
    }
    let h = total_mins / 60;
    let m = total_mins % 60;
    match (h, m) {
        (0, mins) => format!("{mins} min"),
        (hrs, 0) => format!("{hrs} h"),
        (hrs, mins) => format!("{hrs} h {mins} min"),
    }
}

pub(crate) fn format_building_banner(target_name: &str, hours_left: f32) -> String {
    format!("Building: {target_name} — {} left", format_hours(hours_left))
}

pub(crate) fn format_cash_cost(have_cash: i32, need_cash: i32) -> String {
    format!("${have_cash}/${need_cash}")
}

pub(crate) fn format_item_cost(have: u32, need: u32) -> String {
    format!("{have}/{need}")
}

pub(crate) fn upgrade_progress_fraction(hours_left: f32, total_hours: f32) -> f32 {
    if !total_hours.is_finite() || total_hours <= 0.0 {
        return if hours_left.is_finite() && hours_left <= 0.0 {
            1.0
        } else {
            0.0
        };
    }
    let remaining = if hours_left.is_finite() {
        hours_left.clamp(0.0, total_hours)
    } else {
        total_hours
    };
    ((total_hours - remaining) / total_hours).clamp(0.0, 1.0)
}

pub(crate) fn wall_hp_fraction(wall_hp: f32, wall_hp_max: f32) -> f32 {
    if !wall_hp_max.is_finite() || wall_hp_max <= 0.0 || !wall_hp.is_finite() {
        return 0.0;
    }
    (wall_hp / wall_hp_max).clamp(0.0, 1.0)
}

pub(crate) fn wall_hp_color(fraction: f32) -> Color32 {
    if fraction > WALL_HP_WARN {
        theme::SAFE
    } else if fraction > WALL_HP_CRIT {
        theme::ACCENT
    } else {
        theme::DANGER
    }
}

pub(crate) fn format_wall_hp_label(wall_hp: f32, wall_hp_max: f32) -> String {
    if !wall_hp_max.is_finite() || wall_hp_max <= 0.0 {
        return "No walls".to_string();
    }
    let cur = if wall_hp.is_finite() {
        wall_hp.clamp(0.0, wall_hp_max)
    } else {
        0.0
    };
    if cur <= 0.0 {
        format!("0 / {wall_hp_max:.0} (Breached)")
    } else {
        format!("{cur:.0} / {wall_hp_max:.0}")
    }
}

pub(crate) fn refund_percent(refund_ratio: f32) -> u32 {
    if !refund_ratio.is_finite() {
        return 0;
    }
    (refund_ratio.clamp(0.0, 1.0) * 100.0).round() as u32
}

pub(crate) fn cancel_button_label(refund_ratio: f32) -> String {
    format!("Cancel (refund {}%)", refund_percent(refund_ratio))
}

pub(crate) fn line_display_name(line: UpgradeLine) -> &'static str {
    match line {
        UpgradeLine::Walls => "Walls & Gates",
        UpgradeLine::Guards => "Guards & Towers",
        UpgradeLine::Workshop => "Workshop",
        UpgradeLine::Traders => "Traders & Services",
    }
}

pub(crate) fn current_summary_text(line: &UpgradeLineView) -> String {
    let trimmed = line.current_summary.trim();
    if !trimmed.is_empty() {
        trimmed.to_string()
    } else if line.current_level == 0 {
        "None".to_string()
    } else {
        format!("Level {}", line.current_level)
    }
}

pub(crate) fn resolve_in_progress_name(
    prog: &UpgradeInProgressView,
    upgrades: &CampUpgradesView,
) -> String {
    let trimmed = prog.target_name.trim();
    if !trimmed.is_empty() {
        return trimmed.to_string();
    }
    if let Some(line_view) = upgrades.line(prog.line)
        && let Some(next) = &line_view.next
        && !next.name.trim().is_empty()
    {
        return next.name.trim().to_string();
    }
    format!("{} Lv.{}", line_display_name(prog.line), prog.target_level)
}

pub(crate) fn effective_cant_start_reason<'a>(
    next: &'a NextUpgradeView,
    in_progress: Option<&UpgradeInProgressView>,
    line: UpgradeLine,
) -> Option<&'a str> {
    if let Some(reason) = next.cant_start_reason.as_deref() {
        let trimmed = reason.trim();
        return Some(if trimmed.is_empty() {
            "Unavailable"
        } else {
            trimmed
        });
    }
    if let Some(prog) = in_progress {
        return Some(if prog.line == line {
            "Currently building"
        } else {
            "Another upgrade is in progress"
        });
    }
    None
}

pub(crate) fn panel_dimensions(screen_rect: Rect, scale: f32) -> Vec2 {
    let max_w = (screen_rect.width() - PANEL_MARGIN_X * scale)
        .max(320.0)
        .min(screen_rect.width());
    let max_h = (screen_rect.height() - PANEL_MARGIN_Y * scale)
        .max(320.0)
        .min(screen_rect.height());
    vec2(
        (BASE_PANEL_W * scale).min(max_w),
        (BASE_PANEL_H * scale).min(max_h),
    )
}

fn scaled_i8(base: f32, scale: f32) -> i8 {
    (base * scale).round().clamp(1.0, f32::from(i8::MAX)) as i8
}

/// Renders the Camp Board screen using the shared menu theme and responsive 4-column layout.
pub fn draw_upgrades_screen(
    ctx: &egui::Context,
    state: &mut CampUiState,
    view: &CampView,
) -> Option<CampAction> {
    let screen_rect = ctx.content_rect();
    let scale = theme::ui_scale(screen_rect.height());

    theme::with_menu_style(ctx, scale, || {
        let mut action = None;

        egui::Area::new(Id::unique("camp_upgrades_bg"))
            .fixed_pos(screen_rect.min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.painter().rect_filled(screen_rect, 0.0, theme::SETTINGS_DIM);
                ui.allocate_rect(screen_rect, Sense::click_and_drag());
            });

        let panel_size = panel_dimensions(screen_rect, scale);
        let pad = scaled_i8(PANEL_PAD, scale);
        let inner_size = vec2(
            (panel_size.x - 2.0 * f32::from(pad)).max(240.0),
            (panel_size.y - 2.0 * f32::from(pad)).max(240.0),
        );

        egui::Area::new(Id::unique("camp_upgrades_panel"))
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let frame = Frame::NONE
                    .fill(theme::PANEL_BG)
                    .inner_margin(Margin::same(pad))
                    .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                    .stroke(Stroke::new(1.0, theme::BORDER));

                let resp = frame.show(ui, |ui| {
                    ui.set_width(inner_size.x);
                    ui.set_min_height(inner_size.y);
                    ui.set_max_height(inner_size.y);
                    ui.spacing_mut().item_spacing = vec2(8.0 * scale, 0.0);

                    action = draw_board_contents(ui, state, view, inner_size, scale);
                });

                theme::paint_top_accent(ui, resp.response.rect, theme::ACCENT, scale);
            });

        action
    })
}

fn draw_board_contents(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    view: &CampView,
    size: Vec2,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;
    let cash = view
        .upgrades
        .as_ref()
        .map_or_else(|| view.player_dollars(), |u| u.player_cash);

    draw_header(ui, state, cash, size.x, scale);
    ui.add_space(6.0 * scale);
    theme::paint_divider(ui, size.x, scale);
    ui.add_space(8.0 * scale);

    if let Some(act) = draw_build_bar(ui, state, view.upgrades.as_ref(), size.x, scale) {
        action = Some(act);
    }

    ui.add_space(10.0 * scale);

    let fixed_outer = (HEADER_H + 6.0 + 1.0 + 8.0 + BUILD_BAR_H + 10.0 + 10.0 + FOOTER_H) * scale;
    let cards_h = (size.y - fixed_outer).max(260.0 * scale);

    if let Some(upgrades) = &view.upgrades {
        let col_gap = 10.0 * scale; // new: DECISIONS 2026-11-21
        let col_w = ((size.x - 3.0 * col_gap) / 4.0).floor().max(120.0);
        let card_size = vec2(col_w, cards_h);

        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
            ui.spacing_mut().item_spacing = vec2(col_gap, 0.0);
            for line_id in UpgradeLine::ALL {
                let card_action = if let Some(line_view) = upgrades.line(line_id) {
                    draw_upgrade_card(ui, line_view, upgrades, card_size, scale)
                } else {
                    let fallback = UpgradeLineView {
                        line: line_id,
                        current_level: 0,
                        max_level: 0,
                        current_summary: String::new(),
                        next: None,
                    };
                    draw_upgrade_card(ui, &fallback, upgrades, card_size, scale)
                };
                if let Some(act) = card_action {
                    action = Some(act);
                }
            }
        });
    } else {
        Frame::NONE
            .fill(theme::CARD_EMPTY_BG)
            .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
            .stroke(Stroke::new(1.0, theme::BORDER))
            .show(ui, |ui| {
                ui.set_width(size.x);
                ui.set_min_height(cards_h);
                ui.set_max_height(cards_h);
                ui.add_space(cards_h * 0.44);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new("Camp upgrade data unavailable.")
                            .size(theme::FONT_BODY * scale)
                            .color(theme::TEXT_MUTED),
                    );
                });
            });
    }

    ui.add_space(10.0 * scale);

    ui.allocate_ui(vec2(size.x, FOOTER_H * scale), |ui| {
        ui.set_min_height(FOOTER_H * scale);
        ui.horizontal(|ui| {
            let close_btn = theme::menu_button(
                ui,
                "Close (Esc)",
                vec2(144.0 * scale, FOOTER_H * scale), // new: DECISIONS 2026-11-21
                scale,
                ButtonTone::Default,
                true,
            );
            if close_btn.clicked() {
                action = Some(CampAction::Close);
            }

            if let Some(upgrades) = &view.upgrades {
                if upgrades.has_doctor {
                    ui.add_space(8.0 * scale);
                    let doc_btn = theme::menu_button(
                        ui,
                        "Doctor: Heal ($30)", // new: DECISIONS 2026-11-21
                        vec2(160.0 * scale, FOOTER_H * scale), // new: DECISIONS 2026-11-21
                        scale,
                        ButtonTone::Safe,
                        true,
                    );
                    if doc_btn.clicked() {
                        action = Some(CampAction::DoctorHeal);
                    }
                }
                if upgrades.has_mechanic {
                    ui.add_space(8.0 * scale);
                    let mech_btn = theme::menu_button(
                        ui,
                        "Mechanic: Repair ($50)", // new: DECISIONS 2026-11-21
                        vec2(180.0 * scale, FOOTER_H * scale), // new: DECISIONS 2026-11-21
                        scale,
                        ButtonTone::Primary,
                        true,
                    );
                    if mech_btn.clicked() {
                        action = Some(CampAction::MechanicRepair);
                    }
                }
            }

            if !state.status().is_empty() {
                ui.add_space(14.0 * scale);
                ui.label(
                    RichText::new(state.status())
                        .size(theme::FONT_BODY * scale)
                        .color(theme::SAFE_HOVER),
                );
            }
        });
    });

    action
}

fn draw_header(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    cash: i32,
    width: f32,
    scale: f32,
) {
    let tabs = [
        CampScreen::Shop,
        CampScreen::Bank,
        CampScreen::Crafting,
        CampScreen::Bounties,
        CampScreen::Upgrades,
    ];
    let tab_w = (TAB_WIDTH * scale).min((width * 0.46) / (tabs.len() as f32));

    ui.allocate_ui(vec2(width, HEADER_H * scale), |ui| {
        ui.set_min_height(HEADER_H * scale);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(CampScreen::Upgrades.title())
                    .size(theme::FONT_HEADING * scale)
                    .color(theme::TEXT_PRIMARY)
                    .strong(),
            );
            ui.add_space(14.0 * scale);

            for tab in tabs {
                let selected = state.screen == tab;
                if theme::tab_button(ui, tab.tab_name(), tab_w, scale, selected).clicked() {
                    state.switch_tab(tab);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("${cash}"))
                        .size(theme::FONT_SUBHEADING * scale)
                        .color(theme::ACCENT_HOVER)
                        .strong(),
                );
                ui.label(
                    RichText::new("CASH")
                        .size(theme::FONT_HELP * scale)
                        .color(theme::TEXT_MUTED)
                        .strong(),
                );
            });
        });
    });
}

fn draw_build_bar(
    ui: &mut egui::Ui,
    state: &mut CampUiState,
    upgrades: Option<&CampUpgradesView>,
    width: f32,
    scale: f32,
) -> Option<CampAction> {
    if upgrades.is_none_or(|u| u.in_progress.is_none())
        && state.selected_row() == Some(CANCEL_CONFIRM_KEY)
    {
        state.clear_selection();
    }

    let pad_x = scaled_i8(12.0, scale); // new: DECISIONS 2026-11-21
    let pad_y = scaled_i8(7.0, scale); // new: DECISIONS 2026-11-21
    let bar_h = BUILD_BAR_H * scale;
    let inner_w = (width - 2.0 * f32::from(pad_x)).max(160.0);
    let inner_h = (bar_h - 2.0 * f32::from(pad_y)).max(24.0);

    let active_pair = upgrades.and_then(|u| u.in_progress.as_ref().map(|p| (u, p)));
    let (fill, stroke_color) = if active_pair.is_some() {
        (theme::CARD_BG, theme::ACCENT)
    } else {
        (theme::CARD_EMPTY_BG, theme::BORDER)
    };

    let mut action = None;

    Frame::NONE
        .fill(fill)
        .inner_margin(Margin::symmetric(pad_x, pad_y))
        .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
        .stroke(Stroke::new(1.0, stroke_color))
        .show(ui, |ui| {
            ui.set_width(inner_w);
            ui.set_min_height(inner_h);
            ui.set_max_height(inner_h);

            if let Some((upg, prog)) = active_pair {
                let gap = 14.0 * scale; // new: DECISIONS 2026-11-21
                let cancel_w = (CANCEL_ZONE_W * scale).min(inner_w * 0.4);
                let left_w = (inner_w - cancel_w - gap).max(120.0);
                let progress = upgrade_progress_fraction(prog.hours_left, prog.total_hours);
                let pct = (progress * 100.0).round() as u32;
                let banner = format_building_banner(
                    &resolve_in_progress_name(prog, upg),
                    prog.hours_left,
                );

                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, 0.0);

                    ui.allocate_ui(vec2(left_w, inner_h), |ui| {
                        ui.set_width(left_w);
                        ui.set_min_height(inner_h);
                        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

                        let label_h = 18.0 * scale; // new: DECISIONS 2026-11-21
                        let (row_rect, _) =
                            ui.allocate_exact_size(vec2(left_w, label_h), Sense::hover());
                        if ui.is_rect_visible(row_rect) {
                            let painter =
                                ui.painter().with_clip_rect(row_rect.intersect(ui.clip_rect()));
                            painter.text(
                                pos2(row_rect.left(), row_rect.center().y),
                                Align2::LEFT_CENTER,
                                banner,
                                FontId::new(theme::FONT_BODY * scale, FontFamily::Proportional),
                                theme::TEXT_PRIMARY,
                            );
                            painter.text(
                                pos2(row_rect.right(), row_rect.center().y),
                                Align2::RIGHT_CENTER,
                                format!("{pct}%"),
                                FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional),
                                theme::ACCENT_HOVER,
                            );
                        }

                        ui.add_space(6.0 * scale);
                        draw_bar(ui, vec2(left_w, 10.0 * scale), progress, theme::ACCENT, scale);
                    });

                    ui.allocate_ui(vec2(cancel_w, inner_h), |ui| {
                        ui.set_width(cancel_w);
                        ui.set_min_height(inner_h);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.spacing_mut().item_spacing = vec2(8.0 * scale, 0.0);
                            let btn_h = (32.0 * scale).min(inner_h);
                            let confirming = state.selected_row() == Some(CANCEL_CONFIRM_KEY);
                            if confirming {
                                if theme::menu_button(
                                    ui,
                                    "Keep",
                                    vec2(72.0 * scale, btn_h), // new: DECISIONS 2026-11-21
                                    scale,
                                    ButtonTone::Default,
                                    true,
                                )
                                .clicked()
                                {
                                    state.clear_selection();
                                }
                                if theme::menu_button(
                                    ui,
                                    "Confirm cancel",
                                    vec2(146.0 * scale, btn_h), // new: DECISIONS 2026-11-21
                                    scale,
                                    ButtonTone::Danger,
                                    true,
                                )
                                .clicked()
                                {
                                    state.clear_selection();
                                    action = Some(CampAction::CancelUpgrade);
                                }
                            } else {
                                let label = cancel_button_label(upg.refund_ratio);
                                if theme::menu_button(
                                    ui,
                                    &label,
                                    vec2(204.0 * scale, btn_h), // new: DECISIONS 2026-11-21
                                    scale,
                                    ButtonTone::Danger,
                                    true,
                                )
                                .clicked()
                                {
                                    state.select_row(CANCEL_CONFIRM_KEY);
                                }
                            }
                        });
                    });
                });
            } else {
                let (rect, _) = ui.allocate_exact_size(vec2(inner_w, inner_h), Sense::hover());
                if ui.is_rect_visible(rect) {
                    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
                    painter.text(
                        pos2(rect.left(), rect.center().y),
                        Align2::LEFT_CENTER,
                        "No upgrade in progress — choose a camp improvement below.",
                        FontId::new(theme::FONT_BODY * scale, FontFamily::Proportional),
                        theme::TEXT_MUTED,
                    );
                    painter.text(
                        pos2(rect.right(), rect.center().y),
                        Align2::RIGHT_CENTER,
                        "One build at a time",
                        FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional),
                        theme::TEXT_DISABLED,
                    );
                }
            }
        });

    action
}

fn draw_upgrade_card(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    upgrades: &CampUpgradesView,
    size: Vec2,
    scale: f32,
) -> Option<CampAction> {
    let building_target = upgrades
        .in_progress
        .as_ref()
        .filter(|p| p.line == line.line)
        .map(|p| p.target_level);
    let is_building = building_target.is_some();
    let is_maxed = line.is_max_level();

    let border_color = if is_building {
        theme::ACCENT
    } else if is_maxed {
        theme::SAFE
    } else {
        theme::BORDER
    };
    let icon_color = if is_maxed { theme::SAFE } else { theme::ACCENT };

    let pad = scaled_i8(CARD_PAD, scale);
    let inner_w = (size.x - 2.0 * f32::from(pad)).max(96.0);
    let inner_h = (size.y - 2.0 * f32::from(pad)).max(200.0);

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
            ui.spacing_mut().item_spacing = vec2(6.0 * scale, 0.0);

            draw_card_header(ui, line.line, icon_color, inner_w, scale);
            ui.add_space(6.0 * scale);

            draw_level_pips(ui, line, building_target, inner_w, scale);
            ui.add_space(6.0 * scale);

            // Short "what you have now" line
            let current_h = CURRENT_H * scale;
            ui.allocate_ui(vec2(inner_w, current_h), |ui| {
                ui.set_width(inner_w);
                ui.set_min_height(current_h);
                ui.set_max_height(current_h);
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0 * scale);
                ui.label(
                    RichText::new("CURRENT")
                        .size(10.5 * scale) // new: DECISIONS 2026-11-21
                        .color(theme::TEXT_DISABLED)
                        .strong(),
                );
                ui.add(
                    egui::Label::new(
                        RichText::new(current_summary_text(line))
                            .size(theme::FONT_HELP * scale)
                            .color(theme::TEXT_PRIMARY),
                    )
                    .wrap_mode(egui::TextWrapMode::Wrap),
                );
            });

            ui.add_space(6.0 * scale);
            draw_extra_slot(ui, line.line, upgrades, is_building, inner_w, scale);
            ui.add_space(6.0 * scale);

            let top_used =
                (ICON_SIZE + 6.0 + PIPS_H + 6.0 + CURRENT_H + 6.0 + EXTRA_BAR_H + 6.0) * scale;
            let next_h = (inner_h - top_used).max(160.0 * scale);
            action = draw_next_box(ui, line, upgrades, vec2(inner_w, next_h), scale);
        });

    action
}

fn draw_card_header(
    ui: &mut egui::Ui,
    line: UpgradeLine,
    icon_color: Color32,
    width: f32,
    scale: f32,
) {
    let icon_sz = ICON_SIZE * scale;
    let (rect, _) = ui.allocate_exact_size(vec2(width, icon_sz), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let icon_rect = Rect::from_min_max(rect.min, pos2(rect.min.x + icon_sz, rect.min.y + icon_sz));
    draw_line_icon(ui.painter(), icon_rect, line, icon_color, scale);

    let title_x = icon_rect.right() + 9.0 * scale; // new: DECISIONS 2026-11-21
    ui.painter()
        .with_clip_rect(rect.intersect(ui.clip_rect()))
        .text(
            pos2(title_x, rect.center().y),
            Align2::LEFT_CENTER,
            line_display_name(line),
            FontId::new(15.5 * scale, FontFamily::Proportional), // new: DECISIONS 2026-11-21
            theme::TEXT_PRIMARY,
        );
}

fn draw_line_icon(
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
            // Battlement merlons, brick wall body, mortar lines, and central gate
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
            // Watchtower roof, cabin with lookout slit, braced legs, and base
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
            // Diagonal wrench handle, jaw head, and cut-out open jaw
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
            // Trade coin disc with inner rim and "$" mark
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
                FontId::new(15.0 * scale, FontFamily::Proportional), // new: DECISIONS 2026-11-21
                ICON_CUTOUT,
            );
        }
    }
}

fn draw_level_pips(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    building_target: Option<u8>,
    width: f32,
    scale: f32,
) {
    let h = PIPS_H * scale;
    let (rect, _) = ui.allocate_exact_size(vec2(width, h), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }

    let painter = ui.painter();
    let label_text = format!("Lv {}/{}", line.current_level, line.max_level);
    let label_w = 50.0 * scale; // new: DECISIONS 2026-11-21
    let pips_count = usize::from(line.max_level.max(1));
    let pip_gap = 5.0 * scale; // new: DECISIONS 2026-11-21
    let pips_area_w = (width - label_w - 6.0 * scale).max(24.0 * scale);
    let pip_w = ((pips_area_w - pip_gap * (pips_count.saturating_sub(1) as f32))
        / (pips_count as f32))
        .max(6.0 * scale);
    let pip_h = 8.0 * scale; // new: DECISIONS 2026-11-21
    let pip_y = rect.center().y - pip_h * 0.5;
    let r = theme::radius(2.0, scale); // new: DECISIONS 2026-11-21
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

fn draw_extra_slot(
    ui: &mut egui::Ui,
    line: UpgradeLine,
    upgrades: &CampUpgradesView,
    is_building: bool,
    width: f32,
    scale: f32,
) {
    let slot_h = EXTRA_BAR_H * scale;
    ui.allocate_ui(vec2(width, slot_h), |ui| {
        ui.set_width(width);
        ui.set_min_height(slot_h);
        ui.set_max_height(slot_h);
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        if line == UpgradeLine::Walls {
            let frac = wall_hp_fraction(upgrades.wall_hp, upgrades.wall_hp_max);
            let bar_color = wall_hp_color(frac);
            let label = format_wall_hp_label(upgrades.wall_hp, upgrades.wall_hp_max);
            let text_color = if upgrades.wall_hp_max <= 0.0 {
                theme::TEXT_DISABLED
            } else if frac <= WALL_HP_CRIT {
                theme::DANGER_HOVER
            } else if frac <= WALL_HP_WARN {
                theme::ACCENT_HOVER
            } else {
                theme::SAFE_HOVER
            };

            let row_h = 15.0 * scale; // new: DECISIONS 2026-11-21
            let (row_rect, _) = ui.allocate_exact_size(vec2(width, row_h), Sense::hover());
            if ui.is_rect_visible(row_rect) {
                let painter = ui.painter().with_clip_rect(row_rect.intersect(ui.clip_rect()));
                let font = FontId::new(11.5 * scale, FontFamily::Proportional); // new: DECISIONS 2026-11-21
                painter.text(
                    pos2(row_rect.left(), row_rect.center().y),
                    Align2::LEFT_CENTER,
                    "Wall HP",
                    font.clone(),
                    theme::TEXT_MUTED,
                );
                painter.text(
                    pos2(row_rect.right(), row_rect.center().y),
                    Align2::RIGHT_CENTER,
                    label,
                    font,
                    text_color,
                );
            }
            ui.add_space(4.0 * scale);
            draw_bar(ui, vec2(width, 9.0 * scale), frac, bar_color, scale);
        } else if is_building {
            ui.add_space(5.0 * scale);
            theme::status_badge(ui, "BUILDING", theme::ACCENT, scale);
        }
    });
}

fn draw_next_box(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    upgrades: &CampUpgradesView,
    size: Vec2,
    scale: f32,
) -> Option<CampAction> {
    let pad = scaled_i8(NEXT_PAD, scale);
    let inner_size = vec2(
        (size.x - 2.0 * f32::from(pad)).max(80.0),
        (size.y - 2.0 * f32::from(pad)).max(120.0),
    );
    let is_maxed = line.is_max_level();
    let border = if is_maxed {
        theme::SAFE
    } else {
        theme::BORDER
    };

    let mut action = None;

    Frame::NONE
        .fill(theme::PANEL_BG)
        .inner_margin(Margin::same(pad))
        .corner_radius(theme::radius(theme::CORNER_CONTROL, scale))
        .stroke(Stroke::new(1.0, border))
        .show(ui, |ui| {
            ui.set_width(inner_size.x);
            ui.set_min_height(inner_size.y);
            ui.set_max_height(inner_size.y);
            ui.spacing_mut().item_spacing = vec2(6.0 * scale, 0.0);

            if !is_maxed
                && let Some(next) = &line.next
            {
                action = draw_next_level_contents(ui, line, next, upgrades, inner_size, scale);
            } else {
                draw_complete_box(ui, inner_size, scale);
            }
        });

    action
}

fn draw_next_level_contents(
    ui: &mut egui::Ui,
    line: &UpgradeLineView,
    next: &NextUpgradeView,
    upgrades: &CampUpgradesView,
    size: Vec2,
    scale: f32,
) -> Option<CampAction> {
    let mut action = None;
    let is_building_this = upgrades
        .in_progress
        .as_ref()
        .is_some_and(|p| p.line == line.line);

    // Header tag: "NEXT — LV.X"
    let tag_h = 16.0 * scale; // new: DECISIONS 2026-11-21
    ui.allocate_ui(vec2(size.x, tag_h), |ui| {
        ui.set_width(size.x);
        ui.set_min_height(tag_h);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("NEXT — LV.{}", line.current_level.saturating_add(1)))
                    .size(11.0 * scale) // new: DECISIONS 2026-11-21
                    .color(theme::ACCENT)
                    .strong(),
            );
            if is_building_this && line.line == UpgradeLine::Walls {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    theme::status_badge(ui, "BUILDING", theme::ACCENT, scale);
                });
            }
        });
    });

    ui.add_space(2.0 * scale);

    // Next level name
    let name_h = 22.0 * scale; // new: DECISIONS 2026-11-21
    ui.allocate_ui(vec2(size.x, name_h), |ui| {
        ui.set_width(size.x);
        ui.set_min_height(name_h);
        ui.add(
            egui::Label::new(
                RichText::new(&next.name)
                    .size(16.0 * scale) // new: DECISIONS 2026-11-21
                    .color(theme::TEXT_PRIMARY)
                    .strong(),
            )
            .wrap_mode(egui::TextWrapMode::Truncate),
        );
    });

    ui.add_space(4.0 * scale);

    // 1-2 line description
    let desc_h = DESC_H * scale;
    ui.allocate_ui(vec2(size.x, desc_h), |ui| {
        ui.set_width(size.x);
        ui.set_min_height(desc_h);
        ui.set_max_height(desc_h);
        ui.add(
            egui::Label::new(
                RichText::new(&next.description)
                    .size(theme::FONT_HELP * scale)
                    .color(theme::TEXT_MUTED),
            )
            .wrap_mode(egui::TextWrapMode::Wrap),
        );
    });

    ui.add_space(6.0 * scale);
    theme::paint_divider(ui, size.x, scale);
    ui.add_space(6.0 * scale);

    // Cost rows (cash + items)
    let costs_h = COSTS_H * scale;
    ui.allocate_ui(vec2(size.x, costs_h), |ui| {
        ui.set_width(size.x);
        ui.set_min_height(costs_h);
        ui.set_max_height(costs_h);
        ScrollArea::vertical()
            .id_salt(format!("upgrade_costs_{}", line.line.as_str()))
            .max_height(costs_h)
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 2.0 * scale);
                if next.cash_cost > 0 {
                    draw_cost_row(
                        ui,
                        "Cash",
                        &format_cash_cost(upgrades.player_cash, next.cash_cost),
                        cash_cost_state(upgrades.player_cash, next.cash_cost),
                        size.x,
                        scale,
                    );
                }
                for item in &next.items {
                    draw_cost_row(
                        ui,
                        &item.name,
                        &format_item_cost(item.have, item.need),
                        item_cost_state(item.have, item.need),
                        size.x,
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
    });

    ui.add_space(6.0 * scale);

    // Build time row
    let time_h = 18.0 * scale; // new: DECISIONS 2026-11-21
    let (time_rect, _) = ui.allocate_exact_size(vec2(size.x, time_h), Sense::hover());
    if ui.is_rect_visible(time_rect) {
        let painter = ui.painter().with_clip_rect(time_rect.intersect(ui.clip_rect()));
        let font = FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional);
        painter.text(
            pos2(time_rect.left(), time_rect.center().y),
            Align2::LEFT_CENTER,
            "Build time",
            font.clone(),
            theme::TEXT_MUTED,
        );
        painter.text(
            pos2(time_rect.right(), time_rect.center().y),
            Align2::RIGHT_CENTER,
            format_hours(next.build_hours),
            font,
            theme::TEXT_PRIMARY,
        );
    }

    ui.add_space(8.0 * scale);

    // Big Build button + disabled reason tooltip / second line
    let blocker = effective_cant_start_reason(next, upgrades.in_progress.as_ref(), line.line);
    let can_start = blocker.is_none();
    let mut build_btn = theme::menu_button(
        ui,
        "Build",
        vec2(size.x, BUILD_BTN_H * scale),
        scale,
        ButtonTone::Primary,
        can_start,
    );
    if let Some(reason) = blocker {
        build_btn = build_btn.on_hover_text(reason);
    }
    if can_start && build_btn.clicked() {
        action = Some(CampAction::StartUpgrade(line.line));
    }

    ui.add_space(4.0 * scale);

    let reason_h = REASON_H * scale;
    ui.allocate_ui(vec2(size.x, reason_h), |ui| {
        ui.set_width(size.x);
        ui.set_min_height(reason_h);
        ui.set_max_height(reason_h);
        if let Some(reason) = blocker {
            let reason_color = if is_building_this {
                theme::ACCENT_HOVER
            } else {
                theme::DANGER_HOVER
            };
            ui.add(
                egui::Label::new(
                    RichText::new(reason)
                        .size(11.5 * scale) // new: DECISIONS 2026-11-21
                        .color(reason_color),
                )
                .wrap_mode(egui::TextWrapMode::Wrap),
            );
        }
    });

    action
}

fn draw_complete_box(ui: &mut egui::Ui, size: Vec2, scale: f32) {
    ui.add_space((size.y * 0.34).max(20.0 * scale));
    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
        theme::status_badge(ui, "COMPLETE", theme::SAFE, scale);
        ui.add_space(10.0 * scale);
        ui.label(
            RichText::new("Maximum level reached")
                .size(theme::FONT_BODY * scale)
                .color(theme::TEXT_PRIMARY)
                .strong(),
        );
        ui.add_space(4.0 * scale);
        ui.add(
            egui::Label::new(
                RichText::new("All upgrades in this line are built.")
                    .size(theme::FONT_HELP * scale)
                    .color(theme::TEXT_MUTED),
            )
            .wrap_mode(egui::TextWrapMode::Wrap),
        );
    });
}

fn draw_cost_row(
    ui: &mut egui::Ui,
    label: &str,
    amount: &str,
    state: CostRowState,
    width: f32,
    scale: f32,
) {
    let row_h = 18.0 * scale; // new: DECISIONS 2026-11-21
    let (rect, _) = ui.allocate_exact_size(vec2(width, row_h), Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let color = cost_row_color(state);
    let dot_r = (3.0 * scale).max(2.0); // new: DECISIONS 2026-11-21
    painter.circle_filled(
        pos2(rect.left() + dot_r + 1.0, rect.center().y),
        dot_r,
        color,
    );

    let font = FontId::new(theme::FONT_HELP * scale, FontFamily::Proportional);
    painter.text(
        pos2(rect.left() + dot_r * 2.0 + 7.0 * scale, rect.center().y),
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
        color,
    );
}

fn draw_bar(
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
        theme::radius(3.0, scale), // new: DECISIONS 2026-11-21
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
        painter.rect_filled(fill_rect, theme::radius(2.0, scale), fill_color); // new: DECISIONS 2026-11-21
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camp_ui::view::UpgradeCostItem;

    #[test]
    fn format_hours_handles_whole_fractional_and_edge_cases() {
        assert_eq!(format_hours(0.0), "0 min");
        assert_eq!(format_hours(-2.5), "0 min");
        assert_eq!(format_hours(f32::NAN), "0 min");
        assert_eq!(format_hours(0.004), "< 1 min");
        assert_eq!(format_hours(0.5), "30 min");
        assert_eq!(format_hours(1.0), "1 h");
        assert_eq!(format_hours(1.5), "1 h 30 min");
        assert_eq!(format_hours(3.0 + 20.0 / 60.0), "3 h 20 min");
        // Rounds up cleanly without producing "0 h 60 min"
        assert_eq!(format_hours(59.9 / 60.0), "1 h");
    }

    #[test]
    fn format_building_banner_matches_expected_style() {
        let text = format_building_banner("Metal Sheet Walls", 3.0 + 20.0 / 60.0);
        assert_eq!(text, "Building: Metal Sheet Walls — 3 h 20 min left");
    }

    #[test]
    fn cost_row_states_and_colors_distinguish_enough_and_short() {
        assert_eq!(cash_cost_state(150, 150), CostRowState::Enough);
        assert_eq!(cash_cost_state(200, 150), CostRowState::Enough);
        assert_eq!(cash_cost_state(149, 150), CostRowState::Short);

        assert_eq!(item_cost_state(20, 20), CostRowState::Enough);
        assert_eq!(item_cost_state(25, 20), CostRowState::Enough);
        assert_eq!(item_cost_state(19, 20), CostRowState::Short);

        assert_eq!(cost_row_color(CostRowState::Enough), theme::SAFE_HOVER);
        assert_eq!(cost_row_color(CostRowState::Short), theme::DANGER_HOVER);
        assert_eq!(format_cash_cost(120, 150), "$120/$150");
        assert_eq!(format_item_cost(16, 20), "16/20");
    }

    #[test]
    fn upgrade_progress_fraction_clamps_and_handles_zero_total() {
        assert!((upgrade_progress_fraction(4.0, 4.0) - 0.0).abs() < 1e-5);
        assert!((upgrade_progress_fraction(2.0, 4.0) - 0.5).abs() < 1e-5);
        assert!((upgrade_progress_fraction(0.0, 4.0) - 1.0).abs() < 1e-5);
        assert!((upgrade_progress_fraction(6.0, 4.0) - 0.0).abs() < 1e-5);
        assert!((upgrade_progress_fraction(-1.0, 4.0) - 1.0).abs() < 1e-5);
        assert!((upgrade_progress_fraction(0.0, 0.0) - 1.0).abs() < 1e-5);
        assert!((upgrade_progress_fraction(f32::NAN, 4.0) - 0.0).abs() < 1e-5);
    }

    #[test]
    fn wall_hp_helpers_cover_none_healthy_damaged_and_breached() {
        assert!((wall_hp_fraction(0.0, 0.0) - 0.0).abs() < 1e-5);
        assert_eq!(format_wall_hp_label(0.0, 0.0), "No walls");

        let healthy = wall_hp_fraction(400.0, 500.0);
        assert!((healthy - 0.8).abs() < 1e-5);
        assert_eq!(wall_hp_color(healthy), theme::SAFE);
        assert_eq!(format_wall_hp_label(400.0, 500.0), "400 / 500");

        let damaged = wall_hp_fraction(200.0, 500.0);
        assert_eq!(wall_hp_color(damaged), theme::ACCENT);

        let critical = wall_hp_fraction(50.0, 500.0);
        assert_eq!(wall_hp_color(critical), theme::DANGER);

        assert_eq!(format_wall_hp_label(0.0, 500.0), "0 / 500 (Breached)");
    }

    #[test]
    fn refund_percent_and_cancel_label_use_ratio() {
        assert_eq!(refund_percent(0.5), 50);
        assert_eq!(refund_percent(1.5), 100);
        assert_eq!(refund_percent(-0.2), 0);
        assert_eq!(refund_percent(f32::NAN), 0);
        assert_eq!(cancel_button_label(0.5), "Cancel (refund 50%)");
    }

    #[test]
    fn resolve_in_progress_name_prefers_explicit_then_next_then_fallback() {
        let mut upgrades = CampUpgradesView::default();
        upgrades.lines[0].next = Some(NextUpgradeView {
            name: "Wood Palisade".to_string(),
            items: vec![UpgradeCostItem {
                name: "Wood".to_string(),
                have: 20,
                need: 20,
            }],
            ..Default::default()
        });

        let explicit = UpgradeInProgressView {
            line: UpgradeLine::Walls,
            target_level: 2,
            target_name: "Metal Sheet Walls".to_string(),
            hours_left: 3.0,
            total_hours: 4.0,
        };
        assert_eq!(resolve_in_progress_name(&explicit, &upgrades), "Metal Sheet Walls");

        let from_next = UpgradeInProgressView {
            line: UpgradeLine::Walls,
            target_level: 1,
            target_name: String::new(),
            hours_left: 1.5,
            total_hours: 2.0,
        };
        assert_eq!(resolve_in_progress_name(&from_next, &upgrades), "Wood Palisade");

        let fallback = UpgradeInProgressView {
            line: UpgradeLine::Guards,
            target_level: 2,
            target_name: String::new(),
            hours_left: 2.0,
            total_hours: 4.0,
        };
        assert_eq!(resolve_in_progress_name(&fallback, &upgrades), "Guards & Towers Lv.2");
    }

    #[test]
    fn effective_cant_start_reason_respects_explicit_and_busy_states() {
        let ready = NextUpgradeView::default();
        assert!(ready.can_start());
        assert_eq!(effective_cant_start_reason(&ready, None, UpgradeLine::Walls), None);

        let busy_walls = UpgradeInProgressView {
            line: UpgradeLine::Walls,
            target_level: 1,
            target_name: "Wood Palisade".to_string(),
            hours_left: 1.0,
            total_hours: 2.0,
        };
        assert_eq!(
            effective_cant_start_reason(&ready, Some(&busy_walls), UpgradeLine::Walls),
            Some("Currently building")
        );
        assert_eq!(
            effective_cant_start_reason(&ready, Some(&busy_walls), UpgradeLine::Workshop),
            Some("Another upgrade is in progress")
        );

        let blocked = NextUpgradeView {
            cant_start_reason: Some("Requires Workshop Lv.1".to_string()),
            ..Default::default()
        };
        assert!(!blocked.can_start());
        assert_eq!(
            effective_cant_start_reason(&blocked, None, UpgradeLine::Traders),
            Some("Requires Workshop Lv.1")
        );
    }

    #[test]
    fn camp_screen_upgrades_metadata_and_view_defaults() {
        assert_eq!(CampScreen::Upgrades.title(), "Camp Board");
        assert_eq!(CampScreen::Upgrades.tab_name(), "Upgrades");
        assert_eq!(CampScreen::Upgrades.as_str(), "upgrades");

        let upgrades = CampUpgradesView {
            player_cash: 325,
            ..Default::default()
        };
        assert_eq!(upgrades.line(UpgradeLine::Walls).map(|l| l.max_level), Some(3));
        assert_eq!(upgrades.line(UpgradeLine::Traders).map(|l| l.max_level), Some(4));
        assert!(upgrades.line(UpgradeLine::Walls).is_some_and(UpgradeLineView::is_max_level));

        let view = CampView {
            upgrades: Some(upgrades),
            ..Default::default()
        };
        assert_eq!(view.player_dollars(), 325);
    }

    #[test]
    fn panel_dimensions_fit_720p_to_1440p() {
        for (w, h) in [(1280.0, 720.0), (1920.0, 1080.0), (2560.0, 1440.0)] {
            let rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(w, h));
            let scale = theme::ui_scale(h);
            let panel = panel_dimensions(rect, scale);
            assert!(panel.x < w, "panel width {} must fit screen width {}", panel.x, w);
            assert!(panel.y < h, "panel height {} must fit screen height {}", panel.y, h);
        }
    }
}
