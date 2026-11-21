//! Input gathering, key actions, hotbar selection, and hand equipment changes.

use engine::Input;
use game::Actions;
use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use super::state::{
    weapon_model_name, ActorsState, KeyMap, PlaySide, RenderState, SessionState, UiState,
    WorldState,
};
use crate::{hub_ui, play};

#[path = "../build_mode/mod.rs"]
pub mod build_mode;

/// Result of input processing for the frame.
pub enum InputOutcome {
    /// A menu is open, so game simulation is paused for this frame.
    MenuOpen,
    /// Player actions to feed into actors and simulation updates.
    Active {
        actions: Actions,
        attack: bool,
        dead: bool,
    },
}

/// Puts a weapon in hand (or empties it): the swing uses its numbers, the model shows in the hand.
pub fn apply_hand(
    change: play::HandChange,
    render: &mut RenderState,
    play: &mut PlaySide,
    actors: &mut ActorsState,
    world: &WorldState,
) {
    let def = match change {
        play::HandChange::Weapon(def) => def,
        play::HandChange::ToggleFlashlight => {
            render.flashlight_on = !render.flashlight_on;
            return;
        }
        play::HandChange::PlaceCampfire => {
            // Godot CampfirePlacer: 1.5 m in front on flat ground, away from other fires
            if let (Some(play_state), Some(p)) = (play.play.as_mut(), actors.player_ctrl.as_ref()) {
                let facing = game::yaw_forward(p.yaw);
                let physics = &world.physics;
                let ground = |spot: Vec3| {
                    physics
                        .raycast(spot + Vec3::Y * 3.0, Vec3::NEG_Y, 6.0)
                        .map(|h| (h.point, h.normal))
                };
                let text = match play_state.camp.campfires.place(
                    p.position,
                    facing,
                    Some(&mut play_state.inventory),
                    ground,
                ) {
                    Ok(_) => "Campfire placed".to_string(),
                    Err(e) => e.message().to_string(),
                };
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                play_state.show_toast(text);
            }
            return;
        }
        play::HandChange::Health(amount) => {
            if let Some(c) = actors.combat.as_mut() {
                if amount > 0.0 {
                    c.health.heal(amount);
                } else {
                    // Spoiled food: sickness damage (no flinch)
                    c.take_bites(&[-amount]);
                }
            }
            return;
        }
    };
    render.pending_weapon_model = Some(def.as_ref().and_then(weapon_model_name));
    // Guns go to the shooting system, everything else to the melee swing
    let is_gun = def.as_ref().is_some_and(|d| d.is_gun());
    if let Some(play_state) = play.play.as_mut() {
        play_state.gun = if is_gun { def.clone() } else { None };
    }
    if let Some(c) = actors.combat.as_mut() {
        c.set_weapon(if is_gun { None } else { def });
    }
}

/// Reads hardware inputs, handles hotkeys for hub and menus, and constructs player Actions.
pub fn process_input(
    input: &Input,
    dt: f32,
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &mut SessionState,
) -> InputOutcome {
    let _phase = engine::timing::PhaseScope::new("input");
    use hub_ui::{HubScreen, HubTab};

    // The player's own keys (Settings > Controls)
    ui.keys = ui
        .menus
        .as_ref()
        .map(|m| KeyMap::from_bindings(&m.keys))
        .unwrap_or_default();
    let keys = ui.keys.clone();
    // In a car, the L (headlights), R (flip recovery) and V (camera) keys belong to the vehicle,
    // not to the on-foot hotkeys
    let in_car = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
    if let Some(m) = ui.menus.as_mut() {
        m.pressed_key = if m.is_open() {
            input.pressed_keys().into_iter().next()
        } else {
            None
        };
    }
    build_mode::check_build_test_flag(play.play.as_mut().map(|p| &mut p.inventory));

    let build_key = ui
        .menus
        .as_ref()
        .and_then(|m| m.keys.keycode_of("build_mode"))
        .unwrap_or(KeyCode::KeyB);

    let dev_typing = ui.dev_panel.open && ui.dev_panel.wants_keyboard_input;
    if !dev_typing {
        if input.pressed(build_key)
            && ui.camp_ui.is_none()
            && !ui.menus.as_ref().is_some_and(|m| m.is_open())
        {
            build_mode::toggle();
            if build_mode::is_active() {
                ui.cursor_captured = true;
            }
        }
        for (key, tab) in [
            (keys.inventory, HubTab::Bag),
            (keys.skills, HubTab::Skills),
            (keys.bounties, HubTab::Bounties),
            (keys.profile, HubTab::Profile),
            (keys.map, HubTab::Map),
        ] {
            if input.pressed(key)
                && ui.camp_ui.is_none()
                && !ui.menus.as_ref().is_some_and(|m| m.is_open())
                && !(in_car && key == keys.skills)
            {
                ui.hub = match &ui.hub {
                    Some(h) if h.screen() == HubScreen::Tab(tab) => None,
                    _ => Some(hub_ui::HubUiState::new(HubScreen::Tab(tab))),
                };
                if ui.hub.is_none() && actors.open_trunk.is_some() {
                    actors.open_trunk = None;
                }
            }
        }
    }
    // F1 toggles the developer panel (not while a menu is open)
    if input.pressed(KeyCode::F1) && !ui.menus.as_ref().is_some_and(|m| m.is_open()) {
        ui.dev_panel.open = !ui.dev_panel.open;
        ui.cursor_captured = !ui.dev_panel.open;
        if !ui.dev_panel.open {
            ui.dev_panel.wants_keyboard_input = false;
        }
    }
    let mut build_escaped = false;
    if input.pressed(KeyCode::Escape) && build_mode::is_active() && !ui.screen_open() {
        build_mode::set_active(false);
        build_escaped = true;
    }
    // Esc closes an open camp or hub screen (the screen itself sees the key), otherwise the menus
    if input.pressed(KeyCode::Escape)
        && !build_escaped
        && !ui.screen_open()
        && !ui.menus.as_ref().is_some_and(|m| m.keys_ui.is_waiting())
        && let Some(m) = ui.menus.as_mut()
    {
        let was_open = m.is_open();
        m.on_escape();
        if !was_open && m.is_open() {
            if let Some(slots) = &session.slots {
                session.slot_cards = crate::session::summaries(slots);
            }
        } else if was_open && !m.is_open() {
            ui.cursor_captured = true;
        }
    }
    if ui.menus.as_ref().is_some_and(|m| m.is_open()) {
        if dt > 0.0 {
            render.fps += (1.0 / dt - render.fps) * (dt * 2.0).min(1.0);
        }
        return InputOutcome::MenuOpen;
    }
    if !ui.cursor_captured && !ui.dev_panel.open && input.mouse_pressed(MouseButton::Left) {
        ui.cursor_captured = true;
    }
    session.play_time += dt as f64;

    let mut actions = if dev_typing {
        Actions::default()
    } else {
        Actions::from_input(input, &keys.movement)
    };
    // V cycles the car camera while driving; the on-foot camera toggle waits until you're out
    if in_car {
        actions.toggle_camera = false;
    }
    if !ui.cursor_captured || ui.screen_open() {
        actions.look_delta = (0.0, 0.0);
    }
    // Arrow keys look around: works without mouse capture and with the dev panel open,
    // but never in the camp/hub screens or while typing in a dev panel field
    if !dev_typing && ui.camp_ui.is_none() && ui.hub.is_none() {
        let key_look = game::input::keyboard_look_delta(
            input.held(keys.movement.look_left),
            input.held(keys.movement.look_right),
            input.held(keys.movement.look_up),
            input.held(keys.movement.look_down),
            dt,
        );
        actions.look_delta.0 += key_look.0;
        actions.look_delta.1 += key_look.1;
    }
    if let Some(m) = &ui.menus {
        let sens = m.settings.mouse_sensitivity;
        actions.look_delta.0 *= sens;
        actions.look_delta.1 *= if m.settings.invert_y { -sens } else { sens };
    }
    if session.autowalk && !dev_typing {
        actions.move_vec.y = -1.0;
    }

    let dead = actors.combat.as_ref().is_some_and(|c| c.is_dead());
    if dead {
        actions = Actions::default();
    }
    // G: the dog stays or follows again (Godot dog_stay)
    if !dev_typing
        && input.pressed(keys.dog_stay)
        && !ui.screen_open()
        && let Some(d) = actors.dogs.as_mut()
        && d.sim.has_dog()
    {
        d.sim.toggle_stay();
        let mode = d.sim.dog().map(|g| format!("{:?}", g.mode())).unwrap_or_default();
        if let Some(play_state) = play.play.as_mut() {
            play_state.show_toast(if mode == "Stay" {
                "Dog: stay".into()
            } else {
                "Dog: follow".into()
            });
        }
    }
    if build_mode::is_active() && !ui.screen_open() && !ui.menus.as_ref().is_some_and(|m| m.is_open()) {
        if input.wheel > 0.0 {
            build_mode::with_state(|s| s.cycle_next());
        } else if input.wheel < 0.0 {
            build_mode::with_state(|s| s.cycle_prev());
        }
        for (idx, key) in [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ]
        .iter()
        .enumerate()
        {
            if input.pressed(*key) {
                build_mode::with_state(|s| s.select_slot(idx));
            }
        }
        if input.pressed(KeyCode::KeyR) {
            build_mode::with_state(|s| s.rotate_90());
        }
        if ui.cursor_captured && input.mouse_pressed(MouseButton::Left) {
            let placed = build_mode::with_state(|s| {
                build_mode::try_place(s, play.play.as_mut().map(|p| &mut p.inventory))
            });
            if placed
                && let (Some(toast), Some(play_state)) = (
                    build_mode::with_state(|s| s.toast.clone()),
                    play.play.as_mut(),
                )
            {
                play_state.show_toast(toast);
            }
        }
        if ui.cursor_captured && input.mouse_pressed(MouseButton::Right) {
            let removed = build_mode::with_state(|s| {
                build_mode::try_remove(s, play.play.as_mut().map(|p| &mut p.inventory))
            });
            if removed
                && let (Some(toast), Some(play_state)) = (
                    build_mode::with_state(|s| s.toast.clone()),
                    play.play.as_mut(),
                )
            {
                play_state.show_toast(toast);
            }
        }
        if input.pressed(keys.interact) {
            let toggled = build_mode::with_state(build_mode::try_toggle_door);
            if toggled
                && let (Some(toast), Some(play_state)) = (
                    build_mode::with_state(|s| s.toast.clone()),
                    play.play.as_mut(),
                )
            {
                play_state.show_toast(toast);
            }
        }
        build_mode::with_state(|s| build_mode::update_raycast(s, &render.camera, &world.physics));
    }

    if !dev_typing
        && !build_mode::is_active()
        && let Some(change) = play.play.as_mut().and_then(|p| p.hotbar_keys(input, &keys.slots))
    {
        apply_hand(change, render, play, actors, world);
    }
    let attack = !dead
        && !build_mode::is_active()
        && !ui.screen_open()
        && (session.autoattack || (ui.cursor_captured && input.mouse_pressed(MouseButton::Left)));

    InputOutcome::Active {
        actions,
        attack,
        dead,
    }
}
