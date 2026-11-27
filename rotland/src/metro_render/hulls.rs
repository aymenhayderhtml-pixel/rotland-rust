//! Collider hulls of towers: one solid upper box per tower (above the walk-in lobby) plus
//! the base model id/pose so its own wall colliders keep the lobby walkable.

use glam::{IVec2, Vec3};
use world::metro::{ClimbTower, DamageSites, StyleDef, TowerSpot, climb_hull_boxes, style_for};

use super::near::{base_point, towers_in_chunk};
use super::parts::base_model_id;

/// Fallback lobby base height when a style is not found in `styles`.
const DEFAULT_BASE_HEIGHT: f32 = 7.6; // data: metro.json baseHeight

/// One upper box hull per tower in chunk `coord`: (base centre, yaw, 8 corner points in tower-local space).
/// Skips towers replaced by major damage set pieces.
#[cfg(test)]
pub fn tower_hulls(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    styles: &[StyleDef],
    damage: &DamageSites,
) -> Vec<(Vec3, f32, Vec<[f32; 3]>)> {
    tower_hulls_with_climb(towers, coord, ground_y, styles, damage, &[])
}

/// Like `tower_hulls`, but climbable towers get `climb_hull_boxes` (hollow stairwell, stop-floor
/// slabs/walls, roof hatch hole) while all other towers keep the single upper box.
pub fn tower_hulls_with_climb(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    styles: &[StyleDef],
    damage: &DamageSites,
    climb: &[ClimbTower],
) -> Vec<(Vec3, f32, Vec<[f32; 3]>)> {
    let mut out = Vec::new();
    for t in towers_in_chunk(towers, coord) {
        let Some(idx) = towers.iter().position(|x| std::ptr::eq(x, t)) else {
            continue;
        };
        if damage.tower_is_replaced(idx) {
            continue;
        }
        let style = style_for(styles, t.style);
        let origin = base_point(t, ground_y);
        if let Some(plan) = climb.iter().find(|c| c.tower == idx)
            && let Some(s) = style
        {
            for [min, max] in climb_hull_boxes(t, s, plan) {
                out.push((origin, t.yaw, aabb_points(min, max)));
            }
        } else {
            let base_y = style.map_or(DEFAULT_BASE_HEIGHT, |s| s.base_height);
            out.push((origin, t.yaw, box_points(t, base_y, roof_top(t, style))));
        }
    }
    out
}

/// Base model id (`tw_<style>_<size>_base`), world position and yaw for each tower in chunk `coord`.
/// Skips towers replaced by major damage set pieces.
pub fn tower_bases(
    towers: &[TowerSpot],
    coord: IVec2,
    ground_y: f32,
    damage: &DamageSites,
) -> Vec<(String, Vec3, f32)> {
    towers_in_chunk(towers, coord)
        .filter_map(|t| {
            let idx = towers.iter().position(|x| std::ptr::eq(x, t))?;
            if damage.tower_is_replaced(idx) {
                return None;
            }
            Some((base_model_id(t), base_point(t, ground_y), t.yaw))
        })
        .collect()
}

/// Collider top: the walkable roof deck for plain crowns (so the player stands on the deck
/// among the roof props and camps), the full height for landmark crowns.
fn roof_top(t: &TowerSpot, style: Option<&StyleDef>) -> f32 {
    match style {
        Some(s) if !t.landmark && s.roof_deck_height > 0.0 => t.height - s.crown_height + s.roof_deck_height,
        _ => t.height,
    }
}

/// The eight corners of a tower's upper footprint box, from `base_height` to `top`.
/// Leaving `0..base_height` open keeps the lobby doorway and interior walkable.
pub fn box_points(t: &TowerSpot, base_height: f32, top: f32) -> Vec<[f32; 3]> {
    aabb_points(
        Vec3::new(-t.half.x, base_height, -t.half.y),
        Vec3::new(t.half.x, top.max(base_height), t.half.y),
    )
}

/// The eight corners of an axis-aligned box `[min, max]`.
pub fn aabb_points(min: Vec3, max: Vec3) -> Vec<[f32; 3]> {
    let mut pts = Vec::with_capacity(8);
    for y in [min.y, max.y] {
        for x in [min.x, max.x] {
            for z in [min.z, max.z] {
                pts.push([x, y, z]);
            }
        }
    }
    pts
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;
    use world::metro::{ClimbKind, StopKind, TowerKind, TowerStyle, default_styles};

    fn test_tower(pos: Vec2, floors: u32) -> TowerSpot {
        TowerSpot {
            pos,
            half: Vec2::splat(12.0),
            yaw: 0.25,
            height: 7.6 + floors as f32 * 3.8 + 2.95,
            floors,
            kind: TowerKind::Midrise,
            landmark: false,
            block: (0, 0),
            style: TowerStyle::Stone,
            size: 24,
        }
    }

    #[test]
    fn non_climb_towers_keep_exact_single_hull_and_climb_gets_multi_box_hull() {
        let styles = default_styles();
        let towers = vec![
            test_tower(Vec2::new(10.0, 10.0), 8),
            test_tower(Vec2::new(40.0, 10.0), 8),
        ];
        let damage = DamageSites::default();
        let plain = tower_hulls(&towers, IVec2::ZERO, 20.0, styles, &damage);
        assert_eq!(plain.len(), 2);

        let plan = ClimbTower {
            tower: 0,
            kind: ClimbKind::Midtown,
            stop_floors: vec![3, 7],
            stop_kinds: vec![StopKind::Office, StopKind::Camp],
            deck_y: 7.6 + 8.0 * 3.8 + 2.95,
        };
        let with_climb = tower_hulls_with_climb(&towers, IVec2::ZERO, 20.0, styles, &damage, &[plan]);
        // Non-climb tower (index 1 at x=40) keeps the exact same single upper box.
        let non_climb_boxes: Vec<_> = with_climb.iter().filter(|(p, _, _)| (p.x - 40.0).abs() < 0.1).collect();
        assert_eq!(non_climb_boxes.len(), 1);
        assert_eq!(*non_climb_boxes[0], plain[1]);

        // Climb tower (index 0 at x=10) gets multiple boxes from climb_hull_boxes.
        let climb_boxes: Vec<_> = with_climb.iter().filter(|(p, _, _)| (p.x - 10.0).abs() < 0.1).collect();
        assert!(climb_boxes.len() > 1);
    }
}
