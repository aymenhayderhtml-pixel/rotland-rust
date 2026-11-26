//! Pooled explosion VFX: flash light, growing fireball, rising dark smoke puffs,
//! fading ground scorch circle, and distance-based camera shake trauma.

use std::path::Path;

use engine::render::{GpuMesh, SkinInstance, SkinnedBatch, SpotLight};
use glam::{Mat4, Vec3};
use pack::PackVertex;
use serde::Deserialize;

/// Maximum simultaneous active explosions in the fixed pool. // data: data/effects.json maxActive
pub const MAX_ACTIVE: usize = 4;

// new: DECISIONS 2026-11-25
const PUFFS_PER_EXPLOSION: usize = 5;
// new: DECISIONS 2026-11-25
const SCORCH_SEGMENTS: usize = 12;
// new: DECISIONS 2026-11-25
const SCORCH_Y_OFFSET: f32 = 0.03;
// new: DECISIONS 2026-11-25
const FLASH_HEIGHT_OFFSET: f32 = 1.5;
// new: DECISIONS 2026-11-25
const FLASH_CONE_HALF_DEG: f32 = 175.0;
// new: DECISIONS 2026-11-25
const SHAKE_PHASE_SPEED: f32 = 42.0;

// Self-lit orange fireball (`alpha = 0` triggers glow in `lit.wgsl`), dark smoke, charcoal scorch.
const FIREBALL_COLOR: [u8; 4] = [255, 165, 40, 0];
const SMOKE_COLOR: [u8; 4] = [128, 124, 120, 255];
const SCORCH_COLOR: [u8; 4] = [62, 50, 40, 255];

/// Data-driven explosion visual and camera-shake parameters (`data/effects.json`).
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplosionConfig {
    pub flash_duration_sec: f32,
    pub flash_radius_mul: f32,
    pub flash_energy: f32,
    pub flash_color: [f32; 3],
    pub fireball_grow_sec: f32,
    pub fireball_radius_mul: f32,
    pub smoke_duration_sec: f32,
    pub smoke_rise_speed: f32,
    pub scorch_duration_sec: f32,
    pub scorch_radius_mul: f32,
    pub shake_radius_mul: f32,
    pub shake_max_trauma: f32,
    pub shake_decay_per_sec: f32,
    pub shake_max_offset_m: f32,
    pub max_active: usize,
}

#[derive(Deserialize)]
struct EffectsFile {
    explosion: ExplosionConfig,
}

impl Default for ExplosionConfig {
    fn default() -> Self {
        Self {
            flash_duration_sec: 0.25, // data: data/effects.json flashDurationSec
            flash_radius_mul: 3.0,    // data: data/effects.json flashRadiusMul
            flash_energy: 8.0,        // data: data/effects.json flashEnergy
            flash_color: [1.0, 0.58, 0.2], // data: data/effects.json flashColor
            fireball_grow_sec: 0.15,  // data: data/effects.json fireballGrowSec
            fireball_radius_mul: 0.6, // data: data/effects.json fireballRadiusMul
            smoke_duration_sec: 2.0,  // data: data/effects.json smokeDurationSec
            smoke_rise_speed: 2.5,    // data: data/effects.json smokeRiseSpeed
            scorch_duration_sec: 20.0, // data: data/effects.json scorchDurationSec
            scorch_radius_mul: 0.75,  // data: data/effects.json scorchRadiusMul
            shake_radius_mul: 3.0,    // data: data/effects.json shakeRadiusMul
            shake_max_trauma: 0.8,    // data: data/effects.json shakeMaxTrauma
            shake_decay_per_sec: 2.0, // data: data/effects.json shakeDecayPerSec
            shake_max_offset_m: 0.25, // data: data/effects.json shakeMaxOffsetM
            max_active: MAX_ACTIVE,   // data: data/effects.json maxActive
        }
    }
}

impl ExplosionConfig {
    /// Loads `explosion` from `data/effects.json`, falling back to defaults if missing/invalid.
    pub fn load_from_file(path: impl AsRef<Path>) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str::<EffectsFile>(&text)
            .map(|f| f.explosion)
            .unwrap_or_default()
    }

    fn total_duration(&self) -> f32 {
        self.scorch_duration_sec
            .max(self.fireball_grow_sec + self.smoke_duration_sec)
            .max(self.flash_duration_sec)
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ExplosionSlot {
    active: bool,
    pos: Vec3,
    radius: f32,
    age: f32,
    seed: u64,
    shake_pending: bool,
}

struct ExplosionMeshes {
    fireball: GpuMesh,
    smoke: GpuMesh,
    scorch: GpuMesh,
}

impl ExplosionMeshes {
    fn new(device: &wgpu::Device) -> Self {
        let mut fb = Vec::with_capacity(24);
        push_octahedron(&mut fb, 1.0, FIREBALL_COLOR);
        let mut sm = Vec::with_capacity(24);
        push_octahedron(&mut sm, 1.0, SMOKE_COLOR);
        let mut sc = Vec::with_capacity(SCORCH_SEGMENTS * 3);
        push_scorch_disc(&mut sc, SCORCH_SEGMENTS, SCORCH_COLOR);
        Self {
            fireball: GpuMesh::new(device, "explosion fireball", &fb),
            smoke: GpuMesh::new(device, "explosion smoke", &sm),
            scorch: GpuMesh::new(device, "explosion scorch", &sc),
        }
    }
}

/// Pooled explosion visual effects and camera shake trauma.
pub struct ExplosionView {
    config: ExplosionConfig,
    slots: [ExplosionSlot; MAX_ACTIVE],
    meshes: Option<ExplosionMeshes>,
    trauma: f32,
    shake_phase: f32,
    rng: u64,
}

impl Default for ExplosionView {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplosionView {
    /// Creates an `ExplosionView` loading tuning from `data/effects.json`.
    pub fn new() -> Self {
        Self::with_config(ExplosionConfig::load_from_file(paths::repo_path("data/effects.json")))
    }

    /// Creates an `ExplosionView` with an explicit config (useful for tests).
    pub fn with_config(config: ExplosionConfig) -> Self {
        Self {
            config,
            slots: [ExplosionSlot::default(); MAX_ACTIVE],
            meshes: None,
            trauma: 0.0,
            shake_phase: 0.0,
            rng: 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Spawns an explosion at `pos` with blast `radius`.
    /// Replaces the oldest active slot when all `MAX_ACTIVE` slots are in use.
    pub fn spawn(&mut self, pos: Vec3, radius: f32) {
        if radius <= 0.0 {
            return;
        }
        let cap = self.config.max_active.clamp(1, MAX_ACTIVE);
        let idx = (0..cap)
            .find(|&i| !self.slots[i].active)
            .unwrap_or_else(|| {
                (0..cap)
                    .max_by(|&a, &b| self.slots[a].age.total_cmp(&self.slots[b].age))
                    .unwrap_or(0)
            });
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.slots[idx] = ExplosionSlot {
            active: true,
            pos,
            radius,
            age: 0.0,
            seed: self.rng,
            shake_pending: true,
        };
    }

    /// Advances all active explosions, applies distance-based trauma for new spawns, and decays shake.
    pub fn update(&mut self, dt: f32, player_pos: Vec3) {
        if dt < 0.0 {
            return;
        }
        let total = self.config.total_duration();
        let shake_mul = self.config.shake_radius_mul;
        let max_trauma = self.config.shake_max_trauma;
        for slot in &mut self.slots {
            if !slot.active {
                continue;
            }
            if slot.shake_pending {
                slot.shake_pending = false;
                let max_dist = slot.radius * shake_mul;
                let dist = slot.pos.distance(player_pos);
                if max_dist > 0.0 && dist < max_dist {
                    let falloff = 1.0 - (dist / max_dist);
                    self.trauma = (self.trauma + max_trauma * falloff).min(max_trauma);
                }
            }
            slot.age += dt;
            if slot.age >= total {
                slot.active = false;
            }
        }
        if self.trauma > 0.0 {
            self.trauma = (self.trauma - self.config.shake_decay_per_sec * dt).max(0.0);
            self.shake_phase += dt * SHAKE_PHASE_SPEED;
        }
    }

    /// Current camera position shake offset in metres.
    pub fn camera_shake_offset(&self) -> Vec3 {
        if self.trauma <= 1e-4 {
            return Vec3::ZERO;
        }
        let mag = self.trauma * self.trauma * self.config.shake_max_offset_m;
        Vec3::new(
            self.shake_phase.sin() * mag,
            (self.shake_phase * 1.37).cos() * 0.7 * mag, // new: DECISIONS 2026-11-25
            (self.shake_phase * 0.83).sin() * 0.5 * mag, // new: DECISIONS 2026-11-25
        )
    }

    /// Brightest active explosion flash light, if any explosion is within its flash window.
    pub fn flash_light(&self) -> Option<SpotLight> {
        let dur = self.config.flash_duration_sec;
        if dur <= 0.0 {
            return None;
        }
        let mut best: Option<SpotLight> = None;
        for slot in &self.slots {
            if !slot.active || slot.age >= dur {
                continue;
            }
            let t = (slot.age / dur).clamp(0.0, 1.0);
            let fade = (1.0 - t) * (1.0 - t);
            let energy = self.config.flash_energy * fade;
            if energy <= 0.01 {
                continue;
            }
            let replace = best.as_ref().is_none_or(|b| energy > b.energy);
            if replace {
                best = Some(SpotLight {
                    position: slot.pos + Vec3::Y * FLASH_HEIGHT_OFFSET,
                    direction: Vec3::NEG_Y,
                    range: slot.radius * self.config.flash_radius_mul,
                    angle_deg: FLASH_CONE_HALF_DEG,
                    color: Vec3::from(self.config.flash_color),
                    energy,
                });
            }
        }
        best
    }

    /// Number of currently active explosions (at most `MAX_ACTIVE`).
    pub fn active_count(&self) -> usize {
        self.slots.iter().filter(|s| s.active).count()
    }

    /// Appends fireball, smoke, and scorch batches to `main` (zero work when idle).
    pub fn draw<'a>(&'a mut self, device: &wgpu::Device, main: &mut Vec<SkinnedBatch<'a>>) {
        if self.active_count() == 0 {
            return;
        }
        let cfg = &self.config;
        let mut fireballs = Vec::new();
        let mut smokes = Vec::new();
        let mut scorches = Vec::new();

        for slot in &self.slots {
            if !slot.active {
                continue;
            }
            if let Some(s) = fireball_scale(slot.age, slot.radius, cfg) {
                let mat = Mat4::from_translation(slot.pos + Vec3::Y * (s * 0.5))
                    * Mat4::from_rotation_y(slot.age * 3.0)
                    * Mat4::from_scale(Vec3::splat(s));
                fireballs.push(SkinInstance::new(mat, 0));
            }
            let smoke_start = cfg.fireball_grow_sec * 0.5;
            if slot.age >= smoke_start && slot.age < smoke_start + cfg.smoke_duration_sec {
                let smoke_age = slot.age - smoke_start;
                let t = (smoke_age / cfg.smoke_duration_sec).clamp(0.0, 1.0);
                let fade = 1.0 - t;
                for i in 0..PUFFS_PER_EXPLOSION {
                    let frac = i as f32 / PUFFS_PER_EXPLOSION as f32;
                    let angle = std::f32::consts::TAU * frac + (slot.seed & 0xFF) as f32 * 0.02;
                    let spread = slot.radius * 0.45 * (0.3 + 0.7 * t);
                    let rise = cfg.smoke_rise_speed * (0.7 + 0.5 * frac) * smoke_age;
                    let pos = slot.pos + Vec3::new(angle.cos() * spread, 0.4 + rise, angle.sin() * spread);
                    let scale = slot.radius * 0.35 * (0.5 + 0.8 * t) * (0.3 + 0.7 * fade);
                    if scale <= 0.02 {
                        continue;
                    }
                    let mat = Mat4::from_translation(pos)
                        * Mat4::from_rotation_y(angle + smoke_age * 0.8)
                        * Mat4::from_scale(Vec3::splat(scale));
                    smokes.push(SkinInstance::with_fade(mat, 0, fade, false));
                }
            }
            if let Some(fade) = scorch_fade(slot.age, cfg.scorch_duration_sec) {
                let r = slot.radius * cfg.scorch_radius_mul;
                let mat = Mat4::from_translation(slot.pos + Vec3::Y * SCORCH_Y_OFFSET)
                    * Mat4::from_scale(Vec3::new(r, 1.0, r));
                scorches.push(SkinInstance::with_fade(mat, 0, fade, false));
            }
        }

        let meshes = self.meshes.get_or_insert_with(|| ExplosionMeshes::new(device));
        if !scorches.is_empty() {
            main.push(SkinnedBatch { mesh: &meshes.scorch, instances: scorches });
        }
        if !smokes.is_empty() {
            main.push(SkinnedBatch { mesh: &meshes.smoke, instances: smokes });
        }
        if !fireballs.is_empty() {
            main.push(SkinnedBatch { mesh: &meshes.fireball, instances: fireballs });
        }
    }
}

fn fireball_scale(age: f32, radius: f32, cfg: &ExplosionConfig) -> Option<f32> {
    if cfg.fireball_grow_sec <= 0.0 || age >= cfg.fireball_grow_sec {
        return None;
    }
    let t = (age / cfg.fireball_grow_sec).clamp(0.0, 1.0);
    let ease = 1.0 - (1.0 - t) * (1.0 - t);
    Some(radius * cfg.fireball_radius_mul * (0.15 + 0.85 * ease))
}

fn scorch_fade(age: f32, duration: f32) -> Option<f32> {
    if duration <= 0.0 || age >= duration {
        return None;
    }
    let t = (age / duration).clamp(0.0, 1.0);
    // Hold full darkness for first half, then fade out linearly over the second half.
    let fade = if t < 0.5 { 1.0 } else { (1.0 - (t - 0.5) * 2.0).clamp(0.0, 1.0) };
    (fade > 0.01).then_some(fade)
}

fn push_tri(out: &mut Vec<PackVertex>, p0: Vec3, p1: Vec3, p2: Vec3, color: [u8; 4]) {
    let n = (p1 - p0).cross(p2 - p0).normalize_or_zero();
    for p in [p0, p1, p2] {
        out.push(PackVertex {
            position: p.to_array(),
            normal: n.to_array(),
            color,
            bone: 0,
            _pad: [0; 3],
        });
    }
}

fn push_octahedron(out: &mut Vec<PackVertex>, scale: f32, color: [u8; 4]) {
    let axis = [
        Vec3::X,
        Vec3::NEG_X,
        Vec3::Y,
        Vec3::NEG_Y,
        Vec3::Z,
        Vec3::NEG_Z,
    ];
    for sx in [0usize, 1] {
        for sy in [2usize, 3] {
            for sz in [4usize, 5] {
                let a = axis[sx] * scale;
                let b = axis[sy] * scale;
                let c = axis[sz] * scale;
                let face_normal = axis[sx] + axis[sy] + axis[sz];
                if (b - a).cross(c - a).dot(face_normal) >= 0.0 {
                    push_tri(out, a, b, c, color);
                } else {
                    push_tri(out, a, c, b, color);
                }
            }
        }
    }
}

fn push_scorch_disc(out: &mut Vec<PackVertex>, segments: usize, color: [u8; 4]) {
    let center = Vec3::ZERO;
    for i in 0..segments {
        let a0 = -std::f32::consts::TAU * i as f32 / segments as f32;
        let a1 = -std::f32::consts::TAU * (i + 1) as f32 / segments as f32;
        let p1 = Vec3::new(a0.cos(), 0.0, a0.sin());
        let p2 = Vec3::new(a1.cos(), 0.0, a1.sin());
        push_tri(out, center, p1, p2, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flash_and_fireball_and_scorch_timings() {
        let cfg = ExplosionConfig::default();
        let mut view = ExplosionView::with_config(cfg.clone());
        view.spawn(Vec3::ZERO, 5.0);

        // At t = 0: flash is full energy, fireball is small, scorch is full.
        let light0 = view.flash_light().expect("flash should be active");
        assert!((light0.range - 15.0).abs() < 1e-4);
        assert!((light0.energy - cfg.flash_energy).abs() < 1e-4);
        assert!(fireball_scale(0.0, 5.0, &cfg).is_some());
        assert_eq!(scorch_fade(0.0, cfg.scorch_duration_sec), Some(1.0));

        // Halfway through flash (0.125s): energy = 0.25 * flash_energy, fireball near full.
        view.update(0.125, Vec3::new(100.0, 0.0, 0.0));
        let light_mid = view.flash_light().expect("flash still active at 0.125s");
        assert!((light_mid.energy - cfg.flash_energy * 0.25).abs() < 1e-3);

        // Past flash (0.3s): flash gone, fireball gone, scorch still full.
        view.update(0.175, Vec3::new(100.0, 0.0, 0.0));
        assert!(view.flash_light().is_none());
        assert!(fireball_scale(0.3, 5.0, &cfg).is_none());

        // Scorch at 75% duration (15s) -> fade = 0.5; past 20s -> expired.
        assert!((scorch_fade(15.0, cfg.scorch_duration_sec).unwrap() - 0.5).abs() < 1e-4);
        view.update(20.0, Vec3::new(100.0, 0.0, 0.0));
        assert_eq!(view.active_count(), 0);
    }

    #[test]
    fn pool_caps_at_4_and_replaces_oldest() {
        let mut view = ExplosionView::with_config(ExplosionConfig::default());
        view.spawn(Vec3::new(0.0, 0.0, 0.0), 4.0);
        view.update(0.1, Vec3::new(100.0, 0.0, 0.0));
        view.spawn(Vec3::new(20.0, 0.0, 0.0), 4.0);
        view.spawn(Vec3::new(40.0, 0.0, 0.0), 4.0);
        view.spawn(Vec3::new(60.0, 0.0, 0.0), 4.0);
        assert_eq!(view.active_count(), MAX_ACTIVE);

        // 5th spawn replaces the oldest (at x = 0).
        view.spawn(Vec3::new(80.0, 0.0, 0.0), 4.0);
        assert_eq!(view.active_count(), MAX_ACTIVE);
        assert!(view.slots.iter().all(|s| s.pos.x > 10.0));
    }

    #[test]
    fn camera_shake_triggers_within_3x_radius_and_decays() {
        let mut view = ExplosionView::with_config(ExplosionConfig::default());
        // Blast radius 5 -> shake radius 15. Player at 20 -> no shake.
        view.spawn(Vec3::ZERO, 5.0);
        view.update(0.016, Vec3::new(20.0, 0.0, 0.0));
        assert_eq!(view.camera_shake_offset(), Vec3::ZERO);

        // Player at 0 -> full trauma (0.8), decays to 0 after 0.5s.
        view.spawn(Vec3::ZERO, 5.0);
        view.update(0.016, Vec3::ZERO);
        assert!(view.camera_shake_offset().length() > 0.0);
        view.update(0.5, Vec3::ZERO);
        assert_eq!(view.camera_shake_offset(), Vec3::ZERO);
    }

    #[test]
    fn meshes_have_ccw_outward_winding() {
        let mut verts = Vec::new();
        push_octahedron(&mut verts, 1.0, FIREBALL_COLOR);
        push_scorch_disc(&mut verts, SCORCH_SEGMENTS, SCORCH_COLOR);
        for tri in verts.as_chunks::<3>().0 {
            let p0 = Vec3::from(tri[0].position);
            let p1 = Vec3::from(tri[1].position);
            let p2 = Vec3::from(tri[2].position);
            let n = Vec3::from(tri[0].normal);
            assert!((p1 - p0).cross(p2 - p0).dot(n) > 0.0);
        }
    }
}
