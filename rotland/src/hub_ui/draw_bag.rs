//! egui 0.37 rendering for the Bag (inventory) tab in the player hub.

use engine::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Grid,
    Rect, RichText, Sense, Stroke,
};
use game::{DEFAULT_SLOT_COUNT, HOTBAR_SIZE};


use super::draw_crafting::draw_crafting_subpanel;
use super::action::HubAction;
use super::draw::parse_hex_color;
use super::draw_bag_details::draw_details_panel;
use super::state::HubUiState;
use super::view::HubView;

pub const OVERWEIGHT_RATIO: f32 = 0.9;
pub const OVERWEIGHT_COLOR: Color32 = Color32::from_rgb(255, 89, 89);
pub const NORMAL_WEIGHT_COLOR: Color32 = Color32::from_rgb(217, 217, 217);

pub const SLOT_SIZE: f32 = 56.0;
pub const GRID_COLUMNS: usize = 6;

/// Draws the Bag tab: 6x4 slot grid, crafting list below, details panel on the right.
pub fn draw_bag(ui: &mut egui::Ui, state: &mut HubUiState, view: &HubView) -> Option<HubAction> {
    let mut action = None;

    // --- Header: Title and Weight ---
    ui.horizontal(|ui| {
        ui.label(RichText::new("Bag").size(18.0).strong().color(Color32::WHITE));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(inv) = view.inventory {
                let total = inv.total_weight();
                let max = inv.max_weight();
                let is_over = total > max * OVERWEIGHT_RATIO;
                let color = if is_over { OVERWEIGHT_COLOR } else { NORMAL_WEIGHT_COLOR };
                ui.label(
                    RichText::new(format!("{total:.1} / {max:.1} kg"))
                        .size(15.0)
                        .strong()
                        .color(color),
                );
            }
        });
    });

    ui.add_space(6.0);
    ui.separator();
    ui.add_space(6.0);

    // --- Main layout: Left (Grid + Crafting) and Right (Details) ---
    ui.horizontal(|ui| {
        // Left Column (Grid + Crafting list)
        ui.vertical(|ui| {
            ui.set_width(380.0);

            // 6x4 Slots Grid
            let grid_action = draw_slot_grid(ui, state, view);
            if let Some(act) = grid_action {
                action = Some(act);
            }

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            // Crafting List under grid
            let craft_action = draw_crafting_subpanel(ui, view);
            if let Some(act) = craft_action {
                action = Some(act);
            }
        });

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(12.0);

        // Right Column: Details Panel
        ui.vertical(|ui| {
            ui.set_width(300.0);
            let details_action = draw_details_panel(ui, state, view, None);
            if let Some(act) = details_action {
                action = Some(act);
            }
        });
    });

    action
}

fn draw_slot_grid(
    ui: &mut egui::Ui,
    state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;
    let inv = view.inventory;
    let equipped_id = view.hotbar.and_then(|h| h.equipped_item_id());
    let slot_count = inv.map(|i| i.slot_count()).unwrap_or(DEFAULT_SLOT_COUNT);

    Grid::new("bag_slot_grid")
        .spacing(vec2(6.0, 6.0))
        .show(ui, |ui| {
            for i in 0..slot_count {
                let stack = inv.and_then(|iv| iv.slot(i));
                let is_equipped = stack.is_some_and(|s| equipped_id == Some(&s.item_id));
                let is_selected = state.selected_slot() == i;
                let is_hotbar = i < HOTBAR_SIZE;

                // Border styling
                let border_color = if is_selected {
                    Color32::WHITE
                } else if is_equipped {
                    Color32::from_rgb(255, 217, 51)
                } else if is_hotbar {
                    Color32::from_rgb(140, 179, 217)
                } else {
                    Color32::from_rgb(89, 89, 89)
                };
                let border_width = if is_selected { 3.0 } else if is_equipped || is_hotbar { 2.0 } else { 1.0 };

                let (rect, response) = ui.allocate_exact_size(
                    vec2(SLOT_SIZE, SLOT_SIZE),
                    Sense::click_and_drag(),
                );

                // Painter drawing
                let painter = ui.painter();
                painter.rect(
                    rect,
                    CornerRadius::same(4),
                    Color32::from_rgba_unmultiplied(26, 26, 26, 220),
                    Stroke::new(border_width, border_color),
                    engine::egui::StrokeKind::Inside,
                );

                // Hotbar key number top-left (1..6)
                if is_hotbar {
                    painter.text(
                        pos2(rect.min.x + 4.0, rect.min.y + 2.0),
                        Align2::LEFT_TOP,
                        (i + 1).to_string(),
                        FontId::proportional(11.0),
                        Color32::from_rgb(180, 180, 180),
                    );
                }

                if let Some(s) = stack {
                    let def = view.item_def(&s.item_id);
                    let color_hex = def.map(|d| d.color.as_str()).unwrap_or("#888888");
                    let swatch_color = parse_hex_color(color_hex);

                    // Swatch rectangle
                    let swatch_rect = Rect::from_center_size(
                        pos2(rect.center().x, rect.center().y - 2.0),
                        vec2(26.0, 16.0),
                    );
                    painter.rect_filled(swatch_rect, CornerRadius::same(2), swatch_color);

                    // Count top-right if > 1
                    if s.count > 1 {
                        painter.text(
                            pos2(rect.max.x - 4.0, rect.min.y + 2.0),
                            Align2::RIGHT_TOP,
                            s.count.to_string(),
                            FontId::proportional(11.0),
                            Color32::WHITE,
                        );
                    }

                    // Item short name at bottom
                    let label = def.map(|d| d.hotbar_label()).unwrap_or(&s.item_id);
                    let display_label = if label.len() > 7 { &label[..7] } else { label };
                    painter.text(
                        pos2(rect.center().x, rect.max.y - 3.0),
                        Align2::CENTER_BOTTOM,
                        display_label,
                        FontId::proportional(10.0),
                        Color32::WHITE,
                    );
                }

                // Clicks and Drag
                if response.clicked() {
                    if state.is_dragging() {
                        if let Some((from, to)) = state.drop_drag(i) {
                            action = Some(HubAction::MoveSlot { from, to });
                        }
                    } else {
                        state.select_slot(i);
                    }
                } else if response.secondary_clicked() {
                    state.select_slot(i);
                    if let Some(s) = stack {
                        let is_weapon = view.item_def(&s.item_id).is_some_and(|d| d.is_weapon());
                        if is_weapon {
                            action = Some(HubAction::EquipItem { slot: i });
                        } else {
                            action = Some(HubAction::UseItem { slot: i });
                        }
                    }
                }

                if response.drag_started() {
                    state.start_drag(i);
                }

                if (i + 1) % GRID_COLUMNS == 0 {
                    ui.end_row();
                }
            }
        });

    action
}

