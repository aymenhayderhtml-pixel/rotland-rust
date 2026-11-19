use std::collections::HashSet;
use std::path::{Path, PathBuf};

use baker::bake_glb;
use pack::ModelPack;

fn repo_root() -> PathBuf {
    paths::repo_root()
}

fn find_glb_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if entry.file_name() == "_previews" {
            continue;
        }
        if path.is_dir() {
            find_glb_files(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("glb") {
            out.push(path);
        }
    }
}

#[test]
fn test_every_glb_bakes_without_error_and_has_triangles() {
    let root = repo_root();
    let models_dir = root.join("assets/models");
    let mut glb_files = Vec::new();
    find_glb_files(&models_dir, &mut glb_files);

    assert!(
        glb_files.len() >= 90,
        "expected at least 90 models in assets/models, found {}",
        glb_files.len()
    );

    for path in glb_files {
        let rel_path = path.strip_prefix(&models_dir).unwrap();
        let folder = rel_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = path.file_stem().unwrap().to_str().unwrap().to_string();

        let model = bake_glb(&path, name.clone(), folder)
            .unwrap_or_else(|e| panic!("failed to bake model {}: {e}", path.display()));

        assert!(
            !model.vertices.is_empty(),
            "model {name} has 0 drawable vertices"
        );
        assert_eq!(
            model.vertices.len() % 3,
            0,
            "model {name} vertex count is not divisible by 3"
        );
    }
}

#[test]
fn test_zombie_01_skeleton_and_clips() {
    let root = repo_root();
    let zombie_path = root.join("assets/models/characters/zombie_01.glb");

    let model = bake_glb(&zombie_path, "zombie_01".to_string(), "characters".to_string())
        .expect("failed to bake zombie_01");

    let skeleton = model.skeleton.expect("zombie_01 must have a skeleton");
    assert_eq!(skeleton.joints.len(), 22, "zombie_01 should have 22 joints");

    let clip_names: Vec<&str> = model.clips.iter().map(|c| c.name.as_str()).collect();
    assert!(
        clip_names.contains(&"walk"),
        "zombie_01 missing 'walk' clip; found: {clip_names:?}"
    );
    assert!(
        clip_names.contains(&"idle"),
        "zombie_01 missing 'idle' clip; found: {clip_names:?}"
    );
    assert!(
        clip_names.contains(&"zombie_shamble"),
        "zombie_01 missing 'zombie_shamble' clip; found: {clip_names:?}"
    );

    for clip in &model.clips {
        assert!(clip.duration > 0.0, "clip {} has zero duration", clip.name);
        assert!(!clip.channels.is_empty(), "clip {} has no channels", clip.name);
    }
}

#[test]
fn test_tree_pine_vertex_colours_in_palette() {
    let root = repo_root();
    let pine_path = root.join("assets/models/nature/tree_pine_01.glb");
    let palette_path = root.join("assets/textures/palette.png");

    let model = bake_glb(&pine_path, "tree_pine_01".to_string(), "nature".to_string())
        .expect("failed to bake tree_pine_01");

    let palette_bytes = std::fs::read(&palette_path).expect("failed to read palette.png");
    let img = image::load_from_memory(&palette_bytes)
        .expect("failed to decode palette.png")
        .to_rgb8();

    let mut palette_colors = HashSet::new();
    for pixel in img.pixels() {
        palette_colors.insert(pixel.0);
    }

    assert!(
        palette_colors.len() <= 64,
        "unexpected palette color count: {}",
        palette_colors.len()
    );

    for v in &model.vertices {
        let rgb = [v.color[0], v.color[1], v.color[2]];
        assert!(
            palette_colors.contains(&rgb),
            "vertex color {rgb:?} in tree_pine_01 is not in the palette"
        );
    }
}

#[test]
fn test_pack_saves_and_loads_back_identically() {
    let root = repo_root();
    let pine_path = root.join("assets/models/nature/tree_pine_01.glb");
    let zombie_path = root.join("assets/models/characters/zombie_01.glb");

    let pine = bake_glb(&pine_path, "tree_pine_01".to_string(), "nature".to_string())
        .expect("failed to bake pine");
    let zombie = bake_glb(&zombie_path, "zombie_01".to_string(), "characters".to_string())
        .expect("failed to bake zombie");

    let pack = ModelPack::new(vec![pine.clone(), zombie.clone()]);

    let temp_file = std::env::temp_dir().join("rotland_baker_test_pack.pack");
    pack.save(&temp_file).expect("failed to save pack");

    let loaded = ModelPack::load(&temp_file).expect("failed to load pack");
    assert_eq!(pack, loaded);
    assert_eq!(loaded.find("tree_pine_01"), Some(&pine));
    assert_eq!(loaded.find("zombie_01"), Some(&zombie));

    let _ = std::fs::remove_file(temp_file);
}

fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn sub3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Bakes `assets/models/<rel>` and asserts every non-degenerate triangle is wound
/// counter-clockwise when seen from outside, i.e. its geometric normal agrees with
/// the averaged vertex normal. This is what the `skinned` pipeline's
/// `FrontFace::Ccw` assumes; a CW model would be culled and vanish.
fn assert_ccw_outward(rel: &str) -> pack::Model {
    let root = repo_root();
    let models_dir = root.join("assets/models");
    let path = models_dir.join(rel);
    let name = path.file_stem().unwrap().to_str().unwrap().to_string();
    let folder = path
        .parent()
        .unwrap()
        .strip_prefix(&models_dir)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let model = bake_glb(&path, name.clone(), folder)
        .unwrap_or_else(|e| panic!("failed to bake {rel}: {e}"));

    assert!(!model.vertices.is_empty(), "{rel} has no drawable vertices");
    assert_eq!(
        model.vertices.len() % 3,
        0,
        "{rel} vertex count is not a triangle list"
    );

    let mut checked = 0_usize;
    let mut degenerate = 0_usize;
    let mut flipped = Vec::new();

    for (tri, chunk) in model.vertices.as_chunks::<3>().0.iter().enumerate() {
        let p: Vec<[f64; 3]> = chunk
            .iter()
            .map(|v| v.position.map(f64::from))
            .collect();
        let n: Vec<[f64; 3]> = chunk.iter().map(|v| v.normal.map(f64::from)).collect();

        let g = cross3(sub3(p[1], p[0]), sub3(p[2], p[0]));
        if dot3(g, g) < 1e-18 {
            degenerate += 1;
            continue;
        }
        let sum = [
            n[0][0] + n[1][0] + n[2][0],
            n[0][1] + n[1][1] + n[2][1],
            n[0][2] + n[1][2] + n[2][2],
        ];

        checked += 1;
        if dot3(g, sum) <= 0.0 {
            flipped.push(tri);
        }
    }

    assert!(
        checked > 0,
        "{rel} has no non-degenerate triangles ({degenerate} degenerate)"
    );
    assert!(
        flipped.is_empty(),
        "{rel}: {} of {checked} triangles are CW (inside-out) instead of CCW; \
         first offenders (triangle index): {:?}",
        flipped.len(),
        &flipped[..flipped.len().min(10)]
    );

    model
}

#[test]
fn baked_human_player_has_ccw_winding_22_joints_and_valid_height() {
    for name in [
        "human_player",
        "npc_trader",
        "npc_banker",
        "npc_guard",
        "npc_survivor",
        "npc_villager",
        "npc_mechanic",
        "npc_scout",
    ] {
        let rel = format!("characters/{name}.glb");
        let model = assert_ccw_outward(&rel);
        let skeleton = model.skeleton.as_ref().unwrap_or_else(|| panic!("{name} must have a skeleton"));
        assert_eq!(skeleton.joints.len(), 22, "{name} should have 22 joints");
        assert!(model.vertices.len() / 3 <= 5000, "{name} exceeds 5000 triangles");
        let height = model.bounds_max[1] - model.bounds_min[1];
        assert!((1.70..=1.94).contains(&height), "{name} height out of range: {height}");
        assert!(model.bounds_min[1].abs() < 0.03, "{name} feet are not at ground: {}", model.bounds_min[1]);
    }
}

/// The renderer draws glTF/Blender models (all props, cars, zombies) plus hand-built
/// prop meshes through the `skinned` pipeline with `FrontFace::Ccw`. Guard that the
/// baked vehicle parts and a representative prop really are CCW-outward, and that the
/// 4x4 body still carries every marker the game code looks up by name.
#[test]
fn baked_vehicle_and_props_have_ccw_outward_winding() {
    assert_ccw_outward("vehicles/car_hatchback_red.glb");
    assert_ccw_outward("vehicles/car_wheel.glb");
    assert_ccw_outward("vehicles/car_door_red.glb");
    assert_ccw_outward("vehicles/car_tailgate_red.glb");
    assert_ccw_outward("nature/tree_pine_01.glb");

    let body = assert_ccw_outward("vehicles/car_hatchback_red.glb");
    let present: HashSet<&str> = body.markers.iter().map(|m| m.name.as_str()).collect();
    for required in [
        "seat_driver",
        "exit_L",
        "trunk",
        "headlight_L",
        "headlight_R",
        "taillight_L",
        "taillight_R",
        "lightbar",
        "exhaust",
        "hood_smoke",
        "door_hinge_L",
        "tailgate_hinge",
        "wheel_FL",
        "wheel_FR",
        "wheel_RL",
        "wheel_RR",
    ] {
        assert!(
            present.contains(required),
            "car_hatchback_red is missing marker {required}; found {present:?}"
        );
    }

    // The wheels sit on the ground: their markers are at wheel-centre height (0.34 m).
    for name in ["wheel_FL", "wheel_FR", "wheel_RL", "wheel_RR"] {
        let marker = body
            .markers
            .iter()
            .find(|m| m.name == name)
            .expect("wheel marker checked above");
        assert!(
            (marker.translation[1] - 0.34).abs() < 0.01,
            "{name} should be at wheel-centre height 0.34 m, got {}",
            marker.translation[1]
        );
    }
}
