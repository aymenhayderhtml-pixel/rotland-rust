//! Lists the models in the pack whose names start with the given prefixes, with their clips.
//! `cargo run -p pack --example list -- deer boar` (no prefix: every model).
fn main() {
    let pack = pack::ModelPack::load(paths::repo_path("rust/assets/models.pack")).expect("models.pack");
    let see_through: usize = pack.models.iter().map(|m| m.vertices.iter().filter(|v| v.color[3] != 255).count()).sum();
    println!("vertices with alpha below 255: {see_through}");
    let wanted: Vec<String> = std::env::args().skip(1).collect();
    for m in &pack.models {
        if !wanted.is_empty() && !wanted.iter().any(|w| m.name.starts_with(w.as_str())) {
            continue;
        }
        let clips: Vec<String> = m.clips.iter().map(|c| format!("{} ({:.2}s)", c.name, c.duration)).collect();
        println!("{}: {} verts, skeleton {}, clips: {}", m.name, m.vertices.len(), m.skeleton.is_some(), clips.join(", "));
        if !m.markers.is_empty() {
            let names: Vec<&str> = m.markers.iter().map(|k| k.name.as_str()).collect();
            println!("  markers: {}", names.join(", "));
        }
    }
}
