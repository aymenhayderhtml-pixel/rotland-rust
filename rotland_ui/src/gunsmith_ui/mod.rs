//! Gunsmith weapon shop UI (Buy, Sell, Repair, Upgrade) using plain view-model structs.

pub mod action;
pub mod draw_buy;
pub mod draw_repair;
pub mod draw_sell;
pub mod draw_upgrade;
pub mod screen;

#[cfg(test)]
mod tests;

pub use action::{GunsmithAction, GunsmithSlot};
pub use screen::{GunsmithLayout, GunsmithScreen, GunsmithTab, GunsmithUiState};

use engine::egui::{
    self, Color32, CornerRadius, Frame, Id, Key, Margin, Order, Rect, RichText, Sense,
    Stroke, StrokeKind, Vec2, pos2, vec2,
};

use crate::hud::IconCache;
use crate::menus::theme::{self, ButtonTone};

/// Plain view-model snapshot passed into the Gunsmith UI each frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithView {
    pub shop_title: String,
    pub subtitle: String,
    pub scrap_count: u32,
    pub buy_items: Vec<GunsmithBuyItem>,
    pub sell_items: Vec<GunsmithSellItem>,
    pub repair_items: Vec<GunsmithRepairItem>,
    pub upgrade_weapons: Vec<GunsmithUpgradeWeapon>,
}

/// One item offered for sale by the gunsmith.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GunsmithBuyItem {
    pub item_id: String,
    pub name: String,
    pub category: String,
    pub price_scrap: u32,
    pub stock: u32,
    pub color_hex: String,
    pub locked_camp_tier: Option<u8>,
    pub blocker_reason: Option<String>,
    pub summary: String,
}

/// One item in the player's bag that can be sold for scrap.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithSellItem {
    pub slot: usize,
    pub item_id: String,
    pub name: String,
    pub count: u32,
    pub scrap_each: u32,
    pub condition: Option<f32>,
    pub mod_count: usize,
    pub color_hex: String,
}

/// One weapon in the player's bag listed on the Repair tab.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithRepairItem {
    pub slot: usize,
    pub item_id: String,
    pub name: String,
    pub condition: f32,
    pub is_broken: bool,
    pub repair_cost_scrap: u32,
    pub color_hex: String,
}

/// Numeric stats used for before/after comparison bars on the Upgrade tab.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GunsmithWeaponStats {
    pub damage: f32,
    pub spread_deg: f32,
    pub kick: f32,
    pub mag_size: u32,
    pub noise_radius: f32,
    pub zoom_x: f32,
}

/// A currently fitted attachment on a weapon slot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GunsmithFittedMod {
    pub id: String,
    pub name: String,
    pub summary: String,
}

/// An attachment option that can be fitted into a weapon slot.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithModOption {
    pub id: String,
    pub name: String,
    pub slot: GunsmithSlot,
    pub price_scrap: u32,
    pub owned_in_bag: bool,
    pub locked_reason: Option<String>,
    pub after_stats: GunsmithWeaponStats,
    pub summary: String,
}

/// State of one of the 5 attachment slots on a weapon.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithSlotState {
    pub slot: GunsmithSlot,
    pub supported: bool,
    pub fitted_mod: Option<GunsmithFittedMod>,
    pub options: Vec<GunsmithModOption>,
}

/// One upgradable weapon in the player's bag.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GunsmithUpgradeWeapon {
    pub slot: usize,
    pub item_id: String,
    pub name: String,
    pub condition: f32,
    pub color_hex: String,
    pub base_stats: GunsmithWeaponStats,
    pub slots: [GunsmithSlotState; 5],
}

/// Draws the Gunsmith screen centered over a dimmed backdrop.
pub fn draw(
    ctx: &egui::Context,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    mut icons: Option<&mut IconCache>,
) -> Option<GunsmithAction> {
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        return Some(GunsmithAction::Close);
    }

    for key in [Key::Num1, Key::Num2, Key::Num3, Key::Num4] {
        if ctx.input(|i| i.key_pressed(key))
            && let Some(tab) = GunsmithTab::from_key(key)
        {
            state.switch_tab(tab);
        }
    }

    let screen_rect = ctx.content_rect();
    let layout = GunsmithLayout::compute(screen_rect);
    let scale = layout.scale;
    let mut action = None;

    theme::with_menu_style(ctx, scale, || {
        egui::Area::new(Id::unique("gunsmith_background"))
            .fixed_pos(screen_rect.min)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.painter().rect_filled(screen_rect, 0.0, theme::SETTINGS_DIM);
                ui.allocate_rect(screen_rect, Sense::click_and_drag());
            });

        let area_pos = pos2(
            screen_rect.min.x + ((screen_rect.width() - layout.panel_width) * 0.5).max(0.0),
            screen_rect.min.y + ((screen_rect.height() - layout.panel_height) * 0.5).max(0.0),
        );
        egui::Area::new(Id::unique("gunsmith_content"))
            .fixed_pos(area_pos)
            .order(Order::Foreground)
            .show(ctx, |ui| {
                let frame = Frame::NONE
                    .fill(theme::PANEL_BG)
                    .inner_margin(Margin::same(14))
                    .corner_radius(theme::radius(theme::CORNER_PANEL, scale))
                    .stroke(Stroke::new(2.0, theme::BORDER));

                let resp = frame.show(ui, |ui| {
                    ui.set_width(layout.inner_width);
                    ui.set_max_width(layout.inner_width);
                    ui.set_min_height(layout.inner_height);
                    ui.set_max_height(layout.inner_height);
                    ui.spacing_mut().item_spacing = vec2(8.0, 6.0);

                    draw_header(ui, state, view, &layout, icons.as_deref_mut());
                    theme::paint_divider(ui, layout.inner_width, scale);

                    ui.allocate_ui(vec2(layout.inner_width, layout.body_height), |ui| {
                        ui.set_min_height(layout.body_height);
                        ui.set_max_height(layout.body_height);
                        action = match state.tab {
                            GunsmithTab::Buy => {
                                draw_buy::draw_buy(ui, state, view, &layout, icons.as_deref_mut())
                            }
                            GunsmithTab::Sell => {
                                draw_sell::draw_sell(ui, state, view, &layout, icons.as_deref_mut())
                            }
                            GunsmithTab::Repair => {
                                draw_repair::draw_repair(ui, view, &layout, icons.as_deref_mut())
                            }
                            GunsmithTab::Upgrade => draw_upgrade::draw_upgrade(
                                ui,
                                state,
                                view,
                                &layout,
                                icons,
                            ),
                        };
                    });

                    theme::paint_divider(ui, layout.inner_width, scale);
                    if let Some(close_act) = draw_footer(ui, state, &layout) {
                        action = Some(close_act);
                    }
                });

                theme::paint_top_accent(ui, resp.response.rect, theme::ACCENT, scale);
                state.last_panel_rect = Some(resp.response.rect);
            });
    });

    action
}

fn draw_header(
    ui: &mut egui::Ui,
    state: &mut GunsmithUiState,
    view: &GunsmithView,
    layout: &GunsmithLayout,
    icons: Option<&mut IconCache>,
) {
    let scale = layout.scale;
    let tab_w = (104.0 * scale).min((layout.inner_width * 0.42) / 4.0);
    let title = if view.shop_title.is_empty() { "Gunsmith" } else { &view.shop_title };

    ui.allocate_ui(vec2(layout.inner_width, layout.header_height), |ui| {
        ui.set_min_height(layout.header_height);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(
                    RichText::new(title)
                        .size((20.0 * scale).clamp(16.0, 28.0))
                        .color(theme::TEXT_PRIMARY)
                        .strong(),
                );
                if !view.subtitle.is_empty() {
                    ui.label(
                        RichText::new(&view.subtitle)
                            .size((11.5 * scale).clamp(10.0, 15.0))
                            .color(theme::TEXT_MUTED),
                    );
                }
            });

            ui.add_space(10.0 * scale);
            for tab in GunsmithTab::ALL {
                if theme::tab_button(ui, tab.label(), tab_w, scale, state.tab == tab).clicked() {
                    state.switch_tab(tab);
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_scrap_badge(
                    ui,
                    icons,
                    view.scrap_count,
                    (17.0 * scale).clamp(14.0, 24.0),
                    Color32::from_rgb(235, 200, 108),
                );
                ui.label(
                    RichText::new("SCRAP")
                        .size((11.5 * scale).clamp(10.0, 14.0))
                        .color(theme::TEXT_MUTED)
                        .strong(),
                );
            });
        });
    });
}

fn draw_footer(
    ui: &mut egui::Ui,
    state: &GunsmithUiState,
    layout: &GunsmithLayout,
) -> Option<GunsmithAction> {
    let mut action = None;
    let scale = layout.scale;
    let btn_size = vec2((136.0 * scale).clamp(112.0, 180.0), (34.0 * scale).clamp(30.0, 42.0));

    ui.allocate_ui(vec2(layout.inner_width, layout.footer_height), |ui| {
        ui.set_min_height(layout.footer_height);
        ui.horizontal(|ui| {
            if theme::menu_button(ui, "Close (Esc)", btn_size, scale, ButtonTone::Default, true).clicked() {
                action = Some(GunsmithAction::Close);
            }
            ui.add_space(10.0);
            let (msg, color) = if !state.status().is_empty() {
                (state.status(), theme::SAFE_HOVER)
            } else {
                ("Keys 1-4 switch tabs  ·  Esc closes", theme::TEXT_MUTED)
            };
            ui.label(RichText::new(msg).size((13.0 * scale).clamp(11.0, 16.0)).color(color));
        });
    });

    action
}

fn parse_hex_color(hex: &str) -> Color32 {
    let s = hex.trim_start_matches('#');
    if s.len() == 6
        && let Ok(r) = u8::from_str_radix(&s[0..2], 16)
        && let Ok(g) = u8::from_str_radix(&s[2..4], 16)
        && let Ok(b) = u8::from_str_radix(&s[4..6], 16)
    {
        Color32::from_rgb(r, g, b)
    } else {
        Color32::from_rgb(100, 108, 118)
    }
}

/// Draws a PNG icon from `IconCache` when available, or a rounded color swatch fallback.
pub(super) fn draw_item_icon(
    ui: &mut egui::Ui,
    icons: Option<&mut IconCache>,
    item_id: &str,
    name: &str,
    color_hex: &str,
    size: Vec2,
) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    if !ui.is_rect_visible(rect) {
        return;
    }
    if let Some(tid) = icons.and_then(|c| c.icon_or_label(ui.ctx(), Some(item_id), name)) {
        ui.painter().image(tid, rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    } else {
        let swatch = if color_hex.is_empty() { Color32::from_rgb(100, 108, 118) } else { parse_hex_color(color_hex) };
        ui.painter().rect(rect, CornerRadius::same(4), swatch, Stroke::new(1.0, theme::BORDER), StrokeKind::Inside);
    }
}

/// Draws the `scrap_metal` icon paired with a scrap count label.
pub(super) fn draw_scrap_badge(
    ui: &mut egui::Ui,
    icons: Option<&mut IconCache>,
    amount: u32,
    font_size: f32,
    text_color: Color32,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        let icon_sz = (font_size + 3.0).clamp(14.0, 24.0);
        draw_item_icon(ui, icons, "scrap_metal", "Scrap Metal", "#8c939b", vec2(icon_sz, icon_sz));
        ui.label(RichText::new(amount.to_string()).size(font_size).color(text_color).strong());
    });
}
