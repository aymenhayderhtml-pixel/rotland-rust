use glam::{IVec2, Vec2, Vec3};
use physics::{CharacterMover, PhysicsWorld, CHUNK_SAMPLES};

use game::{Actions, CameraRig, Player, SPRINT_SPEED, WALK_SPEED};

fn setup_flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

#[test]
fn test_walking_forward_moves_at_4_mps() {
    let world = setup_flat_world();
    let mover = CharacterMover::default();
    let mut player = Player::new(Vec3::new(10.0, 0.0, 10.0));

    let dt = 1.0 / 60.0;
    let actions = Actions {
        move_vec: Vec2::new(0.0, -1.0), // forward
        ..Default::default()
    };

    // Step for 1 second (60 frames) to reach terminal walk speed
    for _ in 0..60 {
        player.update(&actions, 0.0, &world, &mover, dt);
    }

    assert!(
        (player.horizontal_speed() - WALK_SPEED).abs() < 0.05,
        "Player should reach 4.0 m/s walking speed, got {}",
        player.horizontal_speed()
    );
    assert!(player.on_floor, "Player should remain on floor");
    assert!(!player.is_sprinting, "Player should not be sprinting");
    // Traveled forward in -Z direction
    assert!(
        player.position.z < 10.0 - 3.5,
        "Player should have traveled ~3.8 m forward, reached z={}",
        player.position.z
    );
}

#[test]
fn test_sprinting_moves_at_7_mps() {
    let world = setup_flat_world();
    let mover = CharacterMover::default();
    let mut player = Player::new(Vec3::new(10.0, 0.0, 10.0));

    let dt = 1.0 / 60.0;
    let actions = Actions {
        move_vec: Vec2::new(0.0, -1.0), // forward
        sprint: true,
        ..Default::default()
    };

    // Step for 1 second (60 frames) to reach terminal sprint speed
    for _ in 0..60 {
        player.update(&actions, 0.0, &world, &mover, dt);
    }

    assert!(
        (player.horizontal_speed() - SPRINT_SPEED).abs() < 0.05,
        "Player should reach 7.0 m/s sprint speed, got {}",
        player.horizontal_speed()
    );
    assert!(player.is_sprinting, "Player should be flagged as sprinting");
    assert!(
        player.stamina.current < 100.0,
        "Stamina should have been consumed by sprinting"
    );
    assert!(
        player.position.z < 10.0 - 6.0,
        "Player should have traveled ~6.7 m forward, reached z={}",
        player.position.z
    );
}

#[test]
fn test_jumping_leaves_the_ground_and_lands() {
    let world = setup_flat_world();
    let mover = CharacterMover::default();
    let mut player = Player::new(Vec3::new(10.0, 0.0, 10.0));

    let dt = 1.0 / 60.0;
    let jump_action = Actions {
        jump: true,
        ..Default::default()
    };
    let idle_action = Actions::default();

    // Frame 1: press jump
    player.update(&jump_action, 0.0, &world, &mover, dt);
    assert_eq!(
        player.velocity.y, 4.8,
        "Initial jump velocity should be 4.8 m/s"
    );

    // Follow ascent: player should leave the ground
    let mut left_ground = false;
    let mut max_y = 0.0f32;
    for _ in 0..30 {
        player.update(&idle_action, 0.0, &world, &mover, dt);
        if !player.on_floor {
            left_ground = true;
        }
        if player.position.y > max_y {
            max_y = player.position.y;
        }
    }

    assert!(left_ground, "Player must leave the ground during jump");
    assert!(
        max_y > 0.8,
        "Apex of jump should reach > 0.8 m, got {}",
        max_y
    );

    // Continue simulating to land back on the ground
    for _ in 0..90 {
        player.update(&idle_action, 0.0, &world, &mover, dt);
    }

    assert!(
        player.on_floor,
        "Player should have landed back on the ground"
    );
    assert!(
        player.position.y.abs() < 0.05,
        "Player position Y should be ~0.0 upon landing, got {}",
        player.position.y
    );
    assert_eq!(
        player.velocity.y, 0.0,
        "Vertical velocity should be zeroed upon landing"
    );
}

#[test]
fn test_player_cannot_walk_through_box() {
    let mut world = setup_flat_world();

    // 2x2x20 box centered at (10.0, 1.0, 10.0) -> West face is at x = 9.0
    let box_points = vec![
        [-1.0, -1.0, -10.0],
        [1.0, -1.0, -10.0],
        [1.0, 1.0, -10.0],
        [-1.0, 1.0, -10.0],
        [-1.0, -1.0, 10.0],
        [1.0, -1.0, 10.0],
        [1.0, 1.0, 10.0],
        [-1.0, 1.0, 10.0],
    ];
    world.add_static(&[box_points], Vec3::new(10.0, 1.0, 10.0), 0.0, 1.0);

    let mover = CharacterMover::default();
    // Start at x = 7.0, facing +X (yaw = -PI/2)
    let mut player = Player::new(Vec3::new(7.0, 0.0, 10.0));
    let camera_yaw = -std::f32::consts::FRAC_PI_2; // looking toward +X

    let dt = 1.0 / 60.0;
    let actions = Actions {
        move_vec: Vec2::new(0.0, -1.0), // forward (+X direction)
        ..Default::default()
    };

    // Walk straight into the box for 2 seconds
    for _ in 0..120 {
        player.update(&actions, camera_yaw, &world, &mover, dt);
        // The box west face is at x = 9.0. Radius is 0.35. Player center cannot penetrate x = 9.0 - 0.35 = 8.65
        assert!(
            player.position.x < 8.7,
            "Player penetrated the box! pos.x = {}, expected < 8.7",
            player.position.x
        );
    }

    assert!(
        player.position.x > 8.5,
        "Player should have reached the box face, got x={}",
        player.position.x
    );
}

#[test]
fn test_camera_pulls_in_when_wall_behind_player() {
    let mut world = setup_flat_world();

    let player_pos = Vec3::new(10.0, 0.0, 10.0);
    let mut rig = CameraRig::new();
    rig.pitch = 0.0; // level view

    // Unobstructed camera
    let cam_free = rig.camera(&player_pos, &world);
    let free_dist =
        (cam_free.position - (player_pos + Vec3::new(0.0, rig.eye_height, 0.0))).length();
    assert!(
        (free_dist - (rig.stored_zoom * rig.stored_zoom + 0.55 * 0.55 + 0.2 * 0.2).sqrt()).abs()
            < 0.1,
        "Free camera should be at full spring arm distance (~4.04 m), got {}",
        free_dist
    );

    // Place a 4x4x1 wall directly behind the player at z = 12.0 (front face at z = 11.5)
    let wall_points = vec![
        [-2.0, 0.0, -0.5],
        [2.0, 0.0, -0.5],
        [2.0, 4.0, -0.5],
        [-2.0, 4.0, -0.5],
        [-2.0, 0.0, 0.5],
        [2.0, 0.0, 0.5],
        [2.0, 4.0, 0.5],
        [-2.0, 4.0, 0.5],
    ];
    world.add_static(&[wall_points], Vec3::new(10.0, 0.0, 12.0), 0.0, 1.0);

    // Camera with wall behind player: spring arm must be pulled in
    let cam_blocked = rig.camera(&player_pos, &world);
    assert!(
        cam_blocked.position.z < 11.5,
        "Camera must be pulled in front of the wall (z < 11.5), got z={}",
        cam_blocked.position.z
    );
    assert!(
        cam_blocked.position.z < cam_free.position.z - 2.0,
        "Camera was not pulled in sufficiently: free z={}, blocked z={}",
        cam_free.position.z,
        cam_blocked.position.z
    );
}
