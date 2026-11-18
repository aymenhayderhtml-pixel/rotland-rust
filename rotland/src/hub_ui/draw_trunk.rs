//! egui 0.37 rendering for the Car Trunk storage screen (TrunkScreen.cs).

use engine::egui::{
    self, vec2, Align2, Color32, CornerRadius, Frame, Margin, Order, RichText, ScrollArea, Stroke,
};

use super::action::HubAction;
use super::draw_bag::item_text::format_trunk_row;
use super::state::HubUiState;
use super::view::HubView;

pub const TRUNK_PANEL_WIDTH: f32 = 640.0;
pub const TRUNK_PANEL_HEIGHT: f32 = 460.0;

/// Draws the Car Trunk screen: player bag on the left, trunk inventory on the right.
pub fn draw_trunk(
    ctx: &egui::Context,
    state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;
    let shift_down = ctx.input(|i| i.modifiers.shift);

    // Fullscreen dim background
    let screen_rect = ctx.content_rect();
    let bg_color = Color32::from_rgba_unmultiplied(0, 0, 0, 140);
    egui::Area::new(egui::Id::unique("trunk_background"))
        .fixed_pos(screen_rect.min)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.painter().rect_filled(screen_rect, 0.0, bg_color);
            ui.allocate_rect(screen_rect, egui::Sense::click_and_drag());
        });

    // Centered modal panel
    egui::Area::new(egui::Id::unique("trunk_content"))
        .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
        .order(Order::Foreground)
        .show(ctx, |ui| {
            Frame::NONE
                .fill(Color32::from_rgb(23, 26, 31))
                .inner_margin(Margin::same(14))
                .corner_radius(CornerRadius::same(8))
                .stroke(Stroke::new(2.0, Color32::from_rgb(89, 97, 107)))
                .show(ui, |ui| {
                    ui.set_width(TRUNK_PANEL_WIDTH);
                    ui.set_min_height(TRUNK_PANEL_HEIGHT);

                    // Header: Title with trunk weights
                    let trunk_weight = view.trunk_inventory.map(|t| t.total_weight()).unwrap_or(0.0);
                    let max_weight = view.trunk_max_weight;
                    let title = format!("Car trunk   ({trunk_weight:.1} / {max_weight:.0} kg)");
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new(title).size(20.0).strong().color(Color32::WHITE));
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Columns: Left = Bag, Right = Trunk
                    ui.horizontal(|ui| {
                        // Left: Your bag
                        ui.vertical(|ui| {
                            ui.set_width(TRUNK_PANEL_WIDTH * 0.48);
                            ui.label(
                                RichText::new("Your bag")
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::from_rgb(204, 217, 230)),
                            );
                            ui.add_space(4.0);

                            ScrollArea::vertical()
                                .id_salt("trunk_bag_scroll")
                                .max_height(260.0)
                                .show(ui, |ui| {
                                    if let Some(bag) = view.inventory {
                                        for (i, slot) in bag.slots().iter().enumerate() {
                                            if let Some(stack) = slot {
                                                let blocker = if let Some(trunk) = view.trunk_inventory {
                                                    let can_fit = match view.item_registry {
                                                        Some(reg) => trunk.can_add_with(&stack.item_id, 1, reg),
                                                        None => trunk.can_add(&stack.item_id, 1),
                                                    };
                                                    if can_fit {
                                                        None
                                                    } else {
                                                        Some("Trunk is full")
                                                    }
                                                } else {
                                                    None
                                                };
                                                let name = view.item_name(&stack.item_id);
                                                let btn_text = format_trunk_row(name, stack.count, blocker);
                                                let btn = egui::Button::new(btn_text)
                                                    .min_size(vec2(ui.available_width(), 28.0));
                                                if ui.add_enabled(blocker.is_none(), btn).clicked() {
                                                    let count = if shift_down { 1 } else { 0 };
                                                    action = Some(HubAction::TrunkMove {
                                                        from_trunk: false,
                                                        slot: i,
                                                        count,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                });
                        });

                        ui.separator();

                        // Right: In the trunk
                        ui.vertical(|ui| {
                            ui.set_width(TRUNK_PANEL_WIDTH * 0.48);
                            ui.label(
                                RichText::new("In the trunk")
                                    .size(15.0)
                                    .strong()
                                    .color(Color32::from_rgb(204, 217, 230)),
                            );
                            ui.add_space(4.0);

                            ScrollArea::vertical()
                                .id_salt("trunk_trunk_scroll")
                                .max_height(260.0)
                                .show(ui, |ui| {
                                    if let Some(trunk) = view.trunk_inventory {
                                        for (i, slot) in trunk.slots().iter().enumerate() {
                                            if let Some(stack) = slot {
                                                let blocker = if let Some(bag) = view.inventory {
                                                    let can_fit = match view.item_registry {
                                                        Some(reg) => bag.can_add_with(&stack.item_id, 1, reg),
                                                        None => bag.can_add(&stack.item_id, 1),
                                                    };
                                                    if can_fit {
                                                        None
                                                    } else {
                                                        Some("Bag is full")
                                                    }
                                                } else {
                                                    None
                                                };
                                                let name = view.item_name(&stack.item_id);
                                                let btn_text = format_trunk_row(name, stack.count, blocker);
                                                let btn = egui::Button::new(btn_text)
                                                    .min_size(vec2(ui.available_width(), 28.0));
                                                if ui.add_enabled(blocker.is_none(), btn).clicked() {
                                                    let count = if shift_down { 1 } else { 0 };
                                                    action = Some(HubAction::TrunkMove {
                                                        from_trunk: true,
                                                        slot: i,
                                                        count,
                                                    });
                                                }
                                            }
                                        }
                                    }
                                });
                        });
                    });

                    ui.add_space(10.0);

                    // Status line
                    if !state.status().is_empty() {
                        ui.label(
                            RichText::new(state.status())
                                .size(13.0)
                                .color(Color32::from_rgb(217, 242, 217)),
                        );
                    }

                    // Hint
                    ui.label(
                        RichText::new("Click moves a whole stack. Shift+click moves one. E or Esc closes.")
                            .size(12.0)
                            .color(Color32::from_rgb(160, 165, 170)),
                    );

                    ui.add_space(8.0);

                    // Footer Close button
                    if ui.add(egui::Button::new("Close (Esc)").min_size(vec2(100.0, 32.0))).clicked() {
                        action = Some(HubAction::Close);
                    }
                });
        });

    action
}
