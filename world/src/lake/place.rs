use glam::{IVec2, Vec2, Vec3};

use crate::generator::WorldGenerator;
use crate::lake::layout::LakeLayout;
use crate::random::DotnetRandom;
use crate::scatter::{PropInstance, ScatterPlacer};

// data: lake.json docks
const DOCK_SECTION_SPACING: f32 = 4.0;
// new: DECISIONS 2026-11-21
const CHUNK_SIZE: f32 = 64.0;

fn in_chunk(p: Vec2, coord: IVec2) -> bool {
    let cx = (p.x / CHUNK_SIZE).floor() as i32;
    let cy = (p.y / CHUNK_SIZE).floor() as i32;
    cx == coord.x && cy == coord.y
}

const BEACH_MODELS: [&str; 4] = [
    "rowboat_flipped",
    "beach_umbrella",
    "beach_chair",
    "fishing_rack",
];

/// Places dock sections, dock ends, boats, buoys, and beach props for one chunk.
pub fn place_lake_props(
    generator: &WorldGenerator,
    lake: &LakeLayout,
    coord: IVec2,
) -> Vec<PropInstance> {
    let mut props = Vec::new();

    // 1. Docks: dock_section every 4m + dock_end at the end
    for dock in &lake.docks {
        let dir = dock.forward();
        let len = dock.length();
        let mut t = 2.0;
        while t < len {
            let p = dock.start + dir * t;
            if in_chunk(p, coord) {
                props.push(PropInstance::new(
                    "dock_section",
                    Vec3::new(p.x, lake.water_level, p.y),
                    dock.yaw,
                    1.0,
                ));
            }
            t += DOCK_SECTION_SPACING;
        }
        if in_chunk(dock.end, coord) {
            props.push(PropInstance::new(
                "dock_end",
                Vec3::new(dock.end.x, lake.water_level, dock.end.y),
                dock.yaw,
                1.0,
            ));
        }

        // 2. Buoy and rowboat on the water near docks
        let perp = Vec2::new(-dir.y, dir.x);
        // Buoy 8m off dock end
        // new: DECISIONS 2026-11-21
        let buoy_pos = dock.end + dir * 8.0;
        if in_chunk(buoy_pos, coord) {
            props.push(PropInstance::new(
                "buoy",
                Vec3::new(buoy_pos.x, lake.water_level, buoy_pos.y),
                0.0,
                1.0,
            ));
        }
        // Rowboat moored 6m to the side near dock end
        // new: DECISIONS 2026-11-21
        let boat_pos = dock.end + perp * 6.0;
        if in_chunk(boat_pos, coord) {
            props.push(PropInstance::new(
                "rowboat",
                Vec3::new(boat_pos.x, lake.water_level, boat_pos.y),
                // new: DECISIONS 2026-11-21
                dock.yaw + 0.2,
                1.0,
            ));
        }
    }

    // 3. Beach props: rowboat_flipped, beach_umbrella, beach_chair, fishing_rack
    // Placed along the beach (~ a few per 100m of beach)
    for (i, &shore_pt) in lake.shore.iter().enumerate() {
        // Every 3rd shore sample (each ~11.4m along shore => ~34m apart => ~3 per 100m)
        if i % 3 != 0 {
            continue;
        }
        let u = (shore_pt - lake.center).normalize();
        // Place in mid-beach:
        // data: lake.json beachWidth
        let beach_offset = lake.config.beach_width * 0.45;
        let p = shore_pt + u * beach_offset;
        if !in_chunk(p, coord) {
            continue;
        }

        // Avoid placing beach props directly over docks
        let too_close_to_dock = lake.docks.iter().any(|d| {
            // new: DECISIONS 2026-11-21
            d.start.distance(p) < 8.0
        });
        if too_close_to_dock {
            continue;
        }

        // new: DECISIONS 2026-11-21
        let seed = ScatterPlacer::stable_seed(generator.seed(), IVec2::new(i as i32, 9417));
        let mut rng = DotnetRandom::new(seed);
        let model_idx = rng.next(BEACH_MODELS.len() as i32) as usize;
        let model = BEACH_MODELS[model_idx];

        let yaw = (-u.x).atan2(-u.y) + (rng.next_double() as f32 - 0.5) * 0.5;
        let y = generator.height(p.x, p.y);
        props.push(PropInstance::new(model, Vec3::new(p.x, y, p.y), yaw, 1.0));
    }

    props
}
