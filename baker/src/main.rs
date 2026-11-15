use std::path::{Path, PathBuf};

use baker::bake_glb;
use pack::ModelPack;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let repo_root = paths::repo_root();

    let models_dir = repo_root.join("assets/models");
    let out_pack = repo_root.join("rust/assets/models.pack");

    let mut glb_files = Vec::new();
    find_glb_files(&models_dir, &mut glb_files)?;
    glb_files.sort();

    println!(
        "Baking {} models from {}...",
        glb_files.len(),
        models_dir.display()
    );

    let mut models = Vec::with_capacity(glb_files.len());
    for path in &glb_files {
        let rel_path = path.strip_prefix(&models_dir)?;
        let folder = rel_path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let model = bake_glb(path, name, folder)?;
        let tris = model.vertices.len() / 3;
        let markers = model.markers.len();
        let colliders = model.colliders.len();
        let joints = model.skeleton.as_ref().map(|s| s.joints.len()).unwrap_or(0);
        let clips = model.clips.len();

        println!(
            "{:<28} {:>6} tris, {:>2} markers, {:>2} colliders, {:>2} joints, {:>2} clips",
            model.name, tris, markers, colliders, joints, clips
        );

        models.push(model);
    }

    let pack = ModelPack::new(models);
    pack.save(&out_pack)?;

    let metadata = std::fs::metadata(&out_pack)?;
    let total_bytes = metadata.len();
    let mb = total_bytes as f64 / (1024.0 * 1024.0);

    println!(
        "\nWrote {} models to {} ({} bytes, {:.2} MB)",
        pack.models.len(),
        out_pack.display(),
        total_bytes,
        mb
    );

    Ok(())
}

fn find_glb_files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if name_str == "_previews" {
            continue;
        }

        if path.is_dir() {
            find_glb_files(&path, out)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("glb") {
            out.push(path);
        }
    }
    Ok(())
}
