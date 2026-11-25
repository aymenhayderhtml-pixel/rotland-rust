//! egui 0.37 rendering for the Crafting list inside the Bag tab (GitHub #9).

use engine::egui::{self, Color32, ProgressBar, RichText, ScrollArea};
use game::{CAMPFIRE_STATION, HAND_STATION};

use super::action::HubAction;
use super::item_text::craft_blocker_reason;
use super::view::HubView;

pub const SHORT_COLOR: Color32 = Color32::from_rgb(255, 102, 89);
pub const NORMAL_COUNT_COLOR: Color32 = Color32::from_rgb(200, 210, 200);

/// Draws the crafting subpanel in the Bag screen, showing recipes, ingredients (have/need),
/// station status, disabled reasons on hover, and active craft progress.
pub fn draw_crafting_subpanel(ui: &mut egui::Ui, view: &HubView) -> Option<HubAction> {
    let mut action = None;

    ui.horizontal(|ui| {
        ui.label(RichText::new("Crafting").size(14.0).strong().color(Color32::WHITE));
        let station_text = if view.near_lit_campfire {
            "[Campfire ready]"
        } else {
            "[Hand]"
        };
        ui.label(RichText::new(station_text).size(12.0).color(Color32::from_rgb(180, 200, 180)));
    });

    ScrollArea::vertical()
        .id_salt("bag_craft_scroll")
        .max_height(140.0)
        .show(ui, |ui| {
            if let (Some(registry), Some(inv)) = (view.recipe_registry, view.inventory) {
                for recipe in registry.all() {
                    let is_hand = recipe.station == HAND_STATION;
                    let is_campfire = recipe.station == CAMPFIRE_STATION;
                    if !is_hand && !is_campfire {
                        continue;
                    }

                    let is_crafting = view.crafting.is_some_and(|c| c.is_crafting());
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

                    ui.horizontal(|ui| {
                        // Recipe name / output
                        ui.label(RichText::new(&recipe.name).size(13.0).color(Color32::WHITE));

                        // Station need indicator
                        if is_campfire {
                            if view.near_lit_campfire {
                                ui.label(
                                    RichText::new("[Campfire]")
                                        .size(11.0)
                                        .color(Color32::from_rgb(235, 140, 70)),
                                );
                            } else {
                                ui.label(
                                    RichText::new("[Needs campfire]")
                                        .size(11.0)
                                        .color(SHORT_COLOR),
                                );
                            }
                        }

                        // Inputs (have/need) with red color for short items
                        for input in &recipe.inputs {
                            let have = inv.count(&input.id);
                            let is_short = have < input.count;
                            let count_color = if is_short { SHORT_COLOR } else { NORMAL_COUNT_COLOR };
                            let item_name = view.item_name(&input.id);
                            ui.label(
                                RichText::new(format!("{item_name} {have}/{}", input.count))
                                    .size(12.0)
                                    .color(count_color),
                            );
                        }

                        ui.label(RichText::new("->").size(12.0).color(Color32::GRAY));

                        // Output items
                        for out in &recipe.outputs {
                            let name = view.item_name(&out.id);
                            ui.label(RichText::new(format!("{name} x{}", out.count)).size(12.0));
                        }

                        // Craft button
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let mut craft_btn = ui.add_enabled(can_craft, egui::Button::new("Craft"));
                            if !can_craft
                                && let Some(reason) = &blocker_reason {
                                    craft_btn = craft_btn.on_disabled_hover_text(reason);
                                }
                            if craft_btn.clicked() {
                                action = Some(HubAction::Craft {
                                    recipe: recipe.id.clone(),
                                });
                            }
                        });
                    });
                    ui.separator();
                }
            }
        });

    // Active craft progress
    if let Some(crafting) = view.crafting
        && let Some(active) = crafting.active()
    {
        ui.horizontal(|ui| {
            let p = crafting.progress();
            ui.label(RichText::new(format!("{}: {:.0}%", active.name, p * 100.0)).size(12.0));
            ui.add(ProgressBar::new(p).desired_width(120.0));
            if ui.button("Cancel").clicked() {
                action = Some(HubAction::CancelCraft);
            }
        });
    }

    action
}
