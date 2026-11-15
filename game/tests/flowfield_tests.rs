use std::time::Instant;

use glam::{IVec2, Vec3};
use game::FlowField;
use physics::{PhysicsWorld, CHUNK_SAMPLES};

/// Flat ground over a 3 x 3 block of chunks (-64 m .. 192 m), so the 128 m grid around (10, 10) is covered.
fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    for cx in -1..=1 {
        for cz in -1..=1 {
            world.add_ground_chunk(IVec2::new(cx, cz), &heights);
        }
    }
    world
}

/// An axis-aligned solid box from (x0, z0) to (x1, z1), standing on the ground, `h` high.
fn add_box(world: &mut PhysicsWorld, x0: f32, z0: f32, x1: f32, z1: f32, h: f32) {
    let corners = vec![
        [x0, 0.0, z0],
        [x1, 0.0, z0],
        [x1, h, z0],
        [x0, h, z0],
        [x0, 0.0, z1],
        [x1, 0.0, z1],
        [x1, h, z1],
        [x0, h, z1],
    ];
    world.add_static(&[corners], Vec3::ZERO, 0.0, 1.0);
}

/// Four boxes forming a closed square ring (2 m thick) with inner corners (x0, z0) .. (x1, z1).
fn add_ring(world: &mut PhysicsWorld, x0: f32, z0: f32, x1: f32, z1: f32) {
    let t = 2.0;
    add_box(world, x0 - t, z0 - t, x1 + t, z0, 3.0);
    add_box(world, x0 - t, z1, x1 + t, z1 + t, 3.0);
    add_box(world, x0 - t, z0, x0, z1, 3.0);
    add_box(world, x1, z0, x1 + t, z1, 3.0);
}

fn flat_dist(a: Vec3, b: Vec3) -> f32 {
    Vec3::new(a.x - b.x, 0.0, a.z - b.z).length()
}

#[test]
fn open_ground_points_straight_at_target() {
    let world = flat_world();
    let mut field = FlowField::default();
    field.rebuild(&world, Vec3::new(10.5, 0.0, 10.5));

    let east = field.direction(Vec3::new(30.5, 0.0, 10.5)).expect("open ground is reachable");
    assert!((east - Vec3::NEG_X).length() < 1e-4, "straight west expected, got {east:?}");

    let diagonal = field.direction(Vec3::new(40.5, 0.0, 40.5)).expect("open ground is reachable");
    let want = Vec3::new(-1.0, 0.0, -1.0).normalize();
    assert!((diagonal - want).length() < 1e-4, "diagonal expected, got {diagonal:?}");
}

#[test]
fn wall_between_walker_and_target_is_walked_around() {
    let mut world = flat_world();
    // Wall at x 19..21, z -10..30: straight line from the walker to the target goes through it.
    add_box(&mut world, 19.0, -10.0, 21.0, 30.0, 3.0);
    let target = Vec3::new(10.5, 0.0, 10.5);
    let mut field = FlowField::default();
    field.rebuild(&world, target);

    let mut pos = Vec3::new(30.5, 0.0, 10.5);
    for _ in 0..400 {
        if flat_dist(pos, target) <= 1.5 {
            return;
        }
        let dir = field.direction(pos).expect("walker is on reachable ground");
        pos += dir * 0.5;
        let inside_wall = pos.x > 19.0 && pos.x < 21.0 && pos.z > -10.0 && pos.z < 30.0;
        assert!(!inside_wall, "walker went through the wall at {pos:?}");
    }
    panic!("walker did not reach the target within 400 steps; stopped at {pos:?}");
}

#[test]
fn walled_in_target_is_unreachable_from_outside() {
    let mut world = flat_world();
    // Closed ring around (10.5, 10.5): inside x 0..22 and z 0..22 (walls 2 m thick).
    add_ring(&mut world, 2.0, 2.0, 20.0, 20.0);
    let mut field = FlowField::default();
    field.rebuild(&world, Vec3::new(10.5, 0.0, 10.5));

    assert!(
        field.direction(Vec3::new(30.5, 0.0, 10.5)).is_none(),
        "outside the closed ring there must be no direction"
    );
    assert!(
        field.direction(Vec3::new(4.5, 0.0, 10.5)).is_some(),
        "inside the ring the walker can still walk to the target"
    );
    assert!(
        field.direction(Vec3::new(500.0, 0.0, 500.0)).is_none(),
        "outside the grid there must be no direction"
    );
}

#[test]
fn steep_cliff_separates_the_two_sides() {
    // Chunks at cx = 0 get a 4 m step at x = 32 (slope about 63 degrees, over the 46 degree limit).
    let n = CHUNK_SAMPLES;
    let flat = vec![0.0f32; n * n];
    let mut cliff = vec![0.0f32; n * n];
    for j in 0..n {
        for i in 16..n {
            cliff[j * n + i] = 4.0;
        }
    }
    let mut world = PhysicsWorld::new();
    for cx in -1..=1 {
        for cz in -1..=1 {
            let heights = if cx == 0 { &cliff } else { &flat };
            world.add_ground_chunk(IVec2::new(cx, cz), heights);
        }
    }
    let mut field = FlowField::default();
    field.rebuild(&world, Vec3::new(10.5, 0.0, 10.5));

    assert!(field.direction(Vec3::new(20.5, 0.0, 10.5)).is_some(), "low side is reachable");
    assert!(
        field.direction(Vec3::new(50.5, 0.0, 10.5)).is_none(),
        "the step is too steep to climb, so the far side is unreachable"
    );
}

#[test]
fn rebuild_of_a_128_cell_grid_is_fast() {
    let mut world = flat_world();
    for k in 0..40 {
        let x = 5.0 + (k % 8) as f32 * 8.0;
        let z = 5.0 + (k / 8) as f32 * 9.0;
        add_box(&mut world, x, z, x + 2.0, z + 2.0, 2.0);
    }
    let target = Vec3::new(10.5, 0.0, 10.5);
    let mut field = FlowField::default();
    let mut best_ms = f64::MAX;
    for _ in 0..5 {
        let start = Instant::now();
        field.rebuild(&world, target);
        best_ms = best_ms.min(start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("flow field rebuild, 128 x 128 cells, fastest of 5: {best_ms:.2} ms");
    assert!(best_ms < 50.0, "rebuild took {best_ms:.2} ms (limit 50 ms in debug)");
}
