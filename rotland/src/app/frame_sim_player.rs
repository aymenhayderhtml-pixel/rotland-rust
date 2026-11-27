//! Player damage reactions, lake water swimming/drowning, and death clip playback.

use glam::Vec3;

use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, WorldState};

/// Applies accumulated bite/animal hits to the player, handling downed state, death, and hit reactions.
pub(super) fn apply_player_bites(
    combat: Option<&mut crate::combat::Combat>,
    downed: &mut crate::downed_play::DownedPlay,
    mut player_visual: Option<&mut crate::player_visual::PlayerVisual>,
    player_ctrl: Option<&game::Player>,
    hit_react: &mut game::HitReactState,
    player: Vec3,
    bites: &[f32],
    hit_from: Option<Vec3>,
    big_knock: Vec3,
    render: &mut RenderState,
    play: &mut PlaySide,
    session: &SessionState,
    audio: &mut AudioState,
) {
    if let Some(c) = combat {
        let was_down = downed.is_down();
        let reduction = play
            .play
            .as_ref()
            .map_or(0.0, |ps| game::armor_reduction(&ps.inventory, &ps.armor_cfg));
        let scaled_bites: Vec<f32> = bites
            .iter()
            .map(|&d| game::apply_armor_damage(d, reduction))
            .collect();
        let ev = c.take_damage_with_downed(
            &scaled_bites,
            &mut downed.machine,
            session.play_time,
            true,
        );
        if ev.downed_now {
            crate::downed_play::on_knocked_down(
                render,
                play,
                downed,
                c,
                player_visual.as_deref_mut(),
                session.anim_shot.is_pose_frozen(),
                &mut audio.audio,
            );
        }
        if ev.died {
            if let Some(pv) = player_visual {
                let player_yaw = player_ctrl.map_or(0.0, |p| p.yaw);
                play_death_clip(
                    &mut pv.animator,
                    game::DeathCause::Zombie,
                    player,
                    player_yaw,
                    hit_from,
                );
            }
            if let Some(play_state) = play.play.as_mut() {
                let seed = (session.time * 1000.0) as u64 ^ session.seed as u64;
                play_state.camp.on_death(&mut play_state.inventory, seed);
                play_state.hotbar.on_inventory_changed(&play_state.inventory);
                super::frame_guild::clear_hooks(&mut play.guild_hooks);
            }
            log::info!("the player died (kills so far: {})", c.kills);
        } else if ev.bitten && !was_down && !ev.downed_now {
            static HIT_CFG: std::sync::OnceLock<game::HitReactConfig> = std::sync::OnceLock::new();
            let hit_cfg = HIT_CFG.get_or_init(|| game::HitReactConfig::from_repo().unwrap_or_default());
            let player_yaw = player_ctrl.map_or(0.0, |p| p.yaw);
            let input = game::HitInput {
                time: session.time as f64,
                player_pos: player,
                player_yaw,
                attacker: hit_from,
                knockback: big_knock,
            };
            if let (Some(r), Some(pv)) = (hit_react.on_hit(&input, hit_cfg), player_visual) {
                if r.full_body {
                    let opts = humanoid::FullBodyOptions {
                        speed: r.speed,
                        hold_last: false,
                        fade_in: r.fade_in,
                        fade_out: r.fade_out,
                        ..humanoid::FullBodyOptions::default()
                    };
                    pv.animator.play_full_body(r.clip, opts);
                } else {
                    pv.animator.play_action(r.clip, r.speed);
                }
            }
            if let Some(a) = audio.audio.as_mut() {
                a.event("playerHurt", None);
            }
        } else if ev.bitten
            && let Some(a) = audio.audio.as_mut()
        {
            a.event("playerHurt", None);
        }
    }
}

/// Lake water effects: swimming, floating, stamina drain, and drowning.
pub(super) fn step_water(
    dt: f32,
    alive: bool,
    world: &WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
    audio: &mut AudioState,
) {
    if let (Some(wg), Some(p)) = (world.world_gen.as_ref(), actors.player_ctrl.as_mut()) {
        let p_pos2 = glam::Vec2::new(p.position.x, p.position.z);
        let swim_cfg = play.water.swim_cfg;
        let lake_opt = wg.lake();
        let lake_zone = lake_opt.map_or(world::LakeZone::Land, |l| l.zone_at(p_pos2));
        let on_dock = lake_opt.is_some_and(|l| game::swimming::is_on_dock(l, p_pos2));
        let (zone, water_level, on_platform) = if lake_zone != world::LakeZone::Land {
            (lake_zone, lake_opt.map_or(0.0, |l| l.water_level), on_dock)
        } else {
            let river_y = wg.rivers().water_at(p.position.x, p.position.z);
            let ground_y = wg.height(p.position.x, p.position.z);
            let r_zone = game::swimming::river_zone_at(p.position.y, ground_y, river_y, &swim_cfg);
            (r_zone, river_y.unwrap_or(0.0), false)
        };
        let water_res = game::swimming::evaluate_water(
            p.position,
            zone,
            water_level,
            p.stamina.current,
            on_platform,
            &swim_cfg,
            dt,
        );
        p.apply_water_movement(&water_res);
        p.water_depth = match water_res.zone {
            world::LakeZone::Shallow | world::LakeZone::Deep if !on_platform => {
                (water_level - p.position.y).max(0.0)
            }
            _ => 0.0,
        };
        // Hands are stowed by frame_water (update_water_play) from this flag
        play.water.in_deep = water_res.disarm_weapons;

        let in_deep_water = water_res.zone == world::LakeZone::Deep && !on_platform;
        if (water_res.drowning_damage > 0.0 || (actors.downed.is_down() && in_deep_water)) && alive
            && let Some(c) = actors.combat.as_mut()
        {
            let ev = crate::downed_play::take_drowning(
                c,
                &mut actors.downed,
                water_res.drowning_damage,
                in_deep_water,
            );
            if ev.died {
                if let Some(pv) = actors.player_visual.as_mut() {
                    play_death_clip(
                        &mut pv.animator,
                        game::DeathCause::Drowned,
                        Vec3::ZERO,
                        0.0,
                        None,
                    );
                }
                if let Some(play_state) = play.play.as_mut() {
                    let seed = (session.time * 1000.0) as u64 ^ session.seed as u64;
                    play_state.camp.on_death(&mut play_state.inventory, seed);
                    play_state.hotbar.on_inventory_changed(&play_state.inventory);
                    super::frame_guild::clear_hooks(&mut play.guild_hooks);
                }
                log::info!("the player drowned");
            } else if let Some(a) = audio.audio.as_mut() {
                a.event("playerHurt", None);
            }
        }
    } else {
        play.water.in_deep = false;
    }
}

fn play_death_clip(
    animator: &mut humanoid::HumanoidAnimator,
    cause: game::DeathCause,
    player_pos: Vec3,
    player_yaw: f32,
    attacker: Option<Vec3>,
) {
    static DEATH_CFG: std::sync::OnceLock<game::DeathAnimConfig> = std::sync::OnceLock::new();
    let cfg = DEATH_CFG.get_or_init(|| game::DeathAnimConfig::from_repo().unwrap_or_default());
    let choice = cfg.pick(cause, player_pos, player_yaw, attacker);
    animator.play_die_clip(choice.clip, choice.speed);
}
