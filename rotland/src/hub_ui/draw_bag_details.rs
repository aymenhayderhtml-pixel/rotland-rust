//! egui 0.37 rendering for the Details subpanel in the Bag tab (InventoryDetailsPanel.cs).

use engine::egui::{self, vec2, Color32, Frame, Grid, Margin, RichText};
use game::ItemCategory;

use super::action::HubAction;
use super::state::HubUiState;
use super::view::HubView;

pub fn draw_details_panel(
    ui: &mut egui::Ui,
    state: &mut HubUiState,
    view: &HubView,
) -> Option<HubAction> {
    let mut action = None;
    let selected = state.selected_slot();
    let stack = view.inventory.and_then(|iv| iv.slot(selected));
    let equipped_id = view.hotbar.and_then(|h| h.equipped_item_id());

    if let Some(s) = stack {
        let def = view.item_def(&s.item_id);
        let name = def.map(|d| d.name.as_str()).unwrap_or(&s.item_id);
        let category = def.map(|d| d.category.as_str()).unwrap_or("item");
        let weight = def.map(|d| d.weight).unwrap_or(0.0);
        let total_weight = weight * s.count as f32;
        let is_weapon = def.is_some_and(|d| d.is_weapon());
        let in_hand = equipped_id == Some(&s.item_id);

        let can_use = def.is_some_and(|d| {
            let usable = matches!(
                d.category,
                ItemCategory::Food
                    | ItemCategory::Drink
                    | ItemCategory::Medical
                    | ItemCategory::Tool
                    | ItemCategory::Placeable
            );
            let full_health = d.category == ItemCategory::Medical
                && view.health.is_some_and(|(curr, max)| curr >= max);
            usable && !full_health
        });

        ui.label(RichText::new(name).size(18.0).strong().color(Color32::WHITE));
        ui.label(RichText::new(category.to_ascii_uppercase()).size(11.0).color(Color32::GRAY));

        if let Some(d) = def
            && !d.description.is_empty() {
                ui.label(RichText::new(&d.description).size(12.0).color(Color32::from_rgb(200, 200, 200)));
            }

        ui.label(
            RichText::new(format!("Weight: {weight:.2} kg each, {total_weight:.2} kg total"))
                .size(12.0)
                .color(Color32::from_rgb(180, 180, 180)),
        );

        // Action buttons grid
        ui.add_space(8.0);
        Grid::new("bag_action_buttons")
            .spacing(vec2(8.0, 8.0))
            .show(ui, |ui| {
                if ui.add_enabled(can_use, egui::Button::new("Use").min_size(vec2(130.0, 30.0))).clicked() {
                    action = Some(HubAction::UseItem { slot: selected });
                }

                let equip_label = if in_hand { "Unequip" } else { "Equip" };
                if ui.add_enabled(is_weapon, egui::Button::new(equip_label).min_size(vec2(130.0, 30.0))).clicked() {
                    action = Some(HubAction::EquipItem { slot: selected });
                }
                ui.end_row();

                if ui.button("Drop 1").clicked() {
                    action = Some(HubAction::DropItem { slot: selected, count: 1 });
                }

                if ui.button("Drop all").clicked() {
                    action = Some(HubAction::DropItem { slot: selected, count: 0 });
                }
                ui.end_row();
            });

        // Split stack controls
        ui.add_space(8.0);
        let can_split = s.count > 1;
        state.clamp_split_count(s.count / 2);
        ui.horizontal(|ui| {
            if ui.add_enabled(can_split, egui::Button::new("Split")).clicked() {
                action = Some(HubAction::SplitStack { slot: selected });
            }
            if can_split {
                if ui.button("-").clicked() {
                    state.step_split_down();
                }
                ui.label(format!("{}", state.split_count()));
                if ui.button("+").clicked() {
                    state.step_split_up(s.count / 2);
                }
            }
        });
    } else {
        ui.label(RichText::new("Empty slot").size(16.0).color(Color32::GRAY));
        ui.add_space(8.0);
        Frame::NONE
            .fill(Color32::from_rgb(20, 20, 20))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.label("Click a slot to view details, use items, equip weapons, or split stacks.");
            });
    }

    action
}
