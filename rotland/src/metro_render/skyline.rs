//! Far skyline: one batch of every tower outside the loaded chunks and within `skylineDistance`
//! of the player. One draw in the main pass (never the shadow pass). Rebuilt only when the
//! loaded chunk set or the player's cell (`rebuild_cell_m`) changes, never every frame.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use engine::render::lit::DrawItem;
use engine::render::{ColorVertex, GpuMesh};
use glam::{IVec2, Vec2, Vec3};
use world::metro::{MetroConfig, MetroLook, TowerSpot};

use super::boxes::{Detail, to_world, tower_mesh};
use super::near::{base_point, tower_chunk};

/// What the skyline is built from: the city's towers, its ground height and the settings.
pub struct SkylineInputs<'a> {
    pub towers: &'a [TowerSpot],
    pub ground_y: f32,
    pub cfg: &'a MetroConfig,
    pub look: &'a MetroLook,
}

/// The far batch. `loaded_key` is the key of the set the mesh was built for (None before the first build).
#[derive(Default)]
pub struct SkylineBatch {
    mesh: Option<GpuMesh>,
    loaded_key: Option<u64>,
    tower_count: usize,
}

impl SkylineBatch {
    /// Rebuilds the batch when the loaded set or the player's cell changed. True when it rebuilt.
    pub fn rebuild_if_needed(
        &mut self,
        device: &wgpu::Device,
        inputs: &SkylineInputs,
        loaded: &HashSet<IVec2>,
        player: Vec2,
    ) -> bool {
        let key = skyline_key(loaded, player, inputs.look.rebuild_cell_m);
        if self.loaded_key == Some(key) {
            return false;
        }
        let indices = skyline_indices(inputs.towers, loaded, player, inputs.cfg);
        let verts = skyline_vertices(inputs.towers, &indices, inputs.ground_y, inputs.look);
        self.mesh = (!verts.is_empty()).then(|| GpuMesh::new(device, "metro skyline", &verts));
        self.loaded_key = Some(key);
        self.tower_count = indices.len();
        true
    }

    /// The batch as draw items for the main pass. Empty when there is nothing to show.
    /// `fog_scale` = skyline fog end / the frame's fog end, so the far towers fade into the haze
    /// thousands of metres out instead of at the normal 190 m (and cast no shadow).
    pub fn draw_items(&self, fog_scale: f32) -> Vec<DrawItem<'_>> {
        self.mesh.iter().map(|mesh| DrawItem { mesh, offset: Vec3::ZERO, fog_scale }).collect()
    }

    /// How many towers the batch holds.
    pub fn tower_count(&self) -> usize {
        self.tower_count
    }
}

/// Indices of the towers the batch shows: chunk not loaded, and within `skylineDistance` of `player`.
pub fn skyline_indices(towers: &[TowerSpot], loaded: &HashSet<IVec2>, player: Vec2, cfg: &MetroConfig) -> Vec<usize> {
    let range = cfg.skyline.skyline_distance;
    towers
        .iter()
        .enumerate()
        .filter(|(_, t)| !loaded.contains(&tower_chunk(t.pos)) && t.pos.distance(player) <= range)
        .map(|(i, _)| i)
        .collect()
}

/// World-space far vertices of the towers at `indices`, merged.
pub fn skyline_vertices(towers: &[TowerSpot], indices: &[usize], ground_y: f32, look: &MetroLook) -> Vec<ColorVertex> {
    let mut out = Vec::new();
    for &i in indices {
        let t = &towers[i];
        let local = tower_mesh(t, look, Detail::Far);
        out.extend(to_world(&local, base_point(t, ground_y), t.yaw));
    }
    out
}

/// Key of the batch's inputs: the sorted loaded set and the player's `cell_m` cell.
/// Same inputs give the same key; a moved set or a new cell gives a new key.
pub fn skyline_key(loaded: &HashSet<IVec2>, player: Vec2, cell_m: f32) -> u64 {
    let mut chunks: Vec<IVec2> = loaded.iter().copied().collect();
    chunks.sort_by_key(|c| (c.x, c.y));
    let cell = IVec2::new((player.x / cell_m).floor() as i32, (player.y / cell_m).floor() as i32);
    let mut hasher = DefaultHasher::new();
    for c in &chunks {
        c.x.hash(&mut hasher);
        c.y.hash(&mut hasher);
    }
    cell.x.hash(&mut hasher);
    cell.y.hash(&mut hasher);
    hasher.finish()
}
