//! Unit tests for menu state machine transitions and settings serialization.
//! egui drawing is deliberately not tested here.

use std::time::{SystemTime, UNIX_EPOCH};

use super::*;
use crate::settings::{
    settings_path, Settings, MAX_FOV, MAX_RENDER_SCALE, MAX_SENSITIVITY, MIN_FOV, MIN_RENDER_SCALE,
    MIN_SENSITIVITY,
};

#[test]
fn test_escape_transitions() {
    let mut menus = Menus::new(Settings::default());

    // 1. In main menu, escape does not close the main menu
    assert_eq!(*menus.screen(), Screen::Main);
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::Main);
    assert!(menus.is_open());
    assert!(menus.wants_cursor());

    // 2. In game (Screen::None), escape transitions to Paused
    menus.set_screen(Screen::None);
    assert!(!menus.is_open());
    assert!(!menus.wants_cursor());
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::Paused);
    assert!(menus.is_open());
    assert!(menus.wants_cursor());

    // 3. In paused, escape resumes (Screen::None)
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::None);
    assert!(!menus.is_open());
    assert!(!menus.wants_cursor());

    // 4. In slot picker, escape returns to Main menu
    menus.set_screen(Screen::SlotPicker {
        mode: SlotMode::NewGame,
    });
    assert!(menus.is_open());
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::Main);
}

#[test]
fn test_continue_disabled_with_all_slots_empty() {
    let mut menus = Menus::new(Settings::default());
    let empty_slots = [
        SlotSummary::default(),
        SlotSummary::default(),
        SlotSummary::default(),
    ];

    // All empty -> any_slot_filled is false
    assert!(!Menus::any_slot_filled(&empty_slots));

    // Pressing continue with empty slots does not transition
    let transitioned = menus.press_continue(&empty_slots);
    assert!(!transitioned);
    assert_eq!(*menus.screen(), Screen::Main);

    // One slot filled -> any_slot_filled is true
    let mut some_filled = empty_slots.clone();
    some_filled[1].state = SlotState::Filled;
    some_filled[1].day = 5;
    some_filled[1].play_time_s = 7200.0;
    some_filled[1].difficulty = "Hard".to_string();

    assert!(Menus::any_slot_filled(&some_filled));
    let transitioned = menus.press_continue(&some_filled);
    assert!(transitioned);
    assert_eq!(
        *menus.screen(),
        Screen::SlotPicker {
            mode: SlotMode::Continue
        }
    );
}

#[test]
fn test_delete_confirm_flow() {
    let mut menus = Menus::new(Settings::default());
    menus.set_screen(Screen::SlotPicker {
        mode: SlotMode::Continue,
    });

    assert!(!menus.is_confirm_delete_open());
    assert_eq!(menus.pending_delete, None);

    // 1. Request delete slot 2 -> confirmation modal is active
    menus.request_delete_slot(2);
    assert!(menus.is_confirm_delete_open());
    assert_eq!(menus.pending_delete, Some(2));

    // 2. User cancels -> confirmation closes, no action emitted
    menus.cancel_delete();
    assert!(!menus.is_confirm_delete_open());
    assert_eq!(menus.pending_delete, None);
    assert_eq!(
        *menus.screen(),
        Screen::SlotPicker {
            mode: SlotMode::Continue
        }
    );

    // 3. Request delete slot 3, then press Escape -> dismisses confirm dialog only
    menus.request_delete_slot(3);
    assert!(menus.is_confirm_delete_open());
    menus.on_escape();
    assert!(!menus.is_confirm_delete_open());
    assert_eq!(menus.pending_delete, None);
    assert_eq!(
        *menus.screen(),
        Screen::SlotPicker {
            mode: SlotMode::Continue
        }
    );

    // 4. Request delete slot 1, then confirm -> emits DeleteSlot { slot: 1 }
    menus.request_delete_slot(1);
    assert!(menus.is_confirm_delete_open());
    let action = menus.confirm_delete();
    assert_eq!(action, Some(MenuAction::DeleteSlot { slot: 1 }));
    assert!(!menus.is_confirm_delete_open());
    assert_eq!(menus.pending_delete, None);
}

#[test]
fn test_settings_back_navigation() {
    let mut menus = Menus::new(Settings::default());

    // 1. From Main menu -> Settings -> Back (press_back) -> Main menu
    assert_eq!(*menus.screen(), Screen::Main);
    menus.press_settings();
    assert_eq!(
        *menus.screen(),
        Screen::Settings {
            from: Box::new(Screen::Main)
        }
    );
    menus.press_back();
    assert_eq!(*menus.screen(), Screen::Main);

    // 2. From Main menu -> Settings -> Back via on_escape -> Main menu
    menus.press_settings();
    assert_eq!(
        *menus.screen(),
        Screen::Settings {
            from: Box::new(Screen::Main)
        }
    );
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::Main);

    // 3. From Paused -> Settings -> Back via on_escape -> Paused
    menus.set_screen(Screen::Paused);
    menus.press_settings();
    assert_eq!(
        *menus.screen(),
        Screen::Settings {
            from: Box::new(Screen::Paused)
        }
    );
    menus.on_escape();
    assert_eq!(*menus.screen(), Screen::Paused);

    // 4. From Paused -> Settings -> press_back -> Paused
    menus.press_settings();
    menus.press_back();
    assert_eq!(*menus.screen(), Screen::Paused);
}

#[test]
fn test_settings_round_trip_through_temp_file() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(424242);
    let temp_file = std::env::temp_dir().join(format!("rotland_menu_settings_test_{nanos}.json"));

    let _ = std::fs::remove_file(&temp_file);

    let mut original = Settings {
        mouse_sensitivity: 2.4,
        invert_y: true,
        field_of_view: 88.0,
        render_scale: 0.85,
        vsync: true,
        master_volume: 0.75,
        music_volume: 0.45,
        sfx_volume: 0.95,
        show_fps: false,
        graphics_preset: "high".to_string(),
        key_bindings: Default::default(),
    };
    original.clamp();

    // 1. Save settings to unique temp file
    let saved = original.save(&temp_file);
    assert!(saved, "Saving settings to temp file should succeed");

    // 2. Load settings back and verify match
    let loaded = Settings::load(&temp_file);
    assert_eq!(loaded, original, "Loaded settings must match saved settings");

    // 3. Cleanup temp file
    let _ = std::fs::remove_file(&temp_file);

    // 4. Loading non-existent file yields default settings
    let non_existent = Settings::load(&temp_file);
    assert_eq!(non_existent, Settings::default());

    // 5. Corrupted JSON yields default settings without crashing
    let corrupt_file = std::env::temp_dir().join(format!("rotland_corrupt_test_{nanos}.json"));
    let _ = std::fs::write(&corrupt_file, "{ not valid json @#$ }");
    let corrupt_loaded = Settings::load(&corrupt_file);
    assert_eq!(corrupt_loaded, Settings::default());
    let _ = std::fs::remove_file(&corrupt_file);
}

#[test]
fn test_settings_clamping() {
    let mut s = Settings {
        mouse_sensitivity: 99.0,
        invert_y: false,
        field_of_view: 200.0,
        render_scale: 5.0,
        vsync: false,
        master_volume: -2.0,
        music_volume: 3.0,
        sfx_volume: 0.5,
        show_fps: true,
        graphics_preset: "invalid_preset_name".to_string(),
        key_bindings: Default::default(),
    };
    s.clamp();
    assert_eq!(s.mouse_sensitivity, MAX_SENSITIVITY);
    assert_eq!(s.field_of_view, MAX_FOV);
    assert_eq!(s.render_scale, MAX_RENDER_SCALE);
    assert_eq!(s.master_volume, 0.0);
    assert_eq!(s.music_volume, 1.0);
    assert_eq!(s.sfx_volume, 0.5);
    assert_eq!(s.graphics_preset, "medium");

    // Test lower bounds
    let mut low = Settings {
        mouse_sensitivity: 0.01,
        invert_y: false,
        field_of_view: 10.0,
        render_scale: 0.1,
        vsync: false,
        master_volume: 0.0,
        music_volume: 0.0,
        sfx_volume: 0.0,
        show_fps: true,
        graphics_preset: "LOW".to_string(),
        key_bindings: Default::default(),
    };
    low.clamp();
    assert_eq!(low.mouse_sensitivity, MIN_SENSITIVITY);
    assert_eq!(low.field_of_view, MIN_FOV);
    assert_eq!(low.render_scale, MIN_RENDER_SCALE);
    assert_eq!(low.graphics_preset, "LOW");
}

#[test]
fn test_settings_path_structure() {
    let p = settings_path();
    assert!(p.ends_with("rotland-rust/settings.json"));
}

#[test]
fn test_slot_formatting_helpers() {
    assert_eq!(menus_draw::format_play_time(0.0), "0h 00m");
    assert_eq!(menus_draw::format_play_time(59.0), "0h 00m");
    assert_eq!(menus_draw::format_play_time(60.0), "0h 01m");
    assert_eq!(menus_draw::format_play_time(3600.0), "1h 00m");
    assert_eq!(menus_draw::format_play_time(3665.0), "1h 01m");
    assert_eq!(menus_draw::format_play_time(7325.0), "2h 02m");

    assert_eq!(menus_draw::format_saved_at(""), "");
    assert_eq!(
        menus_draw::format_saved_at("2026-11-16T08:15:00Z"),
        " · saved 2026-11-16 08:15"
    );
}
