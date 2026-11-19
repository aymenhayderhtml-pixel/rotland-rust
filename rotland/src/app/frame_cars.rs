//! Vehicle simulation and collision ramming against zombies and wildlife.

use glam::{Mat4, Vec3};
use world::WorldGenerator;

// TODO(R11): move to game/ with tests
pub fn car_ramming(
    dt: f32,
    cars: &mut game::Cars,
    car_input: &game::CarInput,
    physics: &mut physics::PhysicsWorld,
    car_solids: &mut crate::car_solid::ParkedCarSolids,
    solids_scratch: &mut Vec<(String, Mat4, bool)>,
    targets_scratch: &mut Vec<(u64, Vec3)>,
    gone_scratch: &mut Vec<glam::IVec2>,
    streamer: Option<&crate::streaming::Streamer>,
    zombies: &mut crate::zombies::Zombies,
    mut wildlife: Option<&mut crate::wildlife::Wildlife>,
    player_pos: Vec3,
    hour: f32,
    time: f32,
    audio: &mut Option<audio::Audio>,
    play: Option<&mut crate::play::PlayState>,
) {
    cars.is_night = !(6.0..20.0).contains(&hour);
    if let Some(st) = streamer {
        for c in st.chunks.keys() {
            cars.respawn_chunk(*c);
        }
        gone_scratch.clear();
        gone_scratch.extend(
            cars.live_cars()
                .filter(|c| !c.is_driven())
                .map(|c| game::chunk_of(c.body.position, WorldGenerator::CHUNK_SIZE))
                .filter(|k| !st.chunks.contains_key(k)),
        );
        for &k in gone_scratch.iter() {
            cars.store_chunk(k);
        }
    }
    let cf = cars.update(dt, car_input, physics);
    let mut solid_count = 0;
    for (i, c) in cars.live_cars().enumerate() {
        solid_count += 1;
        let mat = c.draw_state().world_matrix;
        let driven = c.is_driven();
        if i < solids_scratch.len() {
            solids_scratch[i].0.clear();
            solids_scratch[i].0.push_str(&c.id);
            solids_scratch[i].1 = mat;
            solids_scratch[i].2 = driven;
        } else {
            solids_scratch.push((c.id.clone(), mat, driven));
        }
    }
    solids_scratch.truncate(solid_count);
    car_solids.sync(physics, solids_scratch);

    targets_scratch.clear();
    targets_scratch.extend(
        zombies
            .list
            .iter()
            .filter(|q| !q.health.is_dead())
            .map(|q| (q.id, q.position)),
    );
    if let Some(w) = wildlife.as_ref() {
        targets_scratch.extend(
            w.sim
                .list
                .iter()
                .filter(|a| !a.is_dead() && !a.retired)
                .map(|a| (a.id, a.position)),
        );
    }
    for (id, dmg, kb) in cars.run_over(targets_scratch) {
        match wildlife.as_deref_mut() {
            Some(w) if id >= crate::wildlife::FIRST_ANIMAL_ID => {
                w.sim.hit(id, dmg, kb);
            }
            _ => {
                zombies.hit(id, dmg, kb);
            }
        }
        if let Some(a) = audio.as_mut() {
            a.play_at("car_impact", player_pos);
        }
    }
    if let Some(a) = audio.as_mut() {
        if cf.horn && ((time * 4.0) as u32).is_multiple_of(4) {
            a.play_at("car_horn", player_pos);
        }
        if cf.crash {
            a.play_at("car_impact", player_pos);
        }
    }
    // The gauge (Godot CarGauge) sits where the E prompt goes while driving
    if let (Some(car), Some(play_state)) = (cars.driven_car(), play) {
        let cfg = &cars.config;
        play_state.set_prompt(game::describe_gauge(
            car.speed_kmh(),
            car.tank.litres(),
            car.tank.fraction(),
            car.condition.current(),
            cfg.health,
            car.condition.is_wrecked(),
            car.tank.is_empty(),
        ));
    }
}
