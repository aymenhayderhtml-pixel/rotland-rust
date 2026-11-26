//! Where New Halden sits: on the swamp/snow seam about `centerDistance` from camp, clear of
//! towns, river crossings and the core. Also the site's local frame (`to_local` / `to_world`).

use std::f32::consts::SQRT_2;

use glam::Vec2;

use crate::biome::BiomeId;
use crate::biome::layout::WorldLayoutData;
use crate::metro::config::MetroConfig;
use crate::random::DotnetRandom;
use crate::rivers::RiversData;
use crate::towns::TownsData;
use crate::towns::place::rotate_xz_ccw_from_above;

/// Degrees in a full turn.
const FULL_TURN_DEG: f32 = 360.0; // new: DECISIONS 2026-11-26 (geometry constant)
/// Half a turn, the wrap point for the shortest arc.
const HALF_TURN_DEG: f32 = 180.0; // new: DECISIONS 2026-11-26 (geometry constant)

/// The placed city square. Local +Y points toward the camp; `yaw` is the Godot +Y rotation in radians.
#[derive(Debug, Clone, PartialEq)]
pub struct MetroSite {
    pub key: String,
    pub name: String,
    pub center: Vec2,
    /// Half the side of the square (`sizeBlocks * blockSize / 2`).
    pub half_size: f32,
    pub yaw: f32,
    /// Flat city ground height, set by the lead from the terrain at `center`.
    pub ground_y: f32,
}

impl MetroSite {
    /// World XZ point to site-local XZ (centre at the origin, local +Y toward camp).
    pub fn to_local(&self, p: Vec2) -> Vec2 {
        rotate_xz(p - self.center, -self.yaw.to_degrees())
    }

    /// Site-local XZ point back to world XZ.
    pub fn to_world(&self, local: Vec2) -> Vec2 {
        self.center + rotate_xz(local, self.yaw.to_degrees())
    }

    /// Distance from `p` to the square: negative inside, positive outside.
    pub fn signed_distance(&self, p: Vec2) -> f32 {
        let q = self.to_local(p).abs() - Vec2::splat(self.half_size);
        q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
    }

    /// The four world-space corners of the square (counter-clockwise in local space).
    pub fn corners(&self) -> [Vec2; 4] {
        let h = self.half_size;
        [Vec2::new(-h, -h), Vec2::new(h, -h), Vec2::new(h, h), Vec2::new(-h, h)]
            .map(|c| self.to_world(c))
    }

    /// True if world point `p` lies inside the square.
    pub fn contains_square(&self, p: Vec2) -> bool {
        self.to_local(p).abs().max_element() <= self.half_size
    }
}

/// Everything placement needs, extracted from the data files (or set by hand in tests).
#[derive(Debug, Clone, PartialEq)]
pub struct MetroPlacementFrame {
    /// Camp-to-Ashford direction the sector angles turn from.
    pub reference_dir: Vec2,
    pub swamp_center_deg: f32,
    pub snow_center_deg: f32,
    /// Town obstacles: (ideal centre, radius including the spiral drift).
    pub towns: Vec<(Vec2, f32)>,
    /// River obstacles: (start, end, half width) in world XZ.
    pub river_chords: Vec<(Vec2, Vec2, f32)>,
    /// Core radius plus blend band, measured from camp.
    pub core_outer_radius: f32,
    pub spiral_step_m: f32,
    pub spiral_max_m: f32,
}

/// Places New Halden from the real data files (`data/world_layout.json`, `towns.json`, `rivers.json`).
/// `cfg.reference_dir` must already be set by the lead.
pub fn place_metro(seed: u64, camp: Vec2, cfg: &MetroConfig) -> MetroSite {
    let layout = WorldLayoutData::load().expect("data/world_layout.json must load");
    let towns = TownsData::load_embedded();
    let rivers = RiversData::load().expect("data/rivers.json must load");
    let base = cfg.reference_dir.try_normalize().unwrap_or(Vec2::X);
    let dir = |deg: f32| rotate_xz_ccw_from_above(base, deg);
    let sector = |biome: BiomeId| layout.sectors.iter().find(|s| s.biome == biome).map(|s| s.center_deg);

    let mut town_obstacles = Vec::new();
    for def in &towns.towns {
        let Some(deg) = def.biome_id().and_then(sector) else {
            eprintln!("[metro] town '{}' has no known sector; not avoided", def.key);
            continue;
        };
        let ideal = camp + dir(deg) * def.distance_m;
        town_obstacles.push((ideal, def.radius + towns.placement.spiral_max_m));
    }
    let river_chords = rivers
        .rivers
        .iter()
        .map(|r| {
            let start = camp + dir(r.start_angle_deg) * r.start_radius_m;
            let end = camp + dir(r.target_angle_deg) * r.target_radius_m;
            (start, end, rivers.width_end_m + rivers.bank_m)
        })
        .collect();
    let frame = MetroPlacementFrame {
        reference_dir: cfg.reference_dir,
        swamp_center_deg: sector(BiomeId::Swamp).expect("world_layout.json needs a swamp sector"),
        snow_center_deg: sector(BiomeId::Snow).expect("world_layout.json needs a snow sector"),
        towns: town_obstacles,
        river_chords,
        core_outer_radius: layout.core_radius + layout.blend_band,
        spiral_step_m: towns.placement.spiral_step_m,
        spiral_max_m: towns.placement.spiral_max_m,
    };
    place_metro_in_frame(seed, camp, cfg, &frame)
}

/// Pure placement from an explicit frame: bearing from the swamp/snow seam, seeded jitter,
/// then steps outward along that bearing until the square is clear of every obstacle.
pub fn place_metro_in_frame(
    seed: u64,
    camp: Vec2,
    cfg: &MetroConfig,
    frame: &MetroPlacementFrame,
) -> MetroSite {
    let mut rng = DotnetRandom::new(seed as i32);
    // new: DECISIONS 2026-11-26 (uniform sample in [-1, 1) scales the jitter)
    let unit = rng.next_double() as f32 * 2.0 - 1.0;
    let jitter = unit * cfg.angle_jitter_deg; // data: metro.json angleJitterDeg
    let bearing = shortest_arc_midpoint_deg(frame.swamp_center_deg, frame.snow_center_deg) + jitter;
    let base = frame.reference_dir.try_normalize().unwrap_or(Vec2::X);
    let dir = rotate_xz_ccw_from_above(base, bearing);

    let half_size = cfg.size_blocks as f32 * cfg.block_size * 0.5; // data: metro.json sizeBlocks, blockSize
    let clearance = half_size * SQRT_2;
    let steps = if frame.spiral_step_m > 0.0 {
        (frame.spiral_max_m / frame.spiral_step_m).floor() as usize
    } else {
        0
    };
    let mut center = camp + dir * cfg.center_distance; // data: metro.json centerDistance
    let mut found = false;
    for k in 0..=steps {
        center = camp + dir * (cfg.center_distance + k as f32 * frame.spiral_step_m);
        if is_clear(center, clearance, camp, frame) {
            found = true;
            break;
        }
    }
    if !found {
        eprintln!(
            "[metro] no clear site within {:.0} m along bearing {:.1} deg; using the last candidate",
            frame.spiral_max_m, bearing
        );
    }

    let to_camp = camp - center;
    MetroSite {
        key: cfg.key.clone(),
        name: cfg.name.clone(),
        center,
        half_size,
        yaw: to_camp.x.atan2(to_camp.y),
        ground_y: 0.0,
    }
}

/// True if a square of conservative radius `clearance` around `center` misses every obstacle.
fn is_clear(center: Vec2, clearance: f32, camp: Vec2, frame: &MetroPlacementFrame) -> bool {
    let towns_ok = frame.towns.iter().all(|&(c, r)| center.distance(c) > r + clearance);
    let rivers_ok = frame
        .river_chords
        .iter()
        .all(|&(a, b, w)| segment_distance(center, a, b) > w + clearance);
    let core_ok = center.distance(camp) > frame.core_outer_radius + clearance;
    towns_ok && rivers_ok && core_ok
}

/// Midpoint of the shorter arc between two bearings in degrees, normalised to `[0, 360)`.
pub fn shortest_arc_midpoint_deg(a: f32, b: f32) -> f32 {
    let mut delta = (b - a).rem_euclid(FULL_TURN_DEG);
    if delta > HALF_TURN_DEG {
        delta -= FULL_TURN_DEG;
    }
    (a + delta * 0.5).rem_euclid(FULL_TURN_DEG)
}

/// Rotates an XZ vector by `deg` (the same Godot +Y rotation as `rotate_xz_ccw_from_above`)
/// without normalising, so lengths survive.
fn rotate_xz(v: Vec2, deg: f32) -> Vec2 {
    let (s, c) = deg.to_radians().sin_cos();
    Vec2::new(v.x * c + v.y * s, -v.x * s + v.y * c)
}

/// Distance from `p` to the segment `a`..`b`.
fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared().max(f32::EPSILON)).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load_cfg() -> MetroConfig {
        MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
    }

    fn frame(cfg: &MetroConfig) -> MetroPlacementFrame {
        MetroPlacementFrame {
            reference_dir: cfg.reference_dir,
            swamp_center_deg: 90.0,
            snow_center_deg: 180.0,
            towns: Vec::new(),
            river_chords: Vec::new(),
            core_outer_radius: 2600.0,
            spiral_step_m: 80.0,
            spiral_max_m: 600.0,
        }
    }

    #[test]
    fn shortest_arc_midpoint_wraps_correctly() {
        assert!((shortest_arc_midpoint_deg(90.0, 180.0) - 135.0).abs() < 1e-4);
        assert!(shortest_arc_midpoint_deg(350.0, 10.0).rem_euclid(FULL_TURN_DEG) < 1e-4);
        assert!(shortest_arc_midpoint_deg(10.0, 350.0).rem_euclid(FULL_TURN_DEG) < 1e-4);
        assert!((shortest_arc_midpoint_deg(0.0, 90.0) - 45.0).abs() < 1e-4);
    }

    #[test]
    fn real_placement_is_deterministic_about_6km_on_the_seam() {
        let cfg = load_cfg();
        let a = place_metro(1234, Vec2::ZERO, &cfg);
        let b = place_metro(1234, Vec2::ZERO, &cfg);
        assert_eq!(a, b);
        let dist = a.center.length();
        assert!((5900.0..=6700.0).contains(&dist), "distance {dist}");
        let seam = rotate_xz_ccw_from_above(Vec2::X, shortest_arc_midpoint_deg(90.0, 180.0));
        assert!(a.center.normalize().dot(seam) > 10f32.to_radians().cos());
    }

    #[test]
    fn yaw_makes_local_plus_y_face_camp() {
        let cfg = load_cfg();
        let camp = Vec2::new(30.0, -20.0);
        let site = place_metro(7, camp, &cfg);
        let local_camp = site.to_local(camp);
        assert!(local_camp.x.abs() < 1e-2, "{local_camp:?}");
        assert!((local_camp.y - camp.distance(site.center)).abs() < 1e-2);
    }

    #[test]
    fn local_and_world_round_trip() {
        let cfg = load_cfg();
        let site = place_metro(99, Vec2::new(-5.0, 12.0), &cfg);
        for p in [Vec2::new(1000.0, -300.0), site.center, Vec2::new(-4000.0, 2500.0)] {
            assert!(site.to_world(site.to_local(p)).distance(p) < 1e-2);
        }
        assert!(site.signed_distance(site.center) < 0.0);
        assert!((site.signed_distance(site.center) + site.half_size).abs() < 1e-3);
        assert_eq!(site.corners().len(), 4);
    }

    #[test]
    fn town_behind_ideal_centre_pushes_site_outward() {
        let cfg = load_cfg();
        let camp = Vec2::ZERO;
        let mut f = frame(&cfg);
        let dir = rotate_xz_ccw_from_above(Vec2::X, 135.0);
        // Obstacle 1148 m inside the ideal centre: the square must step out to clear it.
        let town = dir * (cfg.center_distance - 1148.0);
        f.towns.push((town, 100.0));
        let site = place_metro_in_frame(5, camp, &cfg, &f);
        let moved = site.center.length();
        assert!(moved > cfg.center_distance + 1.0, "moved {moved}");
        assert!(moved <= cfg.center_distance + f.spiral_max_m + 1e-3);
        assert!(site.center.distance(town) > 100.0 + site.half_size * SQRT_2);
    }

    #[test]
    fn river_chord_across_the_ideal_centre_pushes_site_outward() {
        let cfg = load_cfg();
        let camp = Vec2::ZERO;
        let mut f = frame(&cfg);
        let dir = rotate_xz_ccw_from_above(Vec2::X, 135.0);
        let across = dir.perp() * 400.0;
        let ideal = dir * cfg.center_distance;
        f.river_chords.push((ideal - across, ideal + across, 50.0));
        let site = place_metro_in_frame(5, camp, &cfg, &f);
        assert!(site.center.length() > cfg.center_distance + 1.0);
        assert!(segment_distance(site.center, ideal - across, ideal + across) > 50.0);
    }
}
