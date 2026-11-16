use std::path::PathBuf;

#[derive(Debug)]
pub enum SaveError {
    Io(std::io::Error),
    Json(serde_json::Error),
    ProtectedPath(PathBuf),
    SlotNotFound(usize),
    UnsupportedVersion(i32),
    Corrupted(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::ProtectedPath(p) => {
                write!(f, "refusing to write or delete protected path: {}", p.display())
            }
            Self::SlotNotFound(slot) => write!(f, "save slot {slot} not found"),
            Self::UnsupportedVersion(v) => write!(f, "unsupported save version: {v}"),
            Self::Corrupted(msg) => write!(f, "corrupted save: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Json(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SaveError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for SaveError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}
