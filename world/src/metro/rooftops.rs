//! Rooftop plank bridges and ladders linking climbable / fire-escape roofs to nearby roofs. Pure logic.

use std::collections::HashSet;

use glam::{Vec2, Vec3};

use crate::metro::climb::{ClimbChunk, ClimbLadder, climb_towers, roof_deck_y};
use crate::metro::fire_escapes::fire_escape_towers;
use crate::metro::loot_helpers::in_chunk;
use crate::metro::state::MetroWorld;
use crate::metro::styles::{StyleDef, default_styles, style_for};
use crate::metro::towers::TowerSpot;
use crate::scatter::PropInstance;

const MIN_GAP_M: f32 = 1.5; // new: DECISIONS 2026-11-27 (R43-02 minimum edge gap)
const PLANK_REST_M: f32 = 2.0; // new: DECISIONS 2026-11-27 (1 m resting on each side)
const PLANK_HALF_WIDTH_M: f32 = 0.6; // new: DECISIONS 2026-11-27 (1.2 m wide plank)
const PLANK_THICK_M: f32 = 0.12; // new: DECISIONS 2026-11-27 (hull-box clearance check)
const STEP_CRATE_THRESHOLD_M: f32 = 0.3; // new: DECISIONS 2026-11-27 (R43-02 parapet step threshold)
const STEP_CRATE_INSET_M: f32 = 0.6; // new: DECISIONS 2026-11-27 (step crate inside parapet)
const LADDER_WALL_CLEAR_M: f32 = 0.15; // new: DECISIONS 2026-11-27 (ladder foot outside taller wall)
const LADDER_DECK_INSET_M: f32 = 1.0; // new: DECISIONS 2026-11-27 (R43-02 deck point 1 m inside)
const PARALLEL_DOT: f32 = 0.999; // new: DECISIONS 2026-11-27 (~2.5 deg tolerance)
const DEFAULT_BASE_HEIGHT: f32 = 7.6; // data: metro.json baseHeight

/// One rooftop link (plank bridge + optional step crates and ladder) between two towers.
#[derive(Debug, Clone, PartialEq)]
pub struct RoofLink {
    pub from_tower: usize,
    pub to_tower: usize,
    pub gap: f32,
    pub overlap: f32,
    pub plank_length: u32,
    pub plank_pos: Vec3,
    pub plank_yaw: f32,
    pub plank_y: f32,
    pub step_crate_from: Option<Vec3>,
    pub step_crate_to: Option<Vec3>,
    pub ladder: Option<ClimbLadder>,
    pub ladder_seg_count: usize,
}

#[derive(Clone)]
struct LinkCandidate {
    partner: usize,
    gap: f32,
    overlap: f32,
    plank_length: u32,
    plank_pos: Vec3,
    plank_yaw: f32,
    plank_y: f32,
    step_crate_from: Option<Vec3>,
    step_crate_to: Option<Vec3>,
    ladder: Option<ClimbLadder>,
    ladder_seg_count: usize,
}

/// Plans every rooftop link from climbable and fire-escape towers to nearby standing roofs.
pub fn roof_links(metro: &MetroWorld) -> Vec<RoofLink> {
    let styles = default_styles();
    let cfg = &metro.config.rooftops;
    let ground_y = metro.layout.site.ground_y;

    let mut sources: Vec<usize> = climb_towers(metro)
        .into_iter()
        .map(|c| c.tower)
        .chain(fire_escape_towers(metro))
        .collect();
    sources.sort_unstable();
    sources.dedup();

    let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
    let mut links = Vec::new();

    for src in sources {
        let mut candidates: Vec<LinkCandidate> = (0..metro.towers.len())
            .filter(|&dst| is_valid_partner(metro, src, dst))
            .filter_map(|dst| build_candidate(metro, styles, ground_y, src, dst))
            .collect();

        candidates.sort_by(|a, b| {
            a.gap
                .partial_cmp(&b.gap)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.partner.cmp(&b.partner))
        });

        let mut taken = 0;
        for c in candidates {
            if taken >= cfg.links_per_tower {
                break;
            }
            let pair = (src.min(c.partner), src.max(c.partner));
            if !seen_pairs.insert(pair) {
                continue;
            }
            links.push(RoofLink {
                from_tower: src,
                to_tower: c.partner,
                gap: c.gap,
                overlap: c.overlap,
                plank_length: c.plank_length,
                plank_pos: c.plank_pos,
                plank_yaw: c.plank_yaw,
                plank_y: c.plank_y,
                step_crate_from: c.step_crate_from,
                step_crate_to: c.step_crate_to,
                ladder: c.ladder,
                ladder_seg_count: c.ladder_seg_count,
            });
            taken += 1;
        }
    }
    links
}

/// Emits rooftop plank, step-crate and ladder props/ladders whose plank centre falls in `chunk`.
pub fn rooftops_in_chunk(metro: &MetroWorld, chunk: (i32, i32)) -> ClimbChunk {
    let mut out = ClimbChunk {
        pois: Vec::new(),
        props: Vec::new(),
        crates: Vec::new(),
        ladders: Vec::new(),
        boxes: Vec::new(),
    };
    for link in roof_links(metro) {
        if !in_chunk(link.plank_pos.x, link.plank_pos.z, chunk) {
            continue;
        }
        out.props.push(PropInstance::new(
            format!("roof_plank_{}", link.plank_length),
            link.plank_pos,
            link.plank_yaw,
            1.0,
        ));
        if let Some(pos) = link.step_crate_from {
            out.props.push(PropInstance::new("roof_step_crate", pos, link.plank_yaw, 1.0));
        }
        if let Some(pos) = link.step_crate_to {
            out.props.push(PropInstance::new("roof_step_crate", pos, link.plank_yaw, 1.0));
        }
        if let Some(ladder) = &link.ladder {
            let flat = Vec2::new(ladder.deck.x - ladder.foot.x, ladder.deck.z - ladder.foot.z);
            let yaw = flat.x.atan2(flat.y);
            for k in 0..link.ladder_seg_count {
                let seg_pos = Vec3::new(ladder.foot.x, ladder.foot.y + k as f32, ladder.foot.z);
                out.props.push(PropInstance::new("roof_ladder_seg", seg_pos, yaw, 1.0));
            }
            out.ladders.push(ladder.clone());
        }
    }
    out
}

fn is_valid_partner(metro: &MetroWorld, src: usize, dst: usize) -> bool {
    if src == dst {
        return false;
    }
    let t = &metro.towers[dst];
    !t.landmark && !metro.damage.tower_is_replaced(dst) && !metro.damage.tower_is_burnt(t.block)
}

fn tower_axes(t: &TowerSpot) -> [Vec2; 4] {
    let (s, c) = t.yaw.sin_cos();
    let u = Vec2::new(c, -s);
    let v = Vec2::new(s, c);
    [u, -u, v, -v]
}

fn build_candidate(
    metro: &MetroWorld,
    styles: &[StyleDef],
    ground_y: f32,
    src: usize,
    dst: usize,
) -> Option<LinkCandidate> {
    let cfg = &metro.config.rooftops;
    let a = &metro.towers[src];
    let b = &metro.towers[dst];
    let style_a = style_for(styles, a.style)?;
    let style_b = style_for(styles, b.style)?;

    let axes_a = tower_axes(a);
    let axes_b = tower_axes(b);

    let mut best_edge: Option<(Vec2, f32, f32, f32, f32, f32)> = None;
    for n_a in axes_a {
        if !axes_b.iter().any(|&n_b| n_a.dot(n_b) < -PARALLEL_DOT) {
            continue;
        }
        let t_a = Vec2::new(-n_a.y, n_a.x);
        let a_edge = a.pos.dot(n_a) + a.half.x;
        let b_edge = b.pos.dot(n_a) - b.half.x;
        let gap = b_edge - a_edge;
        if !(MIN_GAP_M..=cfg.max_gap_m).contains(&gap) {
            continue;
        }
        let a_t = a.pos.dot(t_a);
        let b_t = b.pos.dot(t_a);
        let ov_min = (a_t - a.half.x).max(b_t - b.half.x);
        let ov_max = (a_t + a.half.x).min(b_t + b.half.x);
        let overlap = ov_max - ov_min;
        if overlap < cfg.min_overlap_m {
            continue;
        }
        let ov_mid = 0.5 * (ov_min + ov_max);
        best_edge = Some((n_a, a_edge, b_edge, gap, overlap, ov_mid));
        break;
    }

    let (n_a, a_edge, b_edge, gap, overlap, ov_mid) = best_edge?;
    let t_a = Vec2::new(-n_a.y, n_a.x);

    let mut lengths = cfg.plank_lengths.clone();
    lengths.sort_unstable();
    let plank_length = lengths
        .into_iter()
        .find(|&l| l as f32 >= gap + PLANK_REST_M)?;

    let deck_a = roof_deck_y(a, style_a, ground_y);
    let deck_b = roof_deck_y(b, style_b, ground_y);
    let para_a = cfg.parapet_top.for_style(a.style);
    let para_b = cfg.parapet_top.for_style(b.style);
    let plank_y = if deck_a <= deck_b { deck_a + para_a } else { deck_b + para_b };
    let higher_deck = deck_a.max(deck_b);
    let dh = higher_deck - plank_y;
    if dh > cfg.max_drop_m {
        return None;
    }

    // Place the plank so it never cuts into a taller tower's upper hull box.
    let half_l = plank_length as f32 * 0.5;
    let gap_mid = 0.5 * (a_edge + b_edge);
    let plank_along = if plank_y >= higher_deck {
        gap_mid
    } else if deck_b > deck_a {
        b_edge - half_l - 0.05 // new: DECISIONS 2026-11-27 (hull-box clearance)
    } else {
        a_edge + half_l + 0.05 // new: DECISIONS 2026-11-27 (hull-box clearance)
    };
    let plank_xz = n_a * plank_along + t_a * ov_mid;
    let plank_yaw = n_a.x.atan2(n_a.y);
    let plank_pos = Vec3::new(plank_xz.x, plank_y, plank_xz.y);

    if plank_hits_any_tower(metro, styles, ground_y, plank_xz, plank_yaw, half_l, plank_y) {
        return None;
    }

    let edge_a_xz = n_a * a_edge + t_a * ov_mid;
    let edge_b_xz = n_a * b_edge + t_a * ov_mid;
    let step_crate_from = (plank_y - deck_a > STEP_CRATE_THRESHOLD_M).then(|| {
        let p = edge_a_xz - n_a * STEP_CRATE_INSET_M;
        Vec3::new(p.x, deck_a, p.y)
    });
    let step_crate_to = (plank_y - deck_b > STEP_CRATE_THRESHOLD_M).then(|| {
        let p = edge_b_xz + n_a * STEP_CRATE_INSET_M;
        Vec3::new(p.x, deck_b, p.y)
    });

    let (ladder, ladder_seg_count) = if dh > cfg.flat_step_m {
        let (taller_edge, inward) = if deck_b > deck_a {
            (edge_b_xz, n_a)
        } else {
            (edge_a_xz, -n_a)
        };
        let foot_xz = taller_edge - inward * LADDER_WALL_CLEAR_M;
        let deck_xz = taller_edge + inward * LADDER_DECK_INSET_M;
        let segs = dh.ceil().max(1.0) as usize;
        (
            Some(ClimbLadder {
                foot: Vec3::new(foot_xz.x, plank_y, foot_xz.y),
                deck: Vec3::new(deck_xz.x, higher_deck, deck_xz.y),
                id: format!("metro_roof_{src}_{dst}"),
            }),
            segs,
        )
    } else {
        (None, 0)
    };

    Some(LinkCandidate {
        partner: dst,
        gap,
        overlap,
        plank_length,
        plank_pos,
        plank_yaw,
        plank_y,
        step_crate_from,
        step_crate_to,
        ladder,
        ladder_seg_count,
    })
}

/// Checks whether the plank's 3D box overlaps any standing tower's upper hull box.
fn plank_hits_any_tower(
    metro: &MetroWorld,
    styles: &[StyleDef],
    ground_y: f32,
    plank_xz: Vec2,
    plank_yaw: f32,
    half_l: f32,
    plank_y: f32,
) -> bool {
    let plank_y_lo = plank_y + 0.02; // new: DECISIONS 2026-11-27 (plank contact tolerance)
    let plank_y_hi = plank_y + PLANK_THICK_M;
    for (idx, t) in metro.towers.iter().enumerate() {
        if metro.damage.tower_is_replaced(idx) {
            continue;
        }
        let Some(style) = style_for(styles, t.style) else {
            continue;
        };
        let hull_lo = ground_y
            + if style.base_height > 0.0 {
                style.base_height
            } else {
                DEFAULT_BASE_HEIGHT
            };
        let hull_hi = if t.landmark {
            ground_y + t.height
        } else {
            roof_deck_y(t, style, ground_y)
        };
        if plank_y_hi <= hull_lo || plank_y_lo >= hull_hi {
            continue;
        }
        if obb_overlaps_square(
            plank_xz,
            Vec2::new(PLANK_HALF_WIDTH_M, half_l),
            plank_yaw,
            t.pos,
            t.half.x,
            t.yaw,
        ) {
            return true;
        }
    }
    false
}

/// Separating-axis test between a 2D OBB `(c1, h1, yaw1)` and a square `(c2, h2, yaw2)`.
pub(crate) fn obb_overlaps_square(
    c1: Vec2,
    h1: Vec2,
    yaw1: f32,
    c2: Vec2,
    h2: f32,
    yaw2: f32,
) -> bool {
    let (s1, c1_r) = yaw1.sin_cos();
    let u1 = Vec2::new(c1_r, -s1);
    let v1 = Vec2::new(s1, c1_r);
    let (s2, c2_r) = yaw2.sin_cos();
    let u2 = Vec2::new(c2_r, -s2);
    let v2 = Vec2::new(s2, c2_r);
    let d = c2 - c1;

    for axis in [u1, v1, u2, v2] {
        let r1 = h1.x * u1.dot(axis).abs() + h1.y * v1.dot(axis).abs();
        let r2 = h2 * u2.dot(axis).abs() + h2 * v2.dot(axis).abs();
        if d.dot(axis).abs() >= r1 + r2 {
            return false;
        }
    }
    true
}
