use glam::{IVec2, Vec3};
use physics::{
    CharacterMover, MoveResult, PhysicsWorld, CHUNK_SAMPLES, CHUNK_SIZE,
};
use world::generator::WorldGenerator;

#[test]
fn test_capsule_dropped_above_flat_chunk_lands_and_reports_on_floor() {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    let _handle = world.add_ground_chunk(IVec2::new(0, 0), &heights);

    let mover = CharacterMover::default();
    let mut pos = Vec3::new(10.0, 5.0, 10.0);
    let mut vel = Vec3::ZERO;
    let gravity = -9.81;
    let dt = 1.0 / 60.0;

    let mut landed = false;
    for _ in 0..120 {
        vel.y += gravity * dt;
        let res: MoveResult = mover.move_and_slide(&world, pos, vel, dt);
        pos = res.position;
        vel = res.velocity;
        if res.on_floor {
            landed = true;
            assert!(
                pos.y.abs() < 0.05,
                "Landed position Y should be ~0.0, got {}",
                pos.y
            );
            assert!(
                res.floor_normal.y > 0.9,
                "Floor normal should point up, got {:?}",
                res.floor_normal
            );
            break;
        }
    }
    assert!(landed, "Capsule dropped above flat chunk should land on it and report on_floor");
}

#[test]
fn test_walking_into_box_slides_and_does_not_pass_through() {
    let mut world = PhysicsWorld::new();
    // Flat ground so the character doesn't fall into the void
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);

    // Box collider: 8 vertices of a 2x2x20 long wall in model space centered around origin
    let box_points = vec![
        [-1.0, 0.0, -10.0],
        [1.0, 0.0, -10.0],
        [1.0, 2.0, -10.0],
        [-1.0, 2.0, -10.0],
        [-1.0, 0.0, 10.0],
        [1.0, 0.0, 10.0],
        [1.0, 2.0, 10.0],
        [-1.0, 2.0, 10.0],
    ];
    // Placed at position (4.0, 0.0, 10.0) with yaw = 0.0, scale = 1.0
    // West face of the box is at x = 3.0 (from x = 4.0 - 1.0) along z from 0.0 to 20.0
    world.add_static(&[box_points], Vec3::new(4.0, 0.0, 10.0), 0.0, 1.0);

    let mover = CharacterMover::default();
    // Start at x = 1.0, z = 5.0
    let mut pos = Vec3::new(1.0, 0.0, 5.0);
    // Move diagonally: +X into the box (at 4 m/s) and +Z along the box face (at 3 m/s)
    let mut vel = Vec3::new(4.0, 0.0, 3.0);
    let dt = 1.0 / 60.0;

    for _ in 0..120 {
        let res = mover.move_and_slide(&world, pos, vel, dt);
        pos = res.position;
        vel = res.velocity;
        // The box west face is at x = 3.0. Character radius is 0.35.
        // The character center must NEVER enter the box (x must stay < 3.0 - 0.35 + margin ~ 2.66).
        assert!(
            pos.x < 2.68,
            "Character passed through box! pos.x = {}, expected < 2.68",
            pos.x
        );
    }

    // Must have slid forward along Z while blocked in X
    assert!(
        pos.z > 10.0,
        "Character should have slid along Z face, pos.z = {}",
        pos.z
    );
    assert!(
        pos.x > 2.5 && pos.x < 2.68,
        "Character should be pressed against box face, pos.x = {}",
        pos.x
    );
}

#[test]
fn test_walking_up_30_degree_ramp_works_60_degree_does_not() {
    let mut world = PhysicsWorld::new();

    // 30 degree ramp convex points (length = 4, height = 4 * tan(30 deg) = 2.309)
    let h30 = 4.0 * 30.0_f32.to_radians().tan();
    let ramp30_points = vec![
        [-2.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, h30, 4.0],
        [-2.0, h30, 4.0],
        [-2.0, 0.0, 4.0],
        [2.0, 0.0, 4.0],
    ];
    world.add_static(&[ramp30_points], Vec3::new(0.0, 0.0, 0.0), 0.0, 1.0);

    // 60 degree ramp convex points (length = 4, height = 4 * tan(60 deg) = 6.928)
    let h60 = 4.0 * 60.0_f32.to_radians().tan();
    let ramp60_points = vec![
        [-2.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, h60, 4.0],
        [-2.0, h60, 4.0],
        [-2.0, 0.0, 4.0],
        [2.0, 0.0, 4.0],
    ];
    world.add_static(&[ramp60_points], Vec3::new(20.0, 0.0, 0.0), 0.0, 1.0);

    let mover = CharacterMover::default();
    let dt = 1.0 / 60.0;

    // 1) Test 30-degree ramp (should climb up)
    let mut pos30 = Vec3::new(0.0, 0.05, 0.2);
    let vel30 = Vec3::new(0.0, -1.0, 3.0);
    for _ in 0..60 {
        let res = mover.move_and_slide(&world, pos30, vel30, dt);
        pos30 = res.position;
    }
    assert!(
        pos30.z > 2.0 && pos30.y > 0.8,
        "Character should walk up 30-degree ramp: reached pos = {:?}",
        pos30
    );

    // 2) Test 60-degree ramp (should NOT climb up)
    let mut pos60 = Vec3::new(20.0, 0.05, 0.2);
    let vel60 = Vec3::new(0.0, -1.0, 3.0);
    for _ in 0..60 {
        let res = mover.move_and_slide(&world, pos60, vel60, dt);
        pos60 = res.position;
    }
    assert!(
        pos60.y < 0.5,
        "Character should NOT climb 60-degree ramp: pos = {:?}",
        pos60
    );
}

#[test]
fn test_ground_height_matches_world_generator() {
    let repo_data = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
    let world_gen = WorldGenerator::from_data_dir(42, repo_data)
        .expect("should load world generator from data dir");

    // The settlement chunk is flattened by SettlementGround::flatten
    let settlement_chunk_coord = IVec2::new(1, -1);
    let chunk = world_gen.build_chunk_ground(settlement_chunk_coord);

    let mut world = PhysicsWorld::new();
    world.add_ground_chunk(settlement_chunk_coord, &chunk.heights);

    let base_x = settlement_chunk_coord.x as f32 * CHUNK_SIZE;
    let base_z = settlement_chunk_coord.y as f32 * CHUNK_SIZE;

    // Test 50 points across the chunk
    for k in 0..50 {
        let x = base_x + 10.0 + (k as f32 * 0.85);
        let z = base_z + 10.0 + (k as f32 * 0.75);

        let gen_height = world_gen.height(x, z);
        let phys_height = world
            .ground_height(x, z)
            .unwrap_or_else(|| panic!("raycast must hit ground at ({x}, {z})"));

        let diff = (gen_height - phys_height).abs();
        assert!(
            diff < 1e-3,
            "ground_height ({}) does not match WorldGenerator::height ({}) within 1e-3 at point ({}, {}), diff={}",
            phys_height,
            gen_height,
            x,
            z,
            diff
        );
    }
}

#[test]
fn test_adding_and_removing_200_chunks_leaves_world_empty() {
    let mut world = PhysicsWorld::new();
    assert!(world.is_empty());
    assert_eq!(world.collider_count(), 0);

    let flat_heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    let mut handles = Vec::with_capacity(200);

    for i in 0..200 {
        let coord = IVec2::new(i % 20, i / 20);
        let handle = world.add_ground_chunk(coord, &flat_heights);
        handles.push(handle);
    }

    assert_eq!(world.collider_count(), 200);
    assert!(!world.is_empty());

    for handle in handles {
        let removed = world.remove(handle);
        assert!(removed, "Handle should be successfully removed");
    }

    assert_eq!(world.collider_count(), 0);
    assert!(world.is_empty(), "World should be empty after removing all chunks");
}
