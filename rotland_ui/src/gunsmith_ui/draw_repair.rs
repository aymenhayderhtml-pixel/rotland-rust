//! Repair tab for the Gunsmith screen: condition bars, repair costs, and Repair all.

use engine::egui::{
    self, Color32, CornerRadius, Frame, Margin, Rect, RichText, ScrollArea, Sense, Stroke, vec2,
};

use crate::hud::IconCache;
use crate::menus::theme::{self, ButtonTone};

use super::action::GunsmithAction;
use super::screen::GunsmithLayout;
use super::{GunsmithRepairItem, GunsmithView, draw_item_icon, draw_scrap_badge};

/// Draws the Repair tab: "Repair all" bar and per-weapon condition & cost rows.
pub fn draw_repair(
    ui: &mut egui::Ui,
    view: &GunsmithView,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;
    let scale = layout.scale;

    let worn_count = view
        .repair_items
        .iter()
        .filter(|it| it.condition < 0.999)
        .count();
    let total_cost: u32 = view
        .repair_items
        .iter()
        .filter(|it| it.condition < 0.999)
        .map(|it| it.repair_cost_scrap)
        .sum();
    let can_repair_all = worn_count > 0 && view.scrap_count >= total_cost;

    Frame::NONE
        .fill(theme::CARD_EMPTY_BG)
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, |ui| {
            ui.set_width((layout.inner_width - 16.0).max(200.0));
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if worn_count == 0 {
                        "All weapons are in full condition.".to_string()
                    } else {
                        format!("{worn_count} worn weapon(s)")
                    })
                    .size(14.0 * scale)
                    .color(theme::TEXT_PRIMARY),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let btn_w = (146.0 * scale).clamp(124.0, 200.0);
                    let btn_h = (32.0 * scale).clamp(28.0, 40.0);
                    let mut repair_all_btn = theme::menu_button(
                        ui,
                        "Repair all",
                        vec2(btn_w, btn_h),
                        scale,
                        if can_repair_all {
                            ButtonTone::Primary
                        } else {
                            ButtonTone::Default
                        },
                        can_repair_all,
                    );
                    if worn_count > 0 && view.scrap_count < total_cost {
                        repair_all_btn = repair_all_btn
                            .on_hover_text(format!("Need {total_cost} scrap metal"));
                    }
                    if can_repair_all && repair_all_btn.clicked() {
                        action = Some(GunsmithAction::RepairAll);
                    }

                    if worn_count > 0 {
                        draw_scrap_badge(
                            ui,
                            icons.as_deref_mut(),
                            total_cost,
                            14.0 * scale,
                            if view.scrap_count >= total_cost {
                                Color32::from_rgb(235, 200, 108)
                            } else {
                                theme::DANGER_HOVER
                            },
                        );
                        ui.label(
                            RichText::new("Total:")
                                .size(13.0 * scale)
                                .color(theme::TEXT_MUTED),
                        );
                    }
                });
            });
        });

    ui.add_space(6.0);

    if view.repair_items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(36.0);
            ui.label(
                RichText::new("You have no weapons in your bag.")
                    .size(15.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        return action;
    }

    let scroll_h = (layout.body_height - 62.0).max(120.0);
    ScrollArea::vertical()
        .id_salt("gunsmith_repair_scroll")
        .max_height(scroll_h)
        .show(ui, |ui| {
            ui.set_width((layout.inner_width - 16.0).max(200.0));
            for item in &view.repair_items {
                if let Some(act) =
                    draw_repair_row(ui, item, view.scrap_count, scale, icons.as_deref_mut())
                {
                    action = Some(act);
                }
                ui.add_space(4.0);
            }
        });

    action
}

fn draw_repair_row(
    ui: &mut egui::Ui,
    item: &GunsmithRepairItem,
    player_scrap: u32,
    scale: f32,
    icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;
    let cond = item.condition.clamp(0.0, 1.0);
    let needs_repair = cond < 0.999 && item.repair_cost_scrap > 0;
    let can_afford = player_scrap >= item.repair_cost_scrap;
    let can_repair = needs_repair && can_afford;

    let cond_color = if cond >= 0.6 {
        theme::SAFE
    } else if cond >= 0.25 {
        theme::ACCENT
    } else {
        theme::DANGER
    };

    Frame::NONE
        .fill(theme::CARD_BG)
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(
            1.0,
            if item.is_broken {
                theme::DANGER
            } else {
                theme::BORDER
            },
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_item_icon(
                    ui,
                    icons,
                    &item.item_id,
                    &item.name,
                    &item.color_hex,
                    vec2(34.0, 34.0),
                );
                ui.add_space(4.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 4.0;
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&item.name)
                                .size(14.5)
                                .color(theme::TEXT_PRIMARY)
                                .strong(),
                        );
                        if item.is_broken {
                            theme::status_badge(ui, "BROKEN", theme::DANGER_HOVER, scale);
                        }
                    });
                    ui.horizontal(|ui| {
                        let (bar_rect, _) =
                            ui.allocate_exact_size(vec2(180.0, 10.0), Sense::hover());
                        ui.painter()
                            .rect_filled(bar_rect, CornerRadius::same(3), Color32::from_rgb(16, 18, 22));
                        let fill_w = (bar_rect.width() * cond)
                            .max(if cond > 0.0 { 2.0 } else { 0.0 })
                            .min(bar_rect.width());
                        let fill_rect =
                            Rect::from_min_size(bar_rect.min, vec2(fill_w, bar_rect.height()));
                        ui.painter()
                            .rect_filled(fill_rect, CornerRadius::same(3), cond_color);

                        let pct = (cond * 100.0).round() as u32;
                        ui.label(
                            RichText::new(format!("{pct}%"))
                                .size(12.5)
                                .color(cond_color)
                                .strong(),
                        );
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !needs_repair {
                        ui.add_enabled(false, egui::Button::new("Full condition"));
                    } else {
                        let mut btn = ui.add_enabled(can_repair, egui::Button::new("Repair"));
                        if !can_afford {
                            btn = btn.on_hover_text(format!(
                                "Need {} scrap metal",
                                item.repair_cost_scrap
                            ));
                        }
                        if btn.clicked() {
                            action = Some(GunsmithAction::Repair { slot: item.slot });
                        }
                        draw_scrap_badge(
                            ui,
                            None,
                            item.repair_cost_scrap,
                            13.5,
                            if can_afford {
                                Color32::from_rgb(235, 200, 108)
                            } else {
                                theme::DANGER_HOVER
                            },
                        );
                    }
                });
            });
        });

    action
}
