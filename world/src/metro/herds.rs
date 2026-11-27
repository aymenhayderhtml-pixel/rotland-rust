//! Slow zombie herd routes along New Halden's avenues. Pure geometry, no GPU.

use std::collections::{HashMap, HashSet, VecDeque};

use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::metro::layout::{MetroDistrict, MetroLayout, MetroStreet};
use crate::random::DotnetRandom;

const FNV_OFFSET: u32 = 2_166_136_261; // new: DECISIONS 2026-11-27
const FNV_PRIME: u32 = 16_777_619; // new: DECISIONS 2026-11-27
const LEN_EPS: f32 = 1e-4; // new: DECISIONS 2026-11-27
const MIN_LOOP_LEGS: usize = 4; // data: metro.json herds.routeLegs[0]
const MAX_ROUTE_ATTEMPTS: usize = 32; // new: DECISIONS 2026-11-27 (route search retries)

/// Herd counts per New Halden district (`herds.count` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HerdsDistrictCount {
    pub edge: usize,
    pub midtown: usize,
    pub core: usize,
}

impl Default for HerdsDistrictCount {
    fn default() -> Self {
        Self {
            edge: 2,    // data: metro.json herds.count.edge
            midtown: 4, // data: metro.json herds.count.midtown
            core: 4,    // data: metro.json herds.count.core
        }
    }
}

/// Settings for avenue zombie herds (`herds` in `data/metro.json`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct HerdsConfig {
    pub count: HerdsDistrictCount,
    pub size: [u32; 2],
    pub speed: f32,
    pub route_legs: [usize; 2],
    pub active_radius: f32,
    pub despawn_radius: f32,
    pub spread_m: f32,
    pub aggro_share: f32,
}

impl Default for HerdsConfig {
    fn default() -> Self {
        Self {
            count: HerdsDistrictCount::default(),
            size: [12, 30],        // data: metro.json size
            speed: 0.55,           // data: metro.json speed
            route_legs: [4, 9],    // data: metro.json routeLegs
            active_radius: 170.0,  // data: metro.json activeRadius
            despawn_radius: 210.0, // data: metro.json despawnRadius
            spread_m: 7.0,         // data: metro.json spreadM
            aggro_share: 0.6,      // data: metro.json aggroShare
        }
    }
}

impl HerdsConfig {
    /// Checks that herd counts, ranges, speed and radii are valid.
    pub fn validate(&self) -> Result<(), String> {
        if self.size[0] == 0 || self.size[0] > self.size[1] {
            return Err("metro.json herds.size must have 1 <= min <= max".into());
        }
        if self.speed <= 0.0 || self.active_radius <= 0.0 || self.despawn_radius < self.active_radius {
            return Err("metro.json herds: speed/activeRadius must be > 0 and despawnRadius >= activeRadius".into());
        }
        if self.route_legs[0] < MIN_LOOP_LEGS || self.route_legs[0] > self.route_legs[1] {
            return Err("metro.json herds.routeLegs must have 4 <= min <= max".into());
        }
        if self.spread_m <= 0.0 || !(0.0..=1.0).contains(&self.aggro_share) {
            return Err("metro.json herds: spreadM > 0 and aggroShare in 0..1 required".into());
        }
        Ok(())
    }
}

/// A closed-loop walking route for one street herd (world XZ points).
#[derive(Debug, Clone, PartialEq)]
pub struct HerdRoute {
    pub id: u32,
    pub points: Vec<Vec2>,
    pub size: u32,
    pub speed: f32,
    pub zone: MetroDistrict,
}

/// Generates deterministic closed-loop herd routes per district.
/// Never enters reserved blocks or outer perimeter street ends.
pub fn herd_routes(layout: &MetroLayout, cfg: &HerdsConfig, seed: u64) -> Vec<HerdRoute> {
    herd_routes_avoiding(layout, cfg, &[], seed)
}

/// Generates closed-loop herd routes while avoiding world XZ points on blocked street segments
/// (such as the fallen-tower street, car-wall ends, and the highway entrance avenue's first block).
pub fn herd_routes_avoiding(
    layout: &MetroLayout,
    cfg: &HerdsConfig,
    blocked_world_points: &[Vec2],
    seed: u64,
) -> Vec<HerdRoute> {
    let Some(grid) = StreetGrid::from_layout(layout, blocked_world_points) else {
        return Vec::new();
    };
    let mut rng = DotnetRandom::new(mix_seed(seed));
    let mut out = Vec::new();
    let mut edge_uses: HashMap<(GridPt, GridPt), usize> = HashMap::new();
    let mut next_id = 1u32;

    for (zone, want) in [
        (MetroDistrict::Edge, cfg.count.edge),
        (MetroDistrict::Midtown, cfg.count.midtown),
        (MetroDistrict::Core, cfg.count.core),
    ] {
        let edges = grid.edges_for_district(zone);
        if edges.is_empty() {
            continue;
        }
        for _ in 0..want {
            let want_legs = rng.next_range(cfg.route_legs[0] as i32, cfg.route_legs[1] as i32 + 1) as usize;
            let size = rng.next_range(cfg.size[0] as i32, cfg.size[1] as i32 + 1) as u32;
            if let Some(loop_pts) = build_closed_loop(&grid, &edges, want_legs, &edge_uses, &mut rng) {
                for pair in loop_pts.windows(2) {
                    mark_segment_used(&mut edge_uses, pair[0], pair[1]);
                }
                if let (Some(&last), Some(&first)) = (loop_pts.last(), loop_pts.first()) {
                    mark_segment_used(&mut edge_uses, last, first);
                }
                let points = loop_pts.into_iter().map(|p| grid.to_world(p)).collect();
                out.push(HerdRoute { id: next_id, points, size, speed: cfg.speed, zone });
                next_id += 1;
            }
        }
    }
    out
}

/// Returns `(world_xz, yaw)` on `route` at `time_s` (looped, Godot yaw: forward = `(-sin yaw, -cos yaw)`).
pub fn route_pos(route: &HerdRoute, time_s: f64) -> (Vec2, f32) {
    let n = route.points.len();
    if n == 0 {
        return (Vec2::ZERO, 0.0);
    }
    if n == 1 {
        return (route.points[0], 0.0);
    }
    let mut total_len = 0.0_f32;
    for i in 0..n {
        total_len += route.points[i].distance(route.points[(i + 1) % n]);
    }
    if total_len <= LEN_EPS {
        return (route.points[0], 0.0);
    }
    let dist = (time_s * f64::from(route.speed)).rem_euclid(f64::from(total_len)) as f32;
    let mut rem = dist;
    for i in 0..n {
        let a = route.points[i];
        let b = route.points[(i + 1) % n];
        let seg = a.distance(b);
        if seg <= LEN_EPS {
            continue;
        }
        if rem <= seg || i + 1 == n {
            let t = (rem / seg).clamp(0.0, 1.0);
            let dir = (b - a) / seg;
            return (a.lerp(b, t), (-dir.x).atan2(-dir.y));
        }
        rem -= seg;
    }
    (route.points[0], 0.0)
}

type GridPt = (i32, i32);

#[derive(Clone, Copy)]
struct GridEdge {
    to: GridPt,
    avenue: bool,
    zone: MetroDistrict,
}

struct StreetGrid<'a> {
    layout: &'a MetroLayout,
    half: f32,
    bs: f32,
    adj: HashMap<GridPt, Vec<GridEdge>>,
}

impl<'a> StreetGrid<'a> {
    fn from_layout(layout: &'a MetroLayout, blocked_world: &[Vec2]) -> Option<Self> {
        let first_block = layout.blocks.first()?;
        let bs = (first_block.rect.1.x - first_block.rect.0.x).abs();
        if bs <= LEN_EPS {
            return None;
        }
        let half = layout.site.half_size;
        let n = ((half * 2.0) / bs).round() as i32;
        let blocked_local: Vec<Vec2> = blocked_world.iter().map(|&p| layout.site.to_local(p)).collect();
        let mut adj: HashMap<GridPt, Vec<GridEdge>> = HashMap::new();

        for s in &layout.streets {
            if is_blocked_street(s, &blocked_local) {
                continue;
            }
            let p0 = to_grid(s.a, half, bs);
            let p1 = to_grid(s.b, half, bs);
            // Keep routes inside the city grid so they never walk along outer perimeter dead-ends.
            if !is_interior(p0, n) || !is_interior(p1, n) {
                continue;
            }
            let Some(zone) = segment_district(layout, p0, p1) else {
                continue;
            };
            adj.entry(p0).or_default().push(GridEdge { to: p1, avenue: s.avenue, zone });
            adj.entry(p1).or_default().push(GridEdge { to: p0, avenue: s.avenue, zone });
        }
        Some(Self { layout, half, bs, adj })
    }

    fn edges_for_district(&self, zone: MetroDistrict) -> Vec<(GridPt, GridEdge)> {
        let mut out = Vec::new();
        let mut keys: Vec<GridPt> = self.adj.keys().copied().collect();
        keys.sort();
        for u in keys {
            for &e in &self.adj[&u] {
                if e.zone == zone && u < e.to {
                    out.push((u, e));
                }
            }
        }
        out
    }

    fn to_world(&self, p: GridPt) -> Vec2 {
        let local = Vec2::new(-self.half + p.0 as f32 * self.bs, -self.half + p.1 as f32 * self.bs);
        self.layout.site.to_world(local)
    }

    fn has_edge(&self, a: GridPt, b: GridPt, zone: MetroDistrict) -> bool {
        self.adj.get(&a).is_some_and(|list| list.iter().any(|e| e.to == b && e.zone == zone))
    }
}

fn to_grid(local: Vec2, half: f32, bs: f32) -> GridPt {
    (((local.x + half) / bs).round() as i32, ((local.y + half) / bs).round() as i32)
}

fn is_interior(p: GridPt, n: i32) -> bool {
    p.0 > 0 && p.0 < n && p.1 > 0 && p.1 < n
}

fn is_blocked_street(s: &MetroStreet, blocked_local: &[Vec2]) -> bool {
    let half_w = s.width * 0.5 + 1.0; // new: DECISIONS 2026-11-27 (street half-width + margin)
    blocked_local.iter().any(|&p| s.contains_point(p) || s.closest_point(p).distance(p) <= half_w)
}

/// Returns the district if both adjacent blocks exist, neither is reserved, and both share the district.
fn segment_district(layout: &MetroLayout, a: GridPt, b: GridPt) -> Option<MetroDistrict> {
    let (b0, b1) = if a.1 == b.1 {
        let ix = a.0.min(b.0);
        (layout.block_at(ix, a.1 - 1)?, layout.block_at(ix, a.1)?)
    } else {
        let iz = a.1.min(b.1);
        (layout.block_at(a.0 - 1, iz)?, layout.block_at(a.0, iz)?)
    };
    if b0.reserved.is_some() || b1.reserved.is_some() || b0.district != b1.district {
        return None;
    }
    Some(b0.district)
}

fn build_closed_loop(
    grid: &StreetGrid<'_>,
    edges: &[(GridPt, GridEdge)],
    want_legs: usize,
    edge_uses: &HashMap<(GridPt, GridPt), usize>,
    rng: &mut DotnetRandom,
) -> Option<Vec<GridPt>> {
    let zone = edges[0].1.zone;
    let ave_starts: Vec<GridPt> = edges
        .iter()
        .filter(|(_, e)| e.avenue)
        .map(|&(u, _)| u)
        .collect();
    let starts: Vec<GridPt> = if ave_starts.is_empty() {
        edges.iter().map(|&(u, _)| u).collect()
    } else {
        ave_starts
    };

    for _ in 0..MAX_ROUTE_ATTEMPTS {
        let start = starts[rng.next(starts.len() as i32) as usize];
        if let Some(loop_pts) = try_walk_loop(grid, start, zone, want_legs, edge_uses, rng) {
            return Some(loop_pts);
        }
    }
    fallback_rectangle_loop(grid, edges, zone, rng)
}

fn try_walk_loop(
    grid: &StreetGrid<'_>,
    start: GridPt,
    zone: MetroDistrict,
    want_legs: usize,
    edge_uses: &HashMap<(GridPt, GridPt), usize>,
    rng: &mut DotnetRandom,
) -> Option<Vec<GridPt>> {
    let mut path = vec![start];
    let mut visited: HashSet<GridPt> = HashSet::from([start]);
    let mut dir = (0, 0);
    let mut legs = 0usize;
    let target_walk_legs = want_legs.saturating_sub(2).max(2);

    while legs < target_walk_legs {
        let cur = *path.last()?;
        let mut cands: Vec<GridEdge> = grid
            .adj
            .get(&cur)?
            .iter()
            .copied()
            .filter(|e| e.zone == zone && !visited.contains(&e.to))
            .filter(|e| (e.to.0 - cur.0, e.to.1 - cur.1) != (-dir.0, -dir.1))
            .collect();
        if cands.is_empty() {
            break;
        }
        cands.sort_by_key(|e| {
            let nd = (e.to.0 - cur.0, e.to.1 - cur.1);
            let straight = if nd == dir { 0usize } else { 1 };
            let ave_pen = if e.avenue { 0usize } else { 2 };
            let uses = edge_uses.get(&ordered_pair(cur, e.to)).copied().unwrap_or(0);
            (uses, ave_pen + straight)
        });
        let top = cands.len().min(3);
        let chosen = cands[rng.next(top as i32) as usize];
        // new: DECISIONS 2026-11-27 (2-4 blocks per avenue leg, 1-2 per street leg)
        let step_len = if chosen.avenue { rng.next_range(2, 5) } else { rng.next_range(1, 3) };
        let nd = (chosen.to.0 - cur.0, chosen.to.1 - cur.1);
        if nd != dir {
            legs += 1;
            dir = nd;
        }
        let mut head = cur;
        for _ in 0..step_len {
            let next = (head.0 + dir.0, head.1 + dir.1);
            if !grid.has_edge(head, next, zone) || visited.contains(&next) {
                break;
            }
            path.push(next);
            visited.insert(next);
            head = next;
        }
    }

    let end = *path.last()?;
    let ret = bfs_return(grid, end, start, zone, &visited, dir)?;
    for p in ret.into_iter().skip(1) {
        if p != start {
            path.push(p);
        }
    }
    let corners = simplify_closed_loop(&path);
    (corners.len() >= MIN_LOOP_LEGS).then_some(corners)
}

fn bfs_return(
    grid: &StreetGrid<'_>,
    from: GridPt,
    goal: GridPt,
    zone: MetroDistrict,
    blocked: &HashSet<GridPt>,
    last_dir: GridPt,
) -> Option<Vec<GridPt>> {
    let mut queue = VecDeque::from([from]);
    let mut parent: HashMap<GridPt, GridPt> = HashMap::new();
    while let Some(cur) = queue.pop_front() {
        if cur == goal {
            let mut rev = vec![goal];
            let mut p = goal;
            while let Some(&prev) = parent.get(&p) {
                rev.push(prev);
                if prev == from {
                    break;
                }
                p = prev;
            }
            rev.reverse();
            return Some(rev);
        }
        let Some(nbrs) = grid.adj.get(&cur) else { continue };
        let mut sorted = nbrs.clone();
        sorted.sort_by_key(|e| (!e.avenue, e.to));
        for e in sorted {
            if e.zone != zone {
                continue;
            }
            if cur == from && (e.to.0 - cur.0, e.to.1 - cur.1) == (-last_dir.0, -last_dir.1) {
                continue;
            }
            if (e.to != goal && blocked.contains(&e.to)) || parent.contains_key(&e.to) || e.to == from {
                continue;
            }
            parent.insert(e.to, cur);
            queue.push_back(e.to);
        }
    }
    None
}

fn fallback_rectangle_loop(
    grid: &StreetGrid<'_>,
    edges: &[(GridPt, GridEdge)],
    zone: MetroDistrict,
    rng: &mut DotnetRandom,
) -> Option<Vec<GridPt>> {
    for _ in 0..MAX_ROUTE_ATTEMPTS {
        let (p, _) = edges[rng.next(edges.len() as i32) as usize];
        let w = rng.next_range(1, 4);
        let h = rng.next_range(1, 4);
        let corners = [p, (p.0 + w, p.1), (p.0 + w, p.1 + h), (p.0, p.1 + h)];
        let valid = corners.iter().enumerate().all(|(i, &a)| {
            let b = corners[(i + 1) % 4];
            segment_all_valid(grid, a, b, zone)
        });
        if valid {
            return Some(corners.to_vec());
        }
    }
    None
}

fn segment_all_valid(grid: &StreetGrid<'_>, a: GridPt, b: GridPt, zone: MetroDistrict) -> bool {
    let step = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
    let mut cur = a;
    while cur != b {
        let next = (cur.0 + step.0, cur.1 + step.1);
        if !grid.has_edge(cur, next, zone) {
            return false;
        }
        cur = next;
    }
    true
}

fn simplify_closed_loop(path: &[GridPt]) -> Vec<GridPt> {
    let n = path.len();
    if n < 3 {
        return path.to_vec();
    }
    let mut corners = Vec::new();
    for i in 0..n {
        let prev = path[(i + n - 1) % n];
        let cur = path[i];
        let next = path[(i + 1) % n];
        let d1 = (cur.0 - prev.0, cur.1 - prev.1);
        let d2 = (next.0 - cur.0, next.1 - cur.1);
        if d1 != d2 {
            corners.push(cur);
        }
    }
    corners
}

fn ordered_pair(a: GridPt, b: GridPt) -> (GridPt, GridPt) {
    if a <= b { (a, b) } else { (b, a) }
}

fn mark_segment_used(uses: &mut HashMap<(GridPt, GridPt), usize>, a: GridPt, b: GridPt) {
    let step = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
    let mut cur = a;
    while cur != b {
        let next = (cur.0 + step.0, cur.1 + step.1);
        *uses.entry(ordered_pair(cur, next)).or_insert(0) += 1;
        cur = next;
    }
}

fn mix_seed(seed: u64) -> i32 {
    let mut hash = FNV_OFFSET;
    for b in seed.to_le_bytes() {
        hash ^= u32::from(b);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash as i32
}
