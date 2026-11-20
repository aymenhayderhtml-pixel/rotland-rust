use std::path::{Path, PathBuf};

use baker::bake_glb;
use baker::budget::{check_model_budget, Budgets};
use pack::ModelPack;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let repo_root = paths::repo_root();

    let models_dir = repo_root.join("assets/models");
    let out_pack = repo_root.join("assets/models.pack");
    let budgets_path = repo_root.join("data/budgets.json");

    let report_budgets = std::env::args().any(|a| a == "--report-budgets");

    let mut glb_files = Vec::new();
    find_glb_files(&models_dir, &mut glb_files)?;
    glb_files.sort();

    let budgets = if budgets_path.exists() {
        match Budgets::load(&budgets_path) {
            Ok(b) => Some(b),
            Err(e) => {
                log::warn!("Could not load budgets from {}: {e}", budgets_path.display());
                None
            }
        }
    } else {
        None
    };

    if report_budgets {
        println!("{:<14} {:<24} {:>8} {:>8}  Status", "Folder", "Model", "Tris", "Cap");
        println!("{:-<14} {:-<24} {:-<8} {:-<8}  ------", "", "", "", "");
        let mut over_count = 0;
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
            let cap = budgets.as_ref().and_then(|b| b.models.get(&model.folder).copied());
            let report = check_model_budget(&model.folder, &model.name, tris, cap);
            let status = if report.over_budget {
                over_count += 1;
                "OVER BUDGET"
            } else {
                "OK"
            };
            let cap_str = if report.cap == u32::MAX { "None".to_string() } else { report.cap.to_string() };
            println!("{:<14} {:<24} {:>8} {:>8}  {}", report.folder, report.model, report.tris, cap_str, status);
        }
        if over_count > 0 {
            println!("\n{over_count} model(s) exceeded budget caps.");
        } else {
            println!("\nAll models within budget caps.");
        }
        return Ok(());
    }

    println!(
        "Baking {} models from {}...",
        glb_files.len(),
        models_dir.display()
    );

    let mut models = Vec::with_capacity(glb_files.len());
    let mut over_budget = Vec::new();

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

        if let Some(b) = budgets.as_ref() {
            let cap = b.models.get(&model.folder).copied().unwrap_or(u32::MAX);
            if tris as u32 > cap {
                eprintln!("BUDGET: {} {} > {}", model.name, tris, cap);
                over_budget.push((model.name.clone(), tris, cap));
            }
        }

        models.push(model);
    }

    if !over_budget.is_empty() {
        return Err(format!("{} model(s) exceeded triangle budget", over_budget.len()).into());
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
