use glam::{IVec2, Vec2, Vec3};
use crate::city::config::District;
use crate::city::layout::CityLayout;
use crate::generator::WorldGenerator;
use crate::random::DotnetRandom;
use crate::scatter::{PropInstance, ScatterPlacer};

// data: city.json lampSpacing
const LAMP_SPACING: f32 = 24.0;
// data: city.json busStopSpacing
const BUS_STOP_SPACING: f32 = 128.0;
// data: city.json roadWreckSpacing
const ROAD_WRECK_SPACING: f32 = 150.0;
// data: city.json sidewalkWidth
const SIDEWALK_WIDTH: f32 = 1.5;
// data: city.json lampSidewalkInset
const LAMP_SIDEWALK_INSET: f32 = 1.0;
// data: city.json minPropDistance
const MIN_PROP_DISTANCE: f32 = 3.0;
// data: city.json roadWreckOffset
const ROAD_WRECK_OFFSET: f32 = 2.0;
// data: city.json blockSize
const BLOCK_SIZE: f32 = 64.0;

struct WeightedChoice {
    model: &'static str,
    weight: u32,
}

// data: city.json blockers
const BLOCKERS: [WeightedChoice; 4] = [
    WeightedChoice { model: "barricade", weight: 3 },
    WeightedChoice { model: "car_wreck_sedan", weight: 2 },
    WeightedChoice { model: "rubble_pile", weight: 2 },
    WeightedChoice { model: "jersey_barrier", weight: 2 },
];
// data: city.json blockers
const TOTAL_BLOCKER_WEIGHT: u32 = 9;

// data: city.json clutter
const CLUTTER: [WeightedChoice; 9] = [
    WeightedChoice { model: "car_wreck", weight: 2 },
    WeightedChoice { model: "car_wreck_sedan", weight: 2 },
    WeightedChoice { model: "car_wreck_van", weight: 1 },
    WeightedChoice { model: "trash_bags", weight: 3 },
    WeightedChoice { model: "dumpster", weight: 2 },
    WeightedChoice { model: "mailbox", weight: 1 },
    WeightedChoice { model: "sign_fallen", weight: 1 },
    WeightedChoice { model: "tire_pile", weight: 1 },
    WeightedChoice { model: "barrel_rusty", weight: 1 },
];
// data: city.json clutter
const TOTAL_CLUTTER_WEIGHT: u32 = 14;

fn pick_weighted(rng: &mut DotnetRandom, choices: &[WeightedChoice], total: u32) -> &'static str {
    let mut roll = rng.next(total as i32) as u32;
    for c in choices {
        if roll < c.weight {
            return c.model;
        }
        roll -= c.weight;
    }
    choices[0].model
}

pub fn is_ground_cover(model: &str) -> bool {
    model.starts_with("grass_")
        || model.starts_with("flowers_")
        || model.starts_with("fern")
        || model.starts_with("mushrooms")
        || model.starts_with("bush")
}

pub fn is_street_prop(model: &str) -> bool {
    model == "street_lamp"
        || model == "bus_stop"
        || model == "fence_broken"
        || model == "barricade"
        || model == "rubble_pile"
        || model == "jersey_barrier"
        || model == "dumpster"
        || model == "trash_bags"
        || model == "mailbox"
        || model == "sign_fallen"
        || model == "tire_pile"
        || model == "barrel_rusty"
        || model.starts_with("car_wreck")
}

fn in_chunk(p: Vec2, coord: IVec2) -> bool {
    let cx = (p.x / BLOCK_SIZE).floor() as i32;
    let cy = (p.y / BLOCK_SIZE).floor() as i32;
    cx == coord.x && cy == coord.y
}

fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

/// Generates street props and road wrecks for one chunk.
pub fn place_street_props(
    generator: &WorldGenerator,
    city: &CityLayout,
    coord: IVec2,
) -> Vec<PropInstance> {
    let mut candidates: Vec<PropInstance> = Vec::new();

    // 1. Street segment props (lamps, blockers, bus stops, clutter)
    for (seg_idx, seg) in city.streets.iter().enumerate() {
        let delta = seg.b - seg.a;
        let seg_len = delta.length();
        if seg_len < 1e-4 {
            continue;
        }
        let dir = delta / seg_len;
        let perp = Vec2::new(-dir.y, dir.x);

        // Street lamps along sidewalks
        // new: DECISIONS 2026-11-21
        let start_offset = (BLOCK_SIZE % LAMP_SPACING) * 0.5;
        let mut t = start_offset;
        let mut lamp_idx = 0;
        while t < seg_len - 1.0 {
            let side = if (seg_idx * 3 + lamp_idx) % 2 == 0 { 1.0 } else { -1.0 };
            let offset = side * (seg.width * 0.5 - LAMP_SIDEWALK_INSET);
            let p = seg.a + dir * t + perp * offset;
            if in_chunk(p, coord) {
                // The lamp model's arm points along its back, so turn it around to hang over the street
                let facing = if side > 0.0 { perp } else { -perp };
                let yaw = dir_yaw(facing);
                let y = generator.height(p.x, p.y);
                candidates.push(PropInstance::new("street_lamp", Vec3::new(p.x, y, p.y), yaw, 1.0));
            }
            t += LAMP_SPACING;
            lamp_idx += 1;
        }

        // Blockers on blocked segments
        if seg.blocked {
            // new: DECISIONS 2026-11-21
            let seed = ScatterPlacer::stable_seed(generator.seed(), IVec2::new(seg_idx as i32, 6928));
            let mut rng = DotnetRandom::new(seed);
            let model_mid = pick_weighted(&mut rng, &BLOCKERS, TOTAL_BLOCKER_WEIGHT);
            // new: DECISIONS 2026-11-21
            let extra = 1 + rng.next(2);

            // Center piece
            // new: DECISIONS 2026-11-21
            let t_mid = 22.0;
            let p_mid = seg.a + dir * t_mid;
            if in_chunk(p_mid, coord) {
                let yaw = dir_yaw(perp);
                let y = generator.height(p_mid.x, p_mid.y);
                candidates.push(PropInstance::new(model_mid, Vec3::new(p_mid.x, y, p_mid.y), yaw, 1.0));
            }

            // Side 1 piece
            // new: DECISIONS 2026-11-21
            let t_s1 = 19.0;
            // new: DECISIONS 2026-11-21
            let off_s1 = -2.4;
            let model_s1 = pick_weighted(&mut rng, &BLOCKERS, TOTAL_BLOCKER_WEIGHT);
            let p_s1 = seg.a + dir * t_s1 + perp * off_s1;
            if in_chunk(p_s1, coord) {
                let yaw = dir_yaw(perp);
                let y = generator.height(p_s1.x, p_s1.y);
                candidates.push(PropInstance::new(model_s1, Vec3::new(p_s1.x, y, p_s1.y), yaw, 1.0));
            }

            // Side 2 piece (if extra == 2)
            if extra == 2 {
                // new: DECISIONS 2026-11-21
                let t_s2 = 25.0;
                // new: DECISIONS 2026-11-21
                let off_s2 = 2.4;
                let model_s2 = pick_weighted(&mut rng, &BLOCKERS, TOTAL_BLOCKER_WEIGHT);
                let p_s2 = seg.a + dir * t_s2 + perp * off_s2;
                if in_chunk(p_s2, coord) {
                    let yaw = dir_yaw(perp);
                    let y = generator.height(p_s2.x, p_s2.y);
                    candidates.push(PropInstance::new(model_s2, Vec3::new(p_s2.x, y, p_s2.y), yaw, 1.0));
                }
            }
        }

        // Bus stop on avenues
        if seg.avenue {
            let is_horizontal = dir.x.abs() > 0.5;
            let seg_step = if is_horizontal {
                ((seg.a.x - city.origin.x) / BLOCK_SIZE).round() as i32
            } else {
                ((seg.a.y - city.origin.y) / BLOCK_SIZE).round() as i32
            };
            // Bus stop every busStopSpacing (128) m
            let segs_per_stop = (BUS_STOP_SPACING / BLOCK_SIZE).round() as i32;
            if segs_per_stop > 0 && seg_step % segs_per_stop == 0 {
                // new: DECISIONS 2026-11-21
                let t = 44.0;
                let side = if (seg_step / segs_per_stop) % 2 == 0 { 1.0 } else { -1.0 };
                let offset = side * (seg.width * 0.5 - SIDEWALK_WIDTH * 0.5);
                let p = seg.a + dir * t + perp * offset;
                if in_chunk(p, coord) {
                    // The shelter model opens to its back side: turn it around so the bench faces the road
                    let facing = if side > 0.0 { perp } else { -perp };
                    let yaw = dir_yaw(facing);
                    let y = generator.height(p.x, p.y);
                    candidates.push(PropInstance::new("bus_stop", Vec3::new(p.x, y, p.y), yaw, 1.0));
                }
            }
        }

        // Clutter on streets/sidewalks
        // new: DECISIONS 2026-11-21
        let clutter_seed = ScatterPlacer::stable_seed(generator.seed(), IVec2::new(seg_idx as i32, 15376));
        let mut clutter_rng = DotnetRandom::new(clutter_seed);
        // data: city.json clutterPerSegment
        let count = 1 + clutter_rng.next(3);
        let mut placed_on_seg: Vec<Vec2> = Vec::new();
        for _ in 0..count {
            let model = pick_weighted(&mut clutter_rng, &CLUTTER, TOTAL_CLUTTER_WEIGHT);
            let is_wreck = model.starts_with("car_wreck");
            let mut accepted_p = None;
            // new: DECISIONS 2026-11-21
            for _ in 0..20 {
                // new: DECISIONS 2026-11-21
                let t = 12.0 + (clutter_rng.next_double() as f32) * 40.0;
                let offset = if is_wreck {
                    let road_half = (seg.width * 0.5 - SIDEWALK_WIDTH).max(1.0);
                    // new: DECISIONS 2026-11-21
                    let usable = (road_half - 0.8).max(0.5);
                    ((clutter_rng.next_double() as f32) * 2.0 - 1.0) * usable
                } else {
                    let side = if clutter_rng.next(2) == 0 { 1.0 } else { -1.0 };
                    side * (seg.width * 0.5 - SIDEWALK_WIDTH * 0.5)
                };
                let p = seg.a + dir * t + perp * offset;

                let mut ok = true;
                for &pos in &placed_on_seg {
                    if (p - pos).length_squared() < MIN_PROP_DISTANCE * MIN_PROP_DISTANCE {
                        ok = false;
                        break;
                    }
                }
                // Avoid lamp and blocker positions
                // new: DECISIONS 2026-11-21
                if ok && (t - 22.0).abs() < 3.5 && offset.abs() < 3.0 { ok = false; }
                // new: DECISIONS 2026-11-21
                if ok && (t - 8.0).abs() < 3.5 && (offset.abs() - (seg.width * 0.5 - LAMP_SIDEWALK_INSET)).abs() < 2.0 { ok = false; }
                // new: DECISIONS 2026-11-21
                if ok && (t - 32.0).abs() < 3.5 && (offset.abs() - (seg.width * 0.5 - LAMP_SIDEWALK_INSET)).abs() < 2.0 { ok = false; }
                // new: DECISIONS 2026-11-21
                if ok && (t - 56.0).abs() < 3.5 && (offset.abs() - (seg.width * 0.5 - LAMP_SIDEWALK_INSET)).abs() < 2.0 { ok = false; }
                // new: DECISIONS 2026-11-21
                if ok && seg.avenue && (t - 44.0).abs() < 3.5 && (offset.abs() - (seg.width * 0.5 - SIDEWALK_WIDTH * 0.5)).abs() < 2.0 { ok = false; }

                if ok {
                    accepted_p = Some(p);
                    placed_on_seg.push(p);
                    break;
                }
            }
            if let Some(p) = accepted_p && in_chunk(p, coord) {
                let yaw = (clutter_rng.next_double() * std::f64::consts::TAU) as f32;
                let y = generator.height(p.x, p.y);
                candidates.push(PropInstance::new(model, Vec3::new(p.x, y, p.y), yaw, 1.0));
            }
        }
    }

    // 2. Broken fence in Outskirts blocks
    for block in &city.blocks {
        if !block.present || block.district != District::Outskirts || block.coord != coord {
            continue;
        }
        // new: DECISIONS 2026-11-21
        let fence_seed = ScatterPlacer::stable_seed(generator.seed(), coord) ^ 20254;
        let mut rng = DotnetRandom::new(fence_seed);
        // data: city.json fenceBrokenPerBlock
        let count = rng.next(3);
        let b_min = Vec2::new(coord.x as f32 * BLOCK_SIZE, coord.y as f32 * BLOCK_SIZE);
        let b_max = b_min + Vec2::splat(BLOCK_SIZE);
        for _ in 0..count {
            let side = rng.next(4);
            // new: DECISIONS 2026-11-21
            let t = 14.0 + (rng.next_double() as f32) * 36.0;
            let (p, yaw) = match side {
                0 => (Vec2::new(b_min.x + t, b_min.y + SIDEWALK_WIDTH), 0.0),
                1 => (Vec2::new(b_min.x + t, b_max.y - SIDEWALK_WIDTH), std::f32::consts::PI),
                2 => (Vec2::new(b_min.x + SIDEWALK_WIDTH, b_min.y + t), std::f32::consts::FRAC_PI_2),
                _ => (Vec2::new(b_max.x - SIDEWALK_WIDTH, b_min.y + t), -std::f32::consts::FRAC_PI_2),
            };
            if in_chunk(p, coord) {
                let y = generator.height(p.x, p.y);
                candidates.push(PropInstance::new("fence_broken", Vec3::new(p.x, y, p.y), yaw, 1.0));
            }
        }
    }

    // 3. Car wrecks along road from camp to city entry
    if city.road.len() >= 2 {
        let start = city.road[0];
        let end = city.road[1];
        let delta = end - start;
        let len = delta.length();
        if len > 0.0 {
            let dir = delta / len;
            let perp = Vec2::new(-dir.y, dir.x);
            let camp_clear = generator.config().clear_radius;
            let mut d = ROAD_WRECK_SPACING;
            let mut idx = 0;
            while d < len {
                let p_center = start + dir * d;
                let side = if idx % 2 == 0 { ROAD_WRECK_OFFSET } else { -ROAD_WRECK_OFFSET };
                let p = p_center + perp * side;
                if (p - start).length() > camp_clear && in_chunk(p, coord) {
                    // new: DECISIONS 2026-11-21
                    let wreck_seed = generator.seed() ^ (31754 + idx * 1013);
                    let mut r = DotnetRandom::new(wreck_seed);
                    let yaw = (r.next_double() * std::f64::consts::TAU) as f32;
                    let y = generator.height(p.x, p.y);
                    candidates.push(PropInstance::new("car_wreck", Vec3::new(p.x, y, p.y), yaw, 1.0));
                }
                d += ROAD_WRECK_SPACING;
                idx += 1;
            }
        }
    }

    // 4. Ensure no prop is within MIN_PROP_DISTANCE (3.0 m) of another
    let mut result: Vec<PropInstance> = Vec::with_capacity(candidates.len());
    for prop in candidates {
        let mut ok = true;
        for existing in &result {
            let dx = prop.position.x - existing.position.x;
            let dz = prop.position.z - existing.position.z;
            if dx * dx + dz * dz < MIN_PROP_DISTANCE * MIN_PROP_DISTANCE - 1e-4 {
                ok = false;
                break;
            }
        }
        if ok {
            result.push(prop);
        }
    }
    result
}
