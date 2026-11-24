use game::{Player, VaultConfig, VaultKind, VaultState, find_vault};
use glam::{IVec2, Vec3};
use physics::{CHUNK_SAMPLES, PhysicsWorld};

fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    world.add_ground_chunk(IVec2::new(0, 0), &heights);
    world
}

/// Adds an axis-aligned box wall with bottom at `base_center.y`.
fn add_box(world: &mut PhysicsWorld, base_center: Vec3, width: f32, height: f32, thickness: f32) {
    let (hx, hz) = (width * 0.5, thickness * 0.5);
    let mut points = Vec::new();
    for px in [-hx, hx] {
        for py in [0.0, height] {
            for pz in [-hz, hz] {
                points.push([px, py, pz]);
            }
        }
    }
    world.add_static(&[points], base_center, 0.0, 1.0);
}

#[test]
fn config_parses_and_equals_default() {
    let loaded = VaultConfig::from_repo().expect("data/vault.json must load");
    assert_eq!(loaded, VaultConfig::default());
}

#[test]
fn one_meter_wall_gives_vault_with_land_beyond() {
    let mut world = flat_world();
    // Player at z = 10 facing -Z; wall front at z = 9.5, back at z = 9.2 (0.3 m thick).
    add_box(&mut world, Vec3::new(10.0, 0.0, 9.35), 4.0, 1.0, 0.3);
    let cfg = VaultConfig::default();
    let feet = Vec3::new(10.0, 0.0, 10.0);

    let plan = find_vault(&world, feet, Vec3::NEG_Z, &cfg).expect("should vault 1 m wall");
    assert_eq!(plan.kind, VaultKind::Vault);
    assert_eq!(plan.kind.clip(), "vault_low");
    assert!(
        plan.land.z < 9.2,
        "land {} must be beyond wall back 9.2",
        plan.land.z
    );
    assert!(plan.land.y.abs() < 1e-3);
}

#[test]
fn low_wall_gives_step_over() {
    let mut world = flat_world();
    add_box(&mut world, Vec3::new(10.0, 0.0, 9.35), 4.0, 0.6, 0.3);
    let cfg = VaultConfig::default();

    let plan = find_vault(&world, Vec3::new(10.0, 0.0, 10.0), Vec3::NEG_Z, &cfg)
        .expect("should step over 0.6 m wall");
    assert_eq!(plan.kind, VaultKind::StepOver);
    assert_eq!(plan.kind.clip(), "step_over");
    assert_eq!(plan.seconds, cfg.step_over_seconds);
}

#[test]
fn unvaultable_obstacles_return_none() {
    let cfg = VaultConfig::default();
    let feet = Vec3::new(10.0, 0.0, 10.0);

    // No wall.
    let open = flat_world();
    assert!(find_vault(&open, feet, Vec3::NEG_Z, &cfg).is_none());

    // 2.0 m tall wall.
    let mut tall = flat_world();
    add_box(&mut tall, Vec3::new(10.0, 0.0, 9.35), 4.0, 2.0, 0.3);
    assert!(find_vault(&tall, feet, Vec3::NEG_Z, &cfg).is_none());

    // 1.0 m tall, 3.0 m thick block.
    let mut thick = flat_world();
    add_box(&mut thick, Vec3::new(10.0, 0.0, 8.0), 4.0, 1.0, 3.0);
    assert!(find_vault(&thick, feet, Vec3::NEG_Z, &cfg).is_none());

    // Valid wall behind the player (facing +Z away from wall at z = 9.35).
    let mut behind = flat_world();
    add_box(&mut behind, Vec3::new(10.0, 0.0, 9.35), 4.0, 1.0, 0.3);
    assert!(find_vault(&behind, feet, Vec3::Z, &cfg).is_none());
}

#[test]
fn try_start_enforces_gates_and_preserves_stamina_on_refusal() {
    let mut world = flat_world();
    add_box(&mut world, Vec3::new(10.0, 0.0, 9.35), 4.0, 1.0, 0.3);
    let cfg = VaultConfig::default();
    let mut state = VaultState::default();

    // Crouching: refused, stamina untouched.
    let mut p = Player::new(Vec3::new(10.0, 0.0, 10.0));
    p.crouching = true;
    let initial_stamina = p.stamina.current;
    assert!(state.try_start(&mut p, &world, 1.0, &cfg).is_none());
    assert_eq!(p.stamina.current, initial_stamina);

    // In air: refused, stamina untouched.
    p.crouching = false;
    p.on_floor = false;
    assert!(state.try_start(&mut p, &world, 1.0, &cfg).is_none());
    assert_eq!(p.stamina.current, initial_stamina);

    // No forward input: refused, stamina untouched.
    p.on_floor = true;
    assert!(state.try_start(&mut p, &world, 0.0, &cfg).is_none());
    assert_eq!(p.stamina.current, initial_stamina);

    // Empty stamina: refused.
    p.stamina.current = 0.0;
    assert!(state.try_start(&mut p, &world, 1.0, &cfg).is_none());
    assert!(!state.is_active());

    // Valid start: spends jump_cost and returns clip.
    p.stamina.current = initial_stamina;
    let clip = state.try_start(&mut p, &world, 1.0, &cfg);
    assert_eq!(clip, Some("vault_low"));
    assert!(state.is_active());
    assert_eq!(p.stamina.current, initial_stamina - p.stamina.jump_cost);
}

#[test]
fn step_reaches_land_clears_wall_and_handles_zero_dt() {
    let mut world = flat_world();
    // Wall spans z in [9.2, 9.5], height 1.0 m.
    add_box(&mut world, Vec3::new(10.0, 0.0, 9.35), 4.0, 1.0, 0.3);
    let cfg = VaultConfig::default();
    let feet = Vec3::new(10.0, 0.0, 10.0);
    let plan = find_vault(&world, feet, Vec3::NEG_Z, &cfg).expect("plan");

    for dt in [1.0 / 60.0_f32, 0.1_f32] {
        let mut p = Player::new(feet);
        let mut state = VaultState::default();
        assert_eq!(
            state.try_start(&mut p, &world, 1.0, &cfg),
            Some("vault_low")
        );

        // dt = 0 changes nothing.
        let before = p.position;
        assert!(state.step(&mut p, 0.0));
        assert_eq!(p.position, before);

        let mut steps = 0;
        while state.step(&mut p, dt) {
            steps += 1;
            assert!(p.position.is_finite());
            if p.position.z <= 9.5 && p.position.z >= 9.2 {
                assert!(
                    p.position.y >= plan.top.y - 0.05,
                    "dipped at z={}: y={} < {}",
                    p.position.z,
                    p.position.y,
                    plan.top.y - 0.05
                );
            }
            assert!(steps < 200, "vault did not finish");
        }

        assert!(!state.is_active());
        assert!(p.on_floor);
        assert!(p.position.distance(plan.land) < 1e-4);
        assert!((p.velocity - Vec3::NEG_Z * cfg.speed_after).length() < 1e-4);
    }
}
