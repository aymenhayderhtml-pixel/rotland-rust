//! Survivor trace placement for New Halden: street-end car walls, facade graffiti and
//! missing-person posters, rooftop camps, sidewalk bodies and lobby dressing. Pure logic: no GPU.

use std::collections::HashSet;
use std::f32::consts::SQRT_2;

use glam::{Vec2, Vec3};

use crate::metro::layout::MetroStreet;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::metro::styles::default_styles;
use crate::random::DotnetRandom;
use crate::scatter::PropInstance;

#[path = "traces_camps.rs"]
mod camps;
#[path = "traces_config.rs"]
mod config;
#[cfg(test)]
#[path = "traces_tests.rs"]
mod tests;
#[path = "traces_walls.rs"]
mod walls;

pub use camps::{CampSite, camp_sites, lobby_dressing};
pub use config::{CarWallsConfig, LobbyDressingEntry, TraceConfig, default_traces, load_traces};
pub use walls::car_wall_sites;

/// One chunk is 64 m square, matching `jam_chunks.rs` and city blocks.
pub(super) const CHUNK_SIZE_M: f32 = 64.0; // data: metro.json blockSize
/// Trace props are built at full world scale.
pub(super) const PROP_SCALE: f32 = 1.0;
const AVENUE_COUNT: usize = 2; // new: DECISIONS 2026-11-26 (the first two avenues by the entrance)
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-26
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-26

/// Emits every survivor-trace prop (`car_wall_stack`, graffiti, posters, rooftop camps, sidewalk
/// bodies, lobby dressing) whose XZ centre falls inside chunk `chunk`.
pub fn traces_props_in_chunk(
    metro: &MetroWorld,
    chunk: (i32, i32),
    height: &dyn Fn(f32, f32) -> f32,
) -> Vec<PropInstance> {
    let cfg = default_traces();
    let styles = default_styles();
    let entrance_aves = entrance_avenue_indices(metro);
    let mut out = Vec::new();

    for (model, wx, wz, yaw) in walls::plan_car_walls(metro, cfg, &entrance_aves) {
        if in_chunk(wx, wz, chunk) {
            out.push(PropInstance::new(model, Vec3::new(wx, height(wx, wz), wz), yaw, PROP_SCALE));
        }
    }

    let chunk_center = Vec2::new(
        (chunk.0 as f32 + 0.5) * CHUNK_SIZE_M,
        (chunk.1 as f32 + 0.5) * CHUNK_SIZE_M,
    );
    let tower_reach_pad = CHUNK_SIZE_M * 0.5 + 4.0; // new: DECISIONS 2026-11-26 (chunk half + wall offset)
    for (idx, tower) in metro.towers.iter().enumerate() {
        if metro.damage.tower_is_replaced(idx)
            || (tower.pos - chunk_center).abs().max_element() > tower_reach_pad + tower.half.x * SQRT_2
        {
            continue;
        }
        camps::append_tower_facade_traces(metro, idx, tower, cfg, chunk, height, &mut out);
        if !metro.damage.tower_is_burnt(tower.block) {
            let ground_y = height(tower.pos.x, tower.pos.y);
            if let Some(camp) = camps::camp_for_tower(metro, idx, tower, cfg, styles, ground_y)
                && in_chunk(camp.pos.x, camp.pos.z, chunk)
            {
                out.push(PropInstance::new("rooftop_camp", camp.pos, camp.yaw, PROP_SCALE));
            }
            camps::append_lobby_dressing(metro, idx, tower, cfg, chunk, height, &mut out);
        }
    }

    let block_reach = CHUNK_SIZE_M * 0.5 + metro.config.block_size * SQRT_2 * 0.5 + 2.0;
    for block in &metro.layout.blocks {
        if block.reserved.is_some() {
            continue;
        }
        let b_center = metro.layout.site.to_world((block.rect.0 + block.rect.1) * 0.5);
        if (b_center - chunk_center).abs().max_element() <= block_reach {
            camps::append_sidewalk_bodies(metro, block, cfg, &entrance_aves, chunk, height, &mut out);
        }
    }

    out
}

pub(super) fn entrance_avenue_indices(metro: &MetroWorld) -> HashSet<usize> {
    let Some(&entrance) = metro.highway.points.last() else {
        return HashSet::new();
    };
    let mut found: Vec<(f32, usize)> = metro
        .layout
        .streets
        .iter()
        .enumerate()
        .filter(|(_, s)| s.avenue && metro.layout.contains(metro.layout.site.to_world((s.a + s.b) * 0.5)))
        .map(|(i, s)| {
            let wa = metro.layout.site.to_world(s.a);
            let wb = metro.layout.site.to_world(s.b);
            (wa.distance(entrance).min(wb.distance(entrance)), i)
        })
        .collect();
    found.sort_by(|a, b| a.0.total_cmp(&b.0));
    found.into_iter().take(AVENUE_COUNT).map(|(_, i)| i).collect()
}

pub(super) fn is_damage_street(metro: &MetroWorld, s: &MetroStreet) -> bool {
    let has_fallen = metro.damage.fallen.pos != Vec2::ZERO || metro.damage.fallen.block != (0, 0);
    has_fallen && s.contains_point(metro.layout.site.to_local(metro.damage.fallen.pos))
}

pub(super) fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

pub(super) fn forward_of(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

pub(super) fn right_of(d: Vec2) -> Vec2 {
    Vec2::new(-d.y, d.x)
}

pub(super) fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
}

pub(super) fn rng_range_f(rng: &mut DotnetRandom, range: [f32; 2]) -> f32 {
    range[0] + (range[1] - range[0]) * rng.next_double() as f32
}

pub(super) fn mix_seed(site: &MetroSite, block: Option<(i32, i32)>, salt: &[u8]) -> i32 {
    let mut hash = FNV_OFFSET;
    let mut feed = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= u32::from(b);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };
    feed(site.key.as_bytes());
    feed(&site.center.x.to_bits().to_le_bytes());
    feed(&site.center.y.to_bits().to_le_bytes());
    if let Some((ix, iz)) = block {
        feed(&ix.to_le_bytes());
        feed(&iz.to_le_bytes());
    }
    feed(salt);
    hash as i32
}

pub(super) fn tower_seed(site: &MetroSite, idx: usize, block: (i32, i32), salt: &[u8]) -> i32 {
    let mut bytes = Vec::with_capacity(salt.len() + std::mem::size_of::<u64>());
    bytes.extend_from_slice(&(idx as u64).to_le_bytes());
    bytes.extend_from_slice(salt);
    mix_seed(site, Some(block), &bytes)
}
