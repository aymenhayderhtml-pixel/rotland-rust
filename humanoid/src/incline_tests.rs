use glam::Vec3;

use crate::anim_config::AnimConfig;
use crate::incline::{InclineConfig, InclineState, STAIRS_DOWN_CLIP, STAIRS_UP_CLIP};
use crate::layers::{LayerId, LayerStack};
use crate::mask::MaskId;

const DT_60: f32 = 1.0 / 60.0; // new: DECISIONS 2026-11-24
const DT_30: f32 = 1.0 / 30.0; // new: DECISIONS 2026-11-24
const DT_120: f32 = 1.0 / 120.0; // new: DECISIONS 2026-11-24
const WALK_SPEED: f32 = 3.0; // data: incline.json tempoRefSpeed
const FULL_GRADE: f32 = 0.6; // data: incline.json fullGrade
const MAX_WEIGHT: f32 = 0.85; // data: incline.json maxWeight
const BUMP_HEIGHT: f32 = 0.2; // new: DECISIONS 2026-11-24
const BUMP_MAX_WEIGHT: f32 = 0.1; // new: DECISIONS 2026-11-24
const FPS_TOLERANCE: f32 = 0.03; // new: DECISIONS 2026-11-24

fn walk_steps(st: &mut InclineState, pos: &mut Vec3, speed: f32, grade: f32, dt: f32, steps: usize) {
    for _ in 0..steps {
        let dh = speed * dt;
        pos.x += dh;
        pos.y += dh * grade;
        st.set_input(*pos, true);
        st.tick(dt);
    }
}

#[test]
fn incline_json_parses_and_matches_defaults() {
    let text = std::fs::read_to_string(paths::repo_path("data/incline.json"))
        .expect("data/incline.json should exist");
    let parsed = InclineConfig::from_json(&text).expect("data/incline.json should parse");
    assert_eq!(parsed, InclineConfig::default());
}

#[test]
fn stairs_clips_exist_in_poses_climb() {
    let text = std::fs::read_to_string(paths::repo_path("tools/blender/characters/poses_climb.py"))
        .expect("poses_climb.py should exist");
    assert!(text.contains(&format!("\"{STAIRS_UP_CLIP}\"")));
    assert!(text.contains(&format!("\"{STAIRS_DOWN_CLIP}\"")));
}

#[test]
fn not_driven_stays_zero() {
    let mut st = InclineState::with_config(InclineConfig::default());
    for _ in 0..60 {
        st.tick(DT_60);
    }
    assert_eq!(st.grade(), 0.0);
    assert_eq!(st.up_weight(), 0.0);
    assert_eq!(st.down_weight(), 0.0);
    assert_eq!(st.tempo(WALK_SPEED), 0.0);
    assert!(!st.wants_incline());
}

#[test]
fn flat_walking_stays_zero() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    walk_steps(&mut st, &mut pos, WALK_SPEED, 0.0, DT_60, 120);
    assert_eq!(st.grade(), 0.0);
    assert_eq!(st.up_weight(), 0.0);
    assert_eq!(st.down_weight(), 0.0);
}

#[test]
fn walking_up_reaches_max_weight_on_up_only() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    walk_steps(&mut st, &mut pos, WALK_SPEED, FULL_GRADE, DT_60, 180);
    assert!((st.grade() - FULL_GRADE).abs() < 0.01);
    assert!((st.up_weight() - MAX_WEIGHT).abs() < 0.01);
    assert_eq!(st.down_weight(), 0.0);
    assert!((st.tempo(WALK_SPEED) - 1.0).abs() < 0.001);
    assert!(st.tempo(4.5) > st.tempo(1.5)); // new: DECISIONS 2026-11-24
}

#[test]
fn walking_down_drives_down_only_and_reversal_stays_exclusive() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    walk_steps(&mut st, &mut pos, WALK_SPEED, -FULL_GRADE, DT_60, 180);
    assert!((st.grade() + FULL_GRADE).abs() < 0.01);
    assert!((st.down_weight() - MAX_WEIGHT).abs() < 0.01);
    assert_eq!(st.up_weight(), 0.0);

    for _ in 0..180 {
        walk_steps(&mut st, &mut pos, WALK_SPEED, FULL_GRADE, DT_60, 1);
        assert!(!(st.up_weight() > 0.0 && st.down_weight() > 0.0));
    }
    assert!((st.up_weight() - MAX_WEIGHT).abs() < 0.01);
    assert_eq!(st.down_weight(), 0.0);
}

#[test]
fn single_bump_on_flat_ground_stays_below_threshold() {
    for dt in [DT_60, DT_30] {
        let mut st = InclineState::with_config(InclineConfig::default());
        let mut pos = Vec3::ZERO;
        st.set_input(pos, true);
        walk_steps(&mut st, &mut pos, WALK_SPEED, 0.0, dt, 30);
        pos.x += WALK_SPEED * dt;
        pos.y += BUMP_HEIGHT;
        st.set_input(pos, true);
        st.tick(dt);
        let mut peak = st.up_weight().max(st.down_weight());
        for _ in 0..60 {
            walk_steps(&mut st, &mut pos, WALK_SPEED, 0.0, dt, 1);
            peak = peak.max(st.up_weight()).max(st.down_weight());
        }
        assert!(peak < BUMP_MAX_WEIGHT, "bump peak {peak} at dt={dt}");
    }
}

#[test]
fn standing_still_and_leaving_floor_fade_to_zero() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    walk_steps(&mut st, &mut pos, WALK_SPEED, FULL_GRADE, DT_60, 120);
    assert!(st.up_weight() > 0.7); // new: DECISIONS 2026-11-24
    for _ in 0..120 {
        st.set_input(pos, true);
        st.tick(DT_60);
    }
    assert!(st.up_weight() < 0.01); // new: DECISIONS 2026-11-24

    walk_steps(&mut st, &mut pos, WALK_SPEED, FULL_GRADE, DT_60, 120);
    assert!(st.up_weight() > 0.7); // new: DECISIONS 2026-11-24
    for _ in 0..120 {
        pos.x += WALK_SPEED * DT_60;
        pos.y += WALK_SPEED * FULL_GRADE * DT_60;
        st.set_input(pos, false);
        st.tick(DT_60);
    }
    assert!(st.up_weight() < 0.01); // new: DECISIONS 2026-11-24
}

#[test]
fn teleport_resets_without_spike() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    walk_steps(&mut st, &mut pos, WALK_SPEED, 0.0, DT_60, 20);
    pos += Vec3::new(1.0, 2.0, 0.0); // new: DECISIONS 2026-11-24
    st.set_input(pos, true);
    st.tick(DT_60);
    assert_eq!(st.grade(), 0.0);
    assert_eq!(st.up_weight(), 0.0);
    assert_eq!(st.down_weight(), 0.0);
}

#[test]
fn framerate_independent_30_vs_120_fps() {
    let mut st30 = InclineState::with_config(InclineConfig::default());
    let mut st120 = InclineState::with_config(InclineConfig::default());
    let (mut p30, mut p120) = (Vec3::ZERO, Vec3::ZERO);
    st30.set_input(p30, true);
    st120.set_input(p120, true);
    walk_steps(&mut st30, &mut p30, WALK_SPEED, FULL_GRADE, DT_30, 15);
    walk_steps(&mut st120, &mut p120, WALK_SPEED, FULL_GRADE, DT_120, 60);
    assert!((st30.grade() - st120.grade()).abs() < FPS_TOLERANCE);
    assert!((st30.up_weight() - st120.up_weight()).abs() < FPS_TOLERANCE);
}

#[test]
fn finite_at_zero_and_large_dt_and_layer_binding() {
    let mut st = InclineState::with_config(InclineConfig::default());
    let mut pos = Vec3::ZERO;
    st.set_input(pos, true);
    for dt in [0.0_f32, -0.05, f32::NAN, 0.1] {
        pos.x += WALK_SPEED * dt.max(0.0);
        pos.y += WALK_SPEED * FULL_GRADE * dt.max(0.0);
        st.set_input(pos, true);
        st.tick(dt);
        assert!(st.grade().is_finite());
        assert!(st.up_weight().is_finite());
        assert!(st.down_weight().is_finite());
        assert!(st.tempo(WALK_SPEED).is_finite());
        assert!(st.tempo(0.0).is_finite());
        assert!(st.tempo(f32::NAN).is_finite());
    }

    let mut layers = LayerStack::new(AnimConfig::load_shared());
    walk_steps(&mut st, &mut pos, WALK_SPEED, FULL_GRADE, DT_60, 120);
    st.apply_air(&mut layers);
    let air = layers.get(LayerId::Air);
    assert_eq!(air.mask, MaskId::Lower);
    assert_eq!(air.clip.as_deref(), Some(STAIRS_UP_CLIP));
    assert!(air.weight > 0.7); // new: DECISIONS 2026-11-24
    st.release_air(&mut layers);
    let air = layers.get(LayerId::Air);
    assert_eq!(air.mask, MaskId::Full);
    assert_eq!(air.weight, 0.0);
}
