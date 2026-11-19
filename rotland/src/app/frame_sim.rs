//! Simulation updates: zombie AI, camp guards, wildlife, companion dogs, and bite damage.

use glam::Vec3;

use super::frame_cars::car_ramming;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::wildlife;

/// Executes entity AI simulations: zombies, POI spawns, voices, guards, wildlife, cars, and dogs.
pub fn update_sim(
    dt: f32,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    _ui: &mut UiState,
    session: &SessionState,
    audio: &mut AudioState,
) {
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
        let mut heard = std::mem::take(&mut actors.heard_scratch);
        heard.clear();
        if let Some(c) = actors.combat.as_ref() {
            heard.extend(c.noises.iter().cloned());
        }
        heard.append(&mut actors.scream_noises);
        let frame = z.update_full(
            dt,
            alive.then_some(actors.player),
            &heard,
            &world.physics,
            player_unreachable,
            player_in_safe_zone,
        );
        actors.heard_scratch = heard;
        actors.scream_noises.extend(frame.noises.iter().cloned());
        if let Some(a) = audio.audio.as_mut() {
            for at in &frame.screams {
                a.play_at("zombie_groan", *at);
            }
        }
        let mut bites = std::mem::take(&mut actors.bites_scratch);
        bites.clear();
        bites.extend(frame.bites.iter().map(|b| b.damage));

        // Buildings near the player wake the zombies waiting inside; a building is cleared when they are all dead
        if let (Some(pz), Some(st), Some(pack)) = (
            world.poi_zombies.as_mut(),
            world.streamer.as_ref(),
            render.pack.as_ref(),
        ) {
            if world.poi_sites_generation != st.generation {
                world.poi_sites_generation = st.generation;
                let markers = &mut world.poi_markers;
                world.poi_zombie_sites.clear();
                for chunk in st.chunks.values() {
                    for poi in &chunk.pois {
                        let spawns = markers
                            .entry(poi.building_id.clone())
                            .or_insert_with(|| {
                                pack.find(&poi.building_id)
                                    .map(|m| {
                                        m.markers
                                            .iter()
                                            .filter(|k| k.name.starts_with("spawn_"))
                                            .map(|k| Vec3::from(k.translation))
                                            .collect()
                                    })
                                    .unwrap_or_default()
                            })
                            .clone();
                        world.poi_zombie_sites.push(game::PoiZombieSite::new(
                            &poi.id,
                            &poi.building_id,
                            poi.position,
                            poi.yaw,
                            spawns,
                        ));
                    }
                }
            }
            let alive_count = z.list.iter().filter(|q| !q.health.is_dead()).count();
            for spawn in pz.update(actors.player, &world.poi_zombie_sites, alive_count) {
                if let Some(id) = z.spawn_kind(&spawn.kind, spawn.position) {
                    pz.on_zombie_spawned(&spawn.poi_id, id);
                }
            }
            // Far behind: gone for good (the building does not wait for them)
            let mut at = std::mem::take(&mut actors.zombie_pos_scratch);
            at.clear();
            at.extend(z.list.iter().map(|q| (q.id, q.position)));
            let far = pz.check_despawns(actors.player, &at);
            actors.zombie_pos_scratch = at;
            z.despawn(&far);

            let mut cleared = std::mem::take(&mut world.cleared_poi_scratch);
            cleared.clear();
            cleared.extend(far.iter().filter_map(|id| pz.on_zombie_despawned(*id)));
            // Dead, or removed by the spawner
            let mut tracked = std::mem::take(&mut world.tracked_zombie_scratch);
            tracked.clear();
            tracked.extend(pz.zombie_to_poi.keys().copied());
            for &id in &tracked {
                if z.list.iter().find(|q| q.id == id).is_none_or(|q| q.health.is_dead())
                    && let Some(poi) = pz.on_zombie_died(id)
                {
                    cleared.push(poi);
                }
            }
            world.tracked_zombie_scratch = tracked;
            if let Some(play_state) = play.play.as_mut() {
                for poi in &cleared {
                    play_state.camp.bounties.on_poi_cleared(poi);
                    play.pois_cleared += 1;
                }
            }
            world.cleared_poi_scratch = cleared;
        }

        // Zombies groan now and then and snarl as they attack (Godot ZombieVoices)
        if let (Some(voices), Some(a)) = (actors.zombie_voices.as_mut(), audio.audio.as_mut()) {
            let mut list = std::mem::take(&mut actors.voice_zombies_scratch);
            list.clear();
            list.extend(z.list.iter().map(|q| {
                let state = if q.health.is_dead() {
                    game::VoiceState::Dead
                } else {
                    q.brain.state().into()
                };
                game::VoiceZombie::new(q.id, q.position, state)
            }));
            for ev in voices.update(dt, actors.player, &list) {
                play.voices_played += 1;
                a.play_at(&ev.sound_id, ev.position);
            }
            actors.voice_zombies_scratch = list;
        }

        // Camp guards shoot zombies near the camp and turn to face them
        if let (Some(g), Some(npcs)) = (actors.camp_guards.as_mut(), actors.camp_npcs.as_mut()) {
            let mut guard_posts = std::mem::take(&mut actors.guard_posts_scratch);
            let guards_iter = npcs.director.fixed_npcs().iter().filter(|n| n.role == "guard");
            let mut post_count = 0;
            for (i, n) in guards_iter.enumerate() {
                post_count += 1;
                if i < guard_posts.len() {
                    guard_posts[i].id.clear();
                    guard_posts[i].id.push_str(&n.id);
                    guard_posts[i].position = n.position;
                    guard_posts[i].yaw = n.yaw;
                } else {
                    guard_posts.push(game::GuardPost {
                        id: n.id.clone(),
                        position: n.position,
                        yaw: n.yaw,
                    });
                }
            }
            guard_posts.truncate(post_count);

            let mut guard_targets = std::mem::take(&mut actors.guard_targets_scratch);
            guard_targets.clear();
            guard_targets.extend(z.list.iter().map(|q| game::GuardTarget {
                id: q.id,
                position: q.position,
                alive: !q.health.is_dead(),
            }));
            let physics = &world.physics;
            let clear = |a: Vec3, b: Vec3| {
                let d = b - a;
                let len = d.length();
                len < 0.5 || physics.raycast(a, d / len, len - 0.3).is_none()
            };
            let gf = g.update(dt, &guard_posts, &guard_targets, &clear);
            for shot in gf.shots {
                let kb = (shot.to - shot.from).normalize_or_zero() * 2.0;
                z.hit_unclaimed(shot.target_id, shot.damage, kb);
            }
            if let Some(a) = audio.audio.as_mut() {
                for (id, at) in &gf.sounds {
                    a.play_at(id, *at);
                }
            }
            npcs.aim.clear();
            npcs.aim.extend(gf.yaws);

            actors.guard_posts_scratch = guard_posts;
            actors.guard_targets_scratch = guard_targets;
        }

        let mut zombie_status = std::mem::take(&mut actors.zombie_status_scratch);
        zombie_status.clear();
        zombie_status.extend(z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())));

        // Wild animals: graze, flee, boars charge the player and fight zombies (Godot Animal/AnimalSpawner)
        if let (Some(w), Some(p), Some(g)) = (
            actors.wildlife.as_mut(),
            actors.player_ctrl.as_ref(),
            world.world_gen.as_ref(),
        ) {
            let biome = match g.biome(actors.player.x, actors.player.z) {
                world::BiomeId::Plains => "plains",
                world::BiomeId::Forest => "forest",
            };
            let no_spawn = |at: Vec3| {
                g.settlements()
                    .iter()
                    .any(|s| s.contains(at, g.config().safe_zone_padding, g.config()))
            };
            let skills = play.play.as_ref().map(|pl| &pl.camp.skills);
            let ctx = game::AnimalUpdateContext {
                player_position: Some(actors.player),
                player_forward: game::yaw_forward(p.yaw),
                player_is_crouching: p.crouching,
                player_is_sprinting: p.is_sprinting,
                player_is_dead: !alive,
                player_hunting_stealth_bonus: skills.map_or(0.0, |sk| sk.get_bonus("animalStealth")),
                player_hunting_level: skills.map_or(1, |sk| sk.get_level("hunting")),
                is_night: !(6.0..20.0).contains(&play.hour),
                biome_name: Some(biome),
                noises: &noises_scratch,
                zombies: &zombie_status,
                in_no_spawn_zone: Some(&no_spawn),
            };
            let af = w.update(dt, &ctx, &world.physics);
            for hit in af.attacks {
                if hit.is_player {
                    if alive {
                        bites.push(hit.damage);
                    }
                } else if let Some(target) = hit.target_id {
                    let mut d = game::DamageInfo::new(hit.damage);
                    d.hit_point = hit.hit_point;
                    d.damage_type = "animal".into();
                    z.hit(target, d, hit.knockback);
                }
            }
            if let Some(a) = audio.audio.as_mut() {
                for (sound, at) in &af.sounds {
                    a.play_at(sound, *at);
                }
            }
            // Kills by the player give hunting XP and count for hunt bounties
            if let Some(play_state) = play.play.as_mut() {
                for species in w.sim.take_kills() {
                    if let Some(xp) = w.sim.config.species(&species).map(|s| s.xp).filter(|&x| x > 0) {
                        play_state.camp.skills.add_xp("hunting", xp as f32);
                    }
                    play_state.camp.bounties.on_animal_killed(&species);
                }
            }
        }

        // Cars: parked ones only live on loaded ground; the driven one takes the keys and runs things over
        if let Some(cars) = actors.cars.as_mut() {
            let mut car_solids_scratch = std::mem::take(&mut actors.car_solids_scratch);
            let mut car_targets_scratch = std::mem::take(&mut actors.car_targets_scratch);
            let mut car_gone_scratch = std::mem::take(&mut actors.car_gone_scratch);
            car_ramming(
                dt,
                cars,
                actors.open_trunk.as_deref(),
                &actors.car_input,
                &mut world.physics,
                &mut actors.car_solids,
                &mut car_solids_scratch,
                &mut car_targets_scratch,
                &mut car_gone_scratch,
                world.streamer.as_ref(),
                z,
                actors.wildlife.as_mut(),
                actors.player,
                play.hour,
                &mut audio.audio,
                play.play.as_mut(),
                &mut audio.car_horn_cooldown,
                &mut actors.car_camera,
                &mut audio.trunk_was_open,
            );
            actors.car_solids_scratch = car_solids_scratch;
            actors.car_targets_scratch = car_targets_scratch;
            actors.car_gone_scratch = car_gone_scratch;
        }

        // The dog follows, barks and bites zombies and animals (Godot Dog / DogCombat)
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

        // Stray dogs wait at the loaded POI buildings (cached in streamer on chunk load/unload)
        let empty_dog_sites = [];
        let dog_sites: &[game::DogPoiSite] = world
            .streamer
            .as_ref()
            .map(|st| st.dog_sites.as_slice())
            .unwrap_or(&empty_dog_sites);

        if let (Some(d), Some(p)) = (actors.dogs.as_mut(), actors.player_ctrl.as_ref()) {
            let mut animal_status = std::mem::take(&mut actors.animal_status_scratch);
            animal_status.clear();
            if let Some(w) = actors.wildlife.as_ref() {
                animal_status.extend(
                    w.sim
                        .list
                        .iter()
                        .filter(|a| !a.retired)
                        .map(|a| (a.id, a.position, a.is_dead())),
                );
            }
            let camp = world
                .world_gen
                .as_ref()
                .and_then(|g| g.settlements().first().map(|s| s.respawn_point(g.config())));
            let ctx = game::DogUpdateContext {
                player_position: Some(actors.player),
                player_forward: game::yaw_forward(p.yaw),
                player_dead: !alive,
                player_sprinting: p.is_sprinting,
                player_velocity: p.velocity,
                zombies: &zombie_status,
                animals: &animal_status,
                is_night: !(6.0..20.0).contains(&play.hour),
                camp_position: camp,
                now_hours: (play.clock.day as f64 - 1.0) * 24.0 + play.clock.hour as f64,
                day: play.clock.day,
                poi_sites: dog_sites,
                world_seed: session.seed,
            };
            let f = d.update(dt, &ctx, &world.physics);
            for bite in f.bites {
                match actors.wildlife.as_mut() {
                    Some(w) if bite.target_id >= wildlife::FIRST_ANIMAL_ID => {
                        w.sim.hit(bite.target_id, bite.damage, bite.knockback);
                    }
                    _ => {
                        z.hit(bite.target_id, bite.damage, bite.knockback);
                    }
                }
            }
            if let Some(a) = audio.audio.as_mut() {
                for (sound, at) in &f.sounds {
                    a.play_at(sound, *at);
                }
            }
            if let (Some(t), Some(play_state)) = (f.toasts.last(), play.play.as_mut()) {
                play_state.show_toast(t.clone());
            }
            actors.animal_status_scratch = animal_status;
        }

        actors.zombie_status_scratch = zombie_status;
        actors.noises_scratch = noises_scratch;

        if let Some(c) = actors.combat.as_mut() {
            let ev = c.take_bites(&bites);
            if ev.died {
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.play_die();
                }
                if let Some(play_state) = play.play.as_mut() {
                    let seed = (session.time * 1000.0) as u64 ^ session.seed as u64;
                    play_state.camp.on_death(&mut play_state.inventory, seed);
                    play_state.hotbar.on_inventory_changed(&play_state.inventory);
                }
                log::info!("the player died (kills so far: {})", c.kills);
            } else if ev.bitten {
                if let Some(pv) = actors.player_visual.as_mut() {
                    pv.animator.play_hit();
                }
                if let Some(a) = audio.audio.as_mut() {
                    a.event("playerHurt", None);
                }
            }
        }
        actors.bites_scratch = bites;
    }
    // Exhaust and damage smoke ticks once per frame whenever cars exist (independent of the zombie sim)
    if let Some(cars) = actors.cars.as_ref() {
        let draw_states = cars.draw_states();
        render.car_smoke.update(dt, &draw_states, &render.car_markers);
    }
}
