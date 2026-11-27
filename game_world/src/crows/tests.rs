use glam::Vec3;

use crate::crows::{Circlers, CrowFlock, CrowModel, CrowsConfig, FlockState, flap_frame};

#[test]
fn player_far_keeps_perched_and_near_startles() {
    let cfg = CrowsConfig::default();
    let perch = Vec3::new(100.0, 2.0, 100.0);
    let mut flock = CrowFlock::new(perch, 0.0, 5, 42);

    // Far away: stays perched
    flock.update(0.1, perch + Vec3::new(30.0, 0.0, 0.0), false, None, &cfg);
    assert_eq!(flock.state(), FlockState::Perched);

    // Walking within 12 m: startles
    flock.update(0.1, perch + Vec3::new(10.0, 0.0, 0.0), false, None, &cfg);
    assert_eq!(flock.state(), FlockState::Startled);
}

#[test]
fn gunshot_startle_radius() {
    let cfg = CrowsConfig::default();
    let perch = Vec3::ZERO;
    let far_player = Vec3::new(200.0, 0.0, 0.0);

    let mut flock_far_shot = CrowFlock::new(perch, 0.0, 4, 7);
    flock_far_shot.update(0.1, far_player, false, Some(Vec3::new(70.0, 0.0, 0.0)), &cfg);
    assert_eq!(flock_far_shot.state(), FlockState::Perched);

    let mut flock_near_shot = CrowFlock::new(perch, 0.0, 4, 7);
    flock_near_shot.update(0.1, far_player, false, Some(Vec3::new(50.0, 0.0, 0.0)), &cfg);
    assert_eq!(flock_near_shot.state(), FlockState::Startled);
}

#[test]
fn crows_rise_to_gone_and_return_only_when_player_away() {
    let cfg = CrowsConfig { return_after: [1.0, 1.0], ..CrowsConfig::default() };
    let perch = Vec3::new(10.0, 0.0, 10.0);
    let mut flock = CrowFlock::new(perch, 0.0, 4, 99);

    flock.update(0.1, perch + Vec3::new(5.0, 0.0, 0.0), false, None, &cfg);
    assert_eq!(flock.state(), FlockState::Startled);

    // Advance until Gone
    for _ in 0..250 {
        flock.update(0.1, perch + Vec3::new(5.0, 0.0, 0.0), false, None, &cfg);
        if flock.state() == FlockState::Gone {
            break;
        }
    }
    assert_eq!(flock.state(), FlockState::Gone);
    assert_eq!(flock.instances(&cfg).count(), 0);

    // Wait past return_after while player is still inside 2x startleRadius (24 m): stays Gone
    for _ in 0..20 {
        flock.update(0.1, perch + Vec3::new(18.0, 0.0, 0.0), false, None, &cfg);
    }
    assert_eq!(flock.state(), FlockState::Gone);

    // Move player outside 2x startleRadius: begins Returning, then lands Perched
    flock.update(0.1, perch + Vec3::new(40.0, 0.0, 0.0), false, None, &cfg);
    assert_eq!(flock.state(), FlockState::Returning);

    for _ in 0..200 {
        flock.update(0.1, perch + Vec3::new(40.0, 0.0, 0.0), false, None, &cfg);
        if flock.state() == FlockState::Perched {
            break;
        }
    }
    assert_eq!(flock.state(), FlockState::Perched);
}

#[test]
fn flap_frame_cycles_in_order() {
    let frames: Vec<usize> = (0..5).map(|i| flap_frame(0.0, i as f32 * 0.25, 1.0)).collect();
    assert_eq!(frames, vec![0, 1, 2, 3, 0]);
}

#[test]
fn same_seed_produces_identical_poses() {
    let cfg = CrowsConfig::default();
    let perch = Vec3::new(5.0, 1.0, -3.0);
    let a = CrowFlock::new(perch, 0.5, 6, 12345);
    let b = CrowFlock::new(perch, 0.5, 6, 12345);
    let poses_a: Vec<_> = a.instances(&cfg).collect();
    let poses_b: Vec<_> = b.instances(&cfg).collect();
    assert_eq!(poses_a, poses_b);

    let ca = Circlers::new(Vec3::ZERO, 777, &cfg);
    let cb = Circlers::new(Vec3::ZERO, 777, &cfg);
    let ca_poses: Vec<_> = ca.instances(&cfg).collect();
    let cb_poses: Vec<_> = cb.instances(&cfg).collect();
    assert_eq!(ca_poses, cb_poses);
    assert!(ca_poses.iter().all(|p| matches!(p.model, CrowModel::Fly(_))));
}
