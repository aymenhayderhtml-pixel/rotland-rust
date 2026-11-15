use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::marker::{ConvexCollider, Marker};
use crate::skeleton::{Clip, Skeleton};
use crate::vertex::PackVertex;

/// Pack version for compatibility checking.
pub const PACK_VERSION: u32 = 1;

/// A single baked 3D asset ready for GPU upload and game logic.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Model {
    /// File name without `.glb`, e.g. "zombie_01", "tree_pine_01_lod1".
    pub name: String,
    /// Subfolder under models directory, e.g. "characters", "nature".
    pub folder: String,
    /// Non-indexed triangle list in model space.
    pub vertices: Vec<PackVertex>,
    /// Minimum model-space coordinates of drawable vertices [x, y, z].
    pub bounds_min: [f32; 3],
    /// Maximum model-space coordinates of drawable vertices [x, y, z].
    pub bounds_max: [f32; 3],
    /// Model-space transforms of empty/marker nodes (excluding skeleton joints).
    pub markers: Vec<Marker>,
    /// Convex colliders from collision meshes.
    pub colliders: Vec<ConvexCollider>,
    /// Optional joint hierarchy and inverse bind matrices for skinned meshes.
    pub skeleton: Option<Skeleton>,
    /// Animation clips targeting skeleton joints.
    pub clips: Vec<Clip>,
}

/// The entire collection of baked game models in one compact binary file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelPack {
    /// Format version number.
    pub version: u32,
    /// All baked models in the pack.
    pub models: Vec<Model>,
}

impl ModelPack {
    /// Create a new pack with the current format version.
    pub fn new(models: Vec<Model>) -> Self {
        Self {
            version: PACK_VERSION,
            models,
        }
    }

    /// Load and deserialize a binary model pack from disk.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let p = path.as_ref();
        let bytes = std::fs::read(p)
            .map_err(|e| format!("failed to read pack file at {}: {e}", p.display()))?;
        postcard::from_bytes(&bytes)
            .map_err(|e| format!("failed to decode pack file at {}: {e}", p.display()))
    }

    /// Serialize and write the binary model pack to disk.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let p = path.as_ref();
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create directory {}: {e}", parent.display()))?;
        }
        let bytes = postcard::to_allocvec(self)
            .map_err(|e| format!("failed to serialize model pack: {e}"))?;
        std::fs::write(p, bytes)
            .map_err(|e| format!("failed to write pack file at {}: {e}", p.display()))
    }

    /// Look up a model by its name (without folder or extension).
    pub fn find(&self, name: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.name == name)
    }
}
