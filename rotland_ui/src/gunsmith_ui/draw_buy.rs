//! Buy tab for the Gunsmith screen: icon grid, scrap prices, camp tier locks, and buy stepper.

use engine::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, vec2};

use crate::hud::IconCache;
use crate::menus::theme;

use super::action::GunsmithAction;
use super::screen::{GunsmithLayout, GunsmithUiState};
use super::{GunsmithBuyItem, GunsmithView, draw_item_icon, draw_scrap_badge};

/// Draws the Buy tab grid and bottom selection bar.
pub fn draw_buy(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    if view.buy_items.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(
                RichText::new("No items in stock right now.")
                    .size(15.0)
                    .color(theme::TEXT_MUTED),
            );
        });
        return None;
    }

    let mut action = None;
    let has_selection = state
        .selected_buy
        .as_ref()
        .is_some_and(|id| view.buy_items.iter().any(|it| &it.item_id == id));
    let bar_h = if has_selection { 62.0 } else { 0.0 };
    let scroll_h = (layout.body_height - bar_h - 8.0).max(120.0);

    ScrollArea::vertical()
        .id_salt("gunsmith_buy_scroll")
        .max_height(scroll_h)
        .show(ui, |ui| {
            let cols = layout.buy_columns.max(1);
            for row_items in view.buy_items.chunks(cols) {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = vec2(layout.buy_gap, layout.buy_gap);
                    for item in row_items {
                        ui.allocate_ui_with_layout(
                            vec2(layout.buy_card_width, 120.0),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(layout.buy_card_width);
                                ui.set_max_width(layout.buy_card_width);
                                if let Some(act) = draw_buy_card(
                                    ui,
                                    item,
                                    state,
                                    view.scrap_count,
                                    layout.buy_card_width,
                                    icons.as_deref_mut(),
                                ) {
                                    action = Some(act);
                                }
                            },
                        );
                    }
                });
                ui.add_space(layout.buy_gap);
            }
        });

    if let Some(selected_id) = state.selected_buy.clone()
        && let Some(item) = view.buy_items.iter().find(|it| it.item_id == selected_id)
    {
        ui.add_space(4.0);
        if let Some(act) = draw_selected_bar(ui, state, item, view.scrap_count, layout.inner_width, icons) {
            action = Some(act);
        }
    }

    action
}

fn draw_buy_card(
    ui: &mut egui::Ui,
    item: &GunsmithBuyItem,
    state: &mut GunsmithUiState,
    player_scrap: u32,
    card_width: f32,
    icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;
    let is_locked = item.locked_camp_tier.is_some();
    let sold_out = item.stock == 0;
    let cant_afford = player_scrap < item.price_scrap;
    let blocked = is_locked || sold_out || cant_afford || item.blocker_reason.is_some();
    let is_selected = state.selected_buy.as_deref() == Some(item.item_id.as_str());

    let fill = if is_locked {
        Color32::from_rgb(20, 22, 25)
    } else if is_selected {
        Color32::from_rgb(36, 42, 46)
    } else {
        theme::CARD_BG
    };
    let border = if is_selected {
        theme::ACCENT
    } else if is_locked {
        Color32::from_rgb(45, 48, 54)
    } else {
        theme::BORDER
    };

    let pad: i8 = 8;
    let inner_w = (card_width - f32::from(pad) * 2.0).max(80.0);

    let frame_resp = Frame::NONE
        .fill(fill)
        .inner_margin(Margin::same(pad))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, border))
        .show(ui, |ui| {
            ui.set_width(inner_w);
            ui.set_max_width(inner_w);
            ui.set_min_height(102.0);
            ui.spacing_mut().item_spacing = vec2(6.0, 5.0);

            ui.horizontal(|ui| {
                draw_item_icon(
                    ui,
                    icons,
                    &item.item_id,
                    &item.name,
                    &item.color_hex,
                    vec2(40.0, 40.0),
                );
                ui.allocate_ui_with_layout(
                    vec2((inner_w - 48.0).max(40.0), 42.0),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width((inner_w - 48.0).max(40.0));
                        ui.spacing_mut().item_spacing.y = 2.0;
                        let name_color = if is_locked || blocked {
                            theme::TEXT_MUTED
                        } else {
                            theme::TEXT_PRIMARY
                        };
                        ui.add(
                            egui::Label::new(RichText::new(&item.name).size(14.0).color(name_color).strong())
                                .truncate(),
                        );
                        ui.horizontal(|ui| {
                            if !item.category.is_empty() {
                                ui.label(
                                    RichText::new(&item.category)
                                        .size(11.0)
                                        .color(theme::TEXT_DISABLED),
                                );
                            }
                            if let Some(tier) = item.locked_camp_tier {
                                ui.label(
                                    RichText::new(format!("Camp tier {tier}"))
                                        .size(11.0)
                                        .color(theme::DANGER_HOVER)
                                        .strong(),
                                );
                            } else if sold_out {
                                ui.label(
                                    RichText::new("Sold out")
                                        .size(11.0)
                                        .color(theme::DANGER_HOVER),
                                );
                            } else {
                                ui.label(
                                    RichText::new(format!("x{}", item.stock))
                                        .size(11.0)
                                        .color(theme::TEXT_MUTED),
                                );
                            }
                        });
                    },
                );
            });

            if !item.summary.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(&item.summary)
                            .size(11.5)
                            .color(if is_locked {
                                theme::TEXT_DISABLED
                            } else {
                                theme::TEXT_MUTED
                            }),
                    )
                    .truncate(),
                );
            }

            ui.horizontal(|ui| {
                let price_color = if is_locked {
                    theme::TEXT_DISABLED
                } else if cant_afford {
                    theme::DANGER_HOVER
                } else {
                    Color32::from_rgb(235, 200, 108)
                };
                draw_scrap_badge(ui, None, item.price_scrap, 13.5, price_color);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(tier) = item.locked_camp_tier {
                        ui.add_enabled(false, egui::Button::new(format!("Camp tier {tier}")));
                    } else {
                        let can_buy = !blocked;
                        if item.stock >= 5
                            && ui
                                .add_enabled(
                                    can_buy && player_scrap >= item.price_scrap.saturating_mul(5),
                                    egui::Button::new("x5"),
                                )
                                .clicked()
                        {
                            action = Some(GunsmithAction::Buy {
                                item_id: item.item_id.clone(),
                                count: 5,
                            });
                        }
                        let mut btn = ui.add_enabled(can_buy, egui::Button::new("Buy"));
                        if let Some(reason) = &item.blocker_reason {
                            btn = btn.on_hover_text(reason);
                        } else if cant_afford {
                            btn = btn.on_hover_text("Not enough scrap metal");
                        }
                        if btn.clicked() {
                            action = Some(GunsmithAction::Buy {
                                item_id: item.item_id.clone(),
                                count: 1,
                            });
                        }
                    }
                });
            });
        });

    let interact = frame_resp.response.interact(egui::Sense::click());
    if interact.clicked() && !is_locked {
        state.select_buy(&item.item_id);
    }

    action
}

fn draw_selected_bar(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    item: &GunsmithBuyItem,
    player_scrap: u32,
    inner_width: f32,
    icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    let mut action = None;
    let max_afford = player_scrap.checked_div(item.price_scrap).unwrap_or(item.stock);
    let max_qty = item.stock.min(max_afford);
    state.clamp_buy(max_qty);

    Frame::NONE
        .fill(theme::CARD_EMPTY_BG)
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, |ui| {
            let bar_w = (inner_width - 16.0).max(200.0);
            ui.set_width(bar_w);
            ui.set_max_width(bar_w);
            ui.horizontal(|ui| {
                draw_item_icon(
                    ui,
                    icons,
                    &item.item_id,
                    &item.name,
                    &item.color_hex,
                    vec2(28.0, 28.0),
                );
                ui.label(RichText::new(&item.name).size(14.0).strong());
                if !item.summary.is_empty() {
                    ui.add(
                        egui::Label::new(
                            RichText::new(format!("· {}", item.summary))
                                .size(12.5)
                                .color(theme::TEXT_MUTED),
                        )
                        .truncate(),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let count = state.buy_count;
                    let total_cost = item.price_scrap.saturating_mul(count);
                    let can_buy = count > 0
                        && item.locked_camp_tier.is_none()
                        && item.blocker_reason.is_none()
                        && player_scrap >= total_cost;

                    if ui
                        .add_enabled(can_buy, egui::Button::new(format!("Buy {count}")))
                        .clicked()
                    {
                        action = Some(GunsmithAction::Buy {
                            item_id: item.item_id.clone(),
                            count,
                        });
                    }
                    draw_scrap_badge(ui, None, total_cost, 13.5, Color32::from_rgb(235, 200, 108));
                    if ui.button("+").clicked() {
                        state.step_up_buy(max_qty);
                    }
                    ui.label(RichText::new(format!("{count}")).size(14.0).strong());
                    if ui.button("-").clicked() {
                        state.step_down_buy();
                    }
                });
            });
        });

    action
}
