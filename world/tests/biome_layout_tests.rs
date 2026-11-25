use glam::Vec2;
use world::biome::layout::{WorldLayout, WorldLayoutData};
use world::biome::registry::BiomeRegistry;
use world::BiomeId;

#[test]
fn registry_loads_real_data_and_round_trips_keys() {
    let registry = BiomeRegistry::load().expect("failed to load data/biomes.json");
    assert_eq!(BiomeId::ALL.len(), 6);

    for id in BiomeId::ALL {
        let key = id.key();
        assert_eq!(BiomeId::from_key(key), Some(id));
        let def = registry.get(id);
        assert_eq!(def.key, key);
        assert!(!def.name.is_empty());
        assert!(!def.why.is_empty());
        assert!(!def.scatter.is_empty());
    }

    assert_eq!(BiomeId::from_key("jungle"), None);
    assert_eq!(BiomeId::from_key(""), None);
}

#[test]
fn registry_missing_biome_returns_err() {
    let path = paths::repo_path("data/biomes.json");
    let raw = std::fs::read_to_string(path).expect("failed to read data/biomes.json");
    let mut parsed: serde_json::Value = serde_json::from_str(&raw).expect("valid json");
    let biomes = parsed["biomes"]
        .as_array_mut()
        .expect("biomes array in data/biomes.json");
    biomes.retain(|entry| entry["key"] != "mountains" && entry["key"] != "desert");

    let broken = serde_json::to_string(&parsed).expect("serialize modified json");
    let err = BiomeRegistry::from_json(&broken).expect_err("expected missing biome error");
    assert!(
        err.contains("desert") && err.contains("mountains"),
        "error should name missing keys, got: {err}"
    );
}

#[test]
fn outer_is_zero_inside_core_and_one_past_blend_band() {
    let data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let origin = Vec2::new(120.0, -80.0);
    let layout = WorldLayout::new(&data, 42, origin, Vec2::X);

    for dist in [0.0_f32, 500.0, 1500.0, data.core_radius] {
        for deg in [0.0_f32, 65.0, 180.0, 295.0] {
            let p = origin + layout.direction_at_deg(deg) * dist;
            let sample = layout.sample(p.x, p.y);
            assert_eq!(sample.outer, 0.0, "expected outer=0 at dist={dist}, deg={deg}");
            assert_eq!(sample.tier, 0, "expected tier=0 inside core at dist={dist}");
        }
    }

    let mid_band = data.core_radius + data.blend_band * 0.5;
    let p_mid = origin + Vec2::X * mid_band;
    let s_mid = layout.sample(p_mid.x, p_mid.y);
    assert!(
        s_mid.outer > 0.0 && s_mid.outer < 1.0,
        "expected partial outer inside blend band, got {}",
        s_mid.outer
    );

    let past_band = data.core_radius + data.blend_band + 10.0;
    let p_far = origin + Vec2::X * past_band;
    let s_far = layout.sample(p_far.x, p_far.y);
    assert_eq!(s_far.outer, 1.0);
}

#[test]
fn sector_centres_at_4000m_return_expected_biome_with_zero_blend() {
    let data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let origin = Vec2::ZERO;
    let layout = WorldLayout::new(&data, 42, origin, Vec2::X);

    let expected = [
        (0.0_f32, BiomeId::Desert, Vec2::new(4000.0, 0.0)),
        (90.0_f32, BiomeId::Swamp, Vec2::new(0.0, -4000.0)),
        (180.0_f32, BiomeId::Snow, Vec2::new(-4000.0, 0.0)),
        (270.0_f32, BiomeId::Mountains, Vec2::new(0.0, 4000.0)),
    ];

    for (deg, biome, expected_pos) in expected {
        let pos = origin + layout.direction_at_deg(deg) * 4000.0;
        assert!(
            (pos - expected_pos).length() < 1e-3,
            "unexpected direction at {deg} deg: got {pos:?}, expected {expected_pos:?}"
        );
        let sample = layout.sample(pos.x, pos.y);
        assert_eq!(sample.primary, biome);
        assert_eq!(sample.secondary, biome);
        assert_eq!(sample.blend, 0.0);
        assert_eq!(sample.outer, 1.0);
    }
}

#[test]
fn sector_border_blends_near_half() {
    let data = WorldLayoutData::load().expect("failed to load data/world_layout.json");

    // 1. Exact unwarped border at 45 deg gives blend = 0.5 between Desert and Swamp.
    let mut unwarped = data.clone();
    unwarped.warp_deg = 0.0;
    let flat_layout = WorldLayout::new(&unwarped, 42, Vec2::ZERO, Vec2::X);
    let border_dir = flat_layout.direction_at_deg(45.0) * 4000.0;
    let exact_border = flat_layout.sample(border_dir.x, border_dir.y);
    assert!(
        (exact_border.blend - 0.5).abs() < 1e-5,
        "expected blend=0.5 on unwarped border, got {}",
        exact_border.blend
    );
    assert_ne!(exact_border.primary, exact_border.secondary);

    // 2. With warp enabled, sweeping across the Desert->Swamp transition reaches blend near 0.5.
    let warped_layout = WorldLayout::new(&data, 42, Vec2::ZERO, Vec2::X);
    let mut max_blend = 0.0_f32;
    for step in 0..=360 {
        let deg = (step as f32) * 0.25;
        let p = warped_layout.direction_at_deg(deg) * 4000.0;
        let s = warped_layout.sample(p.x, p.y);
        if s.blend > max_blend {
            max_blend = s.blend;
        }
    }
    assert!(
        max_blend > 0.45 && max_blend <= 0.5,
        "expected max blend near 0.5 across warped sector border, got {max_blend}"
    );
}

#[test]
fn danger_tiers_follow_distance_bands_and_cap_at_five() {
    let data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let layout = WorldLayout::new(&data, 42, Vec2::ZERO, Vec2::X);

    let cases = [
        (0.0_f32, 0_u8),
        (2199.0, 0),
        (2201.0, 1),
        (3699.0, 1),
        (3701.0, 2),
        (5201.0, 3),
        (6701.0, 4),
        (8201.0, 5),
        (12000.0, 5),
        (50000.0, 5),
    ];

    for (dist, expected_tier) in cases {
        let sample = layout.sample(dist, 0.0);
        assert_eq!(
            sample.tier, expected_tier,
            "tier mismatch at dist={dist}: got {}, expected {expected_tier}",
            sample.tier
        );
    }
}

#[test]
fn layout_is_deterministic_and_seed_shifts_borders() {
    let data = WorldLayoutData::load().expect("failed to load data/world_layout.json");
    let origin = Vec2::new(50.0, -30.0);
    let ref_dir = Vec2::new(-384.0, 1792.0);

    let a1 = WorldLayout::new(&data, 42, origin, ref_dir);
    let a2 = WorldLayout::new(&data, 42, origin, ref_dir);
    let b = WorldLayout::new(&data, 9001, origin, ref_dir);

    for i in -8..=8 {
        for j in -8..=8 {
            let x = (i as f32) * 750.0;
            let z = (j as f32) * 750.0;
            assert_eq!(a1.sample(x, z), a2.sample(x, z));
        }
    }

    let mut border_differences = 0;
    for k in 0..40 {
        let dist = 2800.0 + (k as f32) * 200.0;
        let deg = 45.0 + ((k % 4) as f32) * 90.0;
        let p = origin + a1.direction_at_deg(deg) * dist;
        let sa = a1.sample(p.x, p.y);
        let sb = b.sample(p.x, p.y);
        if sa.primary != sb.primary || (sa.blend - sb.blend).abs() > 0.02 {
            border_differences += 1;
        }
    }
    assert!(
        border_differences > 5,
        "expected different seed to shift warped sector borders, got {border_differences} diffs"
    );
}
