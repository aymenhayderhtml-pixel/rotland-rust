//! Weapon attachment mesh loading, mount-point placement, and 1st/3rd-person batching.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::Path;

use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::{AttachSlot, AttachmentRegistry};
use glam::{Mat4, Vec3};
use pack::ModelPack;

/// Engine-space `[x, y, z]` mount points per weapon id and slot (`data/weapon_mounts.json`).
#[derive(Clone, Debug, Default)]
struct WeaponMounts {
    weapons: BTreeMap<String, BTreeMap<String, [f32; 3]>>,
}

impl WeaponMounts {
    /// Loads `data/weapon_mounts.json` from the repo root.
    fn from_repo() -> Result<Self, String> {
        Self::load(&paths::repo_path("data/weapon_mounts.json"))
    }

    fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("could not read {}: {e}", path.display()))?;
        Self::from_json(&text)
            .map_err(|e| format!("could not parse {}: {e}", path.display()))
    }

    fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        let weapons: BTreeMap<String, BTreeMap<String, [f32; 3]>> = serde_json::from_str(text)?;
        Ok(Self { weapons })
    }

    /// Returns the weapon-local mount offset for `(weapon_id, slot)`, if present.
    fn get(&self, weapon_id: &str, slot: AttachSlot) -> Option<Vec3> {
        self.weapons
            .get(weapon_id)?
            .get(slot.as_str())
            .map(|&[x, y, z]| Vec3::new(x, y, z))
    }
}

/// Computes an attachment's world matrix from the parent weapon's world matrix and local mount offset.
fn attachment_world_matrix(weapon_world: Mat4, mount_local: Vec3) -> Mat4 {
    weapon_world * Mat4::from_translation(mount_local)
}

fn attachment_model_stem(model_path: &str, fallback_id: &str) -> String {
    let file = model_path.rsplit('/').next().unwrap_or("");
    let stem = file.trim_end_matches(".glb");
    if stem.is_empty() {
        fallback_id.to_string()
    } else {
        stem.to_string()
    }
}

/// Resolves `(model_stem, world_matrix)` pairs for every valid attachment fitted to `weapon_id`.
/// Enforces at most one attachment per `AttachSlot`, matching `game::effective_stats`.
fn resolve_attachment_transforms(
    weapon_id: &str,
    mods: &[String],
    reg: &AttachmentRegistry,
    mounts: &WeaponMounts,
    weapon_world: Mat4,
) -> Vec<(String, Mat4)> {
    let mut used_slots = BTreeSet::new();
    let mut out = Vec::new();
    for mod_id in mods {
        let Some(def) = reg.get(mod_id) else {
            continue;
        };
        if !def.fits_weapon(weapon_id) || !used_slots.insert(def.slot) {
            continue;
        }
        let Some(mount_local) = mounts.get(weapon_id, def.slot) else {
            continue;
        };
        let stem = attachment_model_stem(&def.model, mod_id);
        out.push((stem, attachment_world_matrix(weapon_world, mount_local)));
    }
    out
}

/// Cached GPU meshes for weapon attachments and mount-point offsets from `data/weapon_mounts.json`.
#[derive(Default)]
pub struct AttachmentView {
    mounts: WeaponMounts,
    meshes: HashMap<String, GpuMesh>,
}

impl AttachmentView {
    /// Loads `data/weapon_mounts.json` and uploads available attachment meshes from `pack`.
    pub fn new(device: &wgpu::Device, pack: &ModelPack) -> Self {
        let mounts = WeaponMounts::from_repo().unwrap_or_else(|e| {
            log::warn!("attachment_view: {e}");
            WeaponMounts::default()
        });
        let mut meshes = HashMap::new();
        match AttachmentRegistry::from_repo() {
            Ok(reg) => {
                for def in reg.iter() {
                    let stem = attachment_model_stem(&def.model, &def.id);
                    if meshes.contains_key(&stem) {
                        continue;
                    }
                    if let Some(model) = pack.find(&stem) {
                        meshes.insert(stem.clone(), GpuMesh::new(device, &stem, &model.vertices));
                    } else {
                        log::debug!("attachment_view: model '{stem}' not in pack");
                    }
                }
            }
            Err(e) => log::warn!("attachment_view: data/attachments.json: {e}"),
        }
        Self { mounts, meshes }
    }

    /// Appends `SkinnedBatch` draws for every fitted attachment on `weapon_id` at `weapon_world`.
    pub fn draw_batches<'a>(
        &'a self,
        weapon_id: &str,
        mods: &[String],
        reg: &AttachmentRegistry,
        weapon_world: Mat4,
        main: &mut Vec<SkinnedBatch<'a>>,
        mut shadow: Option<&mut Vec<SkinnedBatch<'a>>>,
    ) {
        if mods.is_empty() {
            return;
        }
        for (stem, mat) in
            resolve_attachment_transforms(weapon_id, mods, reg, &self.mounts, weapon_world)
        {
            let Some(mesh) = self.meshes.get(&stem) else {
                continue;
            };
            main.push(SkinnedBatch {
                mesh,
                instances: vec![SkinInstance::new(mat, 0)],
            });
            if let Some(sh) = shadow.as_deref_mut() {
                sh.push(SkinnedBatch {
                    mesh,
                    instances: vec![SkinInstance::new(mat, 0)],
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mounts_load_and_place_attachments_along_weapon_transform() {
        let mounts = WeaponMounts::from_repo().expect("data/weapon_mounts.json");
        let reg = AttachmentRegistry::from_repo().expect("data/attachments.json");

        let weapon_world =
            Mat4::from_translation(Vec3::new(10.0, 2.0, -5.0)) * Mat4::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let mods = vec![
            "scope_4x".to_string(),
            "scope_2x".to_string(), // duplicate optic slot: ignored
            "suppressor".to_string(),
            "drum_mag".to_string(), // doesn't fit rifle: ignored
            "unknown_mod".to_string(),
        ];

        let resolved =
            resolve_attachment_transforms("rifle", &mods, &reg, &mounts, weapon_world);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].0, "scope_4x");
        assert_eq!(resolved[1].0, "suppressor");

        let optic = mounts.get("rifle", AttachSlot::Optic).expect("rifle optic");
        let expected_pos = weapon_world.transform_point3(optic);
        let got_pos = resolved[0].1.transform_point3(Vec3::ZERO);
        assert!((got_pos - expected_pos).length() < 1e-5);
    }

    #[test]
    fn crossbow_mounts_resolve_scope_and_quiver() {
        let mounts = WeaponMounts::from_repo().expect("data/weapon_mounts.json");
        let reg = AttachmentRegistry::from_repo().expect("data/attachments.json");
        let mods = vec!["scope_2x".to_string(), "bolt_quiver".to_string()];

        let resolved =
            resolve_attachment_transforms("crossbow", &mods, &reg, &mounts, Mat4::IDENTITY);
        assert_eq!(resolved.len(), 2);
        assert_eq!(resolved[0].0, "scope_2x");
        assert_eq!(resolved[1].0, "bolt_quiver");
    }
}
