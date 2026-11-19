//! Vehicle simulation and collision ramming against zombies and wildlife.

use glam::{Mat4, Vec3};
use world::WorldGenerator;

// TODO(R11): move to game/ with tests
pub fn car_ramming(
    dt: f32,
    cars: &mut game::Cars,
    open_trunk: Option<&str>,
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
    audio: &mut Option<audio::Audio>,
    play: Option<&mut crate::play::PlayState>,
    car_horn_cooldown: &mut f32,
    car_camera: &mut game::CarCamera,
    trunk_was_open: &mut bool,
) {
    cars.is_night = !(6.0..20.0).contains(&hour);

    // The tailgate: swinging shut (trunk screen closed) gets the door thunk
    let trunk_open_now = open_trunk.is_some();
    if *trunk_was_open
        && !trunk_open_now
        && let Some(a) = audio.as_mut()
    {
        a.play_at("car_door", player_pos);
    }
    *trunk_was_open = trunk_open_now;

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
    // The tailgate swings open for the car whose trunk screen is open
    for c in cars.live_cars_mut() {
        c.trunk_target_open = !c.is_driven() && open_trunk == Some(c.id.as_str());
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
    let mut hits_this_frame = 0usize;
    for (id, dmg, kb) in cars.run_over(targets_scratch) {
        hits_this_frame += 1;
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
    // Impact camera shake feeding off each event
    if cf.crash {
        car_camera.add_shake(0.35 + cf.crash_intensity * 0.65);
    }
    if hits_this_frame > 0 {
        car_camera.add_shake(0.28);
    }
    if let Some(a) = audio.as_mut() {
        // A horn blips once per press, not every frame it is held
        *car_horn_cooldown = (*car_horn_cooldown - dt).max(0.0);
        if cf.horn && *car_horn_cooldown <= 0.0 {
            a.play_at("car_horn", player_pos);
            *car_horn_cooldown = 0.45;
        }
        if cf.crash {
            a.play_at("car_impact", player_pos);
        }
    }
    // Righted the 4x4: audio + toast + a solid camera jolt
    if cf.flip_recovered {
        car_camera.add_shake(0.40);
        if let Some(a) = audio.as_mut() {
            a.play_at("car_impact", player_pos);
        }
        if let Some(play_state) = play {
            play_state.show_toast("Righted the 4x4".into());
        }
    }
}
