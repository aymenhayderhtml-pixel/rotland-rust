//! egui 0.37 rendering for the Banker screen.

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};

use super::action::CampAction;
use super::state::CampUiState;
use super::view::CampView;

/// Draws the Bank screen: Bag (store) on the left, Bank (take out) on the right, plus cash actions.
pub fn draw_bank(ui: &mut egui::Ui, state: &mut CampUiState, view: &CampView) -> Option<CampAction> {
    let mut action = None;

    ui.label(
        RichText::new(format!("Bank balance: ${}", view.bank_dollars()))
            .size(16.0)
            .color(Color32::from_rgb(255, 230, 102))
            .strong(),
    );
    ui.add_space(4.0);

    ui.columns(2, |columns| {
        // --- Left column: Your bag (store) ---
        let left = &mut columns[0];
        left.label(
            RichText::new("Your bag (store)")
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        left.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("bank_bag_scroll")
            .max_height(340.0)
            .show(left, |ui| {
                if let Some(bag) = view.inventory {
                    let mut empty = true;
                    for (slot_idx, stack_opt) in bag.slots().iter().enumerate() {
                        let Some(stack) = stack_opt else { continue };
                        empty = false;
                        let item_id = &stack.item_id;
                        let def = view.item_def(item_id);
                        let name = def.map(|d| d.name.as_str()).unwrap_or(item_id);
                        let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                        let icon_color = super::draw::parse_hex_color(color_hex);

                        let bank_full = view.bank.is_some_and(|b| !b.items().can_add(item_id, 1));
                        let row_key = format!("bank_store_{slot_idx}_{item_id}");
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

                                    let info_text = if bank_full {
                                        format!("{name}   x{}   (Bank full)", stack.count)
                                    } else {
                                        format!("{name}   x{}", stack.count)
                                    };

                                    let label = RichText::new(info_text).size(14.0).color(if bank_full {
                                        Color32::from_rgb(140, 140, 145)
                                    } else {
                                        Color32::WHITE
                                    });

                                    if ui.button(label).clicked() {
                                        state.select_row(&row_key);
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_enabled(!bank_full, egui::Button::new("Deposit all")).clicked() {
                                            action = Some(CampAction::Deposit {
                                                item: item_id.clone(),
                                                count: stack.count,
                                            });
                                        }
                                        if ui.add_enabled(!bank_full, egui::Button::new("Deposit 1")).clicked() {
                                            action = Some(CampAction::Deposit {
                                                item: item_id.clone(),
                                                count: 1,
                                            });
                                        }
                                    });
                                });

                                if is_selected && !bank_full {
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
                                        if ui.button(format!("Store {}", state.count())).clicked() {
                                            action = Some(CampAction::Deposit {
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

        // --- Right column: In the bank (take out) ---
        let right = &mut columns[1];
        right.label(
            RichText::new("In the bank (take out)")
                .size(16.0)
                .color(Color32::from_rgb(204, 217, 230))
                .strong(),
        );
        right.add_space(6.0);

        ScrollArea::vertical()
            .id_salt("bank_vault_scroll")
            .max_height(340.0)
            .show(right, |ui| {
                if let Some(bank) = view.bank {
                    let mut empty = true;
                    for (slot_idx, stack_opt) in bank.items().slots().iter().enumerate() {
                        let Some(stack) = stack_opt else { continue };
                        empty = false;
                        let item_id = &stack.item_id;
                        let def = view.item_def(item_id);
                        let name = def.map(|d| d.name.as_str()).unwrap_or(item_id);
                        let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                        let icon_color = super::draw::parse_hex_color(color_hex);

                        let bag_full = view.inventory.is_some_and(|bag| !bag.can_add(item_id, 1));
                        let row_key = format!("bank_take_{slot_idx}_{item_id}");
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

                                    let info_text = if bag_full {
                                        format!("{name}   x{}   (Bag full)", stack.count)
                                    } else {
                                        format!("{name}   x{}", stack.count)
                                    };

                                    let label = RichText::new(info_text).size(14.0).color(if bag_full {
                                        Color32::from_rgb(140, 140, 145)
                                    } else {
                                        Color32::WHITE
                                    });

                                    if ui.button(label).clicked() {
                                        state.select_row(&row_key);
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_enabled(!bag_full, egui::Button::new("Withdraw all")).clicked() {
                                            action = Some(CampAction::Withdraw {
                                                item: item_id.clone(),
                                                count: stack.count,
                                            });
                                        }
                                        if ui.add_enabled(!bag_full, egui::Button::new("Withdraw 1")).clicked() {
                                            action = Some(CampAction::Withdraw {
                                                item: item_id.clone(),
                                                count: 1,
                                            });
                                        }
                                    });
                                });

                                if is_selected && !bag_full {
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
                                        if ui.button(format!("Take {}", state.count())).clicked() {
                                            action = Some(CampAction::Withdraw {
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
                        ui.label(RichText::new("Bank vault is empty.").color(Color32::GRAY));
                    }
                } else {
                    ui.label("Bank vault unavailable.");
                }
            });
    });

    ui.add_space(10.0);

    // --- Bank dollars actions row ---
    ui.horizontal(|ui| {
        let player_has_cash = view.player_dollars() > 0;
        let bank_has_cash = view.bank_dollars() > 0;

        if ui.add_enabled(view.player_dollars() >= 10, egui::Button::new("Deposit $10")).clicked() {
            action = Some(CampAction::DepositDollars { amount: 10 });
        }
        if ui.add_enabled(view.player_dollars() >= 100, egui::Button::new("Deposit $100")).clicked() {
            action = Some(CampAction::DepositDollars { amount: 100 });
        }
        if ui.add_enabled(player_has_cash, egui::Button::new("Deposit all")).clicked() {
            action = Some(CampAction::DepositDollars { amount: i32::MAX });
        }

        ui.separator();

        if ui.add_enabled(view.bank_dollars() >= 10, egui::Button::new("Withdraw $10")).clicked() {
            action = Some(CampAction::WithdrawDollars { amount: 10 });
        }
        if ui.add_enabled(view.bank_dollars() >= 100, egui::Button::new("Withdraw $100")).clicked() {
            action = Some(CampAction::WithdrawDollars { amount: 100 });
        }
        if ui.add_enabled(bank_has_cash, egui::Button::new("Withdraw all")).clicked() {
            action = Some(CampAction::WithdrawDollars { amount: i32::MAX });
        }
    });

    action
}
