use std::f32::consts::TAU;

use glam::{IVec2, Vec2};
use world::biome::BiomeId;
use world::{GodotNoise, SettlementGround, WorldGenerator, lerp, smooth_step};

fn load_full_world(seed: i32) -> WorldGenerator {
    WorldGenerator::from_data_dir(seed, paths::repo_path("data"))
        .expect("failed to load world from data dir")
}

fn find_biome_point(generator: &WorldGenerator, target: BiomeId, radius: f32) -> Vec2 {
    let camp = generator.settlements()[0].center;
    let steps = 180;
    for i in 0..steps {
        let angle = (i as f32) * TAU / (steps as f32);
        let x = camp.x + radius * angle.cos();
        let z = camp.z + radius * angle.sin();
        let sample = generator.biome_sample(x, z);
        if sample.outer >= 0.99 && sample.primary == target && sample.blend < 0.05 {
            return Vec2::new(x, z);
        }
    }
    panic!("could not find biome {target:?} on ring radius {radius}");
}

#[test]
fn core_points_match_legacy_exact() {
    let seed = 42;
    let generator = load_full_world(seed);
    let camp = generator.settlements()[0].center;

    let legacy_biome = GodotNoise::simplex(seed, 0.0025);
    let legacy_height = GodotNoise::fbm(seed + 1, 0.006, 4);
    let legacy_detail = GodotNoise::simplex(seed + 2, 0.05);

    let legacy_raw = |x: f32, z: f32| -> f32 {
        let b = legacy_biome.get_noise_2d(x, z) * 0.5 + 0.5;
        let fw = smooth_step(0.45, 0.6, b);
        let n = legacy_height.get_noise_2d(x, z);
        lerp(n * 4.0 + 2.0, n * 16.0 + 6.0, fw) + legacy_detail.get_noise_2d(x, z) * 0.4
    };

    let mut checked = 0usize;
    for dz in -10..=10 {
        for dx in -10..=10 {
            let x = camp.x + dx as f32 * 80.0;
            let z = camp.z + dz as f32 * 80.0;
            let sample = generator.biome_sample(x, z);
            assert_eq!(sample.outer, 0.0);
            assert_eq!(generator.tier(x, z), 0);

            let mut expected_h = legacy_raw(x, z);
            if let (Some(cg), Some(city)) = (generator.city_ground(), generator.city()) {
                expected_h = cg.flatten_road(
                    expected_h,
                    x,
                    z,
                    &city.road,
                    generator.settlements(),
                    generator.config(),
                    &legacy_raw,
                );
            }
            expected_h = SettlementGround::flatten(
                expected_h,
                x,
                z,
                generator.settlements(),
                generator.config(),
            );
            if let Some(cg) = generator.city_ground() {
                expected_h = cg.flatten_city(expected_h, x, z);
            }
            if let Some(lake) = generator.lake()
                && lake.contains_region(Vec2::new(x, z))
            {
                expected_h = lake.ground_height(Vec2::new(x, z), expected_h);
            }

            let actual_h = generator.height(x, z);
            assert!(
                (actual_h - expected_h).abs() < 1e-5,
                "core height mismatch at ({x}, {z}): {actual_h} vs {expected_h}"
            );
            checked += 1;
        }
    }
    assert!(checked > 100);
}

#[test]
fn mountains_at_5000m_reach_over_60m() {
    let generator = load_full_world(42);
    let center = find_biome_point(&generator, BiomeId::Mountains, 5000.0);

    let mut max_h = f32::MIN;
    let mut z = -500.0;
    while z <= 500.0 {
        let mut x = -500.0;
        while x <= 500.0 {
            let h = generator.height(center.x + x, center.y + z);
            max_h = max_h.max(h);
            x += 20.0;
        }
        z += 20.0;
    }
    assert!(
        max_h > 60.0,
        "expected mountains peak > 60 m in 1 km scan near {center:?}, got {max_h}"
    );
}

#[test]
fn swamp_stays_under_4m() {
    let generator = load_full_world(42);
    let center = find_biome_point(&generator, BiomeId::Swamp, 5000.0);

    let mut max_h = f32::MIN;
    let mut sampled = 0usize;
    let mut z = -500.0;
    while z <= 500.0 {
        let mut x = -500.0;
        while x <= 500.0 {
            let wx = center.x + x;
            let wz = center.y + z;
            let sample = generator.biome_sample(wx, wz);
            if sample.outer >= 0.99 && sample.primary == BiomeId::Swamp && sample.blend <= 0.01 {
                let h = generator.height(wx, wz);
                max_h = max_h.max(h);
                sampled += 1;
            }
            x += 20.0;
        }
        z += 20.0;
    }
    assert!(sampled > 50, "expected pure swamp samples near {center:?}");
    assert!(
        max_h < 4.0,
        "expected swamp height < 4 m, got {max_h}"
    );
}

#[test]
fn no_seams_across_core_edge_and_sector_borders() {
    let generator = load_full_world(42);
    let camp = generator.settlements()[0].center;
    let step = WorldGenerator::CELL;

    // Radial sweeps from the core center out to 6000 m across core edge and sector borders
    let rays = 16;
    for r_idx in 0..rays {
        let angle = (r_idx as f32) * TAU / (rays as f32);
        let dir = Vec2::new(angle.cos(), angle.sin());
        let mut dist = 0.0;
        let mut prev_h = generator.height(camp.x, camp.z);
        while dist <= 6000.0 {
            dist += step;
            let p = Vec2::new(camp.x, camp.z) + dir * dist;
            let h = generator.height(p.x, p.y);
            let diff = (h - prev_h).abs();
            assert!(
                diff < 3.5,
                "radial seam at dist={dist} angle={angle}: step {diff} >= 3.5 m"
            );
            prev_h = h;
        }
    }

    // Tangential ring sweeps across all sector boundaries at 2500 m and 5000 m
    for radius in [2500.0_f32, 5000.0_f32] {
        let arc_steps = ((TAU * radius) / step).ceil() as usize;
        let mut prev_h = generator.height(camp.x + radius, camp.z);
        for s in 1..=arc_steps {
            let angle = (s as f32) * TAU / (arc_steps as f32);
            let x = camp.x + radius * angle.cos();
            let z = camp.z + radius * angle.sin();
            let h = generator.height(x, z);
            let diff = (h - prev_h).abs();
            assert!(
                diff < 3.5,
                "tangential seam at radius={radius} step={s}: diff {diff} >= 3.5 m"
            );
            prev_h = h;
        }
    }
}

#[test]
fn chunk_edges_match_in_outer_biomes() {
    let generator = load_full_world(42);
    let n = WorldGenerator::SAMPLES;
    let outer_chunks = [
        IVec2::new(78, 0),
        IVec2::new(-78, 45),
        IVec2::new(0, 78),
        IVec2::new(40, -68),
    ];

    for c in outer_chunks {
        let center = generator.build_chunk_ground(c);
        let right = generator.build_chunk_ground(c + IVec2::new(1, 0));
        let down = generator.build_chunk_ground(c + IVec2::new(0, 1));

        for j in 0..n {
            let h0 = center.heights[j * n + (n - 1)];
            let h1 = right.heights[j * n];
            assert!(
                (h0 - h1).abs() < 1e-5,
                "outer horizontal chunk seam at j={j} for {c:?}: {h0} vs {h1}"
            );
        }

        for i in 0..n {
            let h0 = center.heights[(n - 1) * n + i];
            let h1 = down.heights[i];
            assert!(
                (h0 - h1).abs() < 1e-5,
                "outer vertical chunk seam at i={i} for {c:?}: {h0} vs {h1}"
            );
        }
    }
}
