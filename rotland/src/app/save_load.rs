//! Save game capture, loading, and menu actions.

use super::state::{ActorsState, PlaySide, SessionState, UiState, WorldState};
use crate::{menus, session};

/// Writes the game into the active slot (over the file it came from).
pub fn save_now(
    session: &mut SessionState,
    world: &WorldState,
    actors: &ActorsState,
    play: &PlaySide,
) {
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
    let snap = session::Snapshot {
        seed: session.seed,
        player: p,
        rig,
        health: c.health.current(),
        spawn: w.settlements()[0].respawn_point(w.config()),
        clock: &play.clock,
        play: play_state,
        play_time: session.play_time,
    };
    let mut file = session::capture(session.loaded_save.as_ref(), &snap);
    if let Some(d) = &actors.dogs {
        file.systems.extra.insert("dog".into(), d.sim.to_save());
    }
    if let Some(c) = &actors.cars {
        file.systems.extra.insert("cars".into(), c.to_save());
    }
    let mut ws = file.systems.worldstate.clone().unwrap_or_default();
    ws.fallen_trees = serde_json::from_value(world.trees.to_save()["fallen_trees"].clone()).unwrap_or_default();
    if let Some(pz) = &world.poi_zombies {
        ws.visited_pois = pz.visited_ids();
    }
    file.systems.worldstate = Some(ws);
    match slots.write(session.active_slot as usize, &file) {
        Ok(()) => {
            log::info!(
                "saved slot {} to {}",
                session.active_slot,
                slots.path_for(session.active_slot as usize).display()
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
    play: &PlaySide,
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
            close_menus(ui);
        }
        MenuAction::Continue { slot } => {
            let read = session.slots.as_ref().map(|s| s.read(slot as usize));
            match read {
                Some(Ok(file)) => {
                    session.active_slot = slot;
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
