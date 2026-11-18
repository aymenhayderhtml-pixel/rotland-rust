//! Simulation updates: zombie AI, camp guards, wildlife, companion dogs, and bite damage.

use glam::Vec3;

use super::frame_cars::car_ramming;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::{poi_links, wildlife};

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
    let noises = actors.combat.as_ref().map(|c| c.noises.clone()).unwrap_or_default();

    if let Some(z) = actors.zombies.as_mut() {
        let frame = z.update(dt, alive.then_some(actors.player), &noises, &world.physics);
        let mut bites: Vec<f32> = frame.bites.iter().map(|b| b.damage).collect();

        // Buildings near the player wake the zombies waiting inside; a building is cleared when they are all dead
        if let (Some(pz), Some(st), Some(pack)) = (
            world.poi_zombies.as_mut(),
            world.streamer.as_ref(),
            render.pack.as_ref(),
        ) {
            let markers = &mut world.poi_markers;
            let sites: Vec<game::PoiZombieSite> = st
                .chunks
                .values()
                .flat_map(|c| c.pois.iter())
                .map(|poi| {
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
                    game::PoiZombieSite::new(&poi.id, &poi.building_id, poi.position, poi.yaw, spawns)
                })
                .collect();
            let alive_count = z.list.iter().filter(|q| !q.health.is_dead()).count();
            for spawn in pz.update(actors.player, &sites, alive_count) {
                if let Some(id) = z.spawn_kind(&spawn.kind, spawn.position) {
                    pz.on_zombie_spawned(&spawn.poi_id, id);
                }
            }
            // Far behind: gone for good (the building does not wait for them)
            let at: Vec<(u64, Vec3)> = z.list.iter().map(|q| (q.id, q.position)).collect();
            let far = pz.check_despawns(actors.player, &at);
            z.despawn(&far);
            let mut cleared: Vec<String> =
                far.iter().filter_map(|id| pz.on_zombie_despawned(*id)).collect();
            // Dead, or removed by the spawner
            let tracked: Vec<u64> = pz.zombie_to_poi.keys().copied().collect();
            for id in tracked {
                if z.list.iter().find(|q| q.id == id).is_none_or(|q| q.health.is_dead())
                    && let Some(poi) = pz.on_zombie_died(id)
                {
                    cleared.push(poi);
                }
            }
            if let Some(play_state) = play.play.as_mut() {
                for poi in cleared {
                    play_state.camp.bounties.on_poi_cleared(&poi);
                    play.pois_cleared += 1;
                }
            }
        }

        // Zombies groan now and then and snarl as they attack (Godot ZombieVoices)
        if let (Some(voices), Some(a)) = (actors.zombie_voices.as_mut(), audio.audio.as_mut()) {
            let list: Vec<game::VoiceZombie> = z
                .list
                .iter()
                .map(|q| {
                    let state = if q.health.is_dead() {
                        game::VoiceState::Dead
                    } else {
                        q.brain.state().into()
                    };
                    game::VoiceZombie::new(q.id, q.position, state)
                })
                .collect();
            for ev in voices.update(dt, actors.player, &list) {
                play.voices_played += 1;
                a.play_at(&ev.sound_id, ev.position);
            }
        }

        // Camp guards shoot zombies near the camp and turn to face them
        if let (Some(g), Some(npcs)) = (actors.camp_guards.as_mut(), actors.camp_npcs.as_mut()) {
            let posts: Vec<game::GuardPost> = npcs
                .director
                .fixed_npcs()
                .iter()
                .filter(|n| n.role == "guard")
                .map(|n| game::GuardPost {
                    id: n.id.clone(),
                    position: n.position,
                    yaw: n.yaw,
                })
                .collect();
            let targets: Vec<game::GuardTarget> = z
                .list
                .iter()
                .map(|q| game::GuardTarget {
                    id: q.id,
                    position: q.position,
                    alive: !q.health.is_dead(),
                })
                .collect();
            let physics = &world.physics;
            let clear = |a: Vec3, b: Vec3| {
                let d = b - a;
                let len = d.length();
                len < 0.5 || physics.raycast(a, d / len, len - 0.3).is_none()
            };
            let gf = g.update(dt, &posts, &targets, &clear);
            for shot in gf.shots {
                let kb = (shot.to - shot.from).normalize_or_zero() * 2.0;
                z.hit_unclaimed(shot.target_id, shot.damage, kb);
            }
            if let Some(a) = audio.audio.as_mut() {
                for (id, at) in &gf.sounds {
                    a.play_at(id, *at);
                }
            }
            npcs.aim = gf.yaws.into_iter().collect();
        }

        // Wild animals: graze, flee, boars charge the player and fight zombies (Godot Animal/AnimalSpawner)
        if let (Some(w), Some(p), Some(g)) = (
            actors.wildlife.as_mut(),
            actors.player_ctrl.as_ref(),
            world.world_gen.as_ref(),
        ) {
            let zl: Vec<(u64, Vec3, bool)> =
                z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())).collect();
            let heard: Vec<(Vec3, f32)> = noises.iter().map(|n| (n.position, n.radius)).collect();
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
                noises: &heard,
                zombies: &zl,
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
            car_ramming(
                dt,
                cars,
                &actors.car_input,
                &mut world.physics,
                &mut actors.car_solids,
                world.streamer.as_ref(),
                z,
                actors.wildlife.as_mut(),
                actors.player,
                play.hour,
                session.time,
                &mut audio.audio,
                play.play.as_mut(),
            );
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

        // Stray dogs wait at the loaded POI buildings
        let dog_sites: Vec<game::DogPoiSite> = match (world.streamer.as_ref(), world.world_gen.as_ref()) {
            (Some(st), Some(g)) => st
                .chunks
                .values()
                .flat_map(|c| poi_links::build_dog_poi_sites(&c.pois, g.poi_config()))
                .collect(),
            _ => Vec::new(),
        };
        if let (Some(d), Some(p)) = (actors.dogs.as_mut(), actors.player_ctrl.as_ref()) {
            let zl: Vec<(u64, Vec3, bool)> =
                z.list.iter().map(|q| (q.id, q.position, q.health.is_dead())).collect();
            let al: Vec<(u64, Vec3, bool)> = actors
                .wildlife
                .as_ref()
                .map(|w| {
                    w.sim
                        .list
                        .iter()
                        .filter(|a| !a.retired)
                        .map(|a| (a.id, a.position, a.is_dead()))
                        .collect()
                })
                .unwrap_or_default();
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
                zombies: &zl,
                animals: &al,
                is_night: !(6.0..20.0).contains(&play.hour),
                camp_position: camp,
                now_hours: (play.clock.day as f64 - 1.0) * 24.0 + play.clock.hour as f64,
                day: play.clock.day,
                poi_sites: &dog_sites,
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
        }

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
    }
}
