//! New Halden's block grid: districts, reserved blocks, ragged corners, streets and flat ground.
//!
//! All stored geometry is site-local (see `MetroSite::to_local`): the square runs from
//! `-half_size` to `+half_size` on both axes, with local +Y pointing toward the camp.

use glam::Vec2;
use serde::Deserialize;

use crate::math::smooth_step;
use crate::metro::config::MetroConfig;
use crate::metro::place::MetroSite;
use crate::random::DotnetRandom;

/// Ring a block belongs to (outer ring first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetroDistrict {
    Edge,
    Midtown,
    Core,
}

/// Kinds of reserved block groups (later rounds fill them with real buildings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Reserved {
    Park,
    Stadium,
    Hospital,
    Police,
    FireStation,
    ArmyLine,
    SafeHouseTower,
    Subway,
}

impl Reserved {
    /// Every kind, in the order the config must list them.
    pub const ALL: [Reserved; 8] = [ // new: DECISIONS 2026-11-26 (the 8 reserved kinds)
        Reserved::Park,
        Reserved::Stadium,
        Reserved::Hospital,
        Reserved::Police,
        Reserved::FireStation,
        Reserved::ArmyLine,
        Reserved::SafeHouseTower,
        Reserved::Subway,
    ];
}

/// One present block of the grid (one chunk-sized cell).
#[derive(Debug, Clone, PartialEq)]
pub struct MetroBlock {
    pub ix: i32,
    pub iz: i32,
    /// Site-local (min, max) corners of the full cell. Streets run on the cell edges, so
    /// lots must be inset by half a street width.
    pub rect: (Vec2, Vec2),
    pub district: MetroDistrict,
    pub reserved: Option<Reserved>,
}

/// A straight street surface on a grid line (site-local centreline from `a` to `b`).
#[derive(Debug, Clone, PartialEq)]
pub struct MetroStreet {
    pub a: Vec2,
    pub b: Vec2,
    pub width: f32,
    pub avenue: bool,
}

impl MetroStreet {
    /// True if `p` (site-local) is on this street's rectangular surface.
    pub fn contains_point(&self, p: Vec2) -> bool {
        let half_w = self.width * 0.5;
        let min = self.a.min(self.b) - Vec2::splat(half_w);
        let max = self.a.max(self.b) + Vec2::splat(half_w);
        p.x >= min.x && p.x <= max.x && p.y >= min.y && p.y <= max.y
    }

    /// Closest point on the centreline to `p` (site-local).
    pub fn closest_point(&self, p: Vec2) -> Vec2 {
        let ab = self.b - self.a;
        let len_sq = ab.length_squared();
        if len_sq < LEN_SQ_EPS {
            return self.a;
        }
        let t = ((p - self.a).dot(ab) / len_sq).clamp(0.0, 1.0);
        self.a + ab * t
    }
}

/// The full New Halden layout. Build once with `MetroLayout::generate`.
#[derive(Debug, Clone, PartialEq)]
pub struct MetroLayout {
    pub site: MetroSite,
    /// Present blocks only (dropped corner blocks are absent).
    pub blocks: Vec<MetroBlock>,
    pub streets: Vec<MetroStreet>,
    /// Blocks per side of the square.
    n: i32,
    /// Block edge length in metres.
    block_size: f32,
    /// Flat-ground blend width outside the square.
    skirt_width: f32,
    /// `grid[iz * n + ix]` = index into `blocks`, or None when absent.
    grid: Vec<Option<usize>>,
}

impl MetroLayout {
    /// Builds the grid for `site` (deterministic for a given key and centre).
    pub fn generate(site: MetroSite, cfg: &MetroConfig) -> MetroLayout {
        let n = cfg.size_blocks;
        let bs = cfg.block_size;
        let half = site.half_size;
        let cells = (n * n) as usize;
        let idx = |ix: i32, iz: i32| (iz * n + ix) as usize;

        let mut reserved_at: Vec<Option<Reserved>> = vec![None; cells];
        for area in &cfg.reserved {
            for z in area.iz..area.iz + area.h {
                for x in area.ix..area.ix + area.w {
                    reserved_at[idx(x, z)] = Some(area.kind);
                }
            }
        }

        let mut present = vec![true; cells];
        let mut rng = DotnetRandom::new(layout_seed(&site));
        for (cx, cz) in [(0, 0), (n - 1, 0), (0, n - 1), (n - 1, n - 1)] {
            // Each corner L: the corner block and its two neighbours along the edges.
            let sx = if cx == 0 { 1 } else { -1 };
            let sz = if cz == 0 { 1 } else { -1 };
            for (x, z) in [(cx, cz), (cx + sx, cz), (cx, cz + sz)] {
                let roll = rng.next_double();
                if reserved_at[idx(x, z)].is_some() || roll >= f64::from(cfg.corner_drop_chance) {
                    continue;
                }
                present[idx(x, z)] = false;
                if !every_side_has_block(&present, n) {
                    present[idx(x, z)] = true;
                }
            }
        }

        let mut blocks = Vec::new();
        let mut grid = vec![None; cells];
        for iz in 0..n {
            for ix in 0..n {
                if !present[idx(ix, iz)] {
                    continue;
                }
                let min = Vec2::new(-half + ix as f32 * bs, -half + iz as f32 * bs);
                grid[idx(ix, iz)] = Some(blocks.len());
                blocks.push(MetroBlock {
                    ix,
                    iz,
                    rect: (min, min + Vec2::splat(bs)),
                    district: district_for(ix, iz, n, cfg),
                    reserved: reserved_at[idx(ix, iz)],
                });
            }
        }

        let is_present = |ix: i32, iz: i32| {
            (0..n).contains(&ix) && (0..n).contains(&iz) && present[idx(ix, iz)]
        };
        let mut streets = Vec::new();
        // Horizontal lines j (y = j * bs), then vertical lines i (x = i * bs).
        for j in 0..=n {
            let avenue = j % cfg.avenue_every == 0;
            let width = if avenue { cfg.avenue_width } else { cfg.street_width };
            for bx in 0..n {
                if !(is_present(bx, j - 1) || is_present(bx, j)) {
                    continue;
                }
                let a = Vec2::new(-half + bx as f32 * bs, -half + j as f32 * bs);
                streets.push(MetroStreet { a, b: a + Vec2::X * bs, width, avenue });
            }
        }
        for i in 0..=n {
            let avenue = i % cfg.avenue_every == 0;
            let width = if avenue { cfg.avenue_width } else { cfg.street_width };
            for by in 0..n {
                if !(is_present(i - 1, by) || is_present(i, by)) {
                    continue;
                }
                let a = Vec2::new(-half + i as f32 * bs, -half + by as f32 * bs);
                streets.push(MetroStreet { a, b: a + Vec2::Y * bs, width, avenue });
            }
        }

        MetroLayout { site, blocks, streets, n, block_size: bs, skirt_width: cfg.skirt_width, grid }
    }

    /// The present block at grid cell `(ix, iz)`, if any.
    pub fn block_at(&self, ix: i32, iz: i32) -> Option<&MetroBlock> {
        if !(0..self.n).contains(&ix) || !(0..self.n).contains(&iz) {
            return None;
        }
        self.grid[(iz * self.n + ix) as usize].map(|i| &self.blocks[i])
    }

    /// The present block containing world point `p`, if any.
    pub fn block_at_point(&self, p: Vec2) -> Option<&MetroBlock> {
        if !self.site.contains_square(p) {
            return None;
        }
        let (ix, iz) = self.cell_of(self.site.to_local(p));
        self.block_at(ix, iz)
    }

    /// District of the present block at `p`, or None outside the blocks.
    pub fn district_at(&self, p: Vec2) -> Option<MetroDistrict> {
        self.block_at_point(p).map(|b| b.district)
    }

    /// The street surface covering world point `p`, if any.
    pub fn street_at(&self, p: Vec2) -> Option<&MetroStreet> {
        let local = self.site.to_local(p);
        self.streets.iter().find(|s| s.contains_point(local))
    }

    /// True on a present block or a street surface.
    pub fn contains(&self, p: Vec2) -> bool {
        self.block_at_point(p).is_some() || self.street_at(p).is_some()
    }

    /// True within `skirtWidth` of the city square: the cheap gate before any per-street test.
    pub fn near(&self, p: Vec2) -> bool {
        self.site.signed_distance(p) <= self.skirt_width
    }

    /// Flat city ground height inside the square, None outside.
    pub fn height_at(&self, p: Vec2) -> Option<f32> {
        self.site.contains_square(p).then_some(self.site.ground_y)
    }

    /// Terrain height with the city flattened: flat inside, a smooth blend over the skirt, raw outside.
    pub fn flatten(&self, raw: f32, p: Vec2) -> f32 {
        let sd = self.site.signed_distance(p);
        if sd <= 0.0 {
            self.site.ground_y
        } else if sd < self.skirt_width {
            let t = smooth_step(0.0, self.skirt_width, sd);
            self.site.ground_y + (raw - self.site.ground_y) * t
        } else {
            raw
        }
    }

    /// Grid cell (clamped to the grid) of a site-local point.
    fn cell_of(&self, local: Vec2) -> (i32, i32) {
        let half = self.site.half_size;
        let to_cell = |v: f32| (((v + half) / self.block_size).floor() as i32).clamp(0, self.n - 1);
        (to_cell(local.x), to_cell(local.y))
    }
}

/// Squared street length below which a street counts as a point.
const LEN_SQ_EPS: f32 = 1e-6; // new: DECISIONS 2026-11-26 (degenerate street guard)

/// Ring-based district: distance from the outer edge decides edge, midtown or core.
pub(crate) fn district_for(ix: i32, iz: i32, n: i32, cfg: &MetroConfig) -> MetroDistrict {
    let ring = ix.min(iz).min(n - 1 - ix).min(n - 1 - iz);
    if ring < cfg.districts.edge_rings {
        MetroDistrict::Edge
    } else if ring < cfg.districts.edge_rings + cfg.districts.midtown_rings {
        MetroDistrict::Midtown
    } else {
        MetroDistrict::Core
    }
}

/// True when each of the four outer rows and columns still has a present block.
fn every_side_has_block(present: &[bool], n: i32) -> bool {
    let at = |ix: i32, iz: i32| present[(iz * n + ix) as usize];
    let row = |iz: i32| (0..n).any(|ix| at(ix, iz));
    let col = |ix: i32| (0..n).any(|iz| at(ix, iz));
    row(0) && row(n - 1) && col(0) && col(n - 1)
}

/// FNV-1a over the site key and centre bits: a stable per-site layout seed.
fn layout_seed(site: &MetroSite) -> i32 {
    // new: DECISIONS 2026-11-26 (FNV-1a 32-bit offset & prime, same scheme as towns)
    let mut hash: u32 = 2_166_136_261;
    let bytes = site
        .key
        .bytes()
        .chain(site.center.x.to_bits().to_le_bytes())
        .chain(site.center.y.to_bits().to_le_bytes());
    for b in bytes {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(16_777_619); // new: DECISIONS 2026-11-26 (FNV-1a prime)
    }
    hash as i32
}
