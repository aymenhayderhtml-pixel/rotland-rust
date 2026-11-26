//! Splits the jam plan into the props of one streaming chunk (pure; the ground height is passed in).

use glam::{IVec2, Vec3};

use crate::metro::jam::JamProp;
use crate::scatter::PropInstance;

/// One chunk is 64 m square, the same grid as the city blocks.
const CHUNK_SIZE_M: f32 = 64.0; // new: DECISIONS 2026-11-26 (one chunk = 64 m, as the city blocks)
/// Jam props are built at full scale.
const JAM_SCALE: f32 = 1.0; // new: DECISIONS 2026-11-26 (full-size props)

/// The jam props whose XZ falls in `coord`. Y is `height(x, z)` plus the plan's own `pos.y`
/// (seat height for bodies, 0 for everything else).
pub fn jam_props_in_chunk(
    plan: &[JamProp],
    coord: IVec2,
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    plan.iter()
        .filter(|p| in_chunk(p.pos.x, p.pos.z, coord))
        .map(|p| {
            let y = height(p.pos.x, p.pos.z) + p.pos.y;
            let position = Vec3::new(p.pos.x, y, p.pos.z);
            PropInstance::new(p.model.clone(), position, p.yaw, JAM_SCALE)
        })
        .collect()
}

/// Same rule as `city::place::in_chunk`: the chunk is `floor(x / 64)`, `floor(z / 64)`.
fn in_chunk(x: f32, z: f32, coord: IVec2) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == coord.x && (z / CHUNK_SIZE_M).floor() as i32 == coord.y
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn prop(model: &str, x: f32, z: f32, seat_y: f32) -> JamProp {
        JamProp { model: model.into(), pos: Vec3::new(x, seat_y, z), yaw: 0.25, pitch: 0.0, roll: 0.0 }
    }

    fn flat_plus_slope(x: f32, z: f32) -> f32 {
        1.0 + 0.1 * x - 0.05 * z
    }

    fn sample_plan() -> Vec<JamProp> {
        vec![
            prop("car_jam_sedan", -0.5, 10.0, 0.0),
            prop("car_jam_hatch", 63.9, 63.9, 0.0),
            prop("car_jam_van", 64.0, 0.0, 0.0),
            prop("body_seated", -70.0, -3.0, 0.6),
            prop("luggage_pile", 130.0, 200.0, 0.0),
            prop("bus_city", 0.0, -0.1, 0.0),
        ]
    }

    #[test]
    fn chunk_edges_follow_the_floor_rule() {
        let plan = sample_plan();
        let in_zero = jam_props_in_chunk(&plan, IVec2::new(0, 0), &flat_plus_slope);
        assert_eq!(in_zero.len(), 1, "x = 63.9 and z = 63.9 are in chunk (0, 0)");
        assert_eq!(in_zero[0].model, "car_jam_hatch");
        let west_edge = jam_props_in_chunk(&plan, IVec2::new(-1, 0), &flat_plus_slope);
        assert_eq!(west_edge[0].model, "car_jam_sedan", "x = -0.5 floors to chunk -1");
        let east = jam_props_in_chunk(&plan, IVec2::new(1, 0), &flat_plus_slope);
        assert_eq!(east.len(), 1);
        assert_eq!(east[0].model, "car_jam_van");
        let west = jam_props_in_chunk(&plan, IVec2::new(-2, -1), &flat_plus_slope);
        assert_eq!(west.len(), 1);
        assert_eq!(west[0].model, "body_seated");
    }

    #[test]
    fn union_over_all_chunks_is_the_whole_plan_once() {
        let plan = sample_plan();
        let mut seen = Vec::new();
        let mut coords = HashSet::new();
        for p in &plan {
            coords.insert(IVec2::new((p.pos.x / CHUNK_SIZE_M).floor() as i32, (p.pos.z / CHUNK_SIZE_M).floor() as i32));
        }
        for coord in coords {
            seen.extend(jam_props_in_chunk(&plan, coord, &flat_plus_slope));
        }
        assert_eq!(seen.len(), plan.len(), "no prop lost or duplicated");
        for p in &plan {
            let hits = seen.iter().filter(|s| s.model == p.model && s.position.x == p.pos.x && s.position.z == p.pos.z).count();
            assert_eq!(hits, 1, "{} appears once", p.model);
        }
    }

    #[test]
    fn height_comes_from_the_ground_plus_the_seat_height() {
        let plan = sample_plan();
        let props = jam_props_in_chunk(&plan, IVec2::new(-2, -1), &flat_plus_slope);
        let body = &props[0];
        let expected = flat_plus_slope(-70.0, -3.0) + 0.6;
        assert!((body.position.y - expected).abs() < 1e-5);
        assert_eq!(body.yaw, 0.25);
        assert_eq!(body.scale, JAM_SCALE);
        let car = jam_props_in_chunk(&plan, IVec2::new(-1, 0), &flat_plus_slope);
        assert!((car[0].position.y - flat_plus_slope(-0.5, 10.0)).abs() < 1e-5);
    }
}
