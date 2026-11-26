//! Gun feel view: camera kick, first-person weapon kick, and muzzle flash rendering.

use engine::render::{GpuMesh, SkinInstance, SkinnedBatch, SpotLight};
use game::gun_feel::{GunFeelConfig, GunFeelState};
use glam::{Mat4, Quat, Vec3};
use pack::PackVertex;

use crate::light;

/// Self-lit yellow-white color (alpha = 0 makes `lit.wgsl` render unlit/emissive).
const FLASH_COLOR: [u8; 4] = [255, 196, 90, 0]; // new: DECISIONS 2026-11-25
const FLASH_HALF_W: f32 = 0.06; // new: DECISIONS 2026-11-25
const FLASH_LEN: f32 = 0.2; // new: DECISIONS 2026-11-25

/// Manages per-frame gun recoil view offsets, FP weapon kick, and the muzzle flash quad + light.
pub struct GunFeelView {
    config: GunFeelConfig,
    state: GunFeelState,
    active_weapon: Option<String>,
    aiming: bool,
    flash_timer: f32,
    flash_pos: Vec3,
    flash_dir: Vec3,
    flash_radius: f32,
    flash_mesh: Option<GpuMesh>,
}

impl Default for GunFeelView {
    fn default() -> Self {
        Self::new()
    }
}

impl GunFeelView {
    pub fn new() -> Self {
        let config = GunFeelConfig::load()
            .unwrap_or_else(|e| panic!("failed to load data/gun_feel.json: {e}"));
        Self {
            config,
            state: GunFeelState::default(),
            active_weapon: None,
            aiming: false,
            flash_timer: 0.0,
            flash_pos: Vec3::ZERO,
            flash_dir: Vec3::NEG_Z,
            flash_radius: 0.0,
            flash_mesh: None,
        }
    }

    /// Lazily creates the 2-crossed-quad star-shaped muzzle flash mesh on the GPU.
    pub fn init(&mut self, device: &wgpu::Device) {
        if self.flash_mesh.is_none() {
            let verts = build_flash_star_vertices();
            self.flash_mesh = Some(GpuMesh::new(device, "muzzle_flash", &verts));
        }
    }

    /// Records a shot fired by `weapon_id`, applying recoil kick and triggering muzzle flash.
    pub fn on_shot(&mut self, weapon_id: &str) {
        self.active_weapon = Some(weapon_id.to_string());
        if let Some(def) = self.config.get(weapon_id) {
            self.state.on_shot(def, self.aiming);
            if !def.no_flash && def.flash_duration_sec > 0.0 {
                self.flash_timer = def.flash_duration_sec;
                self.flash_radius = def.flash_light;
            }
        }
    }

    /// Sets the world-space muzzle position and forward direction for the flash quad and light.
    pub fn set_muzzle(&mut self, pos: Vec3, dir: Vec3) {
        self.flash_pos = pos;
        let norm = dir.normalize_or_zero();
        self.flash_dir = if norm.length_squared() > 1e-6 {
            norm
        } else {
            Vec3::NEG_Z
        };
    }

    /// Updates the active weapon id and aiming state used for kick recovery and aim multiplier.
    pub fn set_context(&mut self, weapon_id: Option<&str>, aiming: bool) {
        self.active_weapon = weapon_id.map(str::to_string);
        self.aiming = aiming;
    }

    /// Advances recoil recovery and muzzle flash timer by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        let def = self
            .active_weapon
            .as_deref()
            .and_then(|id| self.config.get(id));
        self.state.update(dt, def);
        self.flash_timer = (self.flash_timer - dt).max(0.0);
    }

    /// Current camera recoil offset `(pitch_deg, yaw_deg)`.
    pub fn view_kick_deg(&self) -> (f32, f32) {
        self.state.view_kick_deg()
    }

    /// Current first-person weapon kick `(fp_back_m, fp_tilt_deg)`.
    pub fn fp_kick(&self) -> (f32, f32) {
        self.state.fp_kick()
    }

    /// Short-lived muzzle flash light when active.
    pub fn flash_light(&self) -> Option<SpotLight> {
        if self.flash_timer > 0.0 && self.flash_radius > 0.0 {
            Some(light::muzzle_flash(
                self.flash_pos,
                self.flash_dir,
                self.flash_radius,
            ))
        } else {
            None
        }
    }

    /// Appends the muzzle flash star mesh to `batches` when the flash is active and initialized.
    pub fn draw_flash<'a>(&'a self, batches: &mut Vec<SkinnedBatch<'a>>) {
        if self.flash_timer > 0.0
            && let Some(mesh) = self.flash_mesh.as_ref()
        {
            let rot = Quat::from_rotation_arc(Vec3::NEG_Z, self.flash_dir);
            let mat = Mat4::from_rotation_translation(rot, self.flash_pos);
            batches.push(SkinnedBatch {
                mesh,
                instances: vec![SkinInstance::new(mat, 0)],
            });
        }
    }
}

/// Builds 2 crossed star-shaped quads (double-sided) along -Z for the muzzle flash.
fn build_flash_star_vertices() -> Vec<PackVertex> {
    let mut verts = Vec::with_capacity(24);
    let planes = [
        (Vec3::X * FLASH_HALF_W, Vec3::Y),
        (Vec3::Y * FLASH_HALF_W, Vec3::X),
    ];
    for (side, normal) in planes {
        let base = Vec3::ZERO;
        let left = Vec3::NEG_Z * (FLASH_LEN * 0.25) - side;
        let right = Vec3::NEG_Z * (FLASH_LEN * 0.25) + side;
        let tip = Vec3::NEG_Z * FLASH_LEN;

        let front_n = normal.to_array();
        let back_n = (-normal).to_array();
        for [a, b, c] in [[base, right, tip], [base, tip, left]] {
            for p in [a, b, c] {
                verts.push(PackVertex {
                    position: p.to_array(),
                    normal: front_n,
                    color: FLASH_COLOR,
                    ..PackVertex::ZERO
                });
            }
            for p in [a, c, b] {
                verts.push(PackVertex {
                    position: p.to_array(),
                    normal: back_n,
                    color: FLASH_COLOR,
                    ..PackVertex::ZERO
                });
            }
        }
    }
    verts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_star_mesh_has_24_double_sided_emissive_vertices() {
        let verts = build_flash_star_vertices();
        assert_eq!(verts.len(), 24);
        assert!(verts.iter().all(|v| v.color[3] == 0));
    }
}
