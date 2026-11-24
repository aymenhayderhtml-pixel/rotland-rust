//! Actor movement, camera rig, melee combat, and tree felling.

use engine::Input;
use game::{Actions, Player};
use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use super::save_load::save_now;
use super::state::{ActorsState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::shooting::TargetWorld;
use crate::{bench, display, shooting, wildlife};

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
    // Death while driving must park the car and restore the on-foot camera.
    if dead {
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
        rig.update(actions, p.crouching, dt);
        p.first_person = rig.is_first_person;
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
                actors.car_camera.orbit(glam::Vec2::new(actions.look_delta.0, actions.look_delta.1));
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
        let in_car = actors.cars.as_ref().is_some_and(|c| c.driven_car_id.is_some());
        // Ladders first: while climbing, the ladder moves the player instead of walking
        let mut grab = input.pressed(ui.keys.interact) && ui.camp_ui.is_none() && ui.hub.is_none() && !dead;
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
                grab = true;
            }
        }
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
        let climbing = !dead && !in_car && game::climb_tick(&mut actors.climb, p, &climb_in, rails, dt);
        ui.e_used = ui.e_used || (grab && (climbing || was_climbing));
        if in_car {
            // the car moves the player (above)
        } else if climbing {
            rig.yaw = p.yaw;
        } else {
            p.update(actions, rig.yaw, &world.physics, &actors.mover, dt);
        }
        let mut swung = None;
        if let (Some(c), Some(z)) = (actors.combat.as_mut(), actors.zombies.as_mut()) {
            let animal_bonus = play
                .play
                .as_ref()
                .map_or(0.0, |pl| pl.camp.skills.get_bonus("animalDamage"));
            let mut prey = wildlife::Prey {
                zombies: z,
                animals: actors.wildlife.as_mut().map(|w| &mut w.sim),
                animal_bonus,
            };
            let ev = c.update(dt, attack, p, &mut prey, &world.physics);
            swung = ev.swung.clone();
            if ev.swing_started.is_some() {
                actors.fp_sway.trigger_swing();
            }
            actors.fp_sway.update(dt, p.horizontal_speed());
            static FP_MOTION_CFG: std::sync::OnceLock<game::FpMotionConfig> =
                std::sync::OnceLock::new();
            let fp_cfg = FP_MOTION_CFG
                .get_or_init(|| game::FpMotionConfig::from_repo().unwrap_or_default());
            let weapon_key = c.weapon_id().map_or(0, |id| {
                use std::hash::{Hash, Hasher};
                let mut h = std::collections::hash_map::DefaultHasher::new();
                id.hash(&mut h);
                h.finish().max(1)
            });
            let stamina_frac = if p.stamina.max > 0.0 {
                p.stamina.current / p.stamina.max
            } else {
                1.0
            };
            let reloading = play.play.as_ref().is_some_and(|pl| pl.gun_reloading());
            let fp_in = game::FpMotionInput {
                yaw: rig.yaw,
                pitch: rig.pitch,
                horizontal_speed: p.horizontal_speed(),
                vertical_velocity: p.velocity.y,
                on_floor: p.on_floor,
                sprinting: p.is_sprinting,
                crouching: p.crouching,
                reloading,
                weapon_key,
                stamina_frac,
            };
            actors.fp_motion.update(dt, &fp_in, fp_cfg);
            actors.fp_sway.motion_offset = actors.fp_motion.offset();
            actors.fp_sway.motion_rot_deg = actors.fp_motion.rot_deg();
            if let Some(a) = audio.as_mut() {
                if ev.swing_started.is_some()
                    && let Some(sound) = c.weapon_id().and_then(|w| a.data.weapon_sound(w, "swing")).map(str::to_string)
                {
                    a.play_at(&sound, p.position);
                }
                if c.hits > 0 {
                    a.event("meleeHit", Some(p.position));
                }
            }
            if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
                pv.animator.play_action(&clip, 1.0);
            }
            if ev.respawn {
                // Clear the death pose, or it stays mixed into the legs after waking up
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.reset_alive();
                }
                if let Some(play_state) = play.play.as_mut() {
                    play_state.needs_respawn();
                }
                // Wake up at the camp (Godot respawns at the last settlement visited)
                if let Some(w) = world.world_gen.as_ref() {
                    let spawn = play
                        .play
                        .as_ref()
                        .and_then(|pl| pl.camp.settlements.try_respawn_point(w.settlements(), w.config()))
                        .unwrap_or_else(|| w.settlements()[0].respawn_point(w.config()));
                    *p = Player::new(spawn);
                    // Face where the camera looks, not the default direction
                    p.yaw = rig.yaw;
                }
            }
        }
        // Tree felling helper
        strike_trees(swung, p.position, world, play, audio);

        let captured = ui.cursor_captured && !ui.screen_open();
        let right_pressed = captured && input.mouse_pressed(MouseButton::Right);
        if let Some(c) = actors.combat.as_mut()
            && !dead
            && !in_car
            && let Some(weapon) = c.weapon_def().cloned()
            && weapon.throwable
            && right_pressed
        {
            if let Some(play_state) = play.play.as_mut() {
                play_state.inventory.remove(&weapon.id, 1);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
            }
            c.set_weapon(None);
            render.pending_weapon_model = Some(None);
            let aim_origin = render.camera.position;
            let aim_dir = render.camera.forward();
            let proj = game::projectiles::spear_throw(aim_origin, aim_dir, &weapon);
            c.projectiles.spawn(proj);
            actors.fp_sway.trigger_swing();
            if let Some(a) = audio.as_mut() {
                a.event("throw", Some(p.position));
            }
        }

        if let Some(c) = actors.combat.as_mut()
            && !dead
            && !in_car
            && let Some(weapon) = c.weapon_def().cloned()
            && weapon.is_bow()
        {
            let attack_held = captured && input.mouse_held(MouseButton::Left);
            let released = c.bow.update(dt, attack_held, weapon.draw_time);
            if c.bow.draw > 0.0 {
                let target_fov = if rig.is_first_person {
                    shooting::FIRST_PERSON_AIM_FOV
                } else {
                    shooting::THIRD_PERSON_AIM_FOV
                };
                render.camera.fov_y_deg = crate::combat::BowState::aim_fov(
                    render.camera.fov_y_deg,
                    c.bow.draw,
                    target_fov,
                );
            }
            if let Some(draw) = released
                && let Some(play_state) = play.play.as_mut()
                && play_state.inventory.count(&weapon.ammo_item) > 0
            {
                play_state.inventory.remove(&weapon.ammo_item, 1);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                let aim_origin = render.camera.position;
                let aim_dir = render.camera.forward();
                let shot = game::projectiles::bow_shot(draw, aim_origin, aim_dir, &weapon);
                c.projectiles.spawn(shot);
                actors.fp_sway.trigger_shot();
                c.noises.push(game::zombie::Noise {
                    position: p.position,
                    radius: weapon.noise_radius,
                });
                if let Some(a) = audio.as_mut() {
                    a.event("bowRelease", Some(p.position));
                }
            }
            if let Some(play_state) = play.play.as_mut() {
                let arrows = play_state.inventory.count(&weapon.ammo_item);
                play_state.last_shot.ammo = (arrows, u32::MAX);
                play_state.gun = Some(weapon.clone());
            }
        }

        if let (Some(c), Some(z)) = (actors.combat.as_mut(), actors.zombies.as_mut())
            && !c.projectiles.is_empty()
        {
            let _projectile_draws = crate::combat::projectiles_view::projectile_instances(&c.projectiles);
            let animal_bonus = play
                .play
                .as_ref()
                .map_or(0.0, |pl| pl.camp.skills.get_bonus("animalDamage"));
            let mut prey = wildlife::Prey {
                zombies: z,
                animals: actors.wildlife.as_mut().map(|w| &mut w.sim),
                animal_bonus,
            };
            let targets = prey.gun_targets();
            let events = c.projectiles.step(dt, |from, to| {
                game::projectiles::raycast_projectile(from, to, &targets, &world.physics)
            });
            for ev in events {
                match ev {
                    game::projectiles::ProjectileEvent::HitZombie { id, damage, headshot } => {
                        c.hits += 1;
                        let dir = render.camera.forward();
                        let knockback = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero() * 2.0;
                        let dmg_info = game::DamageInfo {
                            amount: damage,
                            source: Some(1),
                            hit_point: p.position,
                            direction: dir,
                            damage_type: "projectile".to_string(),
                            headshot,
                            leg_hit: false,
                        };
                        if prey.hit(id, dmg_info, knockback) {
                            c.kills += 1;
                        }
                        if let Some(a) = audio.as_mut() {
                            a.event("meleeHit", Some(p.position));
                        }
                    }
                    game::projectiles::ProjectileEvent::HitWorld { point, pickup } => {
                        if let Some(item_id) = pickup
                            && let Some(play_state) = play.play.as_mut()
                        {
                            play_state.drop_on_ground(&item_id, 1, point);
                        }
                    }
                    game::projectiles::ProjectileEvent::Expired => {}
                }
            }
        }

        actors.player = p.position;
        render.camera = rig.camera(&actors.player, &world.physics);
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
        // A gun's aim zoom (the rifle scope zooms much further, CameraRig/GunHandler); moot while driving
        let aiming = !driving && ui.cursor_captured && input.mouse_held(MouseButton::Right);
        ui.right_held = aiming;
        if let Some(pv) = actors.player_visual.as_mut() { pv.animator.set_aiming(aiming); pv.animator.set_look_pitch(rig.pitch); }
        if let (true, Some(gun)) = (aiming, play.play.as_ref().and_then(|pl| pl.gun.as_ref()))
            && let Some(fov) = shooting::aim_fov(gun, rig.is_first_person)
        {
            render.camera.fov_y_deg = fov;
        }
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

/// Checks scripted test timers (die_at and save_at).
pub fn check_scripted_timers(
    session: &mut SessionState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
) {
    if let (Some(t), Some(c)) = (session.die_at, actors.combat.as_mut())
        && session.time >= t
    {
        session.die_at = None;
        if c.kill()
            && let Some(pv) = actors.player_visual.as_mut()
        {
            pv.animator.play_die();
        }
    }
    if let Some(t) = session.save_at
        && session.time >= t
    {
        session.save_at = None;
        save_now(session, world, actors, play);
        engine::timing::count_save_written();
    }
}
