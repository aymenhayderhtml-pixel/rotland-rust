//! egui 0.37 rendering for dog interaction screens: DogMenu, DogPackScreen, and DogNameDialog.

use engine::egui::{
    self, vec2, Align2, Color32, CornerRadius, Frame, Margin, Order, RichText, ScrollArea, Stroke,
    TextEdit,
};

use super::action::{DogCommand, HubAction};
use super::state::HubUiState;
use super::view::HubView;

pub const DOG_MENU_WIDTH: f32 = 300.0;
pub const DOG_MENU_HEIGHT: f32 = 240.0;

pub const DOG_PACK_WIDTH: f32 = 600.0;
pub const DOG_PACK_HEIGHT: f32 = 420.0;

pub const DOG_DIALOG_WIDTH: f32 = 360.0;
pub const DOG_DIALOG_HEIGHT: f32 = 150.0;

/// Draws the DogMenu (quick interaction radial/dialog opened with E on dog).
pub fn draw_dog_menu(
    ctx: &egui::Context,
    _state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;

    dim_background(ctx, "dog_menu_dim");

    egui::Area::new(egui::Id::unique("dog_menu_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(23, 26, 31))
                .inner_margin(Margin::same(14))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(89, 97, 107)))
                .show(ui, |ui| {
                    ui.set_width(DOG_MENU_WIDTH);
                    ui.set_min_height(DOG_MENU_HEIGHT);

                    let dog = view.dog.as_ref();
                    let name = dog.map(|d| d.name.as_str()).unwrap_or("Dog");
                    let hp = dog.map(|d| d.health).unwrap_or(0.0);
                    let max_hp = dog.map(|d| d.max_health).unwrap_or(1.0);
                    let title = format!("{name}  ({hp:.0}/{max_hp:.0} HP)");

                    ui.label(RichText::new(title).size(18.0).strong().color(Color32::WHITE));
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(10.0);

                    // Pack button
                    let pack_btn = egui::Button::new("Pack").min_size(vec2(ui.available_width(), 34.0));
                    if ui.add(pack_btn).clicked() {
                        action = Some(HubAction::DogCommand(DogCommand::OpenPack));
                    }
                    ui.add_space(6.0);

                    // Feed button
                    let has_food = dog.is_some_and(|d| d.has_food_in_bag);
                    let feed_text = if has_food {
                        "Feed (1 meat)"
                    } else {
                        "Feed (needs raw or cooked meat)"
                    };
                    let feed_btn = egui::Button::new(feed_text).min_size(vec2(ui.available_width(), 34.0));
                    if ui.add_enabled(has_food, feed_btn).clicked() {
                        action = Some(HubAction::DogCommand(DogCommand::Feed));
                    }
                    ui.add_space(6.0);

                    // Stay / Follow button
                    let is_stay = dog.is_some_and(|d| d.mode.eq_ignore_ascii_case("stay"));
                    let stay_text = if is_stay { "Follow (G)" } else { "Stay (G)" };
                    let stay_btn = egui::Button::new(stay_text).min_size(vec2(ui.available_width(), 34.0));
                    if ui.add(stay_btn).clicked() {
                        action = Some(HubAction::DogCommand(DogCommand::ToggleStay));
                    }
                    ui.add_space(6.0);

                    // Close button
                    let close_btn = egui::Button::new("Close").min_size(vec2(ui.available_width(), 34.0));
                    if ui.add(close_btn).clicked() {
                        action = Some(HubAction::DogCommand(DogCommand::Close));
                    }
                });
        });

    action
}

/// Draws the DogPackScreen (storage between player bag and dog backpack).
pub fn draw_dog_pack(
    ctx: &egui::Context,
    _state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;
    let shift_down = ctx.input(|i| i.modifiers.shift);

    dim_background(ctx, "dog_pack_dim");

    egui::Area::new(egui::Id::unique("dog_pack_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(23, 26, 31))
                .inner_margin(Margin::same(14))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(89, 97, 107)))
                .show(ui, |ui| {
                    ui.set_width(DOG_PACK_WIDTH);
                    ui.set_min_height(DOG_PACK_HEIGHT);

                    let dog = view.dog.as_ref();
                    let name = dog.map(|d| d.name.as_str()).unwrap_or("Dog");
                    ui.label(
                        RichText::new(format!("{name}'s pack"))
                            .size(20.0)
                            .strong()
                            .color(Color32::WHITE),
                    );

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Two columns: Left = Bag, Right = Pack
                    ui.horizontal(|ui| {
                        // Left: Your bag (click to store)
                        ui.vertical(|ui| {
                            ui.set_width(DOG_PACK_WIDTH * 0.48);
                            ui.label(
                                RichText::new("Your bag (click to store)")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(204, 217, 230)),
                            );
                            ui.add_space(4.0);

                            ScrollArea::vertical()
                                .id_salt("dog_pack_bag_scroll")
                                .max_height(220.0)
                                .show(ui, |ui| {
                                    if let Some(bag) = view.inventory {
                                        for (i, slot) in bag.slots().iter().enumerate() {
                                            if let Some(stack) = slot {
                                                let item_name = view.item_name(&stack.item_id);
                                                let btn_text = format!("{item_name}   x{}", stack.count);
                                                let btn = egui::Button::new(btn_text)
                                                    .min_size(vec2(ui.available_width(), 28.0));
                                                if ui.add(btn).clicked() {
                                                    let count = if shift_down { 1 } else { 0 };
                                                    action = Some(HubAction::DogCommand(
                                                        DogCommand::StorePack {
                                                            bag_slot: i,
                                                            count,
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                });
                        });

                        ui.separator();

                        // Right: In the pack (click to take out)
                        ui.vertical(|ui| {
                            ui.set_width(DOG_PACK_WIDTH * 0.48);
                            ui.label(
                                RichText::new("In the pack (click to take out)")
                                    .size(14.0)
                                    .strong()
                                    .color(Color32::from_rgb(204, 217, 230)),
                            );
                            ui.add_space(4.0);

                            ScrollArea::vertical()
                                .id_salt("dog_pack_pack_scroll")
                                .max_height(220.0)
                                .show(ui, |ui| {
                                    if let Some(pack) = dog.and_then(|d| d.pack_inventory) {
                                        for (i, slot) in pack.slots().iter().enumerate() {
                                            if let Some(stack) = slot {
                                                let item_name = view.item_name(&stack.item_id);
                                                let btn_text = format!("{item_name}   x{}", stack.count);
                                                let btn = egui::Button::new(btn_text)
                                                    .min_size(vec2(ui.available_width(), 28.0));
                                                if ui.add(btn).clicked() {
                                                    let count = if shift_down { 1 } else { 0 };
                                                    action = Some(HubAction::DogCommand(
                                                        DogCommand::TakePack {
                                                            pack_slot: i,
                                                            count,
                                                        },
                                                    ));
                                                }
                                            }
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(10.0);

                    // Weight line
                    if let Some(d) = dog {
                        let used_slots = d.pack_inventory.map(|p| p.slots().iter().filter(|s| s.is_some()).count()).unwrap_or(0);
                        let total_weight = d.pack_inventory.map(|p| p.total_weight()).unwrap_or(0.0);
                        ui.label(
                            RichText::new(format!(
                                "Pack: {used_slots} of {} slots, {total_weight:.1} of {:.0} kg",
                                d.pack_slots, d.pack_max_weight
                            ))
                            .size(13.0)
                            .color(Color32::from_rgb(200, 210, 220)),
                        );
                    }

                    ui.label(
                        RichText::new("Shift+click moves one.")
                            .size(12.0)
                            .color(Color32::from_rgb(160, 165, 170)),
                    );

                    ui.add_space(8.0);

                    // Close button
                    if ui.add(egui::Button::new("Close").min_size(vec2(100.0, 32.0))).clicked() {
                        action = Some(HubAction::DogCommand(DogCommand::Close));
                    }
                });
        });

    action
}

/// Draws the DogNameDialog ("Name your puppy").
pub fn draw_dog_name_dialog(
    ctx: &egui::Context,
    state: &mut HubUiState,
    _view: &HubView,
) -> Option<HubAction> {
    let mut action = None;

    dim_background(ctx, "dog_name_dim");

    egui::Area::new(egui::Id::unique("dog_name_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(23, 26, 31))
                .inner_margin(Margin::same(16))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(89, 97, 107)))
                .show(ui, |ui| {
                    ui.set_width(DOG_DIALOG_WIDTH);
                    ui.set_min_height(DOG_DIALOG_HEIGHT);

                    ui.label(RichText::new("Name your puppy").size(18.0).strong().color(Color32::WHITE));
                    ui.add_space(10.0);

                    let text_edit = TextEdit::singleline(&mut state.dog_name_input)
                        .char_limit(12)
                        .desired_width(ui.available_width());
                    let edit_response = ui.add(text_edit);

                    ui.add_space(12.0);

                    let enter_pressed = edit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                    if ui.add(egui::Button::new("OK").min_size(vec2(100.0, 32.0))).clicked() || enter_pressed {
                        action = Some(HubAction::RenameDog {
                            name: state.dog_name_input.trim().to_string(),
                        });
                    }
                });
        });

    action
}

fn dim_background(ctx: &egui::Context, id_str: &str) {
    let screen_rect = ctx.content_rect();
    let bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, 140);
    egui::Area::new(egui::Id::unique(id_str))
        .fixed_pos(screen_rect.min)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, bg_color);
            ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
        });
}
