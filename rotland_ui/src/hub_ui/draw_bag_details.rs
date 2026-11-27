//! egui 0.37 rendering for the Details subpanel in the Bag tab (InventoryDetailsPanel.cs).

use engine::egui::{self, vec2, Color32, Frame, Grid, Margin, RichText, TextureId};
use game::ItemCategory;

use super::action::HubAction;
use super::item_text::{
    action_button, format_item_category, format_item_effects, format_item_weight,
    format_weapon_stats, get_weapon_def,
};
use super::state::HubUiState;
use super::view::HubView;

pub fn draw_details_panel(
    ui: &mut egui::Ui,
    state: &mut HubUiState,
    view: &HubView,
    icon_texture: Option<TextureId>,
) -> Option<HubAction> {
    let mut action = None;
    let selected = state.selected_slot();
    let stack = view.inventory.and_then(|iv| iv.slot(selected));

    if let Some(s) = stack {
        let def = view.item_def(&s.item_id);
        let name = def.map(|d| d.name.as_str()).unwrap_or(&s.item_id);
        let category = def.map(|d| d.category.as_str()).unwrap_or("item");
        let weight = def.map(|d| d.weight).unwrap_or(0.0);
        let is_weapon = def.is_some_and(|d| d.is_weapon());
        let in_hand = view.is_in_hand(&s.item_id);

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

        // Larger item picture if icon cache exists (draws nothing when None)
        if let Some(tex) = icon_texture {
            ui.vertical_centered(|ui| {
                ui.image((tex, vec2(96.0, 96.0)));
            });
            ui.add_space(4.0);
        }

        ui.label(RichText::new(name).size(18.0).strong().color(Color32::WHITE));
        ui.label(
            RichText::new(format_item_category(category).to_ascii_uppercase())
                .size(11.0)
                .color(Color32::GRAY),
        );

        ui.label(
            RichText::new(format_item_weight(weight, s.count))
                .size(12.0)
                .color(Color32::from_rgb(180, 180, 180)),
        );

        // Action buttons grid: Use, Equip/Unequip, Drop 1, Drop all, Split share same size & style
        ui.add_space(8.0);
        let can_split = s.count > 1;
        state.clamp_split_count(s.count / 2);

        Grid::new("bag_action_buttons")
            .spacing(vec2(8.0, 8.0))
            .show(ui, |ui| {
                if action_button(ui, "Use", can_use).clicked() {
                    action = Some(HubAction::UseItem { slot: selected });
                }

                let equip_label = if in_hand { "Unequip" } else { "Equip" };
                if action_button(ui, equip_label, is_weapon).clicked() {
                    action = Some(HubAction::EquipItem { slot: selected });
                }
                ui.end_row();

                if action_button(ui, "Drop 1", true).clicked() {
                    action = Some(HubAction::DropItem { slot: selected, count: 1 });
                }

                if action_button(ui, "Drop all", true).clicked() {
                    action = Some(HubAction::DropItem { slot: selected, count: 0 });
                }
                ui.end_row();

                if action_button(ui, "Split", can_split).clicked() {
                    action = Some(HubAction::SplitStack { slot: selected });
                }

                ui.horizontal(|ui| {
                    if can_split {
                        if ui.button("-").clicked() {
                            state.step_split_down();
                        }
                        ui.label(RichText::new(format!("{}", state.split_count())).size(14.0).strong());
                        if ui.button("+").clicked() {
                            state.step_split_up(s.count / 2);
                        }
                    }
                });
                ui.end_row();
            });

        // Details below buttons: description, weapon stats, food/drink/medical effects
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(6.0);

        if let Some(d) = def
            && !d.description.is_empty()
        {
            ui.label(
                RichText::new(&d.description)
                    .size(12.0)
                    .color(Color32::from_rgb(200, 200, 200)),
            );
            ui.add_space(4.0);
        }

        if is_weapon {
            if let Some(d) = def
                && let Some(weapon) = get_weapon_def(&d.weapon_id) {
                    let ammo_name = if weapon.is_gun() && !weapon.ammo_item.is_empty() {
                        Some(view.item_name(&weapon.ammo_item))
                    } else {
                        None
                    };
                    let stats = format_weapon_stats(&weapon, ammo_name);
                    for line in stats {
                        ui.label(
                            RichText::new(line)
                                .size(12.0)
                                .color(Color32::from_rgb(220, 210, 180)),
                        );
                    }
                }
        } else if let Some(d) = def {
            let effects_str = format_item_effects(&d.effects);
            if !effects_str.is_empty() {
                let prefix = match d.category {
                    ItemCategory::Food | ItemCategory::Drink => "Restores: ",
                    ItemCategory::Medical => "Heals: ",
                    _ => "Effects: ",
                };
                ui.label(
                    RichText::new(format!("{prefix}{effects_str}"))
                        .size(12.0)
                        .color(Color32::from_rgb(143, 194, 74)),
                );
            }
            static ARMOR_CFG: std::sync::OnceLock<game::ArmorConfig> = std::sync::OnceLock::new();
            let armor_cfg = ARMOR_CFG.get_or_init(|| game::ArmorConfig::from_repo().unwrap_or_default());
            if let Some(armor) = armor_cfg.items.get(&d.id) {
                ui.label(
                    RichText::new(format!("Bite damage -{:.0}%", armor.reduce * 100.0)) // new: DECISIONS 2026-11-27
                        .size(12.0)
                        .color(Color32::from_rgb(143, 194, 74)),
                );
            }
        }
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
