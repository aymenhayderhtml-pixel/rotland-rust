//! Melee combat, tree felling, guard/shove, throwables, bows, and projectiles.

use engine::Input;
use game::Player;
use glam::Vec3;
use winit::event::MouseButton;

use super::state::{ActorsState, PlaySide, RenderState, SessionState, WorldState};
use crate::shooting::TargetWorld;
use crate::{shooting, wildlife};

/// Steps melee combat, first-person weapon sway/motion, melee audio, and player respawn.
/// Returns any weapon swing that landed this frame for tree felling.
pub(super) fn step_melee_and_respawn(
    dt: f32,
    attack: bool,
    downed: bool,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    audio: &mut Option<audio::Audio>,
) -> Option<(game::WeaponDef, Vec3)> {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_mut(), actors.camera_rig.as_ref()) else {
        return None;
    };
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
        let ev = c.update(
            dt,
            attack && !downed && !actors.guard.is_blocking(),
            p,
            &mut prey,
            &world.physics,
        );
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
                && let Some(sound) = c
                    .weapon_id()
                    .and_then(|w| a.data.weapon_sound(w, "swing"))
                    .map(str::to_string)
            {
                a.play_at(&sound, p.position);
            }
            if c.hits > 0 {
                a.event("meleeHit", Some(p.position));
            }
        }
        let swing_speed = if ev.swing_speed > 0.0 {
            ev.swing_speed
        } else {
            1.0
        };
        if let (Some(clip), Some(pv)) = (ev.swing_started, actors.player_visual.as_mut()) {
            pv.animator.play_action(&clip, swing_speed);
        }
        if ev.respawn {
            actors.downed.reset();
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
                    .and_then(|pl| {
                        pl.camp
                            .settlements
                            .try_respawn_point(w.settlements(), w.config())
                    })
                    .unwrap_or_else(|| w.settlements()[0].respawn_point(w.config()));
                *p = Player::new(spawn);
                // Face where the camera looks, not the default direction
                p.yaw = rig.yaw;
            }
        }
    }
    swung
}

/// Updates melee guard blocking, shoves, and guard-break feedback.
pub(super) fn step_guard(
    input: &Input,
    dt: f32,
    dead: bool,
    downed: bool,
    in_car: bool,
    climbing: bool,
    captured: bool,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
) {
    let Some(p) = actors.player_ctrl.as_mut() else {
        return;
    };
    static GUARD_CFG: std::sync::OnceLock<game::GuardConfig> = std::sync::OnceLock::new();
    let guard_cfg = GUARD_CFG.get_or_init(|| game::GuardConfig::from_repo().unwrap_or_default());
    let guard_weapon = actors
        .combat
        .as_ref()
        .and_then(|c| c.weapon_def())
        .is_some_and(|w| !w.is_gun() && !w.is_bow() && !w.throwable && w.id != "fishing_rod");
    let swing_busy = actors.combat.as_ref().is_some_and(|c| c.swing_busy());
    let guard_in = game::GuardInput {
        held: (captured && input.mouse_held(MouseButton::Right)) || session.anim_shot.block,
        can_guard: !dead
            && !downed
            && !in_car
            && !climbing
            && !play.build_mode.active
            && guard_weapon
            && !swing_busy,
    };
    match actors.guard.update(dt, guard_in, &mut p.stamina, guard_cfg) {
        game::GuardEvent::Shove => {
            if let Some(z) = actors.zombies.as_mut() {
                let facing = Vec3::new(-p.yaw.sin(), 0.0, -p.yaw.cos());
                game::shove_targets(
                    p.position,
                    facing,
                    &z.targets(),
                    guard_cfg,
                    &mut actors.shove_scratch,
                );
                for &(id, push) in &actors.shove_scratch {
                    z.shove(
                        id,
                        push,
                        guard_cfg.shove.stagger_seconds,
                        &guard_cfg.shove.immune_kinds,
                    );
                }
            }
            if let Some(pv) = actors.player_visual.as_mut() {
                pv.animator
                    .play_action(&guard_cfg.shove.clip, guard_cfg.shove.clip_speed);
            }
        }
        game::GuardEvent::GuardBroken => {
            if let Some(play_state) = play.play.as_mut() {
                play_state.show_toast("Guard broken!".to_string());
            }
        }
        _ => {}
    }
}

/// Handles throwable spears, bow draw/release, crossbow fire/reload, and active projectile stepping.
#[allow(clippy::too_many_arguments)]
pub(super) fn step_ranged_and_projectiles(
    input: &Input,
    dt: f32,
    dead: bool,
    downed: bool,
    in_car: bool,
    captured: bool,
    right_pressed: bool,
    reload_pressed: bool,
    render: &mut RenderState,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    audio: &mut Option<audio::Audio>,
) {
    let (Some(p), Some(rig)) = (actors.player_ctrl.as_ref(), actors.camera_rig.as_ref()) else {
        return;
    };
    if let Some(c) = actors.combat.as_mut()
        && !dead
        && !downed
        && !in_car
        && let Some(weapon) = c.weapon_def().cloned()
        && weapon.throwable
        && weapon.id != "grenade"
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
        && !downed
        && !in_car
        && let Some(weapon) = c.weapon_def().cloned()
        && weapon.is_bow()
    {
        if weapon.id == "crossbow" {
            let fire_pressed = captured && input.mouse_pressed(MouseButton::Left);
            let spare_bolts = play
                .play
                .as_ref()
                .map_or(0, |pl| pl.inventory.count(&weapon.ammo_item));
            // data: data/weapons.json crossbow.reloadTime
            let reload_time = if weapon.reload_time > 0.0 {
                weapon.reload_time
            } else {
                2.0
            };
            let step = c.bow.update_crossbow(
                dt,
                fire_pressed,
                reload_pressed,
                spare_bolts > 0,
                reload_time,
            );
            if step.fired {
                let aim_origin = render.camera.position;
                let aim_dir = render.camera.forward();
                let shot = game::projectiles::bow_shot(1.0, aim_origin, aim_dir, &weapon);
                c.projectiles.spawn(shot);
                c.shots += 1;
                actors.fp_sway.trigger_shot();
                render.gun_feel.set_context(Some(&weapon.id), false);
                render.gun_feel.on_shot(&weapon.id);
                c.noises.push(game::zombie::Noise {
                    position: p.position,
                    radius: weapon.noise_radius,
                });
                if let Some(a) = audio.as_mut() {
                    a.event("bowRelease", Some(p.position));
                }
            }
            if step.reload_started {
                if let Some(pv) = actors.player_visual.as_mut() {
                    let clip = weapon.anim.reload.as_deref().unwrap_or("crossbow_reload");
                    pv.animator.play_action(clip, 1.0);
                }
                if let Some(a) = audio.as_mut() {
                    a.event("reload", Some(p.position));
                }
            }
            if step.reload_finished
                && let Some(play_state) = play.play.as_mut()
            {
                play_state.inventory.remove(&weapon.ammo_item, 1);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
            }
            if let Some(play_state) = play.play.as_mut() {
                let bolts = play_state.inventory.count(&weapon.ammo_item);
                play_state.last_shot.ammo = (u32::from(c.bow.crossbow_loaded), bolts);
                play_state.gun = Some(weapon.clone());
            }
        } else {
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
                render.gun_feel.set_context(Some(&weapon.id), c.bow.draw > 0.0);
                render.gun_feel.on_shot(&weapon.id);
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
                game::projectiles::ProjectileEvent::HitZombie {
                    id,
                    damage,
                    headshot,
                } => {
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
}
