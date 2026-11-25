//! Non-zombie simulation steps: camp guards, wildlife, cars, and companion/stray dogs.

use glam::{Mat4, Vec3};

use super::frame_cars::car_ramming;
use super::state::{AudioState, PlaySide, WorldState};
use crate::wildlife;

/// Camp guards shoot zombies near the camp and turn to face them.
pub(super) fn step_camp_guards(
    dt: f32,
    z: &mut crate::zombies::Zombies,
    camp_guards: Option<&mut game::CampGuards>,
    camp_npcs: Option<&mut crate::camp_npcs::CampNpcs>,
    guard_posts_scratch: &mut Vec<game::GuardPost>,
    guard_targets_scratch: &mut Vec<game::GuardTarget>,
    physics: &physics::PhysicsWorld,
    audio: &mut AudioState,
) {
    if let (Some(g), Some(npcs)) = (camp_guards, camp_npcs) {
        let mut guard_posts = std::mem::take(guard_posts_scratch);
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

        let mut guard_targets = std::mem::take(guard_targets_scratch);
        guard_targets.clear();
        guard_targets.extend(z.list.iter().map(|q| game::GuardTarget {
            id: q.id,
            position: q.position,
            alive: !q.health.is_dead(),
        }));
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

        *guard_posts_scratch = guard_posts;
        *guard_targets_scratch = guard_targets;
    }
}

/// Wild animals: graze, flee, boars charge the player and fight zombies (Godot Animal/AnimalSpawner).
pub(super) fn step_wildlife(
    dt: f32,
    z: &mut crate::zombies::Zombies,
    wildlife_opt: Option<&mut wildlife::Wildlife>,
    player: Vec3,
    player_ctrl: Option<&game::Player>,
    alive: bool,
    world_gen: Option<&world::WorldGenerator>,
    physics: &physics::PhysicsWorld,
    play: &mut PlaySide,
    hud: &mut crate::hud::Hud,
    audio: &mut AudioState,
    noises_scratch: &[(Vec3, f32)],
    zombie_status: &[(u64, Vec3, bool)],
    bites: &mut Vec<f32>,
) {
    if let (Some(w), Some(p), Some(g)) = (wildlife_opt, player_ctrl, world_gen) {
        let biome = match g.biome(player.x, player.z) {
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
            player_position: Some(player),
            player_forward: game::yaw_forward(p.yaw),
            player_is_crouching: p.crouching,
            player_is_sprinting: p.is_sprinting,
            player_is_dead: !alive,
            player_hunting_stealth_bonus: skills.map_or(0.0, |sk| sk.get_bonus("animalStealth")),
            player_hunting_level: skills.map_or(1, |sk| sk.get_level("hunting")),
            is_night: !(6.0..20.0).contains(&play.hour),
            biome_name: Some(biome),
            noises: noises_scratch,
            zombies: zombie_status,
            in_no_spawn_zone: Some(&no_spawn),
        };
        let af = w.update(dt, &ctx, physics);
        for hit in af.attacks {
            if hit.is_player {
                if alive {
                    bites.push(hit.damage);
                    if let Some(attacker) = af.draws.iter().find(|d| d.id == hit.animal_id) {
                        hud.on_damage(attacker.position);
                    }
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
}

/// Cars: parked ones only live on loaded ground; the driven one takes the keys and runs things over.
pub(super) fn step_cars(
    dt: f32,
    z: &mut crate::zombies::Zombies,
    cars: Option<&mut game::Cars>,
    open_trunk: Option<&str>,
    car_input: &game::CarInput,
    physics: &mut physics::PhysicsWorld,
    car_solids: &mut crate::car_solid::ParkedCarSolids,
    car_solids_scratch: &mut Vec<(String, Mat4, bool)>,
    car_targets_scratch: &mut Vec<(u64, Vec3)>,
    car_gone_scratch: &mut Vec<glam::IVec2>,
    streamer: Option<&crate::streaming::Streamer>,
    wildlife_opt: Option<&mut wildlife::Wildlife>,
    player: Vec3,
    play: &mut PlaySide,
    audio: &mut AudioState,
    car_camera: &mut game::CarCamera,
) {
    if let Some(cars) = cars {
        let mut solids_buf = std::mem::take(car_solids_scratch);
        let mut targets_buf = std::mem::take(car_targets_scratch);
        let mut gone_buf = std::mem::take(car_gone_scratch);
        car_ramming(
            dt,
            cars,
            open_trunk,
            car_input,
            physics,
            car_solids,
            &mut solids_buf,
            &mut targets_buf,
            &mut gone_buf,
            streamer,
            z,
            wildlife_opt,
            player,
            play.hour,
            &mut audio.audio,
            play.play.as_mut(),
            &mut audio.car_horn_cooldown,
            car_camera,
            &mut audio.trunk_was_open,
        );
        *car_solids_scratch = solids_buf;
        *car_targets_scratch = targets_buf;
        *car_gone_scratch = gone_buf;

        // Zombies attack parked or occupied cars within reach
        let zombie_attackers: Vec<Vec3> = z
            .list
            .iter()
            .filter(|q| {
                !q.health.is_dead()
                    && matches!(
                        q.brain.state(),
                        game::zombie::ZombieState::Chase | game::zombie::ZombieState::Attack
                    )
            })
            .map(|q| q.position)
            .collect();
        cars.apply_zombie_damage(&zombie_attackers, dt);
    }
}

/// The dog follows, barks and bites zombies and animals (Godot Dog / DogCombat).
pub(super) fn step_dogs(
    dt: f32,
    z: &mut crate::zombies::Zombies,
    dogs: Option<&mut crate::dog_view::DogView>,
    mut wildlife_opt: Option<&mut wildlife::Wildlife>,
    player: Vec3,
    player_ctrl: Option<&game::Player>,
    alive: bool,
    zombie_status: &[(u64, Vec3, bool)],
    animal_status_scratch: &mut Vec<(u64, Vec3, bool)>,
    world: &WorldState,
    play: &mut PlaySide,
    session_seed: i32,
    audio: &mut AudioState,
) {
    // Stray dogs wait at the loaded POI buildings (cached in streamer on chunk load/unload)
    let empty_dog_sites = [];
    let dog_sites: &[game::DogPoiSite] = world
        .streamer
        .as_ref()
        .map(|st| st.dog_sites.as_slice())
        .unwrap_or(&empty_dog_sites);

    if let (Some(d), Some(p)) = (dogs, player_ctrl) {
        let mut animal_status = std::mem::take(animal_status_scratch);
        animal_status.clear();
        if let Some(w) = wildlife_opt.as_deref() {
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
            player_position: Some(player),
            player_forward: game::yaw_forward(p.yaw),
            player_dead: !alive,
            player_sprinting: p.is_sprinting,
            player_velocity: p.velocity,
            zombies: zombie_status,
            animals: &animal_status,
            is_night: !(6.0..20.0).contains(&play.hour),
            camp_position: camp,
            now_hours: (play.clock.day as f64 - 1.0) * 24.0 + play.clock.hour as f64,
            day: play.clock.day,
            poi_sites: dog_sites,
            world_seed: session_seed,
        };
        let f = d.update(dt, &ctx, &world.physics);
        for bite in f.bites {
            match wildlife_opt.as_deref_mut() {
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
        *animal_status_scratch = animal_status;
    }
}
