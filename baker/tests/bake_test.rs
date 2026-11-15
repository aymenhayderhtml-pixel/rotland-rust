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
