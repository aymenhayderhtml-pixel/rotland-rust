use glam::IVec2;
use world::{BiomeId, SettlementConfig, SettlementConfigData, WorldGenerator};

fn load_config() -> SettlementConfigData {
    let candidates = [
        "data/settlements.json",
        "../data/settlements.json",
    ];
    for c in candidates {
        let p = std::path::Path::new(c);
        if p.exists() {
            return SettlementConfig::load_from_file(p).expect("failed to load settlements.json");
        }
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = std::path::PathBuf::from(dir).join("../data/settlements.json");
        if p.exists() {
            return SettlementConfig::load_from_file(p).expect("failed to load settlements.json");
        }
    }
    panic!("Could not find data/settlements.json");
}

#[test]
fn test_determinism() {
    let cfg = load_config();
    let gen1 = WorldGenerator::new(42, cfg.clone());
    let gen2 = WorldGenerator::new(42, cfg);

    // Settlements match
    assert_eq!(gen1.settlements().len(), gen2.settlements().len());
    assert_eq!(gen1.settlements()[0].center, gen2.settlements()[0].center);

    // Points match exactly
    for z in -5..=5 {
        for x in -5..=5 {
            let px = x as f32 * 37.5;
            let pz = z as f32 * 41.2;
            let (h1, f1) = gen1.sample(px, pz);
            let (h2, f2) = gen2.sample(px, pz);
            assert_eq!(h1, h2, "height mismatch at ({px}, {pz})");
            assert_eq!(f1, f2, "forest weight mismatch at ({px}, {pz})");
            assert_eq!(gen1.biome(px, pz), gen2.biome(px, pz));
            assert_eq!(gen1.raw_height(px, pz), gen2.raw_height(px, pz));
        }
    }

    // Chunk ground matches exactly
    let chunk1 = gen1.build_chunk_ground(IVec2::new(1, -2));
    let chunk2 = gen2.build_chunk_ground(IVec2::new(1, -2));
    assert_eq!(chunk1.heights, chunk2.heights);
    assert_eq!(chunk1.triangles.len(), chunk2.triangles.len());
    for (t1, t2) in chunk1.triangles.iter().zip(chunk2.triangles.iter()) {
        assert_eq!(t1.positions, t2.positions);
        assert_eq!(t1.normal, t2.normal);
        assert_eq!(t1.color, t2.color);
    }
}

#[test]
fn test_different_seeds() {
    let cfg = load_config();
    let gen_a = WorldGenerator::new(42, cfg.clone());
    let gen_b = WorldGenerator::new(7, cfg);

    assert_ne!(
        gen_a.settlements()[0].center,
        gen_b.settlements()[0].center,
        "Settlement center should differ for different seeds"
    );

    let mut differences = 0;
    for i in 0..50 {
        let x = (i as f32) * 20.0;
        let z = (i as f32) * -15.0;
        if (gen_a.height(x, z) - gen_b.height(x, z)).abs() > 0.01 {
            differences += 1;
        }
    }
    assert!(differences > 40, "Most points should differ between seeds");
}

#[test]
fn test_heights_in_sane_range() {
    let cfg = load_config();
    let generator = WorldGenerator::new(42, cfg);

    let mut min_h = f32::MAX;
    let mut max_h = f32::MIN;

    for z_idx in -20..=20 {
        for x_idx in -20..=20 {
            let x = x_idx as f32 * 50.0;
            let z = z_idx as f32 * 50.0;
            let h = generator.height(x, z);
            assert!(h.is_finite(), "Height at ({x}, {z}) was not finite");
            min_h = min_h.min(h);
            max_h = max_h.max(h);
        }
    }

    // Typical terrain heights stay between -30 and 150
    assert!(
        min_h > -30.0,
        "Terrain minimum height {min_h} is too low"
    );
    assert!(
        max_h < 150.0,
        "Terrain maximum height {max_h} is too high"
    );
}

#[test]
fn test_chunk_edges_match_seamlessly() {
    let cfg = load_config();
    let generator = WorldGenerator::new(42, cfg);

    let test_coords = [
        (IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)),
        (IVec2::new(-2, 3), IVec2::new(-1, 3), IVec2::new(-2, 4)),
        (IVec2::new(5, -4), IVec2::new(6, -4), IVec2::new(5, -3)),
    ];

    let n = WorldGenerator::SAMPLES;

    for (c, right_c, down_c) in test_coords {
        let chunk_center = generator.build_chunk_ground(c);
        let chunk_right = generator.build_chunk_ground(right_c);
        let chunk_down = generator.build_chunk_ground(down_c);

        // Right edge of chunk_center (i = 32) must equal left edge of chunk_right (i = 0)
        for j in 0..n {
            let h_center = chunk_center.heights[j * n + (n - 1)];
            let h_right = chunk_right.heights[j * n];
            assert!(
                (h_center - h_right).abs() < 1e-5,
                "Horizontal seam at j={j} between {c:?} and {right_c:?}: {h_center} vs {h_right}"
            );
        }

        // Bottom edge of chunk_center (j = 32) must equal top edge of chunk_down (j = 0)
        for i in 0..n {
            let h_center = chunk_center.heights[(n - 1) * n + i];
            let h_down = chunk_down.heights[i];
            assert!(
                (h_center - h_down).abs() < 1e-5,
                "Vertical seam at i={i} between {c:?} and {down_c:?}: {h_center} vs {h_down}"
            );
        }
    }
}

#[test]
fn test_settlement_pad_flattening() {
    let cfg = load_config();
    let generator = WorldGenerator::new(42, cfg.clone());
    let site = &generator.settlements()[0];

    // Center height equals site center Y
    let center_h = generator.height(site.center.x, site.center.z);
    assert!(
        (center_h - site.center.y).abs() < 1e-4,
        "Center height {center_h} does not match site.center.y {}",
        site.center.y
    );

    // Inner points within flat_radius should be completely flat at site.center.y
    let inner_offsets = [
        (0.0, 5.0),
        (10.0, 0.0),
        (-15.0, 10.0),
        (site.center.x.signum() * 18.0, 0.0),
    ];
    for (ox, oz) in inner_offsets {
        let h = generator.height(site.center.x + ox, site.center.z + oz);
        assert!(
            (h - site.center.y).abs() < 1e-4,
            "Height at offset ({ox}, {oz}) is {h}, expected flat {}",
            site.center.y
        );
    }
}

#[test]
fn test_biome_classification() {
    let cfg = load_config();
    let generator = WorldGenerator::new(42, cfg);

    for idx in 0..50 {
        let x = (idx as f32) * 53.0;
        let z = (idx as f32) * 29.0;
        let fw = generator.forest_weight(x, z);
        let b = generator.biome(x, z);
        if fw > 0.5 {
            assert_eq!(b, BiomeId::Forest);
        } else {
            assert_eq!(b, BiomeId::Plains);
        }
    }
}
