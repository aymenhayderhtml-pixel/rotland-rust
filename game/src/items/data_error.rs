//! Data loading errors and safe world generator loading.

use std::path::Path;

/// Error when loading required game data files. Names the specific file that failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataError {
    pub file_name: String,
    pub message: String,
}

impl std::fmt::Display for DataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "failed to load '{}': {}", self.file_name, self.message)
    }
}

impl std::error::Error for DataError {}

/// Loads a `WorldGenerator` from the given data directory with clean file-naming error reporting.
pub fn load_world_generator(
    seed: i32,
    data_dir: impl AsRef<Path>,
) -> Result<world::WorldGenerator, DataError> {
    let dir = data_dir.as_ref();
    world::WorldGenerator::from_data_dir(seed, dir).map_err(|err| {
        let (file_name, msg) = if let Some((file, rest)) = err.split_once(':') {
            (file.trim().to_string(), rest.trim().to_string())
        } else {
            ("data files".to_string(), err)
        };
        DataError {
            file_name,
            message: msg,
        }
    })
}
