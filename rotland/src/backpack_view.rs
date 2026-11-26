//! Visible backpack mesh attached to the player's `chest` bone (Godot `PlayerBackpack.cs`).
//!
//! Draws `backpack_small` or `backpack_large` from `ModelPack` in third person when the player
//! carries the corresponding backpack item; hidden in first person or when no backpack is carried.

use engine::anim::Rig;
use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::items::Inventory;
use glam::{EulerRot, Mat4, Vec3};
use pack::ModelPack;
use serde::Deserialize;

use crate::player_visual::PlayerVisual;

/// Which backpack model to display on the player's back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackpackKind {
    Small,
    Large,
}

impl BackpackKind {
    /// Model name inside `ModelPack` (`assets/models/gear/<name>.glb`).
    pub fn model_name(self) -> &'static str {
        match self {
            Self::Small => "backpack_small",
            Self::Large => "backpack_large",
        }
    }
}

/// Bone-local attachment offset and Euler rotation (degrees) for a backpack model.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackpackAttach {
    pub offset: Vec3,
    pub rotation_deg: Vec3,
}

impl Default for BackpackAttach {
    fn default() -> Self {
        Self {
            // data: data/backpack.json offset
            offset: Vec3::new(0.0, 0.14, 0.10),
            // data: data/backpack.json rotation_deg
            rotation_deg: Vec3::ZERO,
        }
    }
}

#[derive(Deserialize)]
struct RawBackpackEntry {
    offset: [f32; 3],
    rotation_deg: [f32; 3],
}

#[derive(Deserialize)]
struct RawBackpackConfig {
    backpack_small: RawBackpackEntry,
    backpack_large: RawBackpackEntry,
}

/// Attachment offsets loaded from `data/backpack.json`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BackpackConfig {
    pub small: BackpackAttach,
    pub large: BackpackAttach,
}

impl BackpackConfig {
    /// Loads `data/backpack.json`, falling back to defaults if missing or invalid.
    pub fn load_shared() -> Self {
        let path = paths::repo_path("data/backpack.json");
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::from_json_str(&text).unwrap_or_else(|e| {
                log::warn!("backpack_view: failed to parse {}: {e}; using defaults", path.display());
                Self::default()
            }),
            Err(e) => {
                log::warn!("backpack_view: could not read {}: {e}; using defaults", path.display());
                Self::default()
            }
        }
    }

    pub fn from_json_str(text: &str) -> Result<Self, serde_json::Error> {
        let raw: RawBackpackConfig = serde_json::from_str(text)?;
        Ok(Self {
            small: BackpackAttach {
                offset: Vec3::from_array(raw.backpack_small.offset),
                rotation_deg: Vec3::from_array(raw.backpack_small.rotation_deg),
            },
            large: BackpackAttach {
                offset: Vec3::from_array(raw.backpack_large.offset),
                rotation_deg: Vec3::from_array(raw.backpack_large.rotation_deg),
            },
        })
    }

    pub fn get(&self, kind: BackpackKind) -> BackpackAttach {
        match kind {
            BackpackKind::Small => self.small,
            BackpackKind::Large => self.large,
        }
    }
}

/// Computes the local offset matrix from the `chest` bone to the backpack origin.
pub fn backpack_local_matrix(attach: BackpackAttach) -> Mat4 {
    Mat4::from_translation(attach.offset)
        * Mat4::from_euler(
            EulerRot::XYZ,
            attach.rotation_deg.x.to_radians(),
            attach.rotation_deg.y.to_radians(),
            attach.rotation_deg.z.to_radians(),
        )
}

/// Computes the world-space transform of a backpack attached to `chest_joint`.
pub fn backpack_world_matrix(
    char_world: Mat4,
    rig: &Rig,
    skin: &[Mat4],
    chest_joint: usize,
    attach: BackpackAttach,
) -> Mat4 {
    let chest_model = rig.joint_model_matrix(skin, chest_joint);
    char_world * chest_model * backpack_local_matrix(attach)
}

/// Picks which backpack model to show based on the player's inventory.
///
/// Prefers `backpack_large` over `backpack_small`; returns `None` if neither is carried.
pub fn select_backpack(inv: &Inventory) -> Option<BackpackKind> {
    if inv.has("backpack_large", 1) {
        // new: DECISIONS 2026-11-25
        Some(BackpackKind::Large)
    } else if inv.has("backpack_small", 1) {
        // new: DECISIONS 2026-11-25
        Some(BackpackKind::Small)
    } else {
        None
    }
}

/// GPU meshes and attachment config for the player's visible backpack.
pub struct BackpackView {
    small_mesh: Option<GpuMesh>,
    large_mesh: Option<GpuMesh>,
    config: BackpackConfig,
}

impl BackpackView {
    /// Loads `backpack_small` and `backpack_large` meshes from `pack` and offsets from `data/backpack.json`.
    pub fn load(device: &wgpu::Device, pack: &ModelPack) -> Self {
        let load_mesh = |name: &str| -> Option<GpuMesh> {
            match pack.find(name) {
                Some(model) => Some(GpuMesh::new(device, name, &model.vertices)),
                None => {
                    log::warn!("backpack_view: model '{name}' not found in pack");
                    None
                }
            }
        };
        Self {
            small_mesh: load_mesh(BackpackKind::Small.model_name()),
            large_mesh: load_mesh(BackpackKind::Large.model_name()),
            config: BackpackConfig::load_shared(),
        }
    }

    /// Returns a `SkinnedBatch` for the equipped backpack in third person, or `None` when hidden/unequipped.
    pub fn draw_batch(
        &self,
        pv: &PlayerVisual,
        kind: Option<BackpackKind>,
        first_person: bool,
        char_world: Mat4,
        bones: &[Mat4],
        bone_base: usize,
    ) -> Option<SkinnedBatch<'_>> {
        if first_person {
            return None;
        }
        let kind = kind?;
        let mesh = match kind {
            BackpackKind::Small => self.small_mesh.as_ref()?,
            BackpackKind::Large => self.large_mesh.as_ref()?,
        };
        let chest_joint = pv.chest_joint()?;
        let skin = bones.get(bone_base..)?;
        let attach = self.config.get(kind);
        let world = backpack_world_matrix(char_world, pv.rig(), skin, chest_joint, attach);
        Some(SkinnedBatch {
            mesh,
            instances: vec![SkinInstance::new(world, 0)],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::items::ItemStack;
    use glam::Quat;
    use pack::{Joint, Skeleton};

    fn test_chest_rig() -> (Rig, Vec<Mat4>) {
        let chest_rest_y = 1.22_f32;
        let skel = Skeleton {
            joints: vec![
                Joint {
                    name: "root".to_string(),
                    parent: None,
                    inverse_bind: Mat4::IDENTITY.to_cols_array(),
                    rest_translation: [0.0, 0.0, 0.0],
                    rest_rotation: [0.0, 0.0, 0.0, 1.0],
                    rest_scale: [1.0, 1.0, 1.0],
                },
                Joint {
                    name: "chest".to_string(),
                    parent: Some(0),
                    inverse_bind: Mat4::from_translation(Vec3::new(0.0, -chest_rest_y, 0.0))
                        .to_cols_array(),
                    rest_translation: [0.0, chest_rest_y, 0.0],
                    rest_rotation: [0.0, 0.0, 0.0, 1.0],
                    rest_scale: [1.0, 1.0, 1.0],
                },
            ],
        };
        let rig = Rig::new(&skel);
        let local = vec![
            (Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
            (Vec3::new(0.0, chest_rest_y, 0.0), Quat::IDENTITY, Vec3::ONE),
        ];
        let mut skin = vec![Mat4::IDENTITY; 2];
        rig.skin(&local, &mut skin);
        (rig, skin)
    }

    #[test]
    fn select_backpack_picks_none_small_or_large() {
        let mut inv = Inventory::new();
        assert_eq!(select_backpack(&inv), None);

        inv.slots_mut()[0] = Some(ItemStack::new("backpack_small", 1));
        assert_eq!(select_backpack(&inv), Some(BackpackKind::Small));
        assert_eq!(BackpackKind::Small.model_name(), "backpack_small");

        inv.slots_mut()[1] = Some(ItemStack::new("backpack_large", 1));
        assert_eq!(select_backpack(&inv), Some(BackpackKind::Large));
        assert_eq!(BackpackKind::Large.model_name(), "backpack_large");
    }

    #[test]
    fn backpack_config_parses_json() {
        let json = r#"{
            "backpack_small": { "bone": "chest", "offset": [0.0, 0.14, 0.10], "rotation_deg": [0.0, 0.0, 0.0] },
            "backpack_large": { "bone": "chest", "offset": [0.0, 0.15, 0.12], "rotation_deg": [5.0, 0.0, 0.0] }
        }"#;
        let cfg = BackpackConfig::from_json_str(json).expect("valid backpack json");
        assert_eq!(cfg.small.offset, Vec3::new(0.0, 0.14, 0.10));
        assert_eq!(cfg.large.offset, Vec3::new(0.0, 0.15, 0.12));
        assert_eq!(cfg.large.rotation_deg, Vec3::new(5.0, 0.0, 0.0));
    }

    #[test]
    fn backpack_world_matrix_attaches_behind_chest_bone() {
        let (rig, skin) = test_chest_rig();
        let chest_joint = rig.joint_index("chest").expect("chest joint");
        let char_world = Mat4::from_translation(Vec3::new(10.0, 0.0, -4.0));
        let attach = BackpackAttach::default();

        let world = backpack_world_matrix(char_world, &rig, &skin, chest_joint, attach);
        let pos = world.transform_point3(Vec3::ZERO);

        // Chest at y=1.22 + offset (0.0, 0.14, 0.10) => (10.0, 1.36, -3.90)
        assert!((pos - Vec3::new(10.0, 1.36, -3.90)).length() < 1e-5);
    }
}
