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
    streamer: Option<&crate::streaming::Streamer>,
    zombies: &mut crate::zombies::Zombies,
    wildlife: Option<&mut crate::wildlife::Wildlife>,
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
        let gone: Vec<glam::IVec2> = cars
            .live_cars()
            .filter(|c| !c.is_driven())
            .map(|c| game::chunk_of(c.body.position, WorldGenerator::CHUNK_SIZE))
            .filter(|k| !st.chunks.contains_key(k))
            .collect();
        for k in gone {
            cars.store_chunk(k);
        }
    }
    let cf = cars.update(dt, car_input, physics);
    let solids: Vec<(String, Mat4, bool)> = cars
        .live_cars()
        .map(|c| (c.id.clone(), c.draw_state().world_matrix, c.is_driven()))
        .collect();
    car_solids.sync(physics, &solids);
    let mut targets: Vec<(u64, Vec3)> = zombies
        .list
        .iter()
        .filter(|q| !q.health.is_dead())
        .map(|q| (q.id, q.position))
        .collect();
    if let Some(w) = wildlife.as_ref() {
        targets.extend(
            w.sim
                .list
                .iter()
                .filter(|a| !a.is_dead() && !a.retired)
                .map(|a| (a.id, a.position)),
        );
    }
    for (id, dmg, kb) in cars.run_over(&targets) {
        match wildlife {
            Some(ref mut w) if id >= crate::wildlife::FIRST_ANIMAL_ID => {
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
