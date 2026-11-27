//! Draws New Halden's silent life: startled/circling crows and flapping flags/plastic (R41-05).

use std::collections::HashMap;

use engine::render::{GpuMesh, SkinInstance, SkinnedBatch};
use game::crows::{Circlers, CrowFlock, CrowModel, CrowsConfig, flap_frame};
use glam::{IVec2, Mat4, Vec3};
use world::WorldGenerator;
use world::metro::life::{CrowPerch, FlapKind, Flapper, crow_perches_in_chunk, flappers_in_chunk};

use crate::streaming::Streamer;

const MODEL_NAMES: [&str; 14] = [
    "crow_perched",
    "crow_peck",
    "crow_fly_f0",
    "crow_fly_f1",
    "crow_fly_f2",
    "crow_fly_f3",
    "flag_torn_f0",
    "flag_torn_f1",
    "flag_torn_f2",
    "flag_torn_f3",
    "plastic_flap_f0",
    "plastic_flap_f1",
    "plastic_flap_f2",
    "plastic_flap_f3",
];

const DEFAULT_LIFE_SEED: u64 = 0x4352_4F57_5334_3100; // new: DECISIONS 2026-11-27

struct LifeModels {
    meshes: [Option<GpuMesh>; 14],
}

impl LifeModels {
    fn load(device: &wgpu::Device, pack: &pack::ModelPack) -> Self {
        let meshes = MODEL_NAMES.map(|name| {
            let m = pack.find(name);
            if m.is_none() {
                log::warn!("metro life model {name} is not in the pack");
            }
            m.map(|asset| GpuMesh::new(device, name, &asset.vertices))
        });
        Self { meshes }
    }
}

struct ChunkLife {
    perches: Vec<CrowPerch>,
    flappers: Vec<Flapper>,
}

/// Manages per-chunk crow perches, active flocks, high circlers, and flapping flags/plastic.
pub struct MetroLife {
    cfg: CrowsConfig,
    seed: u64,
    models: Option<LifeModels>,
    chunk_cache: HashMap<IVec2, ChunkLife>,
    flocks: HashMap<(IVec2, usize), CrowFlock>,
    gone_until: HashMap<(IVec2, usize), f32>,
    circlers: Option<Circlers>,
}

impl Default for MetroLife {
    fn default() -> Self {
        Self::new(DEFAULT_LIFE_SEED)
    }
}

impl MetroLife {
    pub fn new(seed: u64) -> Self {
        let cfg = CrowsConfig::from_repo().unwrap_or_default();
        Self {
            cfg,
            seed,
            models: None,
            chunk_cache: HashMap::new(),
            flocks: HashMap::new(),
            gone_until: HashMap::new(),
            circlers: None,
        }
    }

    /// Updates cached chunk life, active crow flocks, and high circlers.
    pub fn update(
        &mut self,
        dt: f32,
        player_pos: Vec3,
        player_running: bool,
        gunshot_at: Option<Vec3>,
        streamer: Option<&Streamer>,
        world_gen: Option<&WorldGenerator>,
    ) {
        let Some(wgen) = world_gen else { return };
        let Some(metro) = wgen.metro_world() else { return };

        if self.circlers.is_none() {
            let center = Vec3::new(
                metro.layout.site.center.x,
                metro.layout.site.ground_y,
                metro.layout.site.center.y,
            );
            self.circlers = Some(Circlers::new(center, self.seed ^ 0x9E37_79B9_7F4A_7C15, &self.cfg));
        }

        if let Some(st) = streamer {
            self.chunk_cache.retain(|coord, _| st.chunks.contains_key(coord));
            for &coord in st.chunks.keys() {
                self.chunk_cache.entry(coord).or_insert_with(|| ChunkLife {
                    perches: crow_perches_in_chunk(metro, (coord.x, coord.y), &|x, z| wgen.height(x, z)),
                    flappers: flappers_in_chunk(metro, (coord.x, coord.y), &|x, z| wgen.height(x, z)),
                });
            }
        }

        self.gone_until.retain(|_, rem| {
            *rem -= dt;
            *rem > 0.0
        });

        let draw_r_sq = self.cfg.draw_radius * self.cfg.draw_radius;
        let cache = &self.chunk_cache;
        let gone_until = &mut self.gone_until;
        self.flocks.retain(|&(coord, idx), flock| {
            let in_range = cache
                .get(&coord)
                .and_then(|c| c.perches.get(idx))
                .is_some_and(|p| p.pos.distance_squared(player_pos) <= draw_r_sq);
            if !in_range {
                if let Some(rem) = flock.gone_remaining() {
                    gone_until.insert((coord, idx), rem);
                }
                return false;
            }
            true
        });

        for (&coord, chunk) in &self.chunk_cache {
            for (idx, perch) in chunk.perches.iter().enumerate() {
                if perch.pos.distance_squared(player_pos) > draw_r_sq {
                    continue;
                }
                let key = (coord, idx);
                if self.flocks.contains_key(&key) {
                    continue;
                }
                let p_seed = perch_seed(self.seed, coord, idx);
                let flock = if let Some(rem) = self.gone_until.remove(&key) {
                    CrowFlock::new_gone(perch.pos, perch.yaw, perch.count, p_seed, rem)
                } else {
                    CrowFlock::new(perch.pos, perch.yaw, perch.count, p_seed)
                };
                self.flocks.insert(key, flock);
            }
        }

        for flock in self.flocks.values_mut() {
            flock.update(dt, player_pos, player_running, gunshot_at, &self.cfg);
        }
        if let Some(circlers) = self.circlers.as_mut() {
            circlers.update(dt);
        }
    }

    /// Appends non-shadow-casting instanced batches for visible crows and flappers to `main`.
    pub fn pose<'a>(
        &'a mut self,
        device: &wgpu::Device,
        pack: &pack::ModelPack,
        player_pos: Vec3,
        time: f32,
        main: &mut Vec<SkinnedBatch<'a>>,
    ) {
        let models = self.models.get_or_insert_with(|| LifeModels::load(device, pack));
        let mut buckets: [Vec<SkinInstance>; 14] = std::array::from_fn(|_| Vec::new());

        let crow_poses = self
            .flocks
            .values()
            .flat_map(|f| f.instances(&self.cfg))
            .chain(self.circlers.iter().flat_map(|c| c.instances(&self.cfg)));

        for pose in crow_poses {
            let slot = match pose.model {
                CrowModel::Perched => 0,
                CrowModel::Peck => 1,
                CrowModel::Fly(f) => 2 + (f as usize).min(3),
            };
            let mat = Mat4::from_translation(pose.pos) * Mat4::from_rotation_y(pose.yaw);
            buckets[slot].push(SkinInstance::new(mat, 0));
        }

        let flapper_r_sq = self.cfg.flapper_draw_radius * self.cfg.flapper_draw_radius;
        for chunk in self.chunk_cache.values() {
            for flapper in &chunk.flappers {
                if flapper.pos.distance_squared(player_pos) > flapper_r_sq {
                    continue;
                }
                let frame = flap_frame(flapper.phase, time, self.cfg.flapper_rate);
                let slot = match flapper.kind {
                    FlapKind::Flag => 6 + frame,
                    FlapKind::Plastic => 10 + frame,
                };
                let mat = Mat4::from_translation(flapper.pos)
                    * Mat4::from_rotation_y(flapper.yaw)
                    * Mat4::from_scale(Vec3::splat(flapper.scale));
                buckets[slot].push(SkinInstance::new(mat, 0));
            }
        }

        for (instances, mesh_opt) in buckets.into_iter().zip(models.meshes.iter()) {
            if !instances.is_empty()
                && let Some(mesh) = mesh_opt
            {
                main.push(SkinnedBatch { mesh, instances });
            }
        }
    }
}

fn perch_seed(base: u64, coord: IVec2, idx: usize) -> u64 {
    let x = (coord.x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15); // new: DECISIONS 2026-11-27
    let z = (coord.y as i64 as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9); // new: DECISIONS 2026-11-27
    let i = (idx as u64).wrapping_mul(0x94D0_49BB_1331_11EB); // new: DECISIONS 2026-11-27
    base ^ x ^ z ^ i
}
