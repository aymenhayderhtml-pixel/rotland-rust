use glam::Vec2;
use std::collections::VecDeque;
use crate::city::config::CityConfigData;
use crate::random::DotnetRandom;

/// A piece of street running along a block boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct StreetSegment {
    pub a: Vec2,
    pub b: Vec2,
    pub width: f32,
    pub blocked: bool,
    pub avenue: bool,
}

impl StreetSegment {
    /// True if point p is on this street segment's rectangular surface.
    pub fn contains_point(&self, p: Vec2) -> bool {
        let half_w = self.width * 0.5;
        let min_x = self.a.x.min(self.b.x) - half_w;
        let max_x = self.a.x.max(self.b.x) + half_w;
        let min_y = self.a.y.min(self.b.y) - half_w;
        let max_y = self.a.y.max(self.b.y) + half_w;
        p.x >= min_x && p.x <= max_x && p.y >= min_y && p.y <= max_y
    }

    /// Closest point on the segment centerline to point p.
    pub fn closest_point(&self, p: Vec2) -> Vec2 {
        let ab = self.b - self.a;
        let len_sq = ab.length_squared();
        if len_sq < 1e-6 {
            return self.a;
        }
        let t = ((p - self.a).dot(ab) / len_sq).clamp(0.0, 1.0);
        self.a + ab * t
    }
}

/// Generates street segments and ensures car connectivity from the entry point to all present blocks.
pub(crate) fn generate_streets(
    origin: Vec2,
    present_grid: &[bool],
    camp: Vec2,
    cfg: &CityConfigData,
    rng: &mut DotnetRandom,
) -> (Vec<StreetSegment>, Vec2) {
    let n = cfg.blocks;
    let is_present = |bx: i32, by: i32| -> bool {
        if bx < 0 || bx >= n || by < 0 || by >= n {
            false
        } else {
            present_grid[(by * n + bx) as usize]
        }
    };

    let mut streets = Vec::new();
    let mut segment_endpoints: Vec<(usize, usize)> = Vec::new();

    // Map grid vertex (gx, gy) to node index.
    let vert_idx = |gx: i32, gy: i32| -> usize {
        (gy * (n + 1) + gx) as usize
    };

    // Horizontal segments along horizontal grid lines j in 0..=n.
    for j in 0..=n {
        let avenue = j % cfg.avenue_every == 0;
        let width = if avenue { cfg.avenue_width } else { cfg.street_width };
        for bx in 0..n {
            let touches = (j > 0 && is_present(bx, j - 1)) || (j < n && is_present(bx, j));
            if !touches {
                continue;
            }
            let a = origin + Vec2::new(bx as f32 * cfg.block_size, j as f32 * cfg.block_size);
            let b = origin + Vec2::new((bx + 1) as f32 * cfg.block_size, j as f32 * cfg.block_size);
            let blocked = if avenue {
                false
            } else {
                rng.next_double() < cfg.blocked_chance as f64
            };

            streets.push(StreetSegment {
                a,
                b,
                width,
                blocked,
                avenue,
            });
            segment_endpoints.push((vert_idx(bx, j), vert_idx(bx + 1, j)));
        }
    }

    // Vertical segments along vertical grid lines i in 0..=n.
    for i in 0..=n {
        let avenue = i % cfg.avenue_every == 0;
        let width = if avenue { cfg.avenue_width } else { cfg.street_width };
        for by in 0..n {
            let touches = (i > 0 && is_present(i - 1, by)) || (i < n && is_present(i, by));
            if !touches {
                continue;
            }
            let a = origin + Vec2::new(i as f32 * cfg.block_size, by as f32 * cfg.block_size);
            let b = origin + Vec2::new(i as f32 * cfg.block_size, (by + 1) as f32 * cfg.block_size);
            let blocked = if avenue {
                false
            } else {
                rng.next_double() < cfg.blocked_chance as f64
            };

            streets.push(StreetSegment {
                a,
                b,
                width,
                blocked,
                avenue,
            });
            segment_endpoints.push((vert_idx(i, by), vert_idx(i, by + 1)));
        }
    }

    // Find entry point: the point on any street centerline closest to camp.
    let mut best_entry = origin;
    let mut best_dist_sq = f32::MAX;
    let mut best_seg_idx = 0;

    for (idx, seg) in streets.iter().enumerate() {
        let cp = seg.closest_point(camp);
        let d2 = (camp - cp).length_squared();
        if d2 < best_dist_sq {
            best_dist_sq = d2;
            best_entry = cp;
            best_seg_idx = idx;
        }
    }

    if !streets.is_empty() {
        streets[best_seg_idx].blocked = false;
    }

    // Build vertex adjacency for connectivity routing.
    let num_verts = ((n + 1) * (n + 1)) as usize;
    let mut adj: Vec<Vec<(usize, usize)>> = vec![Vec::new(); num_verts];
    for (seg_idx, &(u, v)) in segment_endpoints.iter().enumerate() {
        adj[u].push((v, seg_idx));
        adj[v].push((u, seg_idx));
    }

    let start_verts = if streets.is_empty() {
        vec![]
    } else {
        let (u, v) = segment_endpoints[best_seg_idx];
        vec![u, v]
    };

    // Keep unblocking segments until every present block is reachable from entry.
    loop {
        // 0-1 BFS from entry segment vertices.
        let mut dist = vec![u32::MAX; num_verts];
        let mut parent: Vec<Option<(usize, usize)>> = vec![None; num_verts];
        let mut deque = VecDeque::new();

        for &sv in &start_verts {
            dist[sv] = 0;
            deque.push_back(sv);
        }

        while let Some(u) = deque.pop_front() {
            let d = dist[u];
            for &(v, seg_idx) in &adj[u] {
                let cost = if streets[seg_idx].blocked { 1 } else { 0 };
                let next_d = d + cost;
                if next_d < dist[v] {
                    dist[v] = next_d;
                    parent[v] = Some((u, seg_idx));
                    if cost == 0 {
                        deque.push_front(v);
                    } else {
                        deque.push_back(v);
                    }
                }
            }
        }

        // Find unreachable present blocks.
        let mut best_target: Option<(u32, usize)> = None;

        for by in 0..n {
            for bx in 0..n {
                let block_idx = (by * n + bx) as usize;
                if !present_grid[block_idx] {
                    continue;
                }
                let corners = [
                    vert_idx(bx, by),
                    vert_idx(bx + 1, by),
                    vert_idx(bx, by + 1),
                    vert_idx(bx + 1, by + 1),
                ];
                let is_reachable = corners.iter().any(|&c| dist[c] == 0);
                if !is_reachable {
                    for &c in &corners {
                        let d = dist[c];
                        if d < u32::MAX && best_target.is_none_or(|(best_d, _)| d < best_d) {
                            best_target = Some((d, c));
                        }
                    }
                }
            }
        }

        let Some((_, mut curr)) = best_target else {
            break;
        };

        // Unblock path from curr back to the tree of reachable vertices.
        while let Some((p_vert, seg_idx)) = parent[curr] {
            streets[seg_idx].blocked = false;
            curr = p_vert;
            if dist[curr] == 0 {
                break;
            }
        }
    }

    (streets, best_entry)
}
