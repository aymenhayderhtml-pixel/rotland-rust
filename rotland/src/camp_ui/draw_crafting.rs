//! egui 0.37 rendering for the Crafting panel.

use engine::egui::{self, vec2, Color32, CornerRadius, Frame, Margin, ProgressBar, RichText, ScrollArea, Stroke};
use game::{CAMPFIRE_STATION, HAND_STATION};

use crate::hub_ui::draw_bag::item_text::craft_blocker_reason;

use super::action::CampAction;
use super::state::CampUiState;
use super::view::CampView;

/// Draws the Crafting screen: recipe list, ingredient status, and active craft progress bar.
pub fn draw_crafting(ui: &mut egui::Ui, _state: &mut CampUiState, view: &CampView) -> Option<CampAction> {
    let mut action = None;

    // --- Station hint banner ---
    let hint_text = if view.near_lit_campfire {
        "At the campfire. Cooking recipes are ready."
    } else {
        "Made by hand. Cooking needs a lit campfire."
    };
    ui.label(
        RichText::new(hint_text)
            .size(13.0)
            .color(Color32::from_rgb(180, 200, 180)),
    );
    ui.add_space(8.0);

    // --- Recipe list ---
    ScrollArea::vertical()
        .id_salt("crafting_recipe_scroll")
        .max_height(360.0)
        .show(ui, |ui| {
            if let (Some(registry), Some(inv)) = (view.recipe_registry, view.inventory) {
                for recipe in registry.all() {
                    let station_ready = recipe.station == HAND_STATION || view.near_lit_campfire;
                    let can_craft = view.crafting.is_some_and(|c| {
                        c.can_craft(
                            &recipe.id,
                            &recipe.station,
                            inv,
                            view.near_lit_campfire,
                            view.is_dead,
                            registry,
                        )
                    });

                    let is_crafting = view.crafting.is_some_and(|c| c.is_crafting());
                    let blocker_reason = if can_craft {
                        None
                    } else {
                        craft_blocker_reason(
                            recipe,
                            Some(inv),
                            view.near_lit_campfire,
                            is_crafting,
                            view.is_dead,
                            |id| view.item_name(id).to_string(),
                        )
                    };

                    Frame::NONE
                        .fill(if station_ready {
                            Color32::from_rgb(28, 30, 34)
                        } else {
                            Color32::from_rgb(22, 24, 26)
                        })
                        .inner_margin(Margin::same(8))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(50, 54, 60)))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                // Recipe name and station tag
                                let name_color = if station_ready {
                                    Color32::WHITE
                                } else {
                                    Color32::from_rgb(130, 130, 130)
                                };
                                ui.label(
                                    RichText::new(&recipe.name)
                                        .size(15.0)
                                        .color(name_color)
                                        .strong(),
                                );

                                if recipe.station == CAMPFIRE_STATION {
                                    if view.near_lit_campfire {
                                        ui.label(
                                            RichText::new("[Campfire]")
                                                .size(12.0)
                                                .color(Color32::from_rgb(235, 140, 70)),
                                        );
                                    } else {
                                        ui.label(
                                            RichText::new("[Needs campfire]")
                                                .size(12.0)
                                                .color(Color32::from_rgb(255, 102, 89)),
                                        );
                                    }
                                } else {
                                    ui.label(
                                        RichText::new("[Hand]")
                                            .size(12.0)
                                            .color(Color32::from_rgb(140, 160, 180)),
                                    );
                                }

                                ui.separator();

                                // Inputs (have / need)
                                for input in &recipe.inputs {
                                    let have = inv.count(&input.id);
                                    let is_short = have < input.count;
                                    let def = view.item_def(&input.id);
                                    let item_name = def.map(|d| d.name.as_str()).unwrap_or(&input.id);
                                    let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                                    let icon_color = super::draw::parse_hex_color(color_hex);

                                    let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, CornerRadius::same(3), icon_color);

                                    let count_color = if is_short {
                                        Color32::from_rgb(255, 102, 89)
                                    } else {
                                        Color32::from_rgb(200, 210, 200)
                                    };
                                    ui.label(
                                        RichText::new(format!("{item_name} {have}/{}", input.count))
                                            .size(13.0)
                                            .color(count_color),
                                    );
                                }

                                ui.label(RichText::new("->").size(14.0).color(Color32::GRAY));

                                // Outputs
                                for output in &recipe.outputs {
                                    let def = view.item_def(&output.id);
                                    let item_name = def.map(|d| d.name.as_str()).unwrap_or(&output.id);
                                    let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#777777");
                                    let icon_color = super::draw::parse_hex_color(color_hex);

                                    let (rect, _) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::hover());
                                    ui.painter().rect_filled(rect, CornerRadius::same(3), icon_color);

                                    ui.label(
                                        RichText::new(format!("{item_name} x{}", output.count))
                                            .size(13.0)
                                            .color(Color32::WHITE),
                                    );
                                }

                                // Craft button
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let mut craft_btn = ui.add_enabled(can_craft, egui::Button::new("Craft"));
                                    if !can_craft {
                                        if let Some(reason) = &blocker_reason {
                                            craft_btn = craft_btn.on_disabled_hover_text(reason);
                                        }
                                    }
                                    if craft_btn.clicked() {
                                        action = Some(CampAction::Craft {
                                            recipe: recipe.id.clone(),
                                        });
                                    }
                                });
                            });
                        });
                    ui.add_space(4.0);
                }
            } else {
                ui.label("Crafting recipes unavailable.");
            }
        });

    ui.add_space(8.0);

    // --- Active craft progress row ---
    if let Some(crafting) = view.crafting
        && let Some(active) = crafting.active()
    {
        Frame::NONE
            .fill(Color32::from_rgb(32, 34, 38))
            .inner_margin(Margin::same(10))
            .corner_radius(CornerRadius::same(6))
            .stroke(Stroke::new(1.0, Color32::from_rgb(70, 75, 85)))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let progress = crafting.progress();
                    let pct = (progress * 100.0) as u32;

                    ui.label(
                        RichText::new(format!("Crafting: {}... {pct}%", active.name))
                            .size(14.0)
                            .strong(),
                    );

                    ui.add(ProgressBar::new(progress).desired_width(180.0));

                    if ui.button("Cancel").clicked() {
                        action = Some(CampAction::CancelCraft);
                    }
                });
            });
    }

    action
}
