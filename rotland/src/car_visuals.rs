//! Car visuals: marker positions from the model pack, self-lit emissive meshes
//! (headlight lenses, brake/tail/reverse lights, roof lightbar), and pooled
//! low-poly exhaust/damage smoke VFX.
//!
//! All hand-built meshes here use CCW outward triangle winding (matching the
//! `skinned` pipeline's `FrontFace::Ccw`). A vertex alpha below 255 makes the
//! `lit` shader add a self-lit `glow = 1.0 - alpha` term.

use engine::render::{GpuMesh, SkinInstance};
use glam::{Mat4, Vec3};
use pack::PackVertex;

/// Smoke kinds, kept in one list so the draw pass stays order-stable (light, dark, ember).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SmokeKind {
    Light,
    Dark,
    Ember,
}

/// Local-space markers of the car (Godot coordinates), taken from the body
/// `.glb`'s empties when the pack has them, else from the vehicle contract.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarMarkers {
    pub trunk: Vec3,
    pub headlight_l: Vec3,
    pub headlight_r: Vec3,
    pub taillight_l: Vec3,
    pub taillight_r: Vec3,
    pub lightbar: Vec3,
    pub exhaust: Vec3,
    pub hood_smoke: Vec3,
    pub door_hinge_l: Vec3,
    pub tailgate_hinge: Vec3,
}

impl Default for CarMarkers {
    fn default() -> Self {
        Self {
            trunk: Vec3::new(0.0, 0.75, 2.35),
            headlight_l: Vec3::new(-0.60, 0.80, -1.98),
            headlight_r: Vec3::new(0.60, 0.80, -1.98),
            taillight_l: Vec3::new(-0.79, 0.84, 1.93),
            taillight_r: Vec3::new(0.79, 0.84, 1.93),
            lightbar: Vec3::new(0.0, 1.68, -0.38),
            exhaust: Vec3::new(0.46, 0.34, 2.04),
            hood_smoke: Vec3::new(0.0, 1.08, -1.45),
            door_hinge_l: Vec3::new(-0.84, 0.72, -0.80),
            tailgate_hinge: Vec3::new(0.74, 0.90, 1.90),
        }
    }
}

impl CarMarkers {
    /// Reads the markers from the red (or blue) hatchback body model in the pack,
    /// falling back to the contract defaults for any missing empty.
    pub fn from_pack(pack: &pack::ModelPack) -> Self {
        let body = pack.find("car_hatchback_red").or_else(|| pack.find("car_hatchback_blue"));
        let m = |name: &str, fallback: Vec3| {
            body.and_then(|b| b.markers.iter().find(|k| k.name == name))
                .map_or(fallback, |k| Vec3::from(k.translation))
        };
        let d = Self::default();
        Self {
            trunk: m("trunk", d.trunk),
            headlight_l: m("headlight_L", d.headlight_l),
            headlight_r: m("headlight_R", d.headlight_r),
            taillight_l: m("taillight_L", d.taillight_l),
            taillight_r: m("taillight_R", d.taillight_r),
            lightbar: m("lightbar", d.lightbar),
            exhaust: m("exhaust", d.exhaust),
            hood_smoke: m("hood_smoke", d.hood_smoke),
            door_hinge_l: m("door_hinge_L", d.door_hinge_l),
            tailgate_hinge: m("tailgate_hinge", d.tailgate_hinge),
        }
    }
}

/// Box corners in CCW order as seen from outside the face (see test at the bottom).
const BOX_FACES: [([f32; 3], [[f32; 3]; 4]); 6] = [
    ([1.0, 0.0, 0.0], [[1.0, -1.0, 1.0], [1.0, -1.0, -1.0], [1.0, 1.0, -1.0], [1.0, 1.0, 1.0]]),
    ([-1.0, 0.0, 0.0], [[-1.0, -1.0, -1.0], [-1.0, -1.0, 1.0], [-1.0, 1.0, 1.0], [-1.0, 1.0, -1.0]]),
    ([0.0, 1.0, 0.0], [[-1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, -1.0], [-1.0, 1.0, -1.0]]),
    ([0.0, -1.0, 0.0], [[-1.0, -1.0, -1.0], [1.0, -1.0, -1.0], [1.0, -1.0, 1.0], [-1.0, -1.0, 1.0]]),
    ([0.0, 0.0, 1.0], [[-1.0, -1.0, 1.0], [1.0, -1.0, 1.0], [1.0, 1.0, 1.0], [-1.0, 1.0, 1.0]]),
    ([0.0, 0.0, -1.0], [[1.0, -1.0, -1.0], [-1.0, -1.0, -1.0], [-1.0, 1.0, -1.0], [1.0, 1.0, -1.0]]),
];

fn push_tri(out: &mut Vec<PackVertex>, p0: Vec3, p1: Vec3, p2: Vec3, color: [u8; 4]) {
    let n = (p1 - p0).cross(p2 - p0).normalize_or_zero();
    for p in [p0, p1, p2] {
        out.push(PackVertex { position: p.to_array(), normal: n.to_array(), color, bone: 0, _pad: [0; 3] });
    }
}

/// Pushes a box centred at `center` with half-size `half`, CCW outward, flat shaded.
fn push_box(out: &mut Vec<PackVertex>, center: Vec3, half: Vec3, color: [u8; 4]) {
    for (_, corners) in BOX_FACES {
        let at = |c: [f32; 3]| center + Vec3::new(c[0] * half.x, c[1] * half.y, c[2] * half.z);
        let v = [at(corners[0]), at(corners[1]), at(corners[2]), at(corners[3])];
        push_tri(out, v[0], v[1], v[2], color);
        push_tri(out, v[0], v[2], v[3], color);
    }
}

/// Pushes a unit octahedron: vertices on the axes, one octant per triangle, CCW outward.
fn push_octahedron(out: &mut Vec<PackVertex>, center: Vec3, scale: f32, color: [u8; 4]) {
    let axis = [
        Vec3::new(1.0, 0.0, 0.0),
        Vec3::new(-1.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(0.0, 0.0, -1.0),
    ];
    for sx in [0usize, 1] {
        for sy in [2usize, 3] {
            for sz in [4usize, 5] {
                let a = axis[sx] * scale + center;
                let b = axis[sy] * scale + center;
                let c = axis[sz] * scale + center;
                // Flip b/c when the winding would face inward
                let normal = (b - a).cross(c - a);
                let face_normal = axis[sx] + axis[sy] + axis[sz];
                if normal.dot(face_normal) >= 0.0 {
                    push_tri(out, a, b, c, color);
                } else {
                    push_tri(out, a, c, b, color);
                }
            }
        }
    }
}

/// Self-lit glow colours for the emissive car meshes (`alpha < 255` => self-lit).
const WARM_WHITE: [u8; 4] = [255, 248, 214, 0];
const LED_WHITE: [u8; 4] = [245, 250, 255, 0];
const TAIL_RED: [u8; 4] = [220, 32, 24, 110];
const BRAKE_RED: [u8; 4] = [255, 28, 18, 0];
const REVERSE_WHITE: [u8; 4] = [248, 250, 255, 0];
const SMOKE_LIGHT: [u8; 4] = [168, 172, 178, 255];
const SMOKE_DARK: [u8; 4] = [42, 40, 42, 255];
const EMBER_ORANGE: [u8; 4] = [255, 130, 30, 0];

/// Emissive meshes for the car's lamps and the smoke/ember effect, built once at start.
pub struct CarEmissiveMeshes {
    pub headlights: GpuMesh,
    pub taillights: GpuMesh,
    pub brakelights: GpuMesh,
    pub reverselights: GpuMesh,
    pub smoke_light: GpuMesh,
    pub smoke_dark: GpuMesh,
    pub ember: GpuMesh,
}

impl CarEmissiveMeshes {
    pub fn new(device: &wgpu::Device, markers: &CarMarkers) -> Self {
        let mut head = Vec::new();
        for hl in [markers.headlight_l, markers.headlight_r] {
            // Sits ~2.5 mm proud of the white lens face — zero z-fighting
            push_box(&mut head, hl + Vec3::new(0.0, 0.0, -0.015), Vec3::new(0.112, 0.066, 0.008), WARM_WHITE);
        }
        // 4 roof LED lightbar pods, a bit proud of the rack glass
        for x in [-0.36, -0.12, 0.12, 0.36] {
            push_box(&mut head, Vec3::new(x, markers.lightbar.y, markers.lightbar.z - 0.033), Vec3::new(0.092, 0.028, 0.006), LED_WHITE);
        }
        let mut tail = Vec::new();
        for tl in [markers.taillight_l, markers.taillight_r] {
            push_box(&mut tail, tl + Vec3::new(0.0, 0.08, 0.022), Vec3::new(0.052, 0.082, 0.005), TAIL_RED);
        }
        let mut brake = Vec::new();
        for tl in [markers.taillight_l, markers.taillight_r] {
            push_box(&mut brake, tl + Vec3::new(0.0, 0.08, 0.025), Vec3::new(0.056, 0.085, 0.006), BRAKE_RED);
        }
        let mut rev = Vec::new();
        for tl in [markers.taillight_l, markers.taillight_r] {
            push_box(&mut rev, tl + Vec3::new(0.0, -0.13, 0.022), Vec3::new(0.052, 0.036, 0.005), REVERSE_WHITE);
        }
        let unit = |device: &wgpu::Device, label: &str, color: [u8; 4]| {
            let mut v = Vec::new();
            push_octahedron(&mut v, Vec3::ZERO, 1.0, color);
            GpuMesh::new(device, label, &v)
        };
        Self {
            headlights: GpuMesh::new(device, "car headlights", &head),
            taillights: GpuMesh::new(device, "car taillights", &tail),
            brakelights: GpuMesh::new(device, "car brake lights", &brake),
            reverselights: GpuMesh::new(device, "car reverse lights", &rev),
            smoke_light: unit(device, "car smoke light", SMOKE_LIGHT),
            smoke_dark: unit(device, "car smoke dark", SMOKE_DARK),
            ember: unit(device, "car ember", EMBER_ORANGE),
        }
    }
}

/// One active puff in the smoke pool.
#[derive(Clone, Copy, Debug)]
struct Puff {
    kind: SmokeKind,
    pos: Vec3,
    vel: Vec3,
    age: f32,
    lifetime: f32,
    base_scale: f32,
    /// Y-axis spin rate over the puff's life (from the spawn jitter).
    spin: f32,
}

/// Spawn accumulators so fractional emit rates become whole puffs smoothly.
#[derive(Clone, Copy, Default)]
struct EmitAcc {
    exhaust: f32,
    steam: f32,
    dark: f32,
    ember: f32,
}

/// xorshift64* — deterministic, fast, good enough for smoke jitter.
struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        // 26 highest bits mapped into [0, 1)
        (x >> 38) as f32 / 67_108_864.0
    }
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }
}

const MAX_PUFFS: usize = 96;

/// Pooled exhaust and damage smoke for all cars (capped, no per-frame allocation growth).
pub struct CarSmokeSystem {
    puffs: Vec<Puff>,
    emit: std::collections::HashMap<String, EmitAcc>,
    rng: Rng,
}

impl Default for CarSmokeSystem {
    fn default() -> Self {
        Self { puffs: Vec::with_capacity(MAX_PUFFS), emit: std::collections::HashMap::new(), rng: Rng(0x9E37_79B9_7F4A_7C15) }
    }
}

impl CarSmokeSystem {
    pub fn clear(&mut self) {
        self.puffs.clear();
        self.emit.clear();
    }

    pub fn is_active(&self) -> bool {
        !self.puffs.is_empty()
    }

    pub fn update(&mut self, dt: f32, cars: &[game::CarDrawState], markers: &CarMarkers) {
        // Advance and drop expired puffs
        let mut i = 0;
        while i < self.puffs.len() {
            let p = &mut self.puffs[i];
            p.age += dt;
            if p.age >= p.lifetime {
                self.puffs.swap_remove(i);
                continue;
            }
            p.pos += p.vel * dt;
            p.vel.y += 0.35 * dt;
            let drag = (1.0 - 1.4 * dt).max(0.0);
            p.vel *= drag;
            i += 1;
        }

        for car in cars {
            let mut acc = *self.emit.get(&car.id).unwrap_or(&EmitAcc::default());
            let can_spawn = |system: &Self, need: usize| system.puffs.len() + need < MAX_PUFFS;

            let spawn = |system: &mut Self, kind: SmokeKind, at: Vec3, vel: Vec3, base_scale: f32, lifetime: f32| {
                if system.puffs.len() >= MAX_PUFFS {
                    return;
                }
                let spin = system.rng.range(-1.5, 1.5);
                system.puffs.push(Puff { kind, pos: at, vel, age: 0.0, lifetime, base_scale, spin });
            };

            // Exhaust: only when the engine runs; rate follows engine rpm
            if car.engine_on {
                acc.exhaust += (3.5 + 8.0 * car.engine_rpm) * dt;
                while acc.exhaust >= 1.0 {
                    acc.exhaust -= 1.0;
                    if !can_spawn(self, 1) {
                        acc.exhaust = acc.exhaust.min(0.0);
                        break;
                    }
                    let rear = car.world_matrix.transform_vector3(Vec3::Z).normalize_or_zero();
                    let at = car.world_matrix.transform_point3(markers.exhaust);
                    let vel = rear * self.rng.range(0.6, 1.0)
                        + Vec3::Y * self.rng.range(0.4, 0.7)
                        + Vec3::new(self.rng.range(-0.06, 0.06), 0.0, self.rng.range(-0.06, 0.06));
                    spawn(self, SmokeKind::Light, at, vel, 0.07, 0.75);
                }
            }

            // Engine damage: rising smoke (steam, then dark smoke + embers)
            if car.health_fraction < 0.55 {
                let at = car.world_matrix.transform_point3(markers.hood_smoke);
                if car.is_wrecked || car.health_fraction < 0.28 {
                    acc.dark += 8.0 * dt;
                    while acc.dark >= 1.0 {
                        acc.dark -= 1.0;
                        if !can_spawn(self, 1) {
                            acc.dark = acc.dark.min(0.0);
                            break;
                        }
                        let vel = Vec3::Y * self.rng.range(0.9, 1.4)
                            + Vec3::new(self.rng.range(-0.15, 0.15), 0.0, self.rng.range(-0.15, 0.15));
                        spawn(self, SmokeKind::Dark, at, vel, 0.15, 1.5);
                    }
                    acc.ember += 2.5 * dt;
                    while acc.ember >= 1.0 {
                        acc.ember -= 1.0;
                        if !can_spawn(self, 1) {
                            acc.ember = acc.ember.min(0.0);
                            break;
                        }
                        let vel = Vec3::Y * self.rng.range(1.4, 2.1)
                            + Vec3::new(self.rng.range(-0.35, 0.35), 0.0, self.rng.range(-0.35, 0.35));
                        spawn(self, SmokeKind::Ember, at, vel, 0.045, 0.65);
                    }
                } else {
                    acc.steam += 4.0 * dt;
                    while acc.steam >= 1.0 {
                        acc.steam -= 1.0;
                        if !can_spawn(self, 1) {
                            acc.steam = acc.steam.min(0.0);
                            break;
                        }
                        let vel = Vec3::Y * self.rng.range(0.7, 1.1)
                            + Vec3::new(self.rng.range(-0.1, 0.1), 0.0, self.rng.range(-0.1, 0.1));
                        spawn(self, SmokeKind::Light, at, vel, 0.11, 1.2);
                    }
                }
            }

            self.emit.insert(car.id.clone(), acc);
        }
    }

    /// Instances for the three smoke/ember meshes (bone 0), grouped light / dark / ember.
    /// Scale envelope: smooth grow then fade, with eased steps (no hard edges).
    pub fn instances(&self) -> (Vec<SkinInstance>, Vec<SkinInstance>, Vec<SkinInstance>) {
        let mut light = Vec::new();
        let mut dark = Vec::new();
        let mut ember = Vec::new();
        for p in &self.puffs {
            let t = (p.age / p.lifetime).clamp(0.0, 1.0);
            let grow = smoothstep(0.0, 0.18, t);
            let fade = 1.0 - smoothstep(0.55, 1.0, t);
            let scale = p.base_scale * (0.5 + 1.6 * grow) * (0.35 + 0.65 * fade);
            let matrix = Mat4::from_translation(p.pos)
                * Mat4::from_rotation_y(p.age * p.spin)
                * Mat4::from_scale(Vec3::splat(scale));
            let instance = SkinInstance::new(matrix, 0);
            match p.kind {
                SmokeKind::Light => light.push(instance),
                SmokeKind::Dark => dark.push(instance),
                SmokeKind::Ember => ember.push(instance),
            }
        }
        (light, dark, ember)
    }
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ccw_check(verts: &[PackVertex]) {
        for tri in verts.as_chunks::<3>().0 {
            let p0 = Vec3::from(tri[0].position);
            let p1 = Vec3::from(tri[1].position);
            let p2 = Vec3::from(tri[2].position);
            let n = Vec3::from(tri[0].normal);
            assert!(
                (p1 - p0).cross(p2 - p0).dot(n) > 0.0,
                "triangle must be wound CCW outward, got normal {n:?}"
            );
        }
    }

    #[test]
    fn box_triangles_are_ccw_outward() {
        for center in [Vec3::ZERO, Vec3::new(1.0, 2.0, 3.0)] {
            for half in [Vec3::new(0.1, 0.05, 0.008), Vec3::new(1.0, 1.0, 1.0)] {
                let mut v = Vec::new();
                push_box(&mut v, center, half, LED_WHITE);
                ccw_check(&v);
            }
        }
    }

    #[test]
    fn octahedron_triangles_are_ccw_outward() {
        for scale in [0.045, 1.0] {
            let mut v = Vec::new();
            push_octahedron(&mut v, Vec3::new(0.5, -1.0, 2.0), scale, SMOKE_DARK);
            ccw_check(&v);
        }
    }
}
