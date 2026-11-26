//! Upgrade tab for the Gunsmith screen: pick a weapon, manage 5 attachment slots, and preview stat bars.

use engine::egui::{
    self, Color32, CornerRadius, Frame, Margin, Rect, RichText, ScrollArea, Sense, Stroke, vec2,
};

use crate::hud::IconCache;
use crate::menus::theme;

use super::action::{GunsmithAction, GunsmithSlot};
use super::screen::{GunsmithLayout, GunsmithUiState};
use super::{
    GunsmithModOption, GunsmithSlotState, GunsmithUpgradeWeapon, GunsmithView,
    GunsmithWeaponStats, draw_item_icon, draw_scrap_badge,
};

/// Draws the Upgrade tab: weapon list on the left, 5 slots + before/after stat bars on the right.
pub fn draw_upgrade(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    if view.upgrade_weapons.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No upgradable weapons in your bag.").size(15.0).color(theme::TEXT_MUTED));
        });
        return None;
    }

    if !state.selected_upgrade_slot.is_some_and(|s| view.upgrade_weapons.iter().any(|w| w.slot == s)) {
        state.selected_upgrade_slot = Some(view.upgrade_weapons[0].slot);
    }

    let mut action = None;
    let selected_slot = state.selected_upgrade_slot.unwrap_or(view.upgrade_weapons[0].slot);
    let selected_weapon = view.upgrade_weapons.iter().find(|w| w.slot == selected_slot).unwrap_or(&view.upgrade_weapons[0]);

    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing.x = layout.upgrade_gap;
        draw_weapon_picker(ui, state, view, selected_weapon.slot, layout, icons.as_deref_mut());

        ui.allocate_ui_with_layout(
            vec2(layout.upgrade_right_width, layout.body_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.set_width(layout.upgrade_right_width);
                ui.set_max_width(layout.upgrade_right_width);
                let mut preview_stats: Option<GunsmithWeaponStats> = None;

                ui.allocate_ui_with_layout(
                    vec2(layout.upgrade_right_width, layout.upgrade_slots_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width(layout.upgrade_right_width);
                        ui.set_min_height(layout.upgrade_slots_height);
                        ui.set_max_height(layout.upgrade_slots_height);
                        let (act, preview) = draw_slots_panel(
                            ui,
                            state,
                            selected_weapon,
                            view.scrap_count,
                            layout,
                            icons,
                        );
                        action = act;
                        preview_stats = preview;
                    },
                );

                ui.add_space(6.0);
                ui.allocate_ui_with_layout(
                    vec2(layout.upgrade_right_width, layout.upgrade_stats_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_max_width(layout.upgrade_right_width);
                        draw_stat_comparison(
                            ui,
                            &selected_weapon.base_stats,
                            preview_stats.as_ref().unwrap_or(&selected_weapon.base_stats),
                            layout.upgrade_right_width,
                        );
                    },
                );
            },
        );
    });

    action
}

fn draw_weapon_picker(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    selected_slot: usize,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) {
    ui.allocate_ui_with_layout(
        vec2(layout.upgrade_left_width, layout.body_height),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
        ui.set_width(layout.upgrade_left_width);
        ui.set_max_width(layout.upgrade_left_width);
        ui.label(RichText::new("Your weapons").size(14.5).color(theme::TEXT_PRIMARY).strong());
        ui.add_space(4.0);
        ScrollArea::vertical()
            .id_salt("gunsmith_upgrade_weapons")
            .max_height((layout.body_height - 26.0).max(100.0))
            .show(ui, |ui| {
                let row_w = (layout.upgrade_left_width - 28.0).max(120.0);
                for weapon in &view.upgrade_weapons {
                    let is_sel = weapon.slot == selected_slot;
                    let fitted = weapon.slots.iter().filter(|s| s.fitted_mod.is_some()).count();
                    let pct = (weapon.condition.clamp(0.0, 1.0) * 100.0).round() as u32;
                    let resp = Frame::NONE
                        .fill(if is_sel { Color32::from_rgb(38, 44, 48) } else { theme::CARD_BG })
                        .inner_margin(Margin::same(7))
                        .corner_radius(CornerRadius::same(6))
                        .stroke(Stroke::new(1.0, if is_sel { theme::ACCENT } else { theme::BORDER }))
                        .show(ui, |ui| {
                            ui.set_width(row_w);
                            ui.horizontal(|ui| {
                                draw_item_icon(ui, icons.as_deref_mut(), &weapon.item_id, &weapon.name, &weapon.color_hex, vec2(30.0, 30.0));
                                ui.vertical(|ui| {
                                    ui.spacing_mut().item_spacing.y = 1.0;
                                    ui.label(RichText::new(&weapon.name).size(13.5).color(theme::TEXT_PRIMARY).strong());
                                    ui.label(RichText::new(format!("{fitted}/5 mods · {pct}%")).size(11.5).color(theme::TEXT_MUTED));
                                });
                            });
                        });
                    if resp.response.interact(Sense::click()).clicked() {
                        state.select_upgrade_weapon(weapon.slot);
                    }
                    ui.add_space(4.0);
                }
            });
    });
}

fn draw_slots_panel(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    weapon: &GunsmithUpgradeWeapon,
    player_scrap: u32,
    layout: &GunsmithLayout,
    mut icons: Option<&mut IconCache>,
) -> (Option<GunsmithAction>, Option<GunsmithWeaponStats>) {
    let mut action = None;
    let mut preview_stats = None;
    let slot_btn_w = ((layout.upgrade_right_width - 32.0) / 5.0).max(60.0);

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        for kind in GunsmithSlot::ALL {
            let slot_state = find_slot(weapon, kind);
            let is_sel = state.selected_attach_slot == kind;
            let label = match slot_state {
                Some(s) if !s.supported => "N/A".to_string(),
                Some(s) => s.fitted_mod.as_ref().map(|m| m.name.clone()).unwrap_or_else(|| "Empty".to_string()),
                None => "Empty".to_string(),
            };
            let resp = Frame::NONE
                .fill(if is_sel { theme::ACCENT_SUBTLE } else { theme::CARD_BG })
                .inner_margin(Margin::same(6))
                .corner_radius(CornerRadius::same(5))
                .stroke(Stroke::new(1.0, if is_sel { theme::ACCENT } else { theme::BORDER }))
                .show(ui, |ui| {
                    ui.set_width((slot_btn_w - 12.0).max(48.0));
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        let title_color = if is_sel { theme::ACCENT_HOVER } else { theme::TEXT_MUTED };
                        ui.label(RichText::new(kind.label()).size(12.0).color(title_color).strong());
                        ui.add(egui::Label::new(RichText::new(label).size(11.5).color(theme::TEXT_PRIMARY)).truncate());
                    });
                });
            if resp.response.interact(Sense::click()).clicked() {
                state.select_attach_slot(kind);
            }
        }
    });

    ui.add_space(4.0);
    let scroll_h = (layout.upgrade_slots_height - 64.0).max(76.0);
    let active_slot = state.selected_attach_slot;

    ScrollArea::vertical()
        .id_salt("gunsmith_slot_options")
        .max_height(scroll_h)
        .show(ui, |ui| {
            let slot_scroll_w = (layout.upgrade_right_width - 14.0).max(200.0);
            ui.set_width(slot_scroll_w);
            ui.set_max_width(slot_scroll_w);
            let Some(slot_info) = find_slot(weapon, active_slot) else {
                ui.label("Slot unavailable.");
                return;
            };
            if !slot_info.supported {
                ui.label(RichText::new(format!("{} does not accept {} attachments.", weapon.name, active_slot.label())).size(13.5).color(theme::TEXT_MUTED));
                return;
            }

            let row_w = (slot_scroll_w - 16.0).max(160.0);
            if let Some(fitted) = &slot_info.fitted_mod {
                Frame::NONE
                    .fill(Color32::from_rgb(28, 38, 30))
                    .inner_margin(Margin::same(7))
                    .corner_radius(CornerRadius::same(5))
                    .stroke(Stroke::new(1.0, theme::SAFE))
                    .show(ui, |ui| {
                        ui.set_width(row_w);
                        ui.set_max_width(row_w);
                        ui.horizontal(|ui| {
                            draw_item_icon(ui, icons.as_deref_mut(), &fitted.id, &fitted.name, "#68a357", vec2(28.0, 28.0));
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 1.0;
                                ui.label(RichText::new(format!("Fitted: {}", fitted.name)).size(13.5).color(theme::SAFE_HOVER).strong());
                                if !fitted.summary.is_empty() {
                                    ui.label(RichText::new(&fitted.summary).size(11.5).color(theme::TEXT_MUTED));
                                }
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Remove").clicked() {
                                    action = Some(GunsmithAction::RemoveMod { weapon_slot: weapon.slot, attachment_id: fitted.id.clone() });
                                }
                            });
                        });
                    });
                ui.add_space(4.0);
            }

            if slot_info.options.is_empty() {
                ui.label(RichText::new("No attachment options available for this slot.").size(13.0).color(theme::TEXT_MUTED));
            } else {
                let mut any_hovered = false;
                for opt in &slot_info.options {
                    let is_fitted = slot_info.fitted_mod.as_ref().is_some_and(|m| m.id == opt.id);
                    let (opt_act, hovered) = draw_mod_option_row(ui, weapon.slot, opt, is_fitted, player_scrap, row_w, icons.as_deref_mut());
                    if hovered {
                        any_hovered = true;
                        state.hovered_mod = Some(opt.id.clone());
                        preview_stats = Some(opt.after_stats);
                    }
                    if opt_act.is_some() {
                        action = opt_act;
                    }
                    ui.add_space(4.0);
                }
                if !any_hovered {
                    state.hovered_mod = None;
                }
            }
        });

    (action, preview_stats)
}

fn draw_mod_option_row(
    ui: &mut egui::Ui,
    weapon_slot: usize,
    opt: &GunsmithModOption,
    is_fitted: bool,
    player_scrap: u32,
    row_w: f32,
    icons: Option<&mut IconCache>,
) -> (Option<GunsmithAction>, bool) {
    let mut action = None;
    let cost = if opt.owned_in_bag { 0 } else { opt.price_scrap };
    let can_afford = player_scrap >= cost;
    let can_fit = !is_fitted && opt.locked_reason.is_none() && can_afford;

    let resp = Frame::NONE
        .fill(theme::CARD_BG)
        .inner_margin(Margin::same(6))
        .corner_radius(CornerRadius::same(5))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, |ui| {
            ui.set_width(row_w);
            ui.set_max_width(row_w);
            ui.horizontal(|ui| {
                draw_item_icon(ui, icons, &opt.id, &opt.name, "#8b95a2", vec2(26.0, 26.0));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    let color = if opt.locked_reason.is_some() { theme::TEXT_DISABLED } else { theme::TEXT_PRIMARY };
                    ui.label(RichText::new(&opt.name).size(13.5).color(color).strong());
                    if !opt.summary.is_empty() {
                        ui.label(RichText::new(&opt.summary).size(11.5).color(theme::TEXT_MUTED));
                    }
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if is_fitted {
                        ui.add_enabled(false, egui::Button::new("Fitted"));
                    } else if let Some(reason) = &opt.locked_reason {
                        ui.add_enabled(false, egui::Button::new(reason)).on_hover_text(reason);
                    } else {
                        let mut btn = ui.add_enabled(can_fit, egui::Button::new("Fit"));
                        if !can_afford {
                            btn = btn.on_hover_text(format!("Need {cost} scrap metal"));
                        }
                        if btn.clicked() {
                            action = Some(GunsmithAction::FitMod { weapon_slot, attachment_id: opt.id.clone() });
                        }
                        if opt.owned_in_bag {
                            ui.label(RichText::new("In bag (Free)").size(12.0).color(theme::SAFE_HOVER));
                        } else {
                            let c = if can_afford { Color32::from_rgb(235, 200, 108) } else { theme::DANGER_HOVER };
                            draw_scrap_badge(ui, None, cost, 13.0, c);
                        }
                    }
                });
            });
        });

    (action, resp.response.hovered())
}

fn find_slot(weapon: &GunsmithUpgradeWeapon, kind: GunsmithSlot) -> Option<&GunsmithSlotState> {
    weapon.slots.iter().find(|s| s.slot == kind)
}

fn draw_stat_comparison(ui: &mut egui::Ui, base: &GunsmithWeaponStats, after: &GunsmithWeaponStats, width: f32) {
    Frame::NONE
        .fill(theme::CARD_EMPTY_BG)
        .inner_margin(Margin::same(8))
        .corner_radius(CornerRadius::same(6))
        .stroke(Stroke::new(1.0, theme::BORDER))
        .show(ui, |ui| {
            let panel_inner_w = (width - 18.0).max(200.0);
            ui.set_width(panel_inner_w);
            ui.set_max_width(panel_inner_w);
            ui.label(RichText::new("Stats (Before -> After)").size(12.5).color(theme::TEXT_MUTED).strong());
            ui.add_space(4.0);

            let col_w = ((panel_inner_w - 12.0) * 0.5).floor().max(96.0);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                ui.vertical(|col0| {
                    col0.set_width(col_w);
                    col0.set_max_width(col_w);
                    draw_stat_bar(col0, "Damage", base.damage, after.damage, 120.0, true, "", col_w);
                    draw_stat_bar(col0, "Kick", base.kick, after.kick, 3.0, false, "", col_w);
                    draw_stat_bar(col0, "Noise", base.noise_radius, after.noise_radius, 90.0, false, "m", col_w);
                });
                ui.vertical(|col1| {
                    col1.set_width(col_w);
                    col1.set_max_width(col_w);
                    draw_stat_bar(col1, "Spread", base.spread_deg, after.spread_deg, 10.0, false, "°", col_w);
                    draw_stat_bar(col1, "Mag", base.mag_size as f32, after.mag_size as f32, 60.0, true, "", col_w);
                    draw_stat_bar(col1, "Zoom", base.zoom_x, after.zoom_x, 6.0, true, "x", col_w);
                });
            });
        });
}

#[allow(clippy::too_many_arguments)]
fn draw_stat_bar(
    ui: &mut egui::Ui,
    label: &str,
    before: f32,
    after: f32,
    max_ref: f32,
    higher_is_better: bool,
    unit: &str,
    col_w: f32,
) {
    let diff = after - before;
    let changed = diff.abs() > 0.005;
    let improved = if higher_is_better { diff > 0.005 } else { diff < -0.005 };
    let val_color = if !changed { theme::TEXT_PRIMARY } else if improved { theme::SAFE_HOVER } else { theme::DANGER_HOVER };
    let fmt = |v: f32| {
        if (v - v.round()).abs() < 0.05 { format!("{:.0}{unit}", v) } else { format!("{v:.1}{unit}") }
    };
    let text = if changed { format!("{} -> {}", fmt(before), fmt(after)) } else { fmt(before) };

    let bar_w = (col_w - 4.0).max(80.0);
    ui.allocate_ui(vec2(bar_w, 16.0), |ui| {
        ui.set_width(bar_w);
        ui.set_max_width(bar_w);
        ui.horizontal(|ui| {
            ui.label(RichText::new(label).size(11.5).color(theme::TEXT_MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(text).size(11.5).color(val_color).strong());
            });
        });
    });
    let (rect, _) = ui.allocate_exact_size(vec2(bar_w, 6.0), Sense::hover());
    ui.painter().rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(20, 22, 26));
    let frac = (after / max_ref.max(1.0)).clamp(0.0, 1.0);
    let fill = Rect::from_min_size(rect.min, vec2(rect.width() * frac, rect.height()));
    let fill_color = if !changed { Color32::from_rgb(130, 150, 175) } else if improved { theme::SAFE } else { theme::DANGER };
    ui.painter().rect_filled(fill, CornerRadius::same(2), fill_color);
    ui.add_space(3.0);
}
