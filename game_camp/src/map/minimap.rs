//! Pure HUD mini-map logic: configuration loading, circular layout, world-to-minimap
//! projection, marker filtering with rim pinning, and nearby zombie filtering.

use std::path::Path;

use glam::{IVec2, Vec2};
use serde::{Deserialize, Serialize};
use world::WorldGenerator;

use super::markers::{MapMarkerKind, collect_markers};
use super::math::world_to_screen;
use super::state::MapState;
use crate::bounties::BountyDef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinimapCorner {
    #[default]
    TopLeft,
    TopRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MinimapConfig {
    pub diameter_px: f32,
    pub margin_px: f32,
    pub radius_metres: f32,
    pub opacity: f32,
    pub border_width_px: f32,
    pub player_arrow_size_px: f32,
    pub corner: MinimapCorner,
    pub show_zombies: bool,
}

impl Default for MinimapConfig {
    fn default() -> Self {
        Self {
            diameter_px: 172.0,
            margin_px: 16.0,
            radius_metres: 60.0,
            opacity: 0.85,
            border_width_px: 2.0,
            player_arrow_size_px: 16.0,
            corner: MinimapCorner::TopLeft,
            show_zombies: true,
        }
    }
}

impl MinimapConfig {
    pub fn load_default() -> Self {
        Self::load(&paths::repo_path("data/minimap.json"))
    }

    pub fn load(path: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        let Ok(mut cfg) = serde_json::from_str::<Self>(&text) else {
            return Self::default();
        };
        cfg.clamp();
        cfg
    }

    pub fn clamp(&mut self) {
        self.diameter_px = self.diameter_px.clamp(80.0, 320.0);
        self.margin_px = self.margin_px.clamp(0.0, 128.0);
        self.radius_metres = self.radius_metres.clamp(15.0, 300.0);
        self.opacity = self.opacity.clamp(0.1, 1.0);
        self.border_width_px = self.border_width_px.clamp(0.5, 6.0);
        self.player_arrow_size_px = self.player_arrow_size_px.clamp(8.0, 28.0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapMarker {
    pub kind: MapMarkerKind,
    pub building_type: String,
    pub dim: bool,
    pub offset_px: Vec2,
    pub pinned: bool,
    pub dir: Vec2,
}

pub fn minimap_circle(screen_size: Vec2, scale: f32, cfg: &MinimapConfig) -> (Vec2, f32) {
    let radius = cfg.diameter_px * scale * 0.5;
    let margin = cfg.margin_px * scale;
    let centre = match cfg.corner {
        MinimapCorner::TopLeft => Vec2::new(margin + radius, margin + radius),
        MinimapCorner::TopRight => Vec2::new(screen_size.x - margin - radius, margin + radius),
    };
    (centre, radius)
}

#[inline]
pub fn world_to_minimap_offset(
    world_xz: Vec2,
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
) -> Vec2 {
    if radius_metres <= 0.0 {
        return Vec2::ZERO;
    }
    world_to_screen(world_xz, player_xz, radius_px / radius_metres, Vec2::ZERO)
}

#[inline]
pub fn minimap_player_axes(yaw_rad: f32) -> (Vec2, Vec2) {
    (
        Vec2::new(-yaw_rad.sin(), -yaw_rad.cos()),
        Vec2::new(yaw_rad.cos(), -yaw_rad.sin()),
    )
}

pub fn collect_minimap_markers(
    map_state: &MapState,
    camps: impl IntoIterator<Item = Vec2>,
    bounties: &[BountyDef],
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
    rim_radius_px: f32,
) -> Vec<MinimapMarker> {
    let mut out = Vec::new();
    let limit = rim_radius_px.max(0.0);
    for marker in collect_markers(map_state, camps, bounties) {
        let raw = world_to_minimap_offset(marker.position, player_xz, radius_metres, radius_px);
        let dist = raw.length();
        let dir = if dist > 1e-4 {
            raw / dist
        } else {
            Vec2::new(0.0, -1.0)
        };
        let pins = matches!(marker.kind, MapMarkerKind::Camp | MapMarkerKind::Waypoint);
        if dist <= limit || pins {
            out.push(MinimapMarker {
                kind: marker.kind,
                building_type: marker.building_type,
                dim: marker.dim,
                offset_px: if dist <= limit { raw } else { dir * limit },
                pinned: dist > limit && pins,
                dir,
            });
        }
    }
    out
}

pub fn collect_minimap_zombies(
    zombies_xz: impl IntoIterator<Item = Vec2>,
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
    rim_radius_px: f32,
) -> Vec<Vec2> {
    let limit_sq = rim_radius_px.max(0.0).powi(2);
    zombies_xz
        .into_iter()
        .map(|pos| world_to_minimap_offset(pos, player_xz, radius_metres, radius_px))
        .filter(|offset| offset.length_squared() <= limit_sq)
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimapCampSite {
    pub center: Vec2,
    pub yaw: f32,
    pub half_size: f32,
    pub gate_half: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimapCampOutline {
    pub segments: Vec<(Vec2, Vec2)>,
    pub gateposts: Vec<Vec2>,
    pub interior_quad: Option<[Vec2; 4]>,
}

pub fn clip_segment_to_circle(a: Vec2, b: Vec2, radius: f32) -> Option<(Vec2, Vec2)> {
    let r = radius.max(0.0);
    let d = b - a;
    let qa = d.length_squared();
    if qa < 1e-8 {
        return (a.length_squared() <= r * r).then_some((a, b));
    }
    let qb = 2.0 * a.dot(d);
    let qc = a.length_squared() - r * r;
    let disc = qb * qb - 4.0 * qa * qc;
    if disc < 0.0 {
        return None;
    }
    let root = disc.sqrt();
    let t0 = ((-qb - root) / (2.0 * qa)).max(0.0);
    let t1 = ((-qb + root) / (2.0 * qa)).min(1.0);
    (t0 <= t1).then(|| (a + d * t0, a + d * t1))
}

pub fn collect_minimap_camp_outlines(
    camps: &[MinimapCampSite],
    player_xz: Vec2,
    radius_metres: f32,
    radius_px: f32,
    clip_radius_px: f32,
) -> Vec<MinimapCampOutline> {
    let mut out = Vec::new();
    let clip_sq = clip_radius_px.max(0.0).powi(2);
    for camp in camps {
        let local_to_off = |lx: f32, lz: f32| {
            let c = camp.yaw.cos();
            let s = camp.yaw.sin();
            let world = camp.center + Vec2::new(c * lx + s * lz, -s * lx + c * lz);
            world_to_minimap_offset(world, player_xz, radius_metres, radius_px)
        };
        let h = camp.half_size;
        let g = camp.gate_half.clamp(0.0, h * 0.5);
        let nw = local_to_off(-h, -h);
        let ne = local_to_off(h, -h);
        let se = local_to_off(h, h);
        let sw = local_to_off(-h, h);
        let gate_l = local_to_off(-g, -h);
        let gate_r = local_to_off(g, -h);
        let walls = [(nw, gate_l), (gate_r, ne), (ne, se), (se, sw), (sw, nw)];
        let segments: Vec<(Vec2, Vec2)> = walls
            .into_iter()
            .filter_map(|(a, b)| clip_segment_to_circle(a, b, clip_radius_px))
            .collect();
        let gateposts: Vec<Vec2> = [gate_l, gate_r]
            .into_iter()
            .filter(|p| p.length_squared() <= clip_sq)
            .collect();
        let interior_quad = [nw, ne, se, sw]
            .iter()
            .all(|p| p.length_squared() <= clip_sq)
            .then_some([nw, ne, se, sw]);
        if !segments.is_empty() || interior_quad.is_some() {
            out.push(MinimapCampOutline {
                segments,
                gateposts,
                interior_quad,
            });
        }
    }
    out
}

pub fn minimap_chunk_window(player_xz: Vec2, radius_metres: f32) -> (IVec2, i32) {
    let size = WorldGenerator::CHUNK_SIZE;
    let chunk = IVec2::new(
        (player_xz.x / size).floor() as i32,
        (player_xz.y / size).floor() as i32,
    );
    let reach = (radius_metres / size).ceil().max(1.0) as i32;
    (chunk, reach)
}
