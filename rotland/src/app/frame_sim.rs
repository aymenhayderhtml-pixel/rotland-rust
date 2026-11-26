//! Simulation updates: zombie AI, camp guards, wildlife, companion dogs, and bite damage.

use super::frame_sim_actors::{step_camp_guards, step_cars, step_dogs, step_wildlife};
use super::frame_sim_player::{apply_player_bites, step_water};
use super::frame_sim_zombies::{step_poi_zombies, step_zombie_voices, step_zombies_and_events};
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};

/// Executes entity AI simulations: zombies, POI spawns, voices, guards, wildlife, cars, and dogs.
pub fn update_sim(
    dt: f32,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    ui: &mut UiState,
    session: &SessionState,
    audio: &mut AudioState,
) {
    let _phase = engine::timing::PhaseScope::new("physics");
    let alive = !actors.combat.as_ref().is_some_and(|c| c.is_dead());
    let mut noises_scratch = std::mem::take(&mut actors.noises_scratch);
    noises_scratch.clear();
    if let Some(c) = actors.combat.as_ref() {
        noises_scratch.extend(c.noises.iter().map(|n| (n.position, n.radius)));
    }

    // Zombies can't reach a player on a ladder (they bang instead), and the camp seals the player off
    let player_unreachable = actors.climb.is_active();
    let player_in_safe_zone = world.world_gen.as_ref().is_some_and(|g| {
        g.settlements().iter().any(|s| s.contains(actors.player, 0.0, g.config()))
    });
    // The last camp entered becomes the respawn camp (SettlementWatcher.cs)
    if let (Some(g), Some(pl)) = (world.world_gen.as_ref(), play.play.as_mut()) {
        pl.camp.settlements.update(actors.player, g.settlements(), g.config());
    }
    if let Some(z) = actors.zombies.as_mut() {
        let mut bites = std::mem::take(&mut actors.bites_scratch);
        bites.clear();

        let z_out = step_zombies_and_events(
            dt,
            z,
            alive,
            actors.player,
            actors.combat.as_ref(),
            actors.player_ctrl.as_mut(),
            actors.player_visual.as_mut(),
            &mut actors.guard,
            &mut actors.heard_scratch,
            &mut actors.scream_noises,
            player_unreachable,
            player_in_safe_zone,
            world.world_gen.as_deref(),
            &world.physics,
            play,
            &mut ui.hud,
            audio,
            &mut bites,
        );
        let (hit_from, big_knock) = (z_out.hit_from, z_out.big_knock);
        // R32-06/07: frozen bites chill the body; bloater bursts make toxic clouds.
        if z_out.chill > 0.0
            && let Some(temp) = play.temperature.as_mut()
        {
            temp.body.chill(z_out.chill);
        }
        for b in z_out.toxic_bursts {
            render.toxic_cloud_view.spawn(b.position, b.radius, b.duration);
            actors.toxic_clouds.spawn(game::zombie::ToxicBurst {
                pos: b.position,
                radius: b.radius,
                seconds: b.duration,
                player_dps: b.player_dps,
                zombie_dps: b.zombie_dps,
            });
        }
        actors.toxic_clouds.update(dt);
        render.toxic_cloud_view.update(dt);
        if alive {
            let (player_dps, _) = actors.toxic_clouds.damage_at(actors.player);
            if player_dps > 0.0
                && let Some(c) = actors.combat.as_mut()
            {
                let now = actors.downed.last_play_time();
                c.take_damage_with_downed(&[player_dps * dt], &mut actors.downed.machine, now, false);
            }
        }
        for zb in &mut z.list {
            let (_, zombie_dps) = actors.toxic_clouds.damage_at(zb.position);
            if zombie_dps > 0.0 && !zb.health.is_dead() {
                let mut info = game::DamageInfo::new(zombie_dps * dt);
                info.damage_type = "toxic".to_string();
                zb.health.take_damage(info);
            }
        }

        step_poi_zombies(
            z,
            world,
            render.pack.as_ref(),
            play,
            actors.player,
            &mut actors.zombie_pos_scratch,
        );

        step_zombie_voices(
            dt,
            z,
            actors.zombie_voices.as_mut(),
            audio,
            actors.player,
            &mut actors.voice_zombies_scratch,
            &mut play.voices_played,
        );

        step_camp_guards(
            dt,
            z,
            actors.camp_guards.as_mut(),
            actors.camp_npcs.as_mut(),
            &mut actors.guard_posts_scratch,
            &mut actors.guard_targets_scratch,
            &world.physics,
            audio,
        );

        let mut zombie_status = std::mem::take(&mut actors.zombie_status_scratch);
        zombie_status.clear();
        zombie_status.extend(z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())));

        step_wildlife(
            dt,
            z,
            actors.wildlife.as_mut(),
            actors.player,
            actors.player_ctrl.as_ref(),
            alive,
            world.world_gen.as_deref(),
            &world.physics,
            play,
            &mut ui.hud,
            audio,
            &noises_scratch,
            &zombie_status,
            &mut bites,
        );

        step_cars(
            dt,
            z,
            actors.cars.as_mut(),
            actors.open_trunk.as_deref(),
            &actors.car_input,
            &mut world.physics,
            &mut actors.car_solids,
            &mut actors.car_solids_scratch,
            &mut actors.car_targets_scratch,
            &mut actors.car_gone_scratch,
            world.streamer.as_ref(),
            actors.wildlife.as_mut(),
            actors.player,
            play,
            audio,
            &mut actors.car_camera,
        );

        // Base siege: zombies chasing the player hit the player's walls (R22-N4)
        super::frame_siege::update_siege(
            dt,
            render.pack.as_ref(),
            play,
            &mut world.physics,
            Some(&*z),
            actors.player,
            alive,
        );

        // Buildings near the player go on the map (Godot MapBuilding / WorldMap)
        if let (Some(st), Some(play_state), Some(g)) = (
            world.streamer.as_ref(),
            play.play.as_mut(),
            world.world_gen.as_ref(),
        ) {
            play_state.note_buildings(
                actors.player,
                st.chunks
                    .values()
                    .map(|c| (c.pois.as_slice(), c.crates.as_slice())),
                g.poi_config(),
            );
        }

        step_dogs(
            dt,
            z,
            actors.dogs.as_mut(),
            actors.wildlife.as_mut(),
            actors.player,
            actors.player_ctrl.as_ref(),
            alive,
            &zombie_status,
            &mut actors.animal_status_scratch,
            world,
            play,
            session.seed,
            audio,
        );

        // Hired teammates walk with the player, guard the camp and fight; zombies hit them back (R23-N5)
        super::frame_team::update(
            &super::frame_team::TeamFrame {
                dt,
                player: actors.player,
                player_yaw: actors.player_ctrl.as_ref().map_or(0.0, |p| p.yaw),
                camp: super::frame_team::camp_point(world.world_gen.as_deref()),
                player_downed: actors.downed.is_down(),
                teammate_range: actors.downed.teammate_range(),
            },
            z,
            &world.physics,
            play,
            &mut ui.hud,
            actors.team_models.as_mut(),
        );
        // Guild task hooks: explore places, clear and ready checks, tracker rows (R24-N6)
        super::frame_guild::update(play, world, actors.player, dt, &mut ui.hud);
        actors.zombie_status_scratch = zombie_status;
        actors.noises_scratch = noises_scratch;

        apply_player_bites(
            actors.combat.as_mut(),
            &mut actors.downed,
            actors.player_visual.as_mut(),
            actors.player_ctrl.as_ref(),
            &mut actors.hit_react,
            actors.player,
            &bites,
            hit_from,
            big_knock,
            render,
            play,
            session,
            audio,
        );
        actors.bites_scratch = bites;
    }
    // Exhaust and damage smoke ticks once per frame whenever cars exist (independent of the zombie sim)
    if let Some(cars) = actors.cars.as_ref() {
        let draw_states = cars.draw_states();
        render.car_smoke.update(dt, &draw_states, &render.car_markers);
    }
    render.explosion_view.update(dt, actors.player);

    step_water(dt, alive, world, actors, play, session, audio);
}
