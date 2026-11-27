//! Shared geometry and seeding helpers for silent life placement. Pure logic: no GPU.

use glam::Vec2;

use crate::metro::layout::MetroBlock;
use crate::metro::life::CHUNK_SIZE_M;
use crate::metro::place::MetroSite;
use crate::metro::state::MetroWorld;
use crate::random::DotnetRandom;

pub(super) const HALF: f32 = 0.5; // new: DECISIONS 2026-11-27
const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27

pub(super) trait BlockKey {
    fn block_key(&self) -> (i32, i32);
}

impl BlockKey for MetroBlock {
    fn block_key(&self) -> (i32, i32) {
        (self.ix, self.iz)
    }
}

pub(super) fn nearest_avenue_yaw(metro: &MetroWorld, wp: Vec2, fallback: f32) -> f32 {
    let local = metro.layout.site.to_local(wp);
    let best = metro
        .layout
        .streets
        .iter()
        .filter(|s| s.avenue)
        .map(|s| {
            let cp = s.closest_point(local);
            (cp.distance_squared(local), cp)
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let Some((_, cp)) = best else { return fallback };
    let target_w = metro.layout.site.to_world(cp);
    let dir = (target_w - wp).normalize_or_zero();
    if dir.length_squared() > 0.0 { dir_yaw(dir) } else { fallback }
}

pub(super) fn pick_lot_spot(
    metro: &MetroWorld,
    block: &MetroBlock,
    clearance: f32,
    rng: &mut DotnetRandom,
) -> Option<(Vec2, f32)> {
    let every = metro.config.avenue_every;
    let w = |line: i32| if line % every == 0 { metro.config.avenue_width } else { metro.config.street_width };
    let lo = block.rect.0 + Vec2::new(w(block.ix), w(block.iz)) * HALF + Vec2::splat(clearance);
    let hi = block.rect.1 - Vec2::new(w(block.ix + 1), w(block.iz + 1)) * HALF - Vec2::splat(clearance);
    if hi.x <= lo.x || hi.y <= lo.y {
        return None;
    }
    for _ in 0..8 {
        let lp = Vec2::new(
            lo.x + (hi.x - lo.x) * rng.next_double() as f32,
            lo.y + (hi.y - lo.y) * rng.next_double() as f32,
        );
        let wp = metro.layout.site.to_world(lp);
        if !metro.in_tower_footprint(wp) {
            let yaw = nearest_avenue_yaw(metro, wp, 0.0);
            return Some((wp, yaw));
        }
    }
    None
}

pub(super) fn overlapping_blocks(metro: &MetroWorld, chunk: (i32, i32)) -> Vec<&MetroBlock> {
    let min = Vec2::new(chunk.0 as f32, chunk.1 as f32) * CHUNK_SIZE_M;
    let max = min + Vec2::splat(CHUNK_SIZE_M);
    let pad = metro.config.avenue_width.max(metro.config.street_width) * HALF + 2.0;
    let site = &metro.layout.site;
    let corners = [
        site.to_local(min),
        site.to_local(Vec2::new(max.x, min.y)),
        site.to_local(Vec2::new(min.x, max.y)),
        site.to_local(max),
    ];
    let local_min = corners.into_iter().reduce(Vec2::min).unwrap_or(Vec2::ZERO) - Vec2::splat(pad);
    let local_max = corners.into_iter().reduce(Vec2::max).unwrap_or(Vec2::ZERO) + Vec2::splat(pad);
    let half = site.half_size;
    if local_max.x < -half || local_min.x > half || local_max.y < -half || local_min.y > half {
        return Vec::new();
    }
    let (n, bs) = (metro.config.size_blocks, metro.config.block_size);
    let cell = |v: f32| (((v + half) / bs).floor() as i32).clamp(0, n - 1);
    let (ix0, ix1) = (cell(local_min.x), cell(local_max.x));
    let (iz0, iz1) = (cell(local_min.y), cell(local_max.y));
    (iz0..=iz1)
        .flat_map(|iz| (ix0..=ix1).filter_map(move |ix| metro.layout.block_at(ix, iz)))
        .collect()
}

pub(super) fn in_chunk(x: f32, z: f32, chunk: (i32, i32)) -> bool {
    (x / CHUNK_SIZE_M).floor() as i32 == chunk.0 && (z / CHUNK_SIZE_M).floor() as i32 == chunk.1
}

pub(super) fn dir_yaw(d: Vec2) -> f32 {
    (-d.x).atan2(-d.y)
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
