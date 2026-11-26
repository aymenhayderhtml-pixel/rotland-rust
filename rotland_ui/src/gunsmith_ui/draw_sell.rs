//! Sell tab for the Gunsmith screen: bag's sellable items and scrap payouts.

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, vec2};

use crate::hud::IconCache;
use crate::menus::theme;

use super::action::GunsmithAction;
use super::screen::{GunsmithLayout, GunsmithUiState};
use super::{GunsmithSellItem, GunsmithView, draw_item_icon, draw_scrap_badge};

/// Draws the Sell tab listing sellable bag items with the scrap payout for each.
pub fn draw_sell(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(
            RichText::new("Sell items from your bag for scrap metal")
                .size(14.0 * layout.scale)
                .color(theme::TEXT_MUTED),
        );
    });
    ui.add_space(6.0);

    if view.sell_items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                RichText::new("No sellable items in your bag.")
                    .size(15.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        return None;
    }

    let scroll_h = (layout.body_height - 30.0).max(120.0);
    ScrollArea::vertical()
        .id_salt("gunsmith_sell_scroll")
        .max_height(scroll_h)
        .show(ui, |ui| {
            ui.set_width((layout.inner_width - 16.0).max(200.0));
            for item in &view.sell_items {
                if let Some(act) = draw_sell_row(ui, state, item, icons.as_deref_mut()) {
                    action = Some(act);
                }
                ui.add_space(4.0);
            }
        });

    action
}

fn draw_sell_row(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    item: &GunsmithSellItem,
    icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;
    let is_selected = state.selected_sell_slot == Some(item.slot);

    let frame_resp = Frame::NONE
        .fill(if is_selected {
            Color32::from_rgb(36, 42, 46)
        } else {
            theme::CARD_BG
        })
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(
            1.0,
            if is_selected { theme::ACCENT } else { theme::BORDER },
        ))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                draw_item_icon(
                    ui,
                    icons,
                    &item.item_id,
                    &item.name,
                    &item.color_hex,
                    vec2(32.0, 32.0),
                );
                ui.add_space(4.0);

                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&item.name)
                                .size(14.5)
                                .color(theme::TEXT_PRIMARY)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(format!("x{}", item.count))
                                .size(13.0)
                                .color(theme::TEXT_MUTED),
                        );
                        if let Some(cond) = item.condition {
                            let pct = (cond.clamp(0.0, 1.0) * 100.0).round() as u32;
                            ui.label(
                                RichText::new(format!("{pct}% cond"))
                                    .size(12.0)
                                    .color(theme::TEXT_MUTED),
                            );
                        }
                        if item.mod_count > 0 {
                            ui.label(
                                RichText::new(format!("+{} mods", item.mod_count))
                                    .size(12.0)
                                    .color(theme::SAFE_HOVER),
                            );
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Pays:")
                                .size(12.0)
                                .color(theme::TEXT_MUTED),
                        );
                        draw_scrap_badge(
                            ui,
                            None,
                            item.scrap_each,
                            12.5,
                            Color32::from_rgb(235, 200, 108),
                        );
                        if item.count > 1 {
                            ui.label(
                                RichText::new("each")
                                    .size(12.0)
                                    .color(theme::TEXT_DISABLED),
                            );
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if item.count > 1
                        && ui
                            .button(format!(
                                "Sell all (+{})",
                                item.scrap_each.saturating_mul(item.count)
                            ))
                            .clicked()
                    {
                        action = Some(GunsmithAction::Sell {
                            slot: item.slot,
                            item_id: item.item_id.clone(),
                            count: item.count,
                        });
                    }
                    if ui
                        .button(format!("Sell 1 (+{})", item.scrap_each))
                        .clicked()
                    {
                        action = Some(GunsmithAction::Sell {
                            slot: item.slot,
                            item_id: item.item_id.clone(),
                            count: 1,
                        });
                    }
                });
            });

            if is_selected && item.count > 1 {
                state.clamp_sell(item.count);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("Amount:")
                            .size(13.0)
                            .color(theme::TEXT_MUTED),
                    );
                    if ui.button("-").clicked() {
                        state.step_down_sell();
                    }
                    ui.label(RichText::new(format!("{}", state.sell_count)).size(14.0).strong());
                    if ui.button("+").clicked() {
                        state.step_up_sell(item.count);
                    }
                    let total = item.scrap_each.saturating_mul(state.sell_count);
                    if ui
                        .button(format!("Sell {} (+{} scrap)", state.sell_count, total))
                        .clicked()
                    {
                        action = Some(GunsmithAction::Sell {
                            slot: item.slot,
                            item_id: item.item_id.clone(),
                            count: state.sell_count.min(item.count),
                        });
                    }
                });
            }
        });

    if frame_resp.response.interact(egui::Sense::click()).clicked() {
        state.select_sell(item.slot);
    }

    action
}
