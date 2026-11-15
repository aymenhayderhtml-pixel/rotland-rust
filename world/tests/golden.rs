//! The Rust terrain must give the same numbers as the Godot version. rust/golden/world.json is written by the
//! Godot self-test GoldenWorldDump (ROTLAND_DUMP_GOLDEN=1). Every check counts what it compared, and the
//! test fails if a section compared nothing, so a changed file layout can never pass silently.

use std::path::PathBuf;

use glam::IVec2;
use serde_json::Value;
use world::{Color, GroundPaint, SettlementConfig, WorldGenerator};

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

    let worlds = golden["worlds"].as_array().expect("worlds");
    assert!(!worlds.is_empty(), "no worlds in the golden file");
    let (mut points, mut heights, mut triangles, mut bad) = (0usize, 0usize, 0usize, Vec::new());

    for w in worlds {
        let seed = w["seed"].as_i64().expect("seed") as i32;
        let world_gen = WorldGenerator::new(seed, config.clone());

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

    println!("compared {points} points, {heights} chunk heights, {triangles} triangles, {} paint samples", samples.len());
    assert!(points > 0 && heights > 0 && triangles > 0 && !samples.is_empty(), "a section compared nothing");
    assert!(bad.is_empty(), "{} mismatches (first 20):\n{}", bad.len(), bad.join("\n"));
}
