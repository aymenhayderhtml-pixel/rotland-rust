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

/// What the skyline is built from: the city's towers, its ground height, haze colour and settings.
pub struct SkylineInputs<'a> {
    pub towers: &'a [TowerSpot],
    pub ground_y: f32,
    pub cfg: &'a MetroConfig,
    pub look: &'a MetroLook,
    /// Metro haze colour (sRGB 0..1) that far tower colours are tinted toward.
    pub haze: Vec3,
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
        let mut verts = skyline_vertices(inputs.towers, &indices, inputs.ground_y, inputs.look);
        tint_skyline_vertices(&mut verts, inputs.haze, inputs.look.skyline_haze_share);
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

/// Blends an sRGB `[u8; 3]` colour toward `haze` (sRGB 0..1) by `share` (0..1).
pub fn tint_toward_haze(color: [u8; 3], haze: Vec3, share: f32) -> [u8; 3] {
    let w = share.clamp(0.0, 1.0);
    let scale = 1.0 / 255.0; // new: DECISIONS 2026-11-26 (sRGB byte to 0..1)
    let blend = |byte: u8, target: f32| -> u8 {
        let c = f32::from(byte) * scale;
        let h = target.clamp(0.0, 1.0);
        ((c + (h - c) * w) * 255.0).round().clamp(0.0, 255.0) as u8
    };
    [blend(color[0], haze.x), blend(color[1], haze.y), blend(color[2], haze.z)]
}

/// Applies `tint_toward_haze` to every vertex in the skyline batch.
pub fn tint_skyline_vertices(verts: &mut [ColorVertex], haze: Vec3, share: f32) {
    for v in verts {
        let tinted = tint_toward_haze([v.color[0], v.color[1], v.color[2]], haze, share);
        v.color[0] = tinted[0];
        v.color[1] = tinted[1];
        v.color[2] = tinted[2];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::metro::{TowerKind, TowerStyle, default_styles, style_for};

    fn test_spot(floors: u32, style: TowerStyle, landmark: bool) -> TowerSpot {
        TowerSpot {
            pos: Vec2::new(6000.0, 0.0),
            half: Vec2::splat(12.0),
            yaw: 0.0,
            height: 7.6 + floors as f32 * 3.8 + 5.0,
            floors,
            kind: if landmark { TowerKind::Landmark } else { TowerKind::Highrise },
            landmark,
            block: (0, 0),
            style,
            size: 24,
        }
    }

    #[test]
    fn window_bands_stay_inside_tris_per_tower_and_landmarks_use_deco() {
        let cfg = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).unwrap();
        let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).unwrap();
        let deco_color = style_for(default_styles(), TowerStyle::Deco).unwrap().color;

        for style in TowerStyle::ALL {
            let verts = tower_mesh(&test_spot(24, style, false), &look, Detail::Far);
            assert!(verts.len() / 3 <= cfg.skyline.tris_per_tower);
            assert!(verts.iter().any(|v| v.color[..3] == look.glass_dark), "has dark window bands");
        }

        let landmark = tower_mesh(&test_spot(42, TowerStyle::Brick, true), &look, Detail::Far);
        assert!(landmark.len() / 3 <= cfg.skyline.tris_per_tower);
        assert!(landmark.iter().any(|v| v.color[..3] == deco_color), "landmark uses Deco color");
    }

    #[test]
    fn tint_moves_color_toward_haze_and_preserves_haze_color() {
        let haze_u8 = [150u8, 168, 150];
        let haze = Vec3::new(150.0 / 255.0, 168.0 / 255.0, 150.0 / 255.0);
        assert_eq!(tint_toward_haze(haze_u8, haze, 0.5), haze_u8);

        let dark = [48u8, 54, 58];
        let tinted = tint_toward_haze(dark, haze, 0.5);
        assert!(tinted[0] > dark[0] && tinted[0] < haze_u8[0]);
        assert!(tinted[1] > dark[1] && tinted[1] < haze_u8[1]);
        assert!(tinted[2] > dark[2] && tinted[2] < haze_u8[2]);
    }
}
