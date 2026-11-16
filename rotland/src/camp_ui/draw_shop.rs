//! egui 0.37 rendering for the Trader Shop screen.

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use game::{buy_blocker, buy_price, sell_price};

use super::action::CampAction;
use super::state::CampUiState;
use super::view::CampView;

/// Draws the Trader Shop screen: Buy on the left, Sell on the right.
pub fn draw_shop(ui: &mut egui::Ui, state: &mut CampUiState, view: &CampView) -> Option<CampAction> {
    let mut action = None;

    ui.columns(2, |columns| {
        // --- Left column: For sale (buy) ---
        let left = &mut columns[0];
        left.label(
            RichText::new("For sale (buy)")
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        left.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("shop_buy_scroll")
            .max_height(380.0)
            .show(left, |ui| {
                if let (Some(stock), Some(registry)) = (view.trader_stock, view.item_registry) {
                    for id in stock.order() {
                        let qty = stock.qty(id);
                        let def = registry.get(id);
                        let name = def.map(|d| d.name.as_str()).unwrap_or(id);
                        let price = def.map(|d| buy_price(d, view.buy_rate())).unwrap_or(1);
                        let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                        let icon_color = super::draw::parse_hex_color(color_hex);

                        let blocker = if let (Some(wallet), Some(bag)) = (view.wallet, view.inventory) {
                            buy_blocker(wallet, bag, stock, id, view.buy_rate(), registry)
                        } else if qty == 0 {
                            Some(game::BuyBlocker::SoldOut)
                        } else if view.player_dollars() < price {
                            Some(game::BuyBlocker::NotEnoughDollars)
                        } else {
                            None
                        };

                        let is_selected = state.selected_row() == Some(id);

                        Frame::NONE
                            .fill(if is_selected {
                                Color32::from_rgb(38, 44, 48)
                            } else {
                                Color32::from_rgb(28, 30, 34)
                            })
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(
                                1.0,
                                if is_selected {
                                    Color32::from_rgb(143, 194, 74)
                                } else {
                                    Color32::from_rgb(50, 54, 60)
                                },
                            ))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, CornerRadius::same(4), icon_color);

                                    let info_text = if let Some(b) = blocker {
                                        format!("{name}   ${price}   x{qty}   ({b})")
                                    } else {
                                        format!("{name}   ${price}   x{qty}")
                                    };

                                    let label = RichText::new(info_text).size(14.0).color(if blocker.is_some() {
                                        Color32::from_rgb(140, 140, 145)
                                    } else {
                                        Color32::WHITE
                                    });

                                    if ui.button(label).clicked() {
                                        state.select_row(id);
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let can_buy = blocker.is_none() && qty > 0;
                                        if ui.add_enabled(can_buy && qty >= 5, egui::Button::new("Buy 5")).clicked() {
                                            action = Some(CampAction::Buy {
                                                item: id.clone(),
                                                count: 5.min(qty),
                                            });
                                        }
                                        if ui.add_enabled(can_buy, egui::Button::new("Buy 1")).clicked() {
                                            action = Some(CampAction::Buy {
                                                item: id.clone(),
                                                count: 1,
                                            });
                                        }
                                    });
                                });

                                if is_selected && blocker.is_none() {
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Amount:").size(13.0).color(Color32::from_rgb(200, 200, 200)));
                                        if ui.button("-").clicked() {
                                            state.step_down();
                                        }
                                        ui.label(RichText::new(format!("{}", state.count())).size(14.0).strong());
                                        if ui.button("+").clicked() {
                                            state.step_up(qty);
                                        }
                                        if ui.button(format!("Buy {}", state.count())).clicked() {
                                            action = Some(CampAction::Buy {
                                                item: id.clone(),
                                                count: state.count(),
                                            });
                                        }
                                    });
                                }
                            });
                        ui.add_space(4.0);
                    }
                } else {
                    ui.label("No trader stock available.");
                }
            });

        // --- Right column: Your bag (sell) ---
        let right = &mut columns[1];
        right.label(
            RichText::new("Your bag (sell)")
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        right.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("shop_sell_scroll")
            .max_height(380.0)
            .show(right, |ui| {
                if let Some(bag) = view.inventory {
                    let mut empty = true;
                    for (slot_idx, stack_opt) in bag.slots().iter().enumerate() {
                        let Some(stack) = stack_opt else { continue };
                        empty = false;
                        let item_id = &stack.item_id;
                        let def = view.item_def(item_id);
                        let name = def.map(|d| d.name.as_str()).unwrap_or(item_id);
                        let price = def.map(|d| sell_price(d, view.sell_rate())).unwrap_or(1);
                        let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                        let icon_color = super::draw::parse_hex_color(color_hex);

                        let row_key = format!("sell_{slot_idx}_{item_id}");
                        let is_selected = state.selected_row() == Some(&row_key);

                        Frame::NONE
                            .fill(if is_selected {
                                Color32::from_rgb(38, 44, 48)
                            } else {
                                Color32::from_rgb(28, 30, 34)
                            })
                            .inner_margin(Margin::same(8))
                            .corner_radius(CornerRadius::same(6))
                            .stroke(Stroke::new(
                                1.0,
                                if is_selected {
                                    Color32::from_rgb(143, 194, 74)
                                } else {
                                    Color32::from_rgb(50, 54, 60)
                                },
                            ))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let (rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, CornerRadius::same(4), icon_color);

                                    let info_text = format!("{name}   ${price}   x{}", stack.count);
                                    if ui.button(RichText::new(info_text).size(14.0)).clicked() {
                                        state.select_row(&row_key);
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_enabled(stack.count >= 5, egui::Button::new("Sell 5")).clicked() {
                                            action = Some(CampAction::Sell {
                                                item: item_id.clone(),
                                                count: 5.min(stack.count),
                                            });
                                        }
                                        if ui.button("Sell 1").clicked() {
                                            action = Some(CampAction::Sell {
                                                item: item_id.clone(),
                                                count: 1,
                                            });
                                        }
                                    });
                                });

                                if is_selected {
                                    ui.add_space(4.0);
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Amount:").size(13.0).color(Color32::from_rgb(200, 200, 200)));
                                        if ui.button("-").clicked() {
                                            state.step_down();
                                        }
                                        ui.label(RichText::new(format!("{}", state.count())).size(14.0).strong());
                                        if ui.button("+").clicked() {
                                            state.step_up(stack.count);
                                        }
                                        if ui.button(format!("Sell {}", state.count())).clicked() {
                                            action = Some(CampAction::Sell {
                                                item: item_id.clone(),
                                                count: state.count().min(stack.count),
                                            });
                                        }
                                    });
                                }
                            });
                        ui.add_space(4.0);
                    }
                    if empty {
                        ui.label(RichText::new("Backpack is empty.").color(Color32::GRAY));
                    }
                } else {
                    ui.label("Backpack unavailable.");
                }
            });
    });

    action
}
