//! The wide highway from the nearer of Blackwater / Frostmere into New Halden's nearest avenue.
//! Pure geometry. The terrain height under it lives in `highway_height.rs`, tests in `highway_tests.rs`.

use std::f32::consts::PI;

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::place::MetroSite;
use crate::random::DotnetRandom;
use crate::roads::RoadsData;
use crate::towns::TownSite;

#[path = "highway_height.rs"]
pub mod height;
#[cfg(test)]
#[path = "highway_tests.rs"]
mod tests;

pub use height::{HighwayHeightField, highway_height_at};

/// Town keys the highway may start from: the swamp and snow towns, nearest one wins.
const CANDIDATE_TOWNS: [&str; 2] = ["blackwater", "frostmere"];

/// Width of the seeded bend range: a `[0, 1)` sample times this, minus 1, spans `[-1, 1)`.
const SIGNED_UNIT_SCALE: f32 = 2.0; // new: DECISIONS 2026-11-26 (maps [0, 1) to [-1, 1))

/// The planned highway: a polyline from `from_town` to the city edge.
#[derive(Debug, Clone, PartialEq)]
pub struct Highway {
    pub from_town: String,
    /// World XZ polyline, start at the town's street, end exactly on the city square edge.
    pub points: Vec<Vec2>,
    /// Road width in metres (`highwayWidth`).
    pub width: f32,
}

/// Plans the highway from the nearer of Blackwater or Frostmere (or the nearest town when
/// neither exists) to the city's nearest avenue entrance.
pub fn plan_highway(
    site: &MetroSite,
    towns: &[TownSite],
    roads_cfg: &RoadsData,
    cfg: &MetroConfig,
) -> Highway {
    let town = pick_town(site, towns);
    let start = town_start(site, town, cfg);
    let end = city_entrance(site, town.center, cfg);
    let points = route(start, end, site, town.center, roads_cfg);
    Highway {
        from_town: town.key.clone(),
        points,
        width: cfg.highway.highway_width, // data: metro.json highway.highwayWidth
    }
}

fn pick_town<'a>(site: &MetroSite, towns: &'a [TownSite]) -> &'a TownSite {
    let candidates = towns.iter().filter(|t| CANDIDATE_TOWNS.contains(&t.key.as_str()));
    nearest(candidates, site.center)
        .or_else(|| nearest(towns.iter(), site.center))
        .expect("metro highway needs at least one town")
}

fn nearest<'a>(towns: impl Iterator<Item = &'a TownSite>, to: Vec2) -> Option<&'a TownSite> {
    towns.min_by(|a, b| {
        a.center
            .distance_squared(to)
            .total_cmp(&b.center.distance_squared(to))
    })
}

/// Where the highway leaves the town's main street: on the side that faces the city,
/// `townSetbackM` beyond the town's edge.
fn town_start(site: &MetroSite, town: &TownSite, cfg: &MetroConfig) -> Vec2 {
    let street = town.street_dir.try_normalize().unwrap_or(Vec2::X);
    let toward_city = (site.center - town.center).dot(street);
    let sign = if toward_city < 0.0 { -1.0 } else { 1.0 };
    let reach = town.radius + cfg.highway.town_setback_m; // data: metro.json highway.townSetbackM
    town.center + street * sign * reach
}

/// The avenue entrance on the city edge that faces `from`: the edge the town lies beyond, and the
/// nearest interior avenue line along that edge (corners are never used as entrances).
fn city_entrance(site: &MetroSite, from: Vec2, cfg: &MetroConfig) -> Vec2 {
    let local = site.to_local(from);
    let half = site.half_size;
    let facing_x = local.x.abs() >= local.y.abs();
    let (across, along) = if facing_x { (local.x, local.y) } else { (local.y, local.x) };
    let edge = if across < 0.0 { -half } else { half };
    let along = nearest_avenue(along, site, cfg);
    let local_end = if facing_x { Vec2::new(edge, along) } else { Vec2::new(along, edge) };
    site.to_world(local_end)
}

/// Snaps a site-local coordinate to the nearest avenue line (`-half + k * blockSize`, with
/// `k % avenueEvery == 0` and `0 < k < sizeBlocks`).
fn nearest_avenue(along: f32, site: &MetroSite, cfg: &MetroConfig) -> f32 {
    let half = site.half_size;
    (1..cfg.size_blocks)
        .filter(|k| k % cfg.avenue_every == 0)
        .map(|k| -half + k as f32 * cfg.block_size) // data: metro.json blockSize
        .min_by(|a, b| (a - along).abs().total_cmp(&(b - along).abs()))
        .unwrap_or(along.clamp(-half, half))
}

/// Straight start-to-end line sampled every `point_spacing`, bent sideways by a seeded amount
/// that fades to zero at both ends. The end point is exact.
fn route(
    start: Vec2,
    end: Vec2,
    site: &MetroSite,
    town_center: Vec2,
    roads_cfg: &RoadsData,
) -> Vec<Vec2> {
    let delta = end - start;
    let dir = delta.try_normalize().unwrap_or(Vec2::Y);
    let normal = Vec2::new(-dir.y, dir.x);
    let bend = seeded_bend(site.center, town_center, roads_cfg.wiggle_m);
    let segments = ((delta.length() / roads_cfg.point_spacing).ceil() as usize).max(1);
    (0..=segments)
        .map(|i| {
            if i == segments {
                return end;
            }
            let t = i as f32 / segments as f32;
            start.lerp(end, t) + normal * bend * (PI * t).sin()
        })
        .collect()
}

/// Deterministic lateral bend in `[-wiggle_m, wiggle_m)` for this site and town.
fn seeded_bend(site_center: Vec2, town_center: Vec2, wiggle_m: f32) -> f32 {
    let seed = site_center.x.to_bits()
        ^ site_center.y.to_bits().rotate_left(11)
        ^ town_center.x.to_bits().rotate_left(7)
        ^ town_center.y.to_bits().rotate_left(19);
    let mut rng = DotnetRandom::new(seed as i32);
    let unit = rng.next_double() as f32; // [0, 1)
    (unit * SIGNED_UNIT_SCALE - 1.0) * wiggle_m
}
