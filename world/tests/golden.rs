use std::path::{Path, PathBuf};
use glam::{IVec2, Vec3};
use serde_json::Value;
use world::{BiomeId, Color, GroundPaint, SettlementConfig, SettlementConfigData, WorldGenerator};

fn find_file(relative_paths: &[&str]) -> Option<PathBuf> {
    for p in relative_paths {
        let path = Path::new(p);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        for p in relative_paths {
            let path = PathBuf::from(&dir).join(p);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

fn load_settlement_config() -> SettlementConfigData {
    let path = find_file(&[
        "data/settlements.json",
        "../data/settlements.json",
        "../../data/settlements.json",
    ])
    .expect("Could not find data/settlements.json");
    SettlementConfig::load_from_file(path).expect("Failed to parse data/settlements.json")
}

// Helpers for tolerant JSON key extraction (supports camelCase and PascalCase)
fn get_field<'a>(obj: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    for k in keys {
        if let Some(v) = obj.get(*k) {
            return Some(v);
        }
    }
    None
}

fn parse_f32(v: &Value) -> Option<f32> {
    v.as_f64().map(|f| f as f32)
}

fn parse_vec3(v: &Value) -> Option<Vec3> {
    if let Some(arr) = v.as_array().filter(|a| a.len() >= 3) {
        return Some(Vec3::new(
            parse_f32(&arr[0])?,
            parse_f32(&arr[1])?,
            parse_f32(&arr[2])?,
        ));
    }
    if v.is_object() {
        let x = get_field(v, &["x", "X"])?.as_f64()? as f32;
        let y = get_field(v, &["y", "Y"])?.as_f64()? as f32;
        let z = get_field(v, &["z", "Z"])?.as_f64()? as f32;
        return Some(Vec3::new(x, y, z));
    }
    None
}

fn parse_ivec2(v: &Value) -> Option<IVec2> {
    if let Some(arr) = v.as_array().filter(|a| a.len() >= 2) {
        return Some(IVec2::new(
            arr[0].as_i64()? as i32,
            arr[1].as_i64()? as i32,
        ));
    }
    if v.is_object() {
        let x = get_field(v, &["x", "X"])?.as_i64()? as i32;
        let y = get_field(v, &["y", "Y", "z", "Z"])?.as_i64()? as i32;
        return Some(IVec2::new(x, y));
    }
    None
}

fn parse_color(v: &Value) -> Option<Color> {
    if let Some(arr) = v.as_array().filter(|a| a.len() >= 3) {
        let r = parse_f32(&arr[0])?;
        let g = parse_f32(&arr[1])?;
        let b = parse_f32(&arr[2])?;
        let a = if arr.len() >= 4 { parse_f32(&arr[3])? } else { 1.0 };
        return Some(Color::new(r, g, b, a));
    }
    if v.is_object() {
        let r = get_field(v, &["r", "R"])?.as_f64()? as f32;
        let g = get_field(v, &["g", "G"])?.as_f64()? as f32;
        let b = get_field(v, &["b", "B"])?.as_f64()? as f32;
        let a = get_field(v, &["a", "A"])
            .and_then(|a| a.as_f64())
            .map(|a| a as f32)
            .unwrap_or(1.0);
        return Some(Color::new(r, g, b, a));
    }
    None
}

fn parse_biome(v: &Value) -> Option<BiomeId> {
    if let Some(s) = v.as_str() {
        match s.to_lowercase().as_str() {
            "plains" => Some(BiomeId::Plains),
            "forest" => Some(BiomeId::Forest),
            _ => None,
        }
    } else if let Some(i) = v.as_i64() {
        match i {
            0 => Some(BiomeId::Plains),
            1 => Some(BiomeId::Forest),
            _ => None,
        }
    } else {
        None
    }
}

#[test]
fn test_golden_world() {
    let golden_path = find_file(&[
        "rust/golden/world.json",
        "golden/world.json",
        "../golden/world.json",
        "../../golden/world.json",
    ]);

    let golden_path = match golden_path {
        Some(p) => p,
        None => {
            println!("NOTE: Golden test skipped: rust/golden/world.json not found.");
            return;
        }
    };

    println!("Loading golden test data from {}", golden_path.display());
    let json_str = std::fs::read_to_string(&golden_path).expect("failed to read golden file");
    let root: Value = serde_json::from_str(&json_str).expect("failed to parse golden json");

    let cfg = load_settlement_config();

    // The golden root might be:
    // 1. Map of seed -> data: {"42": {...}, "7": {...}}
    // 2. Array of items with "seed": [{seed: 42, ...}, ...]
    // 3. Object with "seeds": {...} or "tests": [...]
    let mut seed_entries: Vec<(i32, &Value)> = Vec::new();

    if let Some(arr) = root.as_array() {
        for item in arr {
            if let Some(s) = get_field(item, &["seed", "Seed"]).and_then(|v| v.as_i64()) {
                seed_entries.push((s as i32, item));
            }
        }
    } else if let Some(_obj) = root.as_object() {
        let container = get_field(&root, &["seeds", "Seeds", "tests", "Tests"]).unwrap_or(&root);
        if let Some(arr) = container.as_array() {
            for item in arr {
                if let Some(s) = get_field(item, &["seed", "Seed"]).and_then(|v| v.as_i64()) {
                    seed_entries.push((s as i32, item));
                }
            }
        } else if let Some(obj) = container.as_object() {
            for (key, val) in obj {
                if let Ok(seed) = key.parse::<i32>() {
                    seed_entries.push((seed, val));
                } else if let Some(s) = get_field(val, &["seed", "Seed"]).and_then(|v| v.as_i64()) {
                    seed_entries.push((s as i32, val));
                }
            }
        }
    }

    if seed_entries.is_empty() {
        println!("NOTE: Golden file was present but no seeds were recognized. Root: {root:?}");
        return;
    }

    for (seed, data) in seed_entries {
        println!("Verifying golden test for seed {seed}...");
        let generator = WorldGenerator::new(seed, cfg.clone());

        // 1. Settlements check (center within 1e-3)
        if let Some(settlements_val) = get_field(data, &["settlements", "Settlements", "settlement", "Settlement"]) {
            let sites_list: Vec<&Value> = if let Some(arr) = settlements_val.as_array() {
                arr.iter().collect()
            } else {
                vec![settlements_val]
            };

            for (i, site_val) in sites_list.iter().enumerate() {
                let expected_center = get_field(site_val, &["center", "Center"]).and_then(parse_vec3);
                let actual_center = generator.settlements().get(i).map(|s| s.center);
                if let (Some(exp), Some(act)) = (expected_center, actual_center) {
                    let diff = (act - exp).abs();
                    assert!(
                        diff.x < 1e-3 && diff.y < 1e-3 && diff.z < 1e-3,
                        "Seed {seed} settlement {i} center mismatch: actual {act:?}, expected {exp:?}"
                    );
                }
            }
        }

        // 2. Points check (height within 1e-4, colors within 1e-4, biomes equal)
        if let Some(points_val) = get_field(data, &["points", "Points"]).and_then(|v| v.as_array()) {
            for pt in points_val {
                let x = get_field(pt, &["x", "X"]).and_then(parse_f32).expect("point missing x");
                let z = get_field(pt, &["z", "Z"]).and_then(parse_f32).expect("point missing z");

                if let Some(exp_h) = get_field(pt, &["height", "Height"]).and_then(parse_f32) {
                    let act_h = generator.height(x, z);
                    assert!(
                        (act_h - exp_h).abs() < 1e-4,
                        "Seed {seed} height mismatch at ({x}, {z}): actual {act_h}, expected {exp_h}"
                    );
                }

                if let Some(exp_raw) = get_field(pt, &["raw", "Raw", "rawHeight", "RawHeight"]).and_then(parse_f32) {
                    let act_raw = generator.raw_height(x, z);
                    assert!(
                        (act_raw - exp_raw).abs() < 1e-4,
                        "Seed {seed} raw height mismatch at ({x}, {z}): actual {act_raw}, expected {exp_raw}"
                    );
                }

                if let Some(exp_f) = get_field(pt, &["forest", "Forest", "forestWeight", "ForestWeight"]).and_then(parse_f32) {
                    let act_f = generator.forest_weight(x, z);
                    assert!(
                        (act_f - exp_f).abs() < 1e-4,
                        "Seed {seed} forest weight mismatch at ({x}, {z}): actual {act_f}, expected {exp_f}"
                    );
                }

                if let Some(exp_b) = get_field(pt, &["biome", "Biome"]).and_then(parse_biome) {
                    let act_b = generator.biome(x, z);
                    assert_eq!(
                        act_b, exp_b,
                        "Seed {seed} biome mismatch at ({x}, {z}): actual {act_b:?}, expected {exp_b:?}"
                    );
                }
            }
        }

        // 3. Chunks check (coord, heights, triangles with positions, normal, color)
        if let Some(chunks_val) = get_field(data, &["chunks", "Chunks"]).and_then(|v| v.as_array()) {
            for chunk_val in chunks_val {
                let coord = get_field(chunk_val, &["coord", "Coord"])
                    .and_then(parse_ivec2)
                    .expect("chunk missing coord");
                let chunk_ground = generator.build_chunk_ground(coord);

                if let Some(heights_arr) = get_field(chunk_val, &["heights", "Heights"]).and_then(|v| v.as_array()) {
                    assert_eq!(
                        chunk_ground.heights.len(),
                        heights_arr.len(),
                        "Seed {seed} chunk {coord:?} heights length mismatch"
                    );
                    for (idx, h_val) in heights_arr.iter().enumerate() {
                        let exp_h = parse_f32(h_val).unwrap();
                        let act_h = chunk_ground.heights[idx];
                        assert!(
                            (act_h - exp_h).abs() < 1e-4,
                            "Seed {seed} chunk {coord:?} height[{idx}] mismatch: actual {act_h}, expected {exp_h}"
                        );
                    }
                }

                if let Some(tris_arr) = get_field(chunk_val, &["triangles", "Triangles"]).and_then(|v| v.as_array()) {
                    assert_eq!(
                        chunk_ground.triangles.len(),
                        tris_arr.len(),
                        "Seed {seed} chunk {coord:?} triangles length mismatch"
                    );
                    for (idx, tri_val) in tris_arr.iter().enumerate() {
                        let act_tri = &chunk_ground.triangles[idx];

                        if let Some(positions_arr) = get_field(tri_val, &["positions", "Positions"]).and_then(|v| v.as_array()) {
                            for k in 0..3 {
                                if let Some(exp_p) = positions_arr.get(k).and_then(parse_vec3) {
                                    let diff = (act_tri.positions[k] - exp_p).abs();
                                    assert!(
                                        diff.x < 1e-4 && diff.y < 1e-4 && diff.z < 1e-4,
                                        "Seed {seed} chunk {coord:?} tri {idx} pos[{k}] mismatch: act {:?}, exp {:?}",
                                        act_tri.positions[k], exp_p
                                    );
                                }
                            }
                        }

                        if let Some(exp_normal) = get_field(tri_val, &["normal", "Normal"]).and_then(parse_vec3) {
                            let diff = (act_tri.normal - exp_normal).abs();
                            assert!(
                                diff.x < 1e-4 && diff.y < 1e-4 && diff.z < 1e-4,
                                "Seed {seed} chunk {coord:?} tri {idx} normal mismatch: act {:?}, exp {:?}",
                                act_tri.normal, exp_normal
                            );
                        }

                        if let Some(exp_c) = get_field(tri_val, &["color", "Color"]).and_then(parse_color) {
                            let diff_r = (act_tri.color.r - exp_c.r).abs();
                            let diff_g = (act_tri.color.g - exp_c.g).abs();
                            let diff_b = (act_tri.color.b - exp_c.b).abs();
                            let diff_a = (act_tri.color.a - exp_c.a).abs();
                            assert!(
                                diff_r < 1e-4 && diff_g < 1e-4 && diff_b < 1e-4 && diff_a < 1e-4,
                                "Seed {seed} chunk {coord:?} tri {idx} color mismatch: act {:?}, exp {:?}",
                                act_tri.color, exp_c
                            );
                        }
                    }
                }
            }
        }

        // 4. GroundPaint check
        if let Some(gp_val) = get_field(data, &["groundPaint", "GroundPaint"]).and_then(|v| v.as_array()) {
            let paint = GroundPaint::new(seed);
            for test_case in gp_val {
                let inputs = get_field(test_case, &["inputs", "Inputs"]).unwrap_or(test_case);
                let grass = get_field(inputs, &["grass", "Grass"])
                    .and_then(parse_color)
                    .unwrap_or(WorldGenerator::PLAINS_GRASS);
                let x = get_field(inputs, &["x", "X"]).and_then(parse_f32).unwrap_or(0.0);
                let z = get_field(inputs, &["z", "Z"]).and_then(parse_f32).unwrap_or(0.0);
                let forest = get_field(inputs, &["forest", "Forest"]).and_then(parse_f32).unwrap_or(0.0);
                let normal_y = get_field(inputs, &["normalY", "NormalY", "normal_y"]).and_then(parse_f32).unwrap_or(1.0);
                let camp_dist = get_field(inputs, &["campDistance", "CampDistance", "camp_distance"]).and_then(parse_f32).unwrap_or(100.0);

                let act_color = paint.paint(grass, x, z, forest, normal_y, camp_dist);

                if let Some(exp_color) = get_field(test_case, &["color", "Color", "output", "Output"]).and_then(parse_color) {
                    let diff_r = (act_color.r - exp_color.r).abs();
                    let diff_g = (act_color.g - exp_color.g).abs();
                    let diff_b = (act_color.b - exp_color.b).abs();
                    let diff_a = (act_color.a - exp_color.a).abs();
                    assert!(
                        diff_r < 1e-4 && diff_g < 1e-4 && diff_b < 1e-4 && diff_a < 1e-4,
                        "Seed {seed} GroundPaint color mismatch: act {act_color:?}, exp {exp_color:?}"
                    );
                }
            }
        }
    }
}
