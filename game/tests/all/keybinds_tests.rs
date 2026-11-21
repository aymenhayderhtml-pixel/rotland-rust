//! Integration tests for key rebinding, conflict detection/swapping, prompts, and settings round-trip.
//! Port of Godot `scripts/Core/KeyBindings.cs` and related test specifications.

use game::{InputBindings, KeyBindings, KEYBINDS_ALL_ACTIONS, KEYBINDS_SHIFT_MASK};
use winit::keyboard::KeyCode;

#[test]
fn keybinds_defaults_match_godot_project() {
    let kb = KeyBindings::default();

    // Exactly 31 bindable actions defined in Godot KeyBindings.cs
    assert_eq!(KEYBINDS_ALL_ACTIONS.len(), 31);

    // Hand-traced default codes from project.godot:
    assert_eq!(kb.code_of("move_forward"), 87); // W
    assert_eq!(kb.code_of("move_back"), 83); // S
    assert_eq!(kb.code_of("move_left"), 65); // A
    assert_eq!(kb.code_of("move_right"), 68); // D
    assert_eq!(kb.code_of("sprint"), 4194325); // Shift
    assert_eq!(kb.code_of("jump"), 32); // Space
    assert_eq!(kb.code_of("crouch"), 67); // C
    assert_eq!(kb.code_of("interact"), 69); // E
    assert_eq!(kb.code_of("reload"), 82); // R
    assert_eq!(kb.code_of("toggle_camera"), 86); // V
    assert_eq!(kb.code_of("quick_save"), 4194336); // F5
    assert_eq!(kb.code_of("quick_load"), 4194340); // F9
    assert_eq!(kb.code_of("slot_1"), 49); // 1
    assert_eq!(kb.code_of("slot_2"), 50); // 2
    assert_eq!(kb.code_of("slot_3"), 51); // 3
    assert_eq!(kb.code_of("slot_4"), 52); // 4
    assert_eq!(kb.code_of("slot_5"), 53); // 5
    assert_eq!(kb.code_of("slot_6"), 54); // 6
    assert_eq!(kb.code_of("inventory"), 4194306); // Tab
    assert_eq!(kb.code_of("toggle_skills"), 76); // L
    assert_eq!(kb.code_of("toggle_bounties"), 74); // J
    assert_eq!(kb.code_of("toggle_map"), 77); // M
    assert_eq!(kb.code_of("toggle_profile"), 80); // P
    assert_eq!(kb.code_of("hub_prev"), 81); // Q
    assert_eq!(kb.code_of("hub_next"), 69); // E
    assert_eq!(kb.code_of("dog_stay"), 71); // G
    assert_eq!(kb.code_of("dog_home"), 71 | KEYBINDS_SHIFT_MASK); // Shift+G (33554503)
    assert_eq!(kb.code_of("look_left"), 4194319); // Left arrow
    assert_eq!(kb.code_of("look_right"), 4194321); // Right arrow
    assert_eq!(kb.code_of("look_up"), 4194320); // Up arrow
    assert_eq!(kb.code_of("look_down"), 4194322); // Down arrow

    // Key names:
    assert_eq!(kb.key_name("jump"), "Space");
    assert_eq!(kb.key_name("interact"), "E");
    assert_eq!(kb.key_name("dog_home"), "Shift+G");
    assert_eq!(kb.key_name("inventory"), "Tab");
    assert_eq!(kb.key_name("quick_save"), "F5");
    assert_eq!(kb.key_name("quick_load"), "F9");
    assert_eq!(kb.key_name("look_left"), "Left");

    // Initially no overrides
    assert!(kb.overrides().is_empty());
}

#[test]
fn keybinds_rebind_changes_action() {
    let mut kb = KeyBindings::default();

    // Rebind Jump to J (ASCII 74)
    // Matches Godot test RebindChangesAction.cs
    kb.set("jump", 74);
    assert_eq!(kb.code_of("jump"), 74);
    assert_eq!(kb.key_name("jump"), "J");
    assert_eq!(kb.keycode_of("jump"), Some(KeyCode::KeyJ));

    // Rebind Jump to Shift+G
    kb.set_key("jump", KeyCode::KeyG, true);
    assert_eq!(kb.code_of("jump"), 71 | KEYBINDS_SHIFT_MASK);
    assert_eq!(kb.key_name("jump"), "Shift+G");
    assert_eq!(kb.key_of("jump"), Some((KeyCode::KeyG, true)));
}

#[test]
fn keybinds_conflict_detects_clash_and_swaps() {
    let mut kb = KeyBindings::default();

    // Key E (code 69) is used by "interact"
    // Matches Godot test RebindConflictSwap.cs
    let clash = kb.conflict_with("jump", 69);
    assert_eq!(clash, Some("interact"));

    let jump_before = kb.code_of("jump"); // 32 (Space)
    assert_eq!(jump_before, 32);

    kb.swap("jump", "interact");
    assert_eq!(kb.code_of("jump"), 69); // E
    assert_eq!(kb.code_of("interact"), jump_before); // Space
    assert_eq!(kb.key_name("jump"), "E");
    assert_eq!(kb.key_name("interact"), "Space");
}

#[test]
fn keybinds_reset_all_restores_defaults() {
    let mut kb = KeyBindings::default();
    kb.set("jump", 74);
    kb.set("crouch", 90);
    assert_eq!(kb.overrides().len(), 2);

    kb.reset_all();
    assert_eq!(kb.code_of("jump"), 32);
    assert_eq!(kb.code_of("crouch"), 67);
    assert!(kb.overrides().is_empty());
}

#[test]
fn keybinds_prompt_and_expand_format_correctly() {
    let mut kb = KeyBindings::default();

    // Matches Godot test PromptFollowsRebinding.cs
    assert_eq!(kb.prompt("interact", "Search crate"), "[E] Search crate");
    assert_eq!(
        kb.expand("{interact} to search, {inventory} to open bag"),
        "E to search, Tab to open bag"
    );

    // Rebind interact to F (ASCII 70)
    kb.set("interact", 70);
    assert_eq!(kb.prompt("interact", "Search crate"), "[F] Search crate");
    assert_eq!(
        kb.expand("{interact} to search, {inventory} to open bag"),
        "F to search, Tab to open bag"
    );
}

#[test]
fn keybinds_save_and_load_round_trip() {
    let mut kb = KeyBindings::default();

    // Rebind crouch to Z (ASCII 90)
    // Matches Godot test RebindSavedInSettings.cs
    kb.set("crouch", 90);

    let overrides = kb.overrides();
    assert_eq!(overrides.len(), 1);
    assert_eq!(overrides.get("crouch").copied(), Some(90));

    let json = kb.to_settings();
    let obj = json.as_object().expect("json object");
    assert_eq!(obj.len(), 1);
    assert_eq!(obj["crouch"].as_u64(), Some(90));

    // Load back into default KeyBindings
    let mut loaded = KeyBindings::default();
    loaded.from_settings(&json);
    assert_eq!(loaded.code_of("crouch"), 90);
    assert_eq!(loaded.code_of("jump"), 32); // Jump keeps default

    // Non-bindable actions in JSON are dropped
    let noisy_json = serde_json::json!({
        "not_an_action": 12,
        "jump": 90
    });
    let mut filtered = KeyBindings::default();
    filtered.from_settings(&noisy_json);
    assert_eq!(filtered.code_of("jump"), 90);
    assert_eq!(filtered.overrides().len(), 1);
}

#[test]
fn keybinds_input_bindings_conversion() {
    let mut kb = KeyBindings::default();
    kb.set("crouch", 90); // Z

    let input_bindings = InputBindings::from_keybindings(&kb);
    assert_eq!(input_bindings.move_forward, KeyCode::KeyW);
    assert_eq!(input_bindings.move_back, KeyCode::KeyS);
    assert_eq!(input_bindings.move_left, KeyCode::KeyA);
    assert_eq!(input_bindings.move_right, KeyCode::KeyD);
    assert_eq!(input_bindings.jump, KeyCode::Space);
    assert_eq!(input_bindings.sprint, KeyCode::ShiftLeft);
    assert_eq!(input_bindings.crouch, KeyCode::KeyZ);
    assert_eq!(input_bindings.toggle_camera, KeyCode::KeyV);
    assert_eq!(input_bindings.look_left, KeyCode::ArrowLeft);
    assert_eq!(input_bindings.look_right, KeyCode::ArrowRight);
    assert_eq!(input_bindings.look_up, KeyCode::ArrowUp);
    assert_eq!(input_bindings.look_down, KeyCode::ArrowDown);
}
