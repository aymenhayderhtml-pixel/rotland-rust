//! Closed-loop walk generation for herd routes (random walk plus BFS return).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::metro::herds::MIN_LOOP_LEGS;
use crate::metro::herds_grid::{GridEdge, GridPt, StreetGrid};
use crate::metro::layout::MetroDistrict;
use crate::random::DotnetRandom;

const MAX_ROUTE_ATTEMPTS: usize = 32; // new: DECISIONS 2026-11-27 (route search retries)

pub(super) fn build_closed_loop(
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

pub(super) fn mark_segment_used(uses: &mut HashMap<(GridPt, GridPt), usize>, a: GridPt, b: GridPt) {
    let step = ((b.0 - a.0).signum(), (b.1 - a.1).signum());
    let mut cur = a;
    while cur != b {
        let next = (cur.0 + step.0, cur.1 + step.1);
        *uses.entry(ordered_pair(cur, next)).or_insert(0) += 1;
        cur = next;
    }
}
