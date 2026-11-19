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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoNotFoundError {
    pub message: String,
}

impl std::fmt::Display for RepoNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RepoNotFoundError {}

/// Pure function to find the repo root given search locations.
///
/// Checks:
/// 1. `env_override` (if it points to a valid repo)
/// 2. `cwd` and its ancestors
/// 3. `exe` and its ancestors
///
/// If none contain both `Cargo.lock` and a `data/` directory, returns a descriptive `RepoNotFoundError`.
pub fn find_repo_root_from(
    env_override: Option<&Path>,
    cwd: Option<&Path>,
    exe: Option<&Path>,
) -> Result<PathBuf, RepoNotFoundError> {
    if let Some(dir) = env_override.filter(|d| is_repo(d)) {
        return Ok(dir.to_path_buf());
    }
    let from_cwd = cwd.and_then(search_up);
    let from_exe = exe.and_then(search_up);
    if let Some(found) = from_cwd.or(from_exe) {
        return Ok(found);
    }

    let mut tried = Vec::new();
    if let Some(env) = env_override {
        tried.push(format!("ROTLAND_REPO={}", env.display()));
    }
    if let Some(c) = cwd {
        tried.push(format!("current directory ({})", c.display()));
    }
    if let Some(e) = exe {
        tried.push(format!("executable directory ({})", e.display()));
    }
    let tried_desc = if tried.is_empty() {
        "current directory or executable directory".to_string()
    } else {
        tried.join(", ")
    };

    Err(RepoNotFoundError {
        message: format!(
            "Could not find the Rotland project folder (Cargo.lock and data/ directory) in: {tried_desc}.\n\
             Fix: set ROTLAND_REPO=/path/to/rotland-rust or run from within the Rotland repository."
        ),
    })
}

/// Attempts to discover the repo root, returning a `RepoNotFoundError` on failure.
pub fn try_repo_root() -> Result<PathBuf, RepoNotFoundError> {
    let env = std::env::var_os("ROTLAND_REPO").map(PathBuf::from);
    let cwd = std::env::current_dir().ok();
    let exe = std::env::current_exe().ok();
    find_repo_root_from(env.as_deref(), cwd.as_deref(), exe.as_deref())
}

/// The project folder. Order: the ROTLAND_REPO variable, then up from the current folder (cargo runs
/// tests from the crate folder), then up from the program file.
/// If not found, prints a clear error message and exits cleanly without a panic backtrace.
pub fn repo_root() -> PathBuf {
    match try_repo_root() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
}

/// A file or folder inside the project, e.g. `repo_path("data/items.json")`.
pub fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

/// Attempts to resolve a file or folder inside the project without exiting.
pub fn try_repo_path(relative: &str) -> Result<PathBuf, RepoNotFoundError> {
    try_repo_root().map(|root| root.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_project_from_a_crate_folder() {
        assert!(super::repo_path("data/settlements.json").is_file());
    }

    #[test]
    fn bad_dir_returns_error_message() {
        let bad = Path::new("/nonexistent/bad/directory/rotland");
        let res = find_repo_root_from(None, Some(bad), None);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.message.contains("Could not find the Rotland project folder"));
        assert!(err.message.contains("ROTLAND_REPO=/path/to/rotland-rust"));
    }
}
