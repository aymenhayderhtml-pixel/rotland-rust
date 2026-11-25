use glam::Vec2;

use world::biome::layout::{WorldLayout, WorldLayoutData};
use world::rivers::{RiverNetwork, RiversData};

fn default_layout() -> WorldLayout {
    let data = WorldLayoutData::load().expect("load data/world_layout.json");
    WorldLayout::new(&data, 42, Vec2::ZERO, Vec2::X)
}

/// Deterministic bumpy outer terrain sloping gently away from snow/mountains toward desert/swamp.
fn hilly_terrain(x: f32, z: f32) -> f32 {
    let slope = 60.0 - 0.004 * x - 0.002 * z;
    let hills = 6.0 * (x * 0.003).sin() * (z * 0.003).cos() + 3.0 * ((x + z) * 0.007).sin();
    slope + hills
}

#[test]
fn rivers_never_enter_core_and_levels_never_rise() {
    let data = RiversData::load().expect("load data/rivers.json");
    let layout = default_layout();
    assert_eq!(data.rivers.len(), 3);

    let net = RiverNetwork::build(&data, &layout, 42, Vec2::ZERO, &hilly_terrain);
    assert_eq!(net.rivers().len(), 3);

    let min_core = layout.data().core_radius + layout.data().blend_band + data.core_margin_m;
    let outer_edge = layout.data().core_radius + layout.data().blend_band;

    for river in net.rivers() {
        assert!(river.points.len() >= 10, "river should trace multiple segments");
        assert_eq!(river.points.len(), river.widths.len());
        assert_eq!(river.points.len(), river.water_levels.len());

        for (i, &p) in river.points.iter().enumerate() {
            let r = p.length();
            assert!(
                r >= min_core - 1e-2,
                "river point {i} entered core exclusion: r={r}, min={min_core}"
            );
            if i > 0 {
                let mid = (river.points[i - 1] + p) * 0.5;
                assert!(
                    mid.length() >= min_core - 1e-2,
                    "river midpoint {i} entered core exclusion: r={}",
                    mid.length()
                );
                assert!(
                    river.water_levels[i] <= river.water_levels[i - 1] + 1e-5,
                    "water rose downstream at {i}: {} > {}",
                    river.water_levels[i],
                    river.water_levels[i - 1]
                );
                assert!(
                    river.widths[i] >= river.widths[i - 1] - 1e-5,
                    "width shrank downstream at {i}"
                );
            }
        }
    }

    // Core interior must remain untouched by river carving or water.
    for angle_deg in (0..360).step_by(30) {
        let dir = layout.direction_at_deg(angle_deg as f32);
        let p = dir * (outer_edge - 5.0);
        assert!(net.carve(p.x, p.y).is_none());
        assert!(net.water_at(p.x, p.y).is_none());
    }
}

#[test]
fn river_aimed_at_origin_deflects_around_core_edge() {
    let layout = default_layout();
    let json = r#"{
        "step_m": 25.0,
        "max_length_m": 2000.0,
        "core_margin_m": 150.0,
        "sample_count": 5,
        "sample_spread_deg": 50.0,
        "wiggle_deg": 0.0,
        "wiggle_frequency": 0.002,
        "target_pull": 0.5,
        "width_start_m": 6.0,
        "width_end_m": 12.0,
        "depth_m": 1.6,
        "bed_depth_factor": 0.5,
        "bank_m": 6.0,
        "mesh_bank_overlap_m": 1.0,
        "grid_cell_m": 64.0,
        "bridge_spans_m": [12.0, 16.0],
        "crossing_merge_dist_m": 8.0,
        "rivers": [
            {
                "name": "inward_test",
                "start_angle_deg": 0.0,
                "start_radius_m": 3100.0,
                "target_angle_deg": 180.0,
                "target_radius_m": 3100.0
            }
        ]
    }"#;
    let data = RiversData::from_json(json).expect("parse custom rivers JSON");
    // Funnel terrain sloping strictly toward the origin.
    let funnel = |x: f32, z: f32| Vec2::new(x, z).length() * 0.05;
    let net = RiverNetwork::build(&data, &layout, 7, Vec2::ZERO, &funnel);

    let min_core = layout.data().core_radius + layout.data().blend_band + data.core_margin_m;
    for &p in &net.rivers()[0].points {
        assert!(
            p.length() >= min_core - 1e-2,
            "deflected river entered core: r={} < {min_core}",
            p.length()
        );
    }
}

#[test]
fn carve_blend_and_water_at_match_profile() {
    let data = RiversData::load().expect("load data/rivers.json");
    let layout = default_layout();
    let net = RiverNetwork::build(&data, &layout, 42, Vec2::ZERO, &hilly_terrain);

    let river = &net.rivers()[0];
    let mid_idx = river.points.len() / 2;
    let p = river.points[mid_idx];
    let next = river.points[mid_idx + 1];
    let dir = (next - p).normalize();
    let perp = Vec2::new(-dir.y, dir.x);
    let half_w = river.widths[mid_idx] * 0.5;
    let level = river.water_levels[mid_idx];

    // On the river line: blend == 1, bed == level - depth * 0.5, water_at == Some(level).
    let (bed, blend) = net.carve(p.x, p.y).expect("carve on centerline");
    assert!((blend - 1.0).abs() < 1e-4, "expected blend 1, got {blend}");
    let expected_bed = level - data.depth_m * data.bed_depth_factor;
    assert!(
        (bed - expected_bed).abs() < 0.05,
        "expected bed ~{expected_bed}, got {bed}"
    );
    let water = net.water_at(p.x, p.y).expect("water_at on centerline");
    assert!((water - level).abs() < 0.05);

    // Mid-bank: blend strictly between 0 and 1, not over water.
    let bank_pt = p + perp * (half_w + data.bank_m * 0.5);
    let (_, bank_blend) = net.carve(bank_pt.x, bank_pt.y).expect("carve on bank");
    assert!(
        bank_blend > 0.0 && bank_blend < 1.0,
        "expected partial bank blend, got {bank_blend}"
    );

    // Beyond the bank: carve is None (blend 0).
    let outside_pt = p + perp * (half_w + data.bank_m + 2.0);
    assert!(net.carve(outside_pt.x, outside_pt.y).is_none());

    // 30 m away: water_at is None.
    let far_pt = p + perp * 30.0;
    assert!(net.water_at(far_pt.x, far_pt.y).is_none());
}

#[test]
fn water_mesh_near_is_populated_near_river_and_empty_far_away() {
    let data = RiversData::load().expect("load data/rivers.json");
    let layout = default_layout();
    let net = RiverNetwork::build(&data, &layout, 42, Vec2::ZERO, &hilly_terrain);

    let river_pt = net.rivers()[0].points[20];
    let mesh = net.water_mesh_near(river_pt, 300.0);
    assert!(!mesh.is_empty(), "mesh should be non-empty near river");
    assert_eq!(mesh.len() % 3, 0, "mesh must be a triangle list");
    assert!(
        mesh.len() < 20_000,
        "vertex count {} exceeded 20k budget",
        mesh.len()
    );

    // Verify upward (+Y) triangle winding matching LakeWaterMesh.
    for tri in mesh.as_chunks::<3>().0 {
        let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]);
        assert!(normal.y > 0.0, "expected +Y normal, got {normal:?}");
    }

    // Core origin is > 2750 m from every river, so radius 300 m must be empty.
    let empty_mesh = net.water_mesh_near(Vec2::ZERO, 300.0);
    assert!(empty_mesh.is_empty(), "mesh should be empty far from rivers");
}

#[test]
fn crossing_found_for_hand_made_road_across_hand_made_river() {
    let layout = default_layout();
    let json = r#"{
        "step_m": 25.0,
        "max_length_m": 600.0,
        "core_margin_m": 150.0,
        "sample_count": 5,
        "sample_spread_deg": 50.0,
        "wiggle_deg": 0.0,
        "wiggle_frequency": 0.002,
        "target_pull": 0.25,
        "width_start_m": 4.0,
        "width_end_m": 4.0,
        "depth_m": 1.6,
        "bed_depth_factor": 0.5,
        "bank_m": 3.0,
        "mesh_bank_overlap_m": 1.0,
        "grid_cell_m": 64.0,
        "bridge_spans_m": [12.0, 16.0],
        "crossing_merge_dist_m": 8.0,
        "rivers": [
            {
                "name": "valley_stream",
                "start_angle_deg": 0.0,
                "start_radius_m": 3200.0,
                "target_angle_deg": 0.0,
                "target_radius_m": 4200.0
            }
        ]
    }"#;
    let data = RiversData::from_json(json).expect("parse hand-made river data");

    // Straight valley along z = 0 sloping downhill toward +X.
    let valley = |x: f32, z: f32| 100.0 - 0.02 * x + 0.1 * z.abs();
    let net = RiverNetwork::build(&data, &layout, 1, Vec2::ZERO, &valley);

    // Hand-made road running north (-Z) across the river at x = 3500.
    let road = [Vec2::new(3500.0, 100.0), Vec2::new(3500.0, -100.0)];
    let crossings = net.crossings_for_polylines(&[&road]);
    assert_eq!(crossings.len(), 1, "expected exactly 1 road-river crossing");

    let (pos, yaw, span) = crossings[0];
    assert!(
        (pos.x - 3500.0).abs() < 1e-2 && pos.y.abs() < 1e-2,
        "unexpected crossing position: {pos:?}"
    );
    // Godot yaw convention: forward = (-sin yaw, -cos yaw) should match road dir (0, -1).
    let fwd = Vec2::new(-yaw.sin(), -yaw.cos());
    assert!(
        fwd.distance(Vec2::new(0.0, -1.0)) < 1e-4,
        "unexpected road heading yaw={yaw}, fwd={fwd:?}"
    );
    // width (4) + 2 * bank (3) = 10 m -> rounded up to 12 m bridge span.
    assert!((span - 12.0).abs() < 1e-4, "expected 12 m span, got {span}");
}
