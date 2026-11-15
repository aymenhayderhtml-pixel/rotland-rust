//! The Rust terrain must give the same numbers as the Godot version. rust/golden/world.json is written by the
//! Godot self-test GoldenWorldDump (ROTLAND_DUMP_GOLDEN=1). Every check counts what it compared, and the
//! test fails if a section compared nothing, so a changed file layout can never pass silently.

use std::path::PathBuf;

use glam::{IVec2, Vec3};
use serde_json::Value;
use world::{Color, GroundPaint, PoiConfig, ScatterConfig, SettlementConfig, WorldGenerator};

const TOL: f32 = 1e-4;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn f(v: &Value) -> f32 {
    v.as_f64().expect("number") as f32
}

fn color(v: &Value) -> Color {
    Color::new(f(&v["r"]), f(&v["g"]), f(&v["b"]), f(&v["a"]))
}

fn close(a: f32, b: f32, tol: f32) -> bool {
    (a - b).abs() <= tol * (1.0 + b.abs())
}

fn same_color(a: Color, b: Color) -> bool {
    let (a, b) = (a.to_array(), b.to_array());
    (0..4).all(|i| close(a[i], b[i], TOL))
}

#[test]
fn terrain_matches_godot() {
    let text = std::fs::read_to_string(repo().join("rust/golden/world.json")).expect("rust/golden/world.json is missing");
    let golden: Value = serde_json::from_str(&text).expect("golden file is not JSON");
    let config = SettlementConfig::load_from_file(repo().join("data/settlements.json")).expect("data/settlements.json");
    let scatter_config = ScatterConfig::load_from_file(repo().join("data/scatter.json")).expect("data/scatter.json");
    let poi_config = PoiConfig::load_from_file(repo().join("data/poi.json")).expect("data/poi.json");

    let worlds = golden["worlds"].as_array().expect("worlds");
    assert!(!worlds.is_empty(), "no worlds in the golden file");
    let (mut points, mut heights, mut triangles, mut pois_compared, mut props_compared, mut bad) =
        (0usize, 0usize, 0usize, 0usize, 0usize, Vec::new());

    for w in worlds {
        let seed = w["seed"].as_i64().expect("seed") as i32;
        let world_gen = WorldGenerator::with_configs(seed, config.clone(), scatter_config.clone(), poi_config.clone());

        let sites = w["settlements"].as_array().expect("settlements");
        assert_eq!(sites.len(), world_gen.settlements().len(), "seed {seed}: settlement count");
        for (g, r) in sites.iter().zip(world_gen.settlements()) {
            let c = &g["center"];
            let ok = close(r.center.x, f(&c["x"]), 1e-3) && close(r.center.y, f(&c["y"]), 1e-3) && close(r.center.z, f(&c["z"]), 1e-3);
            assert!(ok, "seed {seed}: camp centre {:?} vs Godot {c}", r.center);
        }

        for p in w["points"].as_array().expect("points") {
            let (x, z) = (f(&p["x"]), f(&p["z"]));
            let checks = [
                ("height", world_gen.height(x, z), f(&p["height"])),
                ("raw", world_gen.raw_height(x, z), f(&p["raw"])),
                ("forest", world_gen.forest_weight(x, z), f(&p["forest"])),
                ("biome", world_gen.biome(x, z) as i32 as f32, f(&p["biome"])),
            ];
            for (name, ours, godot) in checks {
                if !close(ours, godot, TOL) && bad.len() < 20 {
                    bad.push(format!("seed {seed} ({x}, {z}) {name}: rust {ours} godot {godot}"));
                }
            }
            points += 1;
        }

        for c in w["chunks"].as_array().expect("chunks") {
            let coord = IVec2::new(c["coord"]["x"].as_i64().unwrap() as i32, c["coord"]["z"].as_i64().unwrap() as i32);
            let ours = world_gen.build_chunk_ground(coord);
            for (i, h) in c["heights"].as_array().unwrap().iter().enumerate() {
                if !close(ours.heights[i], f(h), TOL) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} height[{i}]: rust {} godot {}", ours.heights[i], f(h)));
                }
                heights += 1;
            }
            let tris = c["triangles"].as_array().unwrap();
            assert_eq!(tris.len(), ours.triangles.len(), "seed {seed} chunk {coord}: triangle count");
            for (k, (g, r)) in tris.iter().zip(&ours.triangles).enumerate() {
                for (v, pos) in g["vertices"].as_array().unwrap().iter().zip(r.positions) {
                    if !(close(pos.x, f(&v["x"]), TOL) && close(pos.y, f(&v["y"]), TOL) && close(pos.z, f(&v["z"]), TOL)) && bad.len() < 20 {
                        bad.push(format!("seed {seed} chunk {coord} tri {k} vertex: rust {pos} godot {v}"));
                    }
                }
                let n = &g["normal"];
                if !(close(r.normal.x, f(&n["x"]), 1e-3) && close(r.normal.y, f(&n["y"]), 1e-3) && close(r.normal.z, f(&n["z"]), 1e-3)) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} tri {k} normal: rust {} godot {n}", r.normal));
                }
                if !same_color(r.color, color(&g["color"])) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} tri {k} colour: rust {:?} godot {}", r.color, g["color"]));
                }
                triangles += 1;
            }

            let (ours_pois, ours_props) = world_gen.chunk_contents(coord);

            let pois = c["pois"].as_array().expect("pois");
            assert_eq!(pois.len(), ours_pois.len(), "seed {seed} chunk {coord}: poi count");
            for (k, (g, r)) in pois.iter().zip(&ours_pois).enumerate() {
                let gid = g["id"].as_str().unwrap();
                let gbid = g["buildingId"].as_str().unwrap();
                if r.id != gid && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} poi {k} id: rust {} godot {gid}", r.id));
                }
                if r.building_id != gbid && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} poi {k} buildingId: rust {} godot {gbid}", r.building_id));
                }
                let p = &g["position"];
                let (gx, gy, gz) = (f(&p["x"]), f(&p["y"]), f(&p["z"]));
                if !(close(r.position.x, gx, TOL) && close(r.position.y, gy, TOL) && close(r.position.z, gz, TOL)) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} poi {k} pos: rust {} godot ({gx}, {gy}, {gz})", r.position));
                }
                let gyaw = f(&g["yaw"]);
                if !close(r.yaw, gyaw, TOL) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} poi {k} yaw: rust {} godot {gyaw}", r.yaw));
                }
                pois_compared += 1;
            }

            let props = c["props"].as_array().expect("props");
            assert_eq!(props.len(), ours_props.len(), "seed {seed} chunk {coord}: prop count");
            for (k, (g, r)) in props.iter().zip(&ours_props).enumerate() {
                let gmodel = g["model"].as_str().unwrap();
                if r.model != gmodel && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} prop {k} model: rust {} godot {gmodel}", r.model));
                }
                let p = &g["position"];
                let (gx, gy, gz) = (f(&p["x"]), f(&p["y"]), f(&p["z"]));
                if !(close(r.position.x, gx, TOL) && close(r.position.y, gy, TOL) && close(r.position.z, gz, TOL)) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} prop {k} pos: rust {} godot ({gx}, {gy}, {gz})", r.position));
                }
                let gyaw = f(&g["yaw"]);
                if !close(r.yaw, gyaw, TOL) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} prop {k} yaw: rust {} godot {gyaw}", r.yaw));
                }
                let gscale = f(&g["scale"]);
                if !close(r.scale, gscale, TOL) && bad.len() < 20 {
                    bad.push(format!("seed {seed} chunk {coord} prop {k} scale: rust {} godot {gscale}", r.scale));
                }
                props_compared += 1;
            }
        }
    }

    let paint = GroundPaint::new(42);
    let samples = golden["groundPaint"].as_array().expect("groundPaint");
    for s in samples {
        let out = paint.paint(color(&s["grass"]), f(&s["x"]), f(&s["z"]), f(&s["forest"]), f(&s["normalY"]), f(&s["campDistance"]));
        if !same_color(out, color(&s["output"])) && bad.len() < 20 {
            bad.push(format!("paint at ({}, {}): rust {:?} godot {}", s["x"], s["z"], out, s["output"]));
        }
    }

    let bm = &golden["benchmark"];
    assert!(!bm.is_null(), "benchmark section missing in golden file");
    let bm_seed = bm["seed"].as_i64().expect("benchmark seed") as i32;
    let bm_gen = WorldGenerator::with_configs(bm_seed, config, scatter_config, poi_config);
    let spawn = Vec3::new(f(&bm["spawn"]["x"]), f(&bm["spawn"]["y"]), f(&bm["spawn"]["z"]));
    let chosen_chunk = bm_gen.densest_forest_near(spawn);
    let expected_chunk = IVec2::new(
        bm["chunk"]["x"].as_i64().unwrap() as i32,
        bm["chunk"]["z"].as_i64().unwrap() as i32,
    );
    assert_eq!(chosen_chunk, expected_chunk, "benchmark densest forest chunk mismatch: rust {chosen_chunk} vs godot {expected_chunk}");
    let benchmark_compared = 1usize;

    println!(
        "compared {points} points, {heights} chunk heights, {triangles} triangles, {pois_compared} pois, {props_compared} props, {benchmark_compared} benchmark chunk, {} paint samples",
        samples.len()
    );
    assert!(
        points > 0 && heights > 0 && triangles > 0 && pois_compared > 0 && props_compared > 0 && benchmark_compared > 0 && !samples.is_empty(),
        "a section compared nothing"
    );
    assert!(bad.is_empty(), "{} mismatches (first 20):\n{}", bad.len(), bad.join("\n"));
}
