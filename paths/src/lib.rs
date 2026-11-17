//! Finds the project folder (the one with `Cargo.lock` and `data/`) while the program runs.
//!
//! Never bake the folder in at build time (`env!("CARGO_MANIFEST_DIR")`): all checkouts share one build
//! folder, so a program built in a worker's checkout can be reused after that checkout is deleted, and
//! then it looks for files in a folder that no longer exists.

use std::path::{Path, PathBuf};

fn is_repo(dir: &Path) -> bool {
    dir.join("Cargo.lock").is_file() && dir.join("data").is_dir()
}

fn search_up(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|d| is_repo(d)).map(Path::to_path_buf)
}

/// The project folder. Order: the ROTLAND_REPO variable, then up from the current folder (cargo runs
/// tests from the crate folder), then up from the program file.
pub fn repo_root() -> PathBuf {
    if let Some(dir) = std::env::var_os("ROTLAND_REPO").map(PathBuf::from).filter(|d| is_repo(d)) {
        return dir;
    }
    let from_cwd = std::env::current_dir().ok().and_then(|d| search_up(&d));
    let from_exe = std::env::current_exe().ok().and_then(|e| search_up(&e));
    from_cwd.or(from_exe).expect("could not find the Rotland project folder (Cargo.lock + data/); set ROTLAND_REPO")
}

/// A file or folder inside the project, e.g. `repo_path("data/items.json")`.
pub fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

#[cfg(test)]
mod tests {
    #[test]
    fn finds_the_project_from_a_crate_folder() {
        assert!(super::repo_path("data/settlements.json").is_file());
    }
}
