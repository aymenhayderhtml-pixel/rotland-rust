//! Save game capture, loading, and menu actions.

use super::state::{ActorsState, PlaySide, SessionState, UiState, WorldState};
use crate::{menus, session};

/// Writes the game into the active slot (over the file it came from).
pub fn save_now(
    session: &mut SessionState,
    world: &WorldState,
    actors: &ActorsState,
    play: &mut PlaySide,
) {
    // Teammates' health goes onto their recruits first, so the guild saves it (R23-N5)
    super::frame_team::write_health_back(play);
    let (Some(slots), Some(p), Some(rig), Some(c), Some(play_state), Some(w)) = (
        session.slots.as_ref(),
        actors.player_ctrl.as_ref(),
        actors.camera_rig.as_ref(),
        actors.combat.as_ref(),
        play.play.as_ref(),
        world.world_gen.as_ref(),
    ) else {
        return;
    };
    let spawn = play_state
        .camp
        .settlements
        .try_respawn_point(w.settlements(), w.config())
        .unwrap_or_else(|| w.settlements()[0].respawn_point(w.config()));
    let snap = session::Snapshot {
        seed: session.seed,
        player: p,
        rig,
        health: c.health.current(),
        spawn,
        clock: &play.clock,
        play: play_state,
        play_time: session.play_time,
        trees: Some(&world.trees),
        poi_zombies: world.poi_zombies.as_ref(),
        weather: play.weather.as_ref().map(|w| w.to_save()),
        body_temp: play.temperature.as_ref().map(|t| t.value()),
    };
    let mut file = session::capture(session.loaded_save.as_ref(), &snap);
    crate::rescue_play::save_rescued_ids(play.rescue.as_ref(), &mut file);
    if let Some(d) = &actors.dogs {
        file.systems.extra.insert("dog".into(), d.sim.to_save());
    }
    if let Some(c) = &actors.cars {
        file.systems.extra.insert("cars".into(), c.to_save());
    }
    if let Some(v) = play_state.metro_herds.to_save() {
        file.systems.extra.insert("street_herds".into(), v);
    }
    if let Some(v) = play.metro_radio.to_save() {
        file.systems.extra.insert("metro_radio".into(), v);
    }
    // Weapons saving: equipped weapon in hand and loaded magazine rounds
    let equipped = if let Some(gun) = &play_state.gun {
        gun.id.clone()
    } else if let Some(wid) = c.weapon_id() {
        wid.to_string()
    } else {
        String::new()
    };
    let mut magazines = std::collections::HashMap::new();
    if let Some(gun) = &play_state.gun {
        magazines.insert(gun.id.clone(), play_state.last_shot.ammo.0 as i32);
    }
    file.systems.weapons = Some(save::WeaponsSaveData {
        equipped,
        magazines,
    });
    // Settlements saving: visited safe zones and respawn camp
    let mut visited: Vec<String> = play_state.camp.settlements.visited().iter().cloned().collect();
    visited.sort();
    file.systems.settlements = Some(save::SettlementsSaveData {
        visited,
        respawn: play_state.camp.settlements.respawn_id().map(|s| s.to_string()),
        camp_upgrades: play_state.camp.upgrades_to_save(),
    });
    // Base building saving: placed base parts
    file.systems.base = Some(play.build_mode.to_save());
    match slots.write(session.active_slot as usize, &file) {
        Ok(()) => {
            log::info!(
                "saved slot {} to {}",
                session.active_slot,
                slots.path_for(session.active_slot as usize).unwrap_or_else(|_| std::path::PathBuf::new()).display()
            );
            session.slot_cards = session::summaries(slots);
            session.loaded_save = Some(file);
        }
        Err(e) => log::warn!("save to slot {} failed: {e:?}", session.active_slot),
    }
}

/// Closes pause and main menus, returning mouse capture to the game.
pub fn close_menus(ui: &mut UiState) {
    if let Some(m) = ui.menus.as_mut() {
        m.set_screen(menus::Screen::None);
    }
    ui.cursor_captured = true;
}

/// Does what the player picked in a menu.
pub fn menu_action(
    action: menus::MenuAction,
    session: &mut SessionState,
    world: &WorldState,
    actors: &ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
) {
    use menus::MenuAction;
    match action {
        MenuAction::Resume => {
            if let Some(m) = ui.menus.as_mut() {
                m.set_screen(menus::Screen::None);
            }
            ui.cursor_captured = true;
        }
        MenuAction::NewGame { slot } => {
            session.active_slot = slot;
            // A fresh world: the seed comes from the clock, like Godot's new game
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(42, |d| d.as_nanos() as i32);
            session.pending_world = Some((seed, None));
            play.build_mode.reset();
            super::frame_siege::clear(play);
            close_menus(ui);
        }
        MenuAction::Continue { slot } => {
            let read = session.slots.as_ref().map(|s| s.read(slot as usize));
            match read {
                Some(Ok(file)) => {
                    session.active_slot = slot;
                    if let Some(base) = &file.systems.base {
                        play.build_mode.load_from_save(base);
                    } else {
                        play.build_mode.reset();
                    }
                    super::frame_siege::clear(play);
                    session.pending_world = Some((
                        session::seed_of(&file).unwrap_or(session.seed),
                        Some(file),
                    ));
                    close_menus(ui);
                }
                Some(Err(e)) => log::warn!("slot {slot} not loaded: {e:?}"),
                None => {}
            }
        }
        MenuAction::DeleteSlot { slot } => {
            if let Some(slots) = &session.slots {
                if let Err(e) = slots.delete(slot as usize) {
                    log::warn!("slot {slot} not deleted: {e:?}");
                }
                session.slot_cards = session::summaries(slots);
            }
        }
        MenuAction::SaveAndQuitToMenu => {
            save_now(session, world, actors, play);
            if let Some(m) = ui.menus.as_mut() {
                m.set_screen(menus::Screen::Main);
            }
        }
        MenuAction::Quit => {
            // Quitting from the pause menu keeps the game; from the main menu there is nothing to save
            if ui.menus.as_ref().is_some_and(|m| *m.screen() == menus::Screen::Paused) {
                save_now(session, world, actors, play);
            }
            session.quit = true;
        }
        MenuAction::SettingsChanged(new) => {
            if let Some(m) = ui.menus.as_mut() {
                m.settings = new;
            }
        }
    }
}
