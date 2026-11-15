use std::time::Instant;

use glam::{IVec2, Vec3};
use pack::ModelPack;
use physics::{CharacterMover, PhysicsWorld, CHUNK_SIZE};
use world::generator::WorldGenerator;

fn main() {
    println!("=== Rotland Physics Benchmark: 25 Characters in Forest (600 frames) ===");

    let repo_data = paths::repo_path("data");
    let pack_path = paths::repo_path("rust/assets/models.pack");

    let world_gen = WorldGenerator::from_data_dir(42, &repo_data)
        .expect("should load world generator from data dir");
    let pack = ModelPack::load(&pack_path)
        .expect("should load models.pack from rust/assets/models.pack");

    let forest_coord = world_gen.densest_forest_near(Vec3::ZERO);
    println!("Densest forest chunk near spawn: {:?}", forest_coord);

    let mut world = PhysicsWorld::new();

    // Load 3x3 chunks around the forest chunk
    let mut total_props = 0;
    for dz in -1..=1 {
        for dx in -1..=1 {
            let coord = forest_coord + IVec2::new(dx, dz);
            let ground = world_gen.build_chunk_ground(coord);
            world.add_ground_chunk(coord, &ground.heights);

            let (pois, props) = world_gen.chunk_contents(coord);
            for prop in &props {
                if let Some(model) = pack.find(&prop.model).filter(|m| !m.colliders.is_empty()) {
                    let collider_points: Vec<Vec<[f32; 3]>> = model
                        .colliders
                        .iter()
                        .map(|c| c.points.clone())
                        .collect();
                    world.add_static(&collider_points, prop.position, prop.yaw, prop.scale);
                    total_props += 1;
                }
            }

            for poi in &pois {
                if let Some(model) = pack.find(&poi.building_id).filter(|m| !m.colliders.is_empty()) {
                    let collider_points: Vec<Vec<[f32; 3]>> = model
                        .colliders
                        .iter()
                        .map(|c| c.points.clone())
                        .collect();
                    world.add_static(&collider_points, poi.position, poi.yaw, 1.0);
                    total_props += 1;
                }
            }
        }
    }

    println!(
        "Loaded 9 terrain chunks with {} static colliders in physics world (total colliders: {})",
        total_props,
        world.collider_count()
    );

    // Spawn 25 characters in a 5x5 grid in the central forest chunk
    let center_x = forest_coord.x as f32 * CHUNK_SIZE + CHUNK_SIZE * 0.5;
    let center_z = forest_coord.y as f32 * CHUNK_SIZE + CHUNK_SIZE * 0.5;

    struct Character {
        pos: Vec3,
        vel: Vec3,
    }

    let mut characters = Vec::with_capacity(25);
    for row in 0..5 {
        for col in 0..5 {
            let x = center_x + (col as f32 - 2.0) * 8.0;
            let z = center_z + (row as f32 - 2.0) * 8.0;
            let ground_y = world.ground_height(x, z).unwrap_or(0.0);
            let angle = (row * 5 + col) as f32 * 0.25;
            let speed = 3.5;
            characters.push(Character {
                pos: Vec3::new(x, ground_y + 0.1, z),
                vel: Vec3::new(angle.cos() * speed, 0.0, angle.sin() * speed),
            });
        }
    }

    let mover = CharacterMover::default();
    let dt = 1.0 / 60.0;
    let frames = 600;
    let gravity = -9.81;

    let start = Instant::now();

    for _frame in 0..frames {
        for ch in &mut characters {
            ch.vel.y += gravity * dt;
            let res = mover.move_and_slide(&world, ch.pos, ch.vel, dt);
            ch.pos = res.position;
            ch.vel = res.velocity;
            // Reverse direction if walking too far from center to keep them active in the forest
            let dist_sq = (ch.pos.x - center_x).powi(2) + (ch.pos.z - center_z).powi(2);
            if dist_sq > 28.0 * 28.0 {
                ch.vel.x = -ch.vel.x;
                ch.vel.z = -ch.vel.z;
            }
        }
    }

    let elapsed = start.elapsed();
    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let ms_per_frame = total_ms / frames as f64;
    let us_per_character = (total_ms * 1000.0) / (frames as f64 * characters.len() as f64);

    println!("Simulation finished:");
    println!("  Total time for {} frames: {:.2} ms", frames, total_ms);
    println!("  Time per frame (25 characters): {:.4} ms", ms_per_frame);
    println!("  Time per character step: {:.2} µs", us_per_character);
}
