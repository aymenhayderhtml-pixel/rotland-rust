//! Guild shop section of the Guild screen (tier-gated inventory and purchasing).

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke};
use game::guild::GuildManager;

use crate::camp_ui::action::GuildCampAction;
use crate::camp_ui::draw::parse_hex_color;
use crate::camp_ui::state::CampUiState;
use crate::camp_ui::view::CampView;

/// Draws the Guild Shop column with available stock and locked higher tiers.
pub fn draw_shop_column(
    ui: &mut egui::Ui,
    guild: &GuildManager,
    state: &mut CampUiState,
    view: &CampView,
    max_height: f32,
) -> Option<GuildCampAction> {
    let mut action = None;
    let shop = guild.shop();
    let player_tier = guild.shop_tier();
    let player_dollars = view.player_dollars();

    let all_items = shop.items();
    let available: Vec<_> = all_items.iter().filter(|i| i.tier <= player_tier).collect();
    let locked: Vec<_> = all_items.iter().filter(|i| i.tier > player_tier).collect();

    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Guild Shop")
                    .size(16.0)
                    .color(Color32::from_rgb(204, 217, 230))
                    .strong(),
            );
            ui.label(
                RichText::new(format!("(Tier {})", player_tier))
                    .size(13.0)
                    .color(Color32::from_rgb(143, 194, 74)),
            );
        });

        ui.add_space(4.0);

        ScrollArea::vertical()
            .id_salt("guild_shop_scroll")
            .max_height(max_height)
            .show(ui, |ui| {
                // --- Available Items ---
                if available.is_empty() {
                    ui.label(
                        RichText::new("No items in stock today.")
                            .size(12.0)
                            .color(Color32::from_rgb(150, 150, 150)),
                    );
                }

                for item in available {
                    let def = view.item_def(&item.item_id);
                    let display_name = def.map(|d| d.name.as_str()).unwrap_or(&item.item_id);
                    let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                    let icon_color = parse_hex_color(color_hex);

                    let is_sold_out = item.stock == 0;
                    let can_afford = player_dollars >= item.price as i32;
                    let can_buy = !is_sold_out && can_afford;
                    let is_selected = state.selected_row() == Some(&item.item_id);

                    Frame::NONE
                        .fill(if is_selected {
                            Color32::from_rgb(38, 44, 48)
                        } else {
                            Color32::from_rgb(26, 28, 32)
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
                                let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, CornerRadius::same(4), icon_color);

                                if ui.button(RichText::new(display_name).size(13.0).strong().color(Color32::WHITE)).clicked() {
                                    state.select_row(&item.item_id);
                                }

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("${}", item.price))
                                            .size(13.0)
                                            .color(Color32::from_rgb(255, 230, 102))
                                            .strong(),
                                    );
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!("Stock: {}/{}", item.stock, item.max_stock))
                                        .size(11.0)
                                        .color(if is_sold_out {
                                            Color32::from_rgb(214, 80, 70)
                                        } else {
                                            Color32::from_rgb(170, 170, 170)
                                        }),
                                );

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.add_enabled(can_buy && item.stock >= 5, egui::Button::new("Buy 5")).clicked() {
                                        action = Some(GuildCampAction::BuyItem {
                                            item_id: item.item_id.clone(),
                                            count: 5.min(item.stock),
                                        });
                                    }
                                    if ui.add_enabled(can_buy, egui::Button::new("Buy 1")).clicked() {
                                        action = Some(GuildCampAction::BuyItem {
                                            item_id: item.item_id.clone(),
                                            count: 1,
                                        });
                                    }
                                });
                            });

                            if is_selected && can_buy {
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Count:").size(12.0).color(Color32::from_rgb(180, 180, 180)));
                                    if ui.button("-").clicked() {
                                        state.step_down();
                                    }
                                    ui.label(RichText::new(format!("{}", state.count())).size(13.0).strong());
                                    if ui.button("+").clicked() {
                                        state.step_up(item.stock);
                                    }
                                    let total_cost = (state.count() * item.price) as i32;
                                    let can_afford_count = player_dollars >= total_cost;
                                    let buy_btn = egui::Button::new(format!("Buy {} (${})", state.count(), total_cost));
                                    if ui.add_enabled(can_afford_count, buy_btn).clicked() {
                                        action = Some(GuildCampAction::BuyItem {
                                            item_id: item.item_id.clone(),
                                            count: state.count(),
                                        });
                                    }
                                });
                            }
                        });
                    ui.add_space(4.0);
                }

                // --- Locked Items Preview ---
                if !locked.is_empty() {
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("Higher Rank Stock")
                            .size(13.0)
                            .color(Color32::from_rgb(150, 150, 155))
                            .strong(),
                    );
                    ui.add_space(2.0);

                    for item in locked {
                        let def = view.item_def(&item.item_id);
                        let display_name = def.map(|d| d.name.as_str()).unwrap_or(&item.item_id);

                        Frame::NONE
                            .fill(Color32::from_rgb(20, 22, 24))
                            .inner_margin(Margin::same(6))
                            .corner_radius(CornerRadius::same(4))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(40, 42, 46)))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(display_name)
                                            .size(12.0)
                                            .color(Color32::from_rgb(120, 120, 120)),
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            RichText::new(format!("Tier {}", item.tier))
                                                .size(11.0)
                                                .color(Color32::from_rgb(180, 110, 60)),
                                        );
                                    });
                                });
                            });
                        ui.add_space(2.0);
                    }
                }
            });
    });

    action
}
