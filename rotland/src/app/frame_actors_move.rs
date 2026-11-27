//! Vehicle driving, test teleports, locomotion (climb/vault/walk), and camera/aim sync.

use engine::Input;
use glam::Vec3;
use winit::event::MouseButton;

use super::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::display;
use crate::shooting;

/// Handles vehicle exit, driving inputs, and car camera orbit/mode switching.
pub(super) fn drive_or_exit_car(
    input: &Input,
    dt: f32,
    actions: &game::Actions,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &SessionState,
    audio: &mut Option<audio::Audio>,
) {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_mut(), actors.camera_rig.as_mut()) else {
        return;
    };
    // In a car: the keys drive it, E gets out, the player rides in the driver's seat
    actors.car_input = game::CarInput::default();
    ui.e_used = false;
    let driven = actors.cars.as_ref().and_then(|c| c.driven_car_id.clone());
    if let (Some(id), Some(cars)) = (driven, actors.cars.as_mut()) {
        use winit::keyboard::KeyCode as K;
        let mut exited = false;
        if input.pressed(ui.keys.interact) && ui.hub.is_none() && ui.camp_ui.is_none() {
            // The car must be slow (or flipped) before the driver can climb out
            let can_exit = cars.get(&id).is_some_and(|car| {
                car.speed_kmh() <= cars.config.exit_max_kmh || car.body.is_flipped()
            });
            if can_exit {
                let spot = cars.exit(&id, &world.physics);
                p.position = spot + Vec3::Y * 0.1;
                p.velocity = Vec3::ZERO;
                ui.e_used = true;
                exited = true;
                actors.car_camera.deactivate();
                if let Some(a) = audio.as_mut() {
                    a.play_at("car_door", spot);
                }
            } else if let Some(play_state) = play.play.as_mut() {
                play_state.show_toast("Slow down to exit".into());
                ui.e_used = true;
            }
        }
        if !exited
            && let Some(car) = cars.get(&id)
        {
            actors.car_input = game::CarInput {
                throttle: -actions.move_vec.y,
                steer: actions.move_vec.x,
                handbrake: input.held(K::Space),
                horn: input.held(K::KeyH),
                lights_toggle: input.pressed(K::KeyL),
                flip_recover: input.pressed(K::KeyR),
                ..Default::default()
            };
            let f = car.body.rotation * Vec3::NEG_Z;
            let car_yaw = (-f.x).atan2(-f.z);
            p.position = car.body.position + Vec3::Y * 0.3;
            p.velocity = Vec3::ZERO;
            p.yaw = car_yaw;
            // Activate the car camera the first frame we drive, feed it the mouse, and let V pick the mode
            if !actors.car_camera.is_active() {
                actors.car_camera.activate();
            }
            // look_delta is already zero when the mouse is free or a screen is open (frame_input),
            // except for arrow-key look, which should turn the car camera too
            actors
                .car_camera
                .orbit(glam::Vec2::new(actions.look_delta.0, actions.look_delta.1));
            if let Some(yaw) = session.test_car_yaw {
                actors.car_camera.set_orbit_yaw(yaw);
            }
            if input.pressed(K::KeyV) && ui.hub.is_none() && ui.camp_ui.is_none() {
                let mode = actors.car_camera.cycle_mode();
                if let Some(play_state) = play.play.as_mut() {
                    play_state.show_toast(format!("Camera: {}", mode.label()));
                }
            }
            // The camera swings back behind the car unless the mouse turns it
            if actions.look_delta.0.abs() < 0.01 {
                let diff = (car_yaw - rig.yaw + std::f32::consts::PI)
                    .rem_euclid(std::f32::consts::TAU)
                    - std::f32::consts::PI;
                rig.yaw += diff * (dt * 2.5).min(1.0);
            }
            ui.e_used = input.pressed(ui.keys.interact) || ui.e_used;
        }
    }
}

/// Applies one-shot `--test-poi`, `--test-chop`, and `--test-climb` teleports.
pub(super) fn apply_test_teleports(
    world: &WorldState,
    actors: &mut ActorsState,
    session: &mut SessionState,
    grab: &mut bool,
) {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_mut(), actors.camera_rig.as_mut()) else {
        return;
    };
    if session.test_poi && session.time > 1.0 {
        session.test_poi = false;
        let me = p.position;
        let nearest = world.streamer.as_ref().and_then(|s| {
            s.chunks
                .values()
                .flat_map(|c| c.pois.iter())
                .min_by(|a, b| a.position.distance(me).total_cmp(&b.position.distance(me)))
                .map(|q| q.position)
        });
        if let Some(poi) = nearest {
            let away = Vec3::new(me.x - poi.x, 0.0, me.z - poi.z).normalize_or(Vec3::X);
            let at = poi + away * 20.0;
            p.position = Vec3::new(
                at.x,
                world.physics.ground_height(at.x, at.z).unwrap_or(at.y) + 0.05,
                at.z,
            );
            p.yaw = away.x.atan2(away.z);
            rig.yaw = p.yaw;
        }
    }
    if session.test_chop && session.time > 1.0 {
        session.test_chop = false;
        let me = p.position;
        let nearest = world.streamer.as_ref().and_then(|s| {
            s.chunks
                .values()
                .flat_map(|c| c.trees.iter())
                .min_by(|a, b| a.position.distance(me).total_cmp(&b.position.distance(me)))
                .map(|t| t.position)
        });
        if let Some(tree) = nearest {
            let away = Vec3::new(me.x - tree.x, 0.0, me.z - tree.z).normalize_or(Vec3::X);
            let at = tree + away * 1.3;
            p.position = Vec3::new(
                at.x,
                world.physics.ground_height(at.x, at.z).unwrap_or(at.y) + 0.05,
                at.z,
            );
            p.yaw = away.x.atan2(away.z);
            rig.yaw = p.yaw;
        }
    }
    if session.test_climb && session.time > 1.0 {
        session.test_climb = false;
        let nearest = world.streamer.as_ref().and_then(|s| {
            s.ladders
                .iter()
                .min_by(|a, b| a.foot.distance(p.position).total_cmp(&b.foot.distance(p.position)))
                .cloned()
        });
        if let Some(rail) = nearest {
            p.position = rail.foot_interaction_point();
            *grab = true;
        }
    }
}

/// Steps ladder climbing, vaulting, ground walking, and one-shot body animation cues.
/// Returns `true` if the player is actively climbing a ladder this frame.
pub(super) fn step_locomotion(
    dt: f32,
    actions: &game::Actions,
    grab: bool,
    dead: bool,
    downed: bool,
    in_car: bool,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
) -> bool {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_mut(), actors.camera_rig.as_mut()) else {
        return false;
    };
    let empty_rails = [];
    let rails: &[game::LadderRail] = if grab || actors.climb.is_active() {
        world
            .streamer
            .as_ref()
            .map(|s| s.ladders.as_slice())
            .unwrap_or(&empty_rails)
    } else {
        &empty_rails
    };
    let was_climbing = actors.climb.is_active();
    let climb_in = game::ClimbInput::new(-actions.move_vec.y, actions.jump, grab);
    let climbing = !dead
        && !downed
        && !in_car
        && game::climb_tick(&mut actors.climb, p, &climb_in, rails, dt);
    ui.e_used = ui.e_used || (grab && (climbing || was_climbing));
    static VAULT_CFG: std::sync::OnceLock<game::VaultConfig> = std::sync::OnceLock::new();
    let vault_cfg = VAULT_CFG.get_or_init(|| game::VaultConfig::from_repo().unwrap_or_default());
    if dead || downed || in_car || climbing {
        actors.vault.cancel();
    }
    if in_car {
        // the car moves the player (above)
    } else if climbing {
        rig.yaw = p.yaw;
    } else if actors.vault.is_active() {
        actors.vault.step(p, dt);
    } else if let Some(clip) = (!dead && !downed && actions.jump)
        .then(|| actors.vault.try_start(p, &world.physics, -actions.move_vec.y, vault_cfg))
        .flatten()
    {
        if let Some(pv) = actors.player_visual.as_mut() {
            // HELPER: no config() accessor on HumanoidAnimator; use FullBodyOptions::default() (AnimConfig defaults) and only force hold_last off.
            let opts = humanoid::FullBodyOptions {
                hold_last: false,
                ..humanoid::FullBodyOptions::default()
            };
            pv.animator.play_full_body(clip, opts);
        }
    } else {
        let saved_speed_mul = p.speed_multiplier;
        let saved_sprint_cost = p.stamina.sprint_cost_per_sec;
        if downed {
            p.speed_multiplier *= actors.downed.crawl_speed_mul();
        }
        if play.play.as_ref().is_some_and(|ps| ps.boosts.free_sprint()) {
            p.stamina.sprint_cost_per_sec = 0.0; // new: DECISIONS 2026-11-27
        }
        p.update(actions, rig.yaw, &world.physics, &actors.mover, dt);
        p.speed_multiplier = saved_speed_mul;
        p.stamina.sprint_cost_per_sec = saved_sprint_cost;
    }
    if let Some(cue) = play.play.as_mut().and_then(|ps| ps.anim_cue.take())
        && !downed
        && let Some(pv) = actors.player_visual.as_mut()
    {
        pv.animator.play_cue(cue);
    }
    climbing
}

/// Syncs `actors.player`, camera rig / car camera, FOV, and aiming animation state.
pub(super) fn update_camera_and_aim(
    input: &Input,
    dt: f32,
    downed: bool,
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &PlaySide,
    ui: &mut UiState,
    session: &SessionState,
) {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_ref(), actors.camera_rig.as_ref()) else {
        return;
    };
    actors.player = p.position;
    render.camera = if let Some(orbit_yaw) = session.anim_shot.orbit_camera_yaw(p.yaw) {
        let mut cam_rig = rig.clone();
        cam_rig.yaw = orbit_yaw;
        cam_rig.is_first_person = false;
        cam_rig.camera(&actors.player, &world.physics)
    } else {
        rig.camera(&actors.player, &world.physics)
    };
    // Driving: the multi-mode car camera (Chase / Close / Hood) with speed pullback and shake
    if actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some())
        && let Some((car_pos, car_rot, car_speed, car_tag)) = actors
            .cars
            .as_ref()
            .and_then(|c| c.driven_car())
            .map(|car| (car.body.position, car.body.rotation, car.speed_kmh(), car.body.self_tag))
    {
        let out = actors
            .car_camera
            .step_drive(dt, car_pos, car_rot, car_speed, car_tag, &world.physics);
        render.camera.position = out.eye;
        render.camera.look_at(out.target);
        render.camera.fov_y_deg = out.fov_y_deg;
    }
    // The player's field of view (Settings) applies on foot; the car camera owns its own FOV
    let driving = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
    if let Some(m) = &ui.menus
        && !driving
    {
        render.camera.fov_y_deg = display::apply_fov(render.camera.fov_y_deg, &m.settings);
    }
    // A gun's aim zoom (the rifle scope zooms much further, CameraRig/GunHandler); moot while driving or downed
    let aiming = !driving
        && !downed
        && ((ui.cursor_captured && input.mouse_held(MouseButton::Right)) || session.anim_shot.block);
    ui.right_held = aiming;
    let melee_in_hand = actors
        .combat
        .as_ref()
        .and_then(|c| c.weapon_def())
        .is_some_and(|w| !w.is_gun() && !w.is_bow() && !w.throwable);
    let anim_aiming = if melee_in_hand {
        actors.guard.is_blocking()
    } else {
        aiming
    };
    if session.anim_shot.pose.is_none()
        && !session.anim_shot.is_pose_frozen()
        && let Some(pv) = actors.player_visual.as_mut()
    {
        pv.animator.set_aiming(anim_aiming);
        pv.animator.set_look_pitch(rig.pitch);
    }
    if let (true, Some(gun)) = (aiming, play.play.as_ref().and_then(|pl| pl.gun.as_ref()))
        && let Some(fov) = shooting::aim_fov(gun, rig.is_first_person)
    {
        render.camera.fov_y_deg = fov;
    }
    render.camera.position += render.explosion_view.camera_shake_offset();
}
