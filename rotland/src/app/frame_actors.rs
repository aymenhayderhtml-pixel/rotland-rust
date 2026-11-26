//! Actor movement, camera rig, melee combat, and tree felling.

use engine::Input;
use game::Actions;
use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use super::frame_actors_combat::{
    step_guard, step_melee_and_respawn, step_ranged_and_projectiles,
};
use super::frame_actors_move::{
    apply_test_teleports, drive_or_exit_car, step_locomotion, update_camera_and_aim,
};
use super::save_load::save_now;
use super::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::bench;

/// Third-person camera like Godot's CameraRig: 4 m spring arm, 0.55 m right shoulder, looking slightly down.
pub fn camera_follow(camera: &mut engine::render::Camera, player: Vec3, yaw: f32) {
    camera.yaw = yaw;
    camera.pitch = -0.25;
    let f = camera.forward();
    let right = f.cross(Vec3::Y).normalize_or_zero();
    let pivot = player + Vec3::new(0.0, 1.6, 0.0) + right * 0.55 + Vec3::Y * 0.2;
    camera.position = pivot - f * 4.0;
}

// TODO(R11): move to game/ with tests
pub fn strike_trees(
    swung: Option<(game::WeaponDef, Vec3)>,
    player_pos: Vec3,
    world: &mut WorldState,
    play: &mut PlaySide,
    audio: &mut Option<audio::Audio>,
) {
    // An axe swing that lands near a tree chops it (Godot MeleeHandler + TreeStrike/TreeFelling)
    if let Some((weapon, dir)) = swung
        && weapon.tags.iter().any(|t| t == "chop")
        && let Some(st) = world.streamer.as_mut()
    {
        let me = player_pos;
        let hit = {
            let near = st.chunks.iter().flat_map(|(c, ch)| {
                ch.trees
                    .iter()
                    .filter(move |t| t.position.distance_squared(me) < 36.0)
                    .map(move |t| game::TreeCandidate::new(*c, t.index, t.position, &t.model))
            });
            game::find_tree_strike(me + Vec3::Y, dir, weapon.range, weapon.arc_deg, near)
        };
        if let Some(hit) = hit {
            let id = game::tree_id(hit.chunk, hit.index);
            let model = st
                .chunks
                .get(&hit.chunk)
                .and_then(|ch| ch.trees.iter().find(|t| t.index == hit.index))
                .map(|t| t.model.clone())
                .unwrap_or_default();
            let r = world.trees.strike(&id, hit.point, &model, Some(&weapon.id), None, me);
            if let (Some(sound), Some(a)) = (r.sound_event, audio.as_mut()) {
                a.play_at(sound, hit.point);
            }
            if r.felled {
                if let Some(slot) = st.fell(&mut world.physics, hit.chunk, hit.index) {
                    world.falling.insert(id, (hit.chunk, slot));
                }
                // The wood lands 1.5 m to the +X side, on the ground (Godot TreeFelling.Fell)
                if r.wood_given > 0 && let Some(play_state) = play.play.as_mut() {
                    let (x, z) = (hit.point.x + 1.5, hit.point.z);
                    let y = world.physics.ground_height(x, z).unwrap_or(hit.point.y) + 0.125;
                    play_state.drop_on_ground("wood", r.wood_given, Vec3::new(x, y, z));
                    play_state.show_toast(format!("Tree felled: +{} wood", r.wood_given));
                }
            }
        }
    }
}

/// Updates player locomotion, camera rigs, ladders, vehicle ride positions, and combat swings.
pub fn update_actors(
    input: &Input,
    dt: f32,
    actions: &Actions,
    attack: bool,
    dead: bool,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &mut SessionState,
    audio: &mut Option<audio::Audio>,
) {
    let _phase = engine::timing::PhaseScope::new("zombies/actors");
    let downed = actors.downed.is_down();
    let self_revive_held = downed
        && input.held(ui.keys.interact)
        && !ui.screen_open()
        && !ui.menus.as_ref().is_some_and(|m| m.is_open());
    actors.downed.set_frame_input(self_revive_held, session.play_time);
    crate::downed_play::sync_downed_visual(
        &mut actors.downed,
        render,
        play,
        actors.combat.as_mut(),
        actors.player_visual.as_mut(),
        session.anim_shot.is_pose_frozen(),
    );
    if session.test_explosion && session.time > 0.05 {
        session.test_explosion = false;
        let forward = game::yaw_forward(
            actors.player_ctrl.as_ref().map_or(0.0, |p| p.yaw),
        );
        let pos = actors.player + forward * 5.0 + Vec3::Y * 0.5;
        crate::explosion_view::ExplosionView::spawn(&mut render.explosion_view, pos, 6.0);
    }
    // Death or knockdown while driving must park the car and restore the on-foot camera.
    if dead || downed {
        if let Some(cars) = actors.cars.as_mut()
            && let Some(id) = cars.driven_car_id.clone()
        {
            let spot = cars.exit(&id, &world.physics);
            if let Some(p) = actors.player_ctrl.as_mut() {
                p.position = spot + Vec3::Y * 0.1;
                p.velocity = Vec3::ZERO;
            }
        }
        actors.car_camera.deactivate();
        actors.car_input = game::CarInput::default();
        if let Some(a) = audio.as_mut() {
            a.set_car_engine(false, 0.0);
        }
    }
    if let (Some(p), Some(rig)) = (actors.player_ctrl.as_mut(), actors.camera_rig.as_mut()) {
        let mut actions = *actions;
        session.anim_shot.apply_actions(&mut actions, session.autowalk);
        if downed {
            actions.sprint = false;
            actions.jump = false;
            actions.crouch = false;
            actions.aim = false;
        }
        let actions = &actions;
        rig.update(actions, p.crouching, dt);
        p.first_person = rig.is_first_person;

        drive_or_exit_car(input, dt, actions, world, actors, play, ui, session, audio);
        let in_car = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
        // Ladders first: while climbing, the ladder moves the player instead of walking
        let mut grab = input.pressed(ui.keys.interact)
            && ui.camp_ui.is_none()
            && ui.hub.is_none()
            && !dead
            && !downed;
        apply_test_teleports(world, actors, session, &mut grab);
        let climbing = step_locomotion(
            dt, actions, grab, dead, downed, in_car, world, actors, play, ui,
        );
        let is_grenade = actors
            .combat
            .as_ref()
            .and_then(|c| c.weapon_def())
            .is_some_and(|w| w.id == "grenade");
        let swung = step_melee_and_respawn(
            dt,
            attack && !is_grenade,
            downed,
            world,
            actors,
            play,
            audio,
        );
        let player_pos = actors
            .player_ctrl
            .as_ref()
            .map_or(actors.player, |pl| pl.position);
        // Tree felling helper
        strike_trees(swung, player_pos, world, play, audio);

        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);
        let reload_pressed = !in_car && captured && input.pressed(ui.keys.reload);
        step_guard(
            input, dt, dead, downed, in_car, climbing, captured, actors, play, session,
        );
        step_ranged_and_projectiles(
            input,
            dt,
            dead,
            downed,
            in_car,
            captured,
            right_pressed,
            reload_pressed,
            render,
            world,
            actors,
            play,
            audio,
        );
        let mut grenades = std::mem::take(&mut actors.grenades);
        grenades.update(
            input, dt, dead, downed, in_car, captured, render, world, actors, play, audio,
        );
        actors.grenades = grenades;
        update_camera_and_aim(input, dt, downed, render, world, actors, play, ui, session);
    }
}

/// Fallback update for the scripted benchmark walk or free camera mode.
pub fn update_walk_or_free(
    input: &Input,
    dt: f32,
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    session: &mut SessionState,
) {
    let _phase = engine::timing::PhaseScope::new("zombies/actors");
    if session.walk {
        session.walk_time += dt;
        let yaw = bench::yaw_at(session.walk_time);
        let forward = game::yaw_forward(yaw);
        actors.player += forward * bench::WALK_SPEED * dt;
        actors.player.y = world.height(actors.player.x, actors.player.z);
        camera_follow(&mut render.camera, actors.player, yaw);
    } else {
        render.camera.yaw -= input.mouse_delta.0 * 0.003;
        render.camera.pitch = (render.camera.pitch - input.mouse_delta.1 * 0.003).clamp(-1.5, 1.5);
        let f = render.camera.forward();
        let right = f.cross(Vec3::Y).normalize_or_zero();
        for (key, dir) in [
            (KeyCode::KeyW, f),
            (KeyCode::KeyS, -f),
            (KeyCode::KeyD, right),
            (KeyCode::KeyA, -right),
        ] {
            if input.held(key) {
                render.camera.position += dir * 20.0 * dt;
            }
        }
        actors.player = render.camera.position;
    }
}

/// Checks scripted test timers (`die_at`, `save_at`, and `test_flags`).
pub fn check_scripted_timers(
    session: &mut SessionState,
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    audio: &mut super::state::AudioState,
) {
    if let (Some(t), Some(c)) = (session.die_at, actors.combat.as_mut())
        && session.time >= t
    {
        session.die_at = None;
        if c.kill() {
            actors.downed.reset();
            if let Some(pv) = actors.player_visual.as_mut() {
                pv.animator.play_die();
            }
        }
    }
    super::test_flags::tick_timers(session, render, actors, play, audio);
    if let Some(t) = session.save_at
        && session.time >= t
    {
        session.save_at = None;
        save_now(session, world, actors, play);
        engine::timing::count_save_written();
    }
}
