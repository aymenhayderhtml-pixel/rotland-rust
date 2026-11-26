//! Unit tests for `tower_parts` damage mode (`_floor_c` share and burnt-block `All`).

use glam::Vec2;

use crate::metro::styles::{TowerStyle, default_styles, style_for};
use crate::metro::tower_parts::{DamageMode, PartKind, tower_parts, tower_parts_with_damage};
use crate::metro::towers::{TowerKind, TowerSpot};

fn brick_spot(floors: u32) -> TowerSpot {
    TowerSpot {
        pos: Vec2::new(100.0, 200.0),
        half: Vec2::splat(8.0),
        yaw: 0.0,
        height: 7.6 + floors as f32 * 3.8 + 3.5,
        floors,
        kind: TowerKind::Midrise,
        landmark: false,
        block: (0, 0),
        style: TowerStyle::Brick,
        size: 16,
    }
}

#[test]
fn damaged_share_quarter_lands_between_15_and_35_percent() {
    let styles = default_styles();
    let style = style_for(styles, TowerStyle::Brick).expect("brick");
    let spot = brick_spot(20);
    let mut damaged = 0usize;
    let mut total = 0usize;
    for seed in 1..=30u64 {
        let parts = tower_parts_with_damage(&spot, style, 20.0, seed * 7919, DamageMode::Share(0.25));
        for p in parts.iter().filter(|p| p.kind == PartKind::Floor) {
            total += 1;
            if p.model.ends_with("_floor_c") {
                damaged += 1;
            }
        }
    }
    let ratio = damaged as f32 / total as f32;
    assert!((0.15..=0.35).contains(&ratio), "ratio={ratio}");
}

#[test]
fn damage_mode_all_makes_every_floor_damaged() {
    let styles = default_styles();
    let style = style_for(styles, TowerStyle::Brick).expect("brick");
    let spot = brick_spot(10);
    let parts = tower_parts_with_damage(&spot, style, 20.0, 42, DamageMode::All);
    let floors: Vec<_> = parts.iter().filter(|p| p.kind == PartKind::Floor).collect();
    assert_eq!(floors.len(), 10);
    assert!(floors.iter().all(|p| p.model == "tw_brick_16_floor_c"));
}

#[test]
fn damage_mode_zero_gives_no_damaged_floors() {
    let styles = default_styles();
    let style = style_for(styles, TowerStyle::Brick).expect("brick");
    let spot = brick_spot(12);
    let parts = tower_parts_with_damage(&spot, style, 20.0, 99, DamageMode::Share(0.0));
    assert!(parts.iter().all(|p| !p.model.ends_with("_floor_c")));
}

#[test]
fn floors_stack_gap_free_with_damaged_floors() {
    let styles = default_styles();
    let style = style_for(styles, TowerStyle::Brick).expect("brick");
    let spot = brick_spot(8);
    let parts = tower_parts_with_damage(&spot, style, 20.0, 7, DamageMode::All);
    let ys: Vec<f32> = parts
        .iter()
        .filter(|p| p.kind == PartKind::Floor)
        .map(|p| p.pos.y)
        .collect();
    assert_eq!(ys.len(), 8);
    for w in ys.windows(2) {
        assert!((w[1] - w[0] - style.floor_height).abs() < 1e-4);
    }
}

#[test]
fn tower_parts_same_seed_is_deterministic_and_varies_across_seeds() {
    let styles = default_styles();
    let style = style_for(styles, TowerStyle::Brick).expect("brick");
    let spot = brick_spot(16);
    let a = tower_parts(&spot, style, 20.0, 123);
    let b = tower_parts(&spot, style, 20.0, 123);
    assert_eq!(a, b);
    let c = tower_parts(&spot, style, 20.0, 456);
    assert_ne!(a, c);
}
