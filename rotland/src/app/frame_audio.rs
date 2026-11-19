//! Audio updates: listener positioning, volume settings, day/night ambient loops, and footsteps.

use super::state::{ActorsState, AudioState, PlaySide, RenderState, UiState};

// TODO(R11): move to game/ with tests
pub fn footsteps_tick(
    dt: f32,
    audio: &mut audio::Audio,
    step_timer: &mut f32,
    player_ctrl: &game::Player,
    alive: bool,
) {
    *step_timer -= dt;
    if player_ctrl.on_floor && player_ctrl.horizontal_speed() >= 0.5 && alive {
        if *step_timer <= 0.0 {
            audio.play_ui("footstep_grass");
            *step_timer = if player_ctrl.is_sprinting { 0.33 } else { 0.5 };
        }
    } else {
        *step_timer = 0.0;
    }
}

/// Updates 3D audio listener, volume levels from settings, day/night sound loops, and footstep audio.
pub fn update_audio(
    dt: f32,
    render: &RenderState,
    actors: &ActorsState,
    play: &PlaySide,
    ui: &UiState,
    audio: &mut AudioState,
) {
    let alive = !actors.combat.as_ref().is_some_and(|c| c.is_dead());
    if let Some(a) = audio.audio.as_mut() {
        a.set_listener(render.camera.position, render.camera.forward());
        if let Some(m) = &ui.menus {
            a.set_volumes(
                m.settings.master_volume,
                m.settings.music_volume,
                m.settings.sfx_volume,
            );
        }
        a.tick(dt, !(6.0..20.0).contains(&play.hour));
        if let Some(p) = actors.player_ctrl.as_ref() {
            footsteps_tick(dt, a, &mut audio.step_timer, p, alive);
        }
        // Continuous RPM-pitched engine loop while the 4x4 is driven (silent when dead / paused)
        let paused = ui.menus.as_ref().is_some_and(|m| m.is_open());
        let (engine_on, rpm) = match (alive, paused, actors.cars.as_ref().and_then(|c| c.driven_car())) {
            (true, false, Some(car)) => (
                car.is_engine_on(),
                car.engine_rpm.clamp(0.0, 1.0),
            ),
            _ => (false, 0.0),
        };
        a.set_car_engine(engine_on, rpm);
    }
}
