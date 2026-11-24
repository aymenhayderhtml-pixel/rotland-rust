use game::guild::companion::CompanionAnim;
use game::guild::RecruitRole;
use game::{
    plan_npc_anim, yaw_forward, NpcAnimInput, NpcAnimRole, NpcAnimTrack, TeamAnimConfig,
};
use glam::Vec3;

#[test]
fn data_team_anim_json_loads_from_repo() {
    let cfg = TeamAnimConfig::from_repo().expect("data/team_anim.json should parse");
    assert_eq!(cfg.roles.fighter.weapon, "bat");
    assert_eq!(cfg.roles.fighter.attack_clip, "melee_2h");
    assert!(!cfg.roles.fighter.is_ranged);

    assert_eq!(cfg.roles.guard.weapon, "rifle");
    assert_eq!(cfg.roles.guard.attack_clip, "rifle_fire");
    assert!(cfg.roles.guard.is_ranged);

    assert_eq!(cfg.roles.scavenger.weapon, "knife");
    assert_eq!(cfg.roles.scavenger.attack_clip, "melee_1h");
    assert!(!cfg.roles.scavenger.is_ranged);

    assert_eq!(cfg.roles.camp_guard.weapon, "rifle");
    assert!(cfg.roles.camp_guard.is_ranged);
}

#[test]
fn lod_intervals_follow_distance_and_visibility() {
    let cfg = TeamAnimConfig::from_repo().expect("data/team_anim.json should parse");
    assert_eq!(cfg.lod.interval(5.0, true), 0.0);
    assert_eq!(cfg.lod.interval(15.0, true), 0.04);
    assert_eq!(cfg.lod.interval(40.0, true), 0.1);
    assert_eq!(cfg.lod.interval(15.0, false), 0.5);
    assert_eq!(cfg.lod.interval(35.0, false), 1.0);
}

#[test]
fn melee_teammate_triggers_swing_once_per_attack_and_flinches_on_hit() {
    let cfg = TeamAnimConfig::from_repo().unwrap();
    let mut track = NpcAnimTrack::default();

    let input = NpcAnimInput {
        role: NpcAnimRole::from(RecruitRole::Fighter),
        companion_anim: Some(CompanionAnim::Attack),
        camp_clip: None,
        aiming_at_target: false,
        just_fired: false,
        just_hit: false,
        velocity: Vec3::ZERO,
        yaw: 0.0,
        distance_to_camera: 6.0,
        on_screen: true,
    };
    let p1 = plan_npc_anim(&cfg, &input, &mut track);
    assert_eq!(p1.weapon, Some("bat"));
    assert_eq!(p1.action_clip, Some(("melee_2h", 1.0)));
    assert!(!p1.aiming);
    assert!(!p1.gun_fired);

    // Second frame of same attack does not retrigger one-shot clip
    let p2 = plan_npc_anim(&cfg, &input, &mut track);
    assert_eq!(p2.action_clip, None);

    // Taking a hit triggers flinch clip
    let hit_input = NpcAnimInput {
        just_hit: true,
        ..input
    };
    let p3 = plan_npc_anim(&cfg, &hit_input, &mut track);
    assert_eq!(p3.action_clip, Some(("hit", 1.0)));
}

#[test]
fn ranged_guard_aims_and_fires_with_recoil_and_directional_velocity() {
    let cfg = TeamAnimConfig::from_repo().unwrap();
    let mut track = NpcAnimTrack::default();

    let input = NpcAnimInput {
        role: NpcAnimRole::CampGuard,
        companion_anim: None,
        camp_clip: Some("walk"),
        aiming_at_target: true,
        just_fired: true,
        just_hit: false,
        velocity: Vec3::new(2.0, 0.0, 0.0),
        yaw: 0.0,
        distance_to_camera: 12.0,
        on_screen: true,
    };
    let plan = plan_npc_anim(&cfg, &input, &mut track);
    assert_eq!(plan.weapon, Some("rifle"));
    assert!(plan.aiming);
    assert!(plan.gun_fired);
    assert_eq!(plan.action_clip, Some(("rifle_fire", 1.0)));
    assert_eq!(plan.move_velocity, Vec3::new(2.0, 0.0, 0.0));
    assert!((plan.move_speed - 2.0).abs() < 1e-5);
    assert_eq!(plan.lod_interval, 0.04);
    assert_eq!(plan.held_pose, None);
}

#[test]
fn downed_and_recovery_emit_die_and_reset_alive_once() {
    let cfg = TeamAnimConfig::from_repo().unwrap();
    let mut track = NpcAnimTrack::default();

    let down_input = NpcAnimInput {
        role: NpcAnimRole::Scavenger,
        companion_anim: Some(CompanionAnim::Downed),
        camp_clip: None,
        aiming_at_target: false,
        just_fired: false,
        just_hit: false,
        velocity: Vec3::new(1.0, 0.0, 0.0),
        yaw: 0.0,
        distance_to_camera: 5.0,
        on_screen: true,
    };
    let p1 = plan_npc_anim(&cfg, &down_input, &mut track);
    assert!(p1.play_die);
    assert!(!p1.reset_alive);
    assert_eq!(p1.move_speed, 0.0);

    let p2 = plan_npc_anim(&cfg, &down_input, &mut track);
    assert!(!p2.play_die);
    assert!(!p2.reset_alive);

    let up_input = NpcAnimInput {
        companion_anim: Some(CompanionAnim::Walk),
        velocity: Vec3::ZERO,
        ..down_input
    };
    let p3 = plan_npc_anim(&cfg, &up_input, &mut track);
    assert!(!p3.play_die);
    assert!(p3.reset_alive);
    assert!((p3.move_speed - cfg.walk_speed).abs() < 1e-5);
    assert!((p3.move_velocity - yaw_forward(0.0) * cfg.walk_speed).length() < 1e-4);
}

#[test]
fn resident_non_locomotion_clip_uses_held_pose() {
    let cfg = TeamAnimConfig::from_repo().unwrap();
    let mut track = NpcAnimTrack::default();

    let input = NpcAnimInput {
        role: NpcAnimRole::Resident,
        companion_anim: None,
        camp_clip: Some("sit"),
        aiming_at_target: false,
        just_fired: false,
        just_hit: false,
        velocity: Vec3::ZERO,
        yaw: 0.0,
        distance_to_camera: 35.0,
        on_screen: false,
    };
    let plan = plan_npc_anim(&cfg, &input, &mut track);
    assert_eq!(plan.weapon, None);
    assert_eq!(plan.held_pose, Some("sit"));
    assert_eq!(plan.move_speed, 0.0);
    assert_eq!(plan.lod_interval, 1.0);
}
