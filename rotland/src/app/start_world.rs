//! World building, entity spawning, and save restoration for fresh or continued games.

use std::sync::Arc;

use engine::Gpu;
use game::{CameraRig, Player};
use glam::{Mat4, Vec3};
use world::WorldGenerator;

use super::frame_actors::camera_follow;
use super::frame_input::apply_hand;
use super::state::{ActorsState, AudioState, PlaySide, RenderState, SessionState, UiState, WorldState};
use crate::{
    camp_npcs, campfire_mesh, car_solid, combat, dog_view, play, player_visual, wildlife, zombies,
};

/// Builds the world for `session.seed` (and restores a save): ground, props, zombies, player, backpack.
pub fn start_world(
    gpu: &Gpu,
    save_file: Option<save::SaveFile>,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    _ui: &mut UiState,
    session: &mut SessionState,
    audio: &mut AudioState,
) {
    actors.car_camera.deactivate();
    actors.open_trunk = None;
    audio.car_horn_cooldown = 0.0;
    audio.trunk_was_open = false;
    if let Some(a) = audio.audio.as_mut() {
        a.set_car_engine(false, 0.0);
    }
    world.physics = physics::PhysicsWorld::default();
    world.streamer = None;
    actors.zombies = None;
    actors.combat = None;
    play.play = None;
    actors.player_visual = None;
    actors.player_ctrl = None;
    actors.camera_rig = None;
    actors.camp_npcs = None;
    actors.camp_guards = None;
    actors.wildlife = None;
    actors.dogs = None;
    actors.cars = None;
    actors.car_solids = car_solid::ParkedCarSolids::default();
    actors.zombie_voices = Some(game::ZombieVoices::new(session.seed));
    world.poi_zombies = Some(game::PoiZombies::new(session.seed));
    world.trees = game::TreeFelling::new();
    world.trees.set_seed(session.seed as u64);
    world.falling.clear();
    play.skill_seen.clear();
    session.play_time = 0.0;
    if session.scene == "play" && !play.clock_frozen {
        play.clock = world::time::Clock::new();
        play.hour = play.clock.hour;
    }
    let started = std::time::Instant::now();
    // A broken data file stops the game with a message naming the file, not a crash (CRASH-002)
    let world_gen = match game::load_world_generator(session.seed, paths::repo_path("data")) {
        Ok(g) => g,
        Err(err) => {
            log::error!("{err}");
            eprintln!("Rotland can't start the world: {err}");
            std::process::exit(1);
        }
    };
    let config = world_gen.config().clone();
    let camp = world_gen.settlements()[0].clone();
    let spawn = camp.respawn_point(&config);
    actors.player = if session.scene == "forest" {
        // The same spot as Godot's benchmark: centre of the densest forest chunk near spawn
        let c = world_gen.densest_forest_near(spawn);
        let (x, z) = (
            (c.x as f32 + 0.5) * WorldGenerator::CHUNK_SIZE,
            (c.y as f32 + 0.5) * WorldGenerator::CHUNK_SIZE,
        );
        Vec3::new(x, world_gen.height(x, z), z)
    } else {
        spawn
    };

    if session.scene == "play" {
        let mut rig = CameraRig::new();
        let mut player = Player::new(actors.player);
        // Start facing where the camera looks (away from it), not toward it
        player.yaw = rig.yaw;
        actors.player_ctrl = Some(player);
        if session.test_first_person {
            rig.is_first_person = true;
        }
        actors.camera_rig = Some(rig);
    }

    if let Some(pack) = &render.pack {
        match game::zombie::ZombieDefs::load_default() {
            Ok(defs) => actors.zombies = Some(zombies::Zombies::new(&gpu.device, pack, defs)),
            Err(e) => log::warn!("data/zombies.json: {e}"),
        }
        if session.scene == "play" {
            match wildlife::Wildlife::new(&gpu.device, pack, session.seed) {
                Ok(w) => actors.wildlife = Some(w),
                Err(e) => log::warn!("data/animals.json: {e}"),
            }
            match game::Cars::from_repo() {
                Ok(mut c) => {
                    c.seed = session.seed;
                    actors.cars = Some(c);
                    actors.car_solids = car_solid::ParkedCarSolids::from_pack(pack);
                    render.car_markers = crate::car_visuals::CarMarkers::from_pack(pack);
                    render.car_emissives = None;
                    render.car_smoke.clear();
                }
                Err(e) => log::warn!("data/vehicles.json: {e}"),
            }
            match dog_view::DogView::new(&gpu.device, pack) {
                Ok(d) => actors.dogs = Some(d),
                Err(e) => log::warn!("data/dog.json: {e}"),
            }
            actors.combat = Some(combat::Combat::new("bat"));
            match play::PlayState::new(session.seed as u64) {
                Ok(state) => play.play = Some(state),
                Err(e) => log::warn!("play systems not loaded: {e}"),
            }
            // Godot's starter weapon is the bat (data/start_inventory.json)
            actors.player_visual = Some(player_visual::PlayerVisual::load(
                &gpu.device,
                pack,
                "human_player",
                Some("bat"),
            ));
        }
    }
    // The camp's people (play scene)
    if let (Some(pack), true) = (&render.pack, session.scene == "play") {
        match game::CampLifeDirector::from_data_dir(camp.clone(), paths::repo_path("data")) {
            Ok(director) => {
                actors.camp_npcs = Some(camp_npcs::CampNpcs::new(&gpu.device, pack, director))
            }
            Err(e) => log::warn!("camp life not loaded: {e}"),
        }
        match game::CampGuards::from_repo() {
            Ok(g) => actors.camp_guards = Some(g.with_camp_center(camp.center).with_seed(session.seed)),
            Err(e) => log::warn!("camp guards not loaded: {e}"),
        }
    }
    // A saved game: the player where they were, the clock, the backpack and the emptied crates
    if let Some(file) = &save_file {
        if let (Some(r), Some(p), Some(rig)) = (
            crate::session::restore(file),
            actors.player_ctrl.as_mut(),
            actors.camera_rig.as_mut(),
        ) {
            p.position = r.position;
            p.yaw = r.body_yaw;
            rig.yaw = r.camera_yaw;
            rig.pitch = r.camera_pitch;
            actors.player = r.position;
            // --hour (screenshots) wins over the saved hour
            if !play.clock_frozen {
                play.clock.set_time(r.hour);
                play.hour = play.clock.hour;
            }
            play.clock.day = r.day.max(1);
            session.play_time = r.play_time;
            if let Some(c) = actors.combat.as_mut() {
                c.health.set_current(r.health);
            }
        }
        if let (Some(c), Some(v)) = (actors.cars.as_mut(), file.systems.extra.get("cars")) {
            c.from_save(v);
        }
        if let Some(ws) = &file.systems.worldstate {
            world
                .trees
                .from_save(&serde_json::json!({ "fallen_trees": ws.fallen_trees }));
            if let Some(pz) = world.poi_zombies.as_mut() {
                pz.from_save(&serde_json::json!({ "visited_pois": ws.visited_pois }));
            }
        }
        if let (Some(d), Some(v)) = (actors.dogs.as_mut(), file.systems.extra.get("dog")) {
            d.sim.from_save(v);
        }
        if let Some(play_state) = play.play.as_mut() {
            let warnings = crate::session::restore_play(file, play_state);
            for w in warnings {
                play_state.show_toast(w);
            }
            // Parity: restore saved weapon if still carried in inventory
            if let Some(weapons) = &file.systems.weapons
                && !weapons.equipped.is_empty()
                && play_state.inventory.count(&weapons.equipped) > 0
                && let Some(weapon_def) = play_state.weapons.get(&weapons.equipped).cloned() {
                    apply_hand(
                        crate::play::HandChange::Weapon(Some(weapon_def)),
                        render,
                        play,
                        actors,
                        world,
                    );
                }
        }
    }
    actors.climb = game::LadderClimb::default();
    // The camp's parked car outside the gate, half a tank (Godot CarManager.EnsureCampCar)
    if let (Some(cars), Some(site)) = (actors.cars.as_mut(), world_gen.settlements().first())
        && !cars.has_record(game::CAMP_CAR_ID)
    {
        let local = site.center + site.local(7.0, 0.0, -27.0);
        let at = Vec3::new(local.x, world_gen.height(local.x, local.z) + 1.0, local.z);
        let cfg = cars.config.clone();
        cars.spawn_car(
            game::CAMP_CAR_ID,
            &cfg.camp_colour,
            at,
            site.yaw,
            cfg.camp_fuel_frac * cfg.tank_litres,
            cfg.health,
        );
    }
    if render.campfire_meshes.is_none() {
        render.campfire_meshes = Some(campfire_mesh::CampfireMeshes::new(&gpu.device));
    }
    // The camp's four beds (a loaded save may already have them) and its fire (the model's marker)
    if let Some(play_state) = play.play.as_mut() {
        if let (Some(pack), Some(site)) = (render.pack.as_ref(), world_gen.settlements().first())
            && let Some(m) = pack.find("settlement")
            && let Some(fire) = game::settlement_campfire_from_markers(
                &m.markers,
                Mat4::from_translation(site.center) * Mat4::from_rotation_y(site.yaw),
            )
        {
            play_state.camp.campfires.add(fire);
        }
        play_state.camp.farm.ensure_camp_plots(world_gen.settlements());
        for (id, n) in &session.give {
            play_state.inventory.add(id, *n);
        }
    }
    if let (Some(yaw), Some(p), Some(rig)) = (
        session.start_yaw,
        actors.player_ctrl.as_mut(),
        actors.camera_rig.as_mut(),
    ) {
        p.yaw = yaw;
        rig.yaw = yaw;
    }
    // --animals=...,campfire also lays a burning fire 4 m ahead (screenshots)
    if let (true, Some(play_state), Some(p)) = (
        session.near_animals.iter().any(|a| a == "campfire"),
        play.play.as_mut(),
        actors.player_ctrl.as_ref(),
    ) {
        let at = actors.player + game::yaw_forward(p.yaw) * 4.0;
        play_state.camp.campfires.add(game::Campfire::create(Vec3::new(
            at.x,
            world_gen.height(at.x, at.z),
            at.z,
        )));
    }
    session.near_animals.retain(|a| a != "campfire");
    // ...and `dog` gives the player a grown dog beside them
    if let (true, Some(d), Some(p)) = (
        session.near_animals.iter().any(|a| a == "dog"),
        actors.dogs.as_mut(),
        actors.player_ctrl.as_ref(),
    ) {
        let at = actors.player + game::yaw_right(p.yaw) * 2.0;
        let health = d.sim.config.max_health;
        d.sim.spawn_dog(
            Vec3::new(at.x, world_gen.height(at.x, at.z), at.z),
            health,
            game::DogMode::Follow,
            "Rex",
            play.clock.day,
            game::DogGrowth::NO_START,
        );
    }
    session.near_animals.retain(|a| a != "dog");
    // ...and `drive` seats the player in the camp's car (with --autowalk it drives off)
    if let (true, Some(cars)) = (
        session.near_animals.iter().any(|a| a == "drive"),
        actors.cars.as_mut(),
    ) && let Some(car) = cars.get_mut(game::CAMP_CAR_ID)
    {
        car.set_driven(true);
        cars.driven_car_id = Some(game::CAMP_CAR_ID.to_string());
    }
    session.near_animals.retain(|a| a != "drive");
    if let (Some(w), Some(p)) = (actors.wildlife.as_mut(), actors.player_ctrl.as_ref()) {
        let ahead = game::yaw_forward(p.yaw);
        let side = game::yaw_right(p.yaw);
        let n = session.near_animals.len() as f32;
        for (i, species) in session.near_animals.iter().enumerate() {
            let at = actors.player + ahead * 9.0 + side * (i as f32 - (n - 1.0) / 2.0) * 3.0;
            w.sim.spawn_at(
                species,
                Vec3::new(at.x, world_gen.height(at.x, at.z), at.z),
                None,
                None,
            );
        }
    }
    session.loaded_save = save_file;
    let world_gen = Arc::new(world_gen);
    let mut streamer = crate::streaming::Streamer::new(world_gen.clone());
    streamer.felled = world
        .trees
        .fallen
        .iter()
        .filter_map(|id| game::parse_tree_id(id))
        .collect();
    // Load the first ring before the first frame (Godot holds the player still until the ground is ready)
    if let Some(pack) = &render.pack {
        while !streamer.is_ready_around(actors.player) {
            streamer.update(
                &gpu.device,
                pack,
                &mut render.library,
                &mut world.physics,
                actors.player,
            );
            std::thread::yield_now();
        }
    }
    // Zombies need the ground's collision, so they spawn after the first ring has loaded. Like the
    // forest25 benchmark: 25 walkers 15-40 m around the player (the real spawner comes in R3-07).
    // In play mode the spawner fills the world over time instead (--zombies=N still forces a ring)
    let forced = session.forced_zombies;
    let ring = if session.scene == "play" {
        forced.unwrap_or(0)
    } else {
        forced.unwrap_or(25)
    };
    if let (Some(z), true) = (actors.zombies.as_mut(), ring > 0) {
        let count = ring;
        z.spawn_ring("walker", actors.player, count, 15.0, 40.0, &world.physics);
    }
    let has_weapon = play.play.as_ref().is_some_and(|p| p.gun.is_some())
        || actors.combat.as_ref().is_some_and(|c| c.weapon_id().is_some());
    if !has_weapon
        && let Some(change) = play.play.as_mut().and_then(|p| p.equip_first())
    {
        apply_hand(change, render, play, actors, world);
    }
    log::info!(
        "scene {} at {:?}: {} chunks, {} placed models, {} zombies, ready in {:.0} ms",
        session.scene,
        actors.player,
        streamer.chunks.len(),
        streamer
            .chunks
            .values()
            .map(|c| c.placed.len())
            .sum::<usize>(),
        actors.zombies.as_ref().map_or(0, |z| z.list.len()),
        started.elapsed().as_secs_f64() * 1000.0
    );
    world.streamer = Some(streamer);
    world.world_gen = Some(world_gen);
    if session.scene == "play" && !session.free_cam {
        if let (Some(rig), Some(p)) = (actors.camera_rig.as_ref(), actors.player_ctrl.as_ref()) {
            render.camera = rig.camera(&p.position, &world.physics);
        }
    } else if session.walk {
        camera_follow(&mut render.camera, actors.player, 0.0);
    } else {
        render.camera.position = actors.player + Vec3::new(0.0, 1.6, 0.0);
        render.camera.look_at(camp.center + Vec3::new(0.0, 1.2, 0.0));
    }
}
