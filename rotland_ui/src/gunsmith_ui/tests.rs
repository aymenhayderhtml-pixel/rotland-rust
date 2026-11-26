//! Unit tests for Gunsmith UI state transitions, keyboard shortcuts, and responsive layout bounds.

use engine::egui::{self, Event, Key, Modifiers, RawInput, Rect, pos2, vec2};

use crate::hud::IconCache;

use super::action::{GunsmithAction, GunsmithSlot};
use super::screen::{GunsmithLayout, GunsmithScreen, GunsmithTab, GunsmithUiState};
use super::{
    GunsmithBuyItem, GunsmithFittedMod, GunsmithModOption, GunsmithRepairItem, GunsmithSellItem,
    GunsmithSlotState, GunsmithUpgradeWeapon, GunsmithView, GunsmithWeaponStats, draw,
};

fn sample_view() -> GunsmithView {
    let base_stats = GunsmithWeaponStats {
        damage: 35.0,
        spread_deg: 3.5,
        kick: 1.6,
        mag_size: 30,
        noise_radius: 80.0,
        zoom_x: 1.0,
    };
    let scoped_stats = GunsmithWeaponStats {
        spread_deg: 2.2,
        zoom_x: 4.0,
        ..base_stats
    };
    let suppressed_stats = GunsmithWeaponStats {
        noise_radius: 25.0,
        kick: 1.3,
        ..base_stats
    };

    GunsmithView {
        shop_title: "Camp Gunsmith".to_string(),
        subtitle: "Camp Tier 2".to_string(),
        scrap_count: 85,
        buy_items: vec![
            GunsmithBuyItem {
                item_id: "pistol".into(),
                name: "Pistol".into(),
                category: "Weapon".into(),
                price_scrap: 25,
                stock: 1,
                color_hex: "#2b2b2b".into(),
                locked_camp_tier: None,
                blocker_reason: None,
                summary: "30 dmg · 12 mag · 9mm".into(),
            },
            GunsmithBuyItem {
                item_id: "rifle_rounds".into(),
                name: "Rifle Rounds".into(),
                category: "Ammo".into(),
                price_scrap: 6,
                stock: 20,
                color_hex: "#a08040".into(),
                locked_camp_tier: None,
                blocker_reason: None,
                summary: "High-velocity rifle ammo".into(),
            },
            GunsmithBuyItem {
                item_id: "ak47".into(),
                name: "AK-47".into(),
                category: "Weapon".into(),
                price_scrap: 90,
                stock: 1,
                color_hex: "#5a4a2e".into(),
                locked_camp_tier: Some(3),
                blocker_reason: Some("Requires Camp tier 3".into()),
                summary: "35 dmg · 30 mag · 7.62".into(),
            },
            GunsmithBuyItem {
                item_id: "grenade".into(),
                name: "Frag Grenade".into(),
                category: "Explosive".into(),
                price_scrap: 18,
                stock: 4,
                color_hex: "#3d4a2f".into(),
                locked_camp_tier: None,
                blocker_reason: None,
                summary: "150 blast dmg · 6m radius".into(),
            },
            GunsmithBuyItem {
                item_id: "crossbow_bolt".into(),
                name: "Crossbow Bolt".into(),
                category: "Ammo".into(),
                price_scrap: 3,
                stock: 12,
                color_hex: "#5c4033".into(),
                locked_camp_tier: None,
                blocker_reason: None,
                summary: "Quiet reusable bolt".into(),
            },
        ],
        sell_items: vec![
            GunsmithSellItem {
                slot: 1,
                item_id: "pistol".into(),
                name: "Pistol".into(),
                count: 1,
                scrap_each: 12,
                condition: Some(0.45),
                mod_count: 1,
                color_hex: "#2b2b2b".into(),
            },
            GunsmithSellItem {
                slot: 4,
                item_id: "pistol_ammo".into(),
                name: "Pistol Ammo".into(),
                count: 24,
                scrap_each: 1,
                condition: None,
                mod_count: 0,
                color_hex: "#908040".into(),
            },
        ],
        repair_items: vec![
            GunsmithRepairItem {
                slot: 0,
                item_id: "pistol".into(),
                name: "Pistol".into(),
                condition: 0.15,
                is_broken: false,
                repair_cost_scrap: 14,
                color_hex: "#2b2b2b".into(),
            },
            GunsmithRepairItem {
                slot: 2,
                item_id: "shotgun".into(),
                name: "Shotgun".into(),
                condition: 0.0,
                is_broken: true,
                repair_cost_scrap: 24,
                color_hex: "#4a3a2a".into(),
            },
        ],
        upgrade_weapons: vec![GunsmithUpgradeWeapon {
            slot: 3,
            item_id: "ak47".into(),
            name: "AK-47".into(),
            condition: 0.85,
            color_hex: "#5a4a2e".into(),
            base_stats,
            slots: [
                GunsmithSlotState {
                    slot: GunsmithSlot::Optic,
                    supported: true,
                    fitted_mod: None,
                    options: vec![GunsmithModOption {
                        id: "scope_4x".into(),
                        name: "4x Scope".into(),
                        slot: GunsmithSlot::Optic,
                        price_scrap: 30,
                        owned_in_bag: false,
                        locked_reason: None,
                        after_stats: scoped_stats,
                        summary: "4.0x zoom · tighter spread".into(),
                    }],
                },
                GunsmithSlotState {
                    slot: GunsmithSlot::Muzzle,
                    supported: true,
                    fitted_mod: Some(GunsmithFittedMod {
                        id: "suppressor".into(),
                        name: "Suppressor".into(),
                        summary: "-68% noise · less kick".into(),
                    }),
                    options: vec![GunsmithModOption {
                        id: "suppressor".into(),
                        name: "Suppressor".into(),
                        slot: GunsmithSlot::Muzzle,
                        price_scrap: 25,
                        owned_in_bag: false,
                        locked_reason: None,
                        after_stats: suppressed_stats,
                        summary: "-68% noise · less kick".into(),
                    }],
                },
                GunsmithSlotState {
                    slot: GunsmithSlot::Magazine,
                    supported: true,
                    fitted_mod: None,
                    options: vec![],
                },
                GunsmithSlotState {
                    slot: GunsmithSlot::Grip,
                    supported: true,
                    fitted_mod: None,
                    options: vec![],
                },
                GunsmithSlotState {
                    slot: GunsmithSlot::Stock,
                    supported: false,
                    fitted_mod: None,
                    options: vec![],
                },
            ],
        }],
    }
}

#[test]
fn tab_names_keys_and_state_transitions() {
    assert_eq!(GunsmithTab::parse_name("buy"), Some(GunsmithTab::Buy));
    assert_eq!(GunsmithTab::parse_name("SELL"), Some(GunsmithTab::Sell));
    assert_eq!(GunsmithTab::parse_name("Repair"), Some(GunsmithTab::Repair));
    assert_eq!(GunsmithTab::parse_name("upgrade"), Some(GunsmithTab::Upgrade));
    assert_eq!(GunsmithTab::parse_name("unknown"), None);

    assert_eq!(GunsmithTab::Buy.as_str(), "buy");
    assert_eq!(GunsmithSlot::Optic.as_str(), "optic");

    let mut screen = GunsmithScreen::new(GunsmithTab::Buy);
    assert_eq!(screen.state.tab(), GunsmithTab::Buy);

    screen.state.select_buy("pistol");
    screen.state.step_up_buy(5);
    screen.state.set_status("Bought 2 Pistol");
    assert_eq!(screen.state.buy_count, 2);
    assert_eq!(screen.state.status(), "Bought 2 Pistol");

    // Switching tabs via key 2 clears transient selection and status
    assert_eq!(screen.handle_key(Key::Num2), None);
    assert_eq!(screen.state.tab(), GunsmithTab::Sell);
    assert_eq!(screen.state.selected_buy, None);
    assert_eq!(screen.state.buy_count, 1);
    assert!(screen.state.status().is_empty());

    assert_eq!(screen.handle_key(Key::Num3), None);
    assert_eq!(screen.state.tab(), GunsmithTab::Repair);

    assert_eq!(screen.handle_key(Key::Num4), None);
    assert_eq!(screen.state.tab(), GunsmithTab::Upgrade);

    assert_eq!(screen.handle_key(Key::Num1), None);
    assert_eq!(screen.state.tab(), GunsmithTab::Buy);

    // Escape returns Close action
    assert_eq!(screen.handle_key(Key::Escape), Some(GunsmithAction::Close));

    // Sell & upgrade state helpers
    screen.state.select_sell(4);
    screen.state.step_up_sell(10);
    assert_eq!(screen.state.sell_count, 2);
    screen.state.step_down_sell();
    assert_eq!(screen.state.sell_count, 1);
    screen.state.clamp_sell(0);
    assert_eq!(screen.state.sell_count, 0);

    screen.state.step_down_buy();
    assert_eq!(screen.state.buy_count, 1);
    screen.state.clear_status();
}

#[test]
fn layout_fits_1920x986_and_1280x720_and_1024x600() {
    for (w, h) in [(1920.0, 986.0), (1280.0, 720.0), (1024.0, 600.0)] {
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h));
        let layout = GunsmithLayout::compute(rect);
        assert!(
            layout.fits_in(vec2(w, h)),
            "layout must fit inside {w}x{h}: {layout:?}"
        );
        assert!(layout.body_height >= 180.0);
        assert!(layout.buy_columns >= 2);
        assert!(layout.upgrade_left_width >= 200.0);
        assert!(layout.upgrade_right_width >= 220.0);
        assert!(layout.upgrade_slots_height >= 120.0);
        assert!(layout.upgrade_stats_height >= 118.0);
    }
}

#[test]
fn headless_draw_all_tabs_fit_1920x986_and_1280x720_without_overflow() {
    let view = sample_view();
    let mut icons = IconCache::new();

    for (w, h) in [(1920.0, 986.0), (1280.0, 720.0)] {
        for tab in GunsmithTab::ALL {
            let ctx = egui::Context::default();
            let mut screen = GunsmithScreen::new(tab);
            screen.state.select_buy("rifle_rounds");
            screen.state.select_sell(4);
            screen.state.select_upgrade_weapon(3);
            screen.state.select_attach_slot(GunsmithSlot::Muzzle);

            let raw = RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(w, h))),
                ..Default::default()
            };

            let mut out = ctx.run_pass(raw, |ctx| {
                let act = screen.draw(ctx, &view, Some(&mut icons));
                assert_eq!(act, None);
            });
            out.textures_delta.clear();

            let panel_rect = screen
                .state
                .last_panel_rect()
                .expect("panel rect must be recorded");
            assert!(
                panel_rect.min.x >= 0.0
                    && panel_rect.min.y >= 0.0
                    && panel_rect.max.x <= w + 0.5
                    && panel_rect.max.y <= h + 0.5,
                "tab {tab:?} overflowed {w}x{h}: panel_rect={panel_rect:?}"
            );
            assert!(panel_rect.width() > 300.0);
            assert!(panel_rect.height() > 300.0);
        }
    }
}

#[test]
fn headless_draw_handles_key_shortcuts_and_empty_view() {
    let ctx = egui::Context::default();
    let empty = GunsmithView::default();
    let mut state = GunsmithUiState::new(GunsmithTab::Buy);

    // Pressing Num4 switches to Upgrade tab
    let raw_tab4 = RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 720.0))),
        events: vec![Event::Key {
            key: Key::Num4,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let mut out1 = ctx.run_pass(raw_tab4, |ctx| {
        let act = draw(ctx, &mut state, &empty, None);
        assert_eq!(act, None);
    });
    out1.textures_delta.clear();
    assert_eq!(state.tab(), GunsmithTab::Upgrade);

    // Pressing Escape emits GunsmithAction::Close
    let raw_esc = RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 720.0))),
        events: vec![Event::Key {
            key: Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Modifiers::NONE,
        }],
        ..Default::default()
    };
    let mut out2 = ctx.run_pass(raw_esc, |ctx| {
        let act = draw(ctx, &mut state, &empty, None);
        assert_eq!(act, Some(GunsmithAction::Close));
    });
    out2.textures_delta.clear();
}
