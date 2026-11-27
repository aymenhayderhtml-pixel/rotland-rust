//! Poses for the `--metro=<spot>` test flags: where the player stands, which way it faces and,
//! for `top`, the camera pitch and spring length. Pure maths; `test_flags.rs` applies them.
//! Facing rule (AGENTS.md): yaw forward = (-sin yaw, -cos yaw), so yaw = atan2(-dx, -dz).

use glam::{Vec2, Vec3Swizzles};

use world::metro::{
    MetroStreet, MetroWorld, TowerSpot, TowerStyle, camp_sites, car_wall_sites,
};

use super::metro_flags_r41::{army_pose, crows_pose, herd_pose};
use super::metro_flags_r42::r42_pose;

/// Edge spot: metres inside the first avenue entrance. // new: DECISIONS 2026-11-26
const EDGE_INSET_M: f32 = 10.0;
/// Highway spot: metres back along the highway from its city end (inside the jam, skyline ahead). // new: DECISIONS 2026-11-26
const HIGHWAY_BACK_M: f32 = 350.0;
/// Highway spot: metres in from the highway's outer edge (on the shoulder, clear of the lanes). // new: DECISIONS 2026-11-26
const SHOULDER_IN_M: f32 = 1.0;
/// Top spot: camera height above the city ground in metres (above the ~160 m landmarks). // new: DECISIONS 2026-11-26
const TOP_HEIGHT_M: f32 = 260.0;
/// Top spot: how far the camera looks down, in degrees. // new: DECISIONS 2026-11-26
const TOP_PITCH_DEG: f32 = 60.0;
/// Landmark spot: metres from the landmark base along the nearest avenue. // new: DECISIONS 2026-11-26
const LANDMARK_VIEW_DIST_M: f32 = 60.0;
/// Leaning spot: metres from the leaning tower base along the nearest avenue. // new: DECISIONS 2026-11-26
const LEANING_VIEW_DIST_M: f32 = 40.0;
/// Fallen spot: metres from the fallen section centre along its street. // new: DECISIONS 2026-11-26
const FALLEN_VIEW_DIST_M: f32 = 55.0;
/// Burnt spot: metres from the burnt block centre on the nearest street. // new: DECISIONS 2026-11-26
const BURNT_VIEW_DIST_M: f32 = 35.0;
/// Carwall spot: metres before the first car wall on its street. // new: DECISIONS 2026-11-26
const CARWALL_VIEW_DIST_M: f32 = 12.0;
/// Camp spot: metres off the rooftop camp centre on the roof. // new: DECISIONS 2026-11-26
const CAMP_VIEW_OFFSET_M: f32 = 6.0;
/// Landmark aim share of total tower height so the crown sits in the upper view. // new: DECISIONS 2026-11-26
const LANDMARK_AIM_SHARE: f32 = 0.85;
/// Landmark spot: default third-person camera spring length in metres. // new: DECISIONS 2026-11-26
const LANDMARK_SPRING_M: f32 = 4.0;
/// Style spot: metres in front of the tower's facade on its front street. // new: DECISIONS 2026-11-26
const STYLE_STREET_DIST_M: f32 = 12.0;

/// The twelve places `--metro=<spot>` can teleport to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetroSpot {
    /// Just inside the city's first avenue entrance, looking in.
    Edge,
    /// The middle of the city, looking along the nearest avenue.
    Core,
    /// On the jam, looking at the skyline.
    Highway,
    /// Beside the nearest town, looking at the city.
    TownView,
    /// High above the Core, looking down.
    Top,
    /// Inside the ground-floor lobby of the tower nearest the Core centre, facing its door.
    Lobby,
    /// On the nearest avenue ~60 m from the tallest landmark, looking up at its crown.
    Landmark,
    /// On the street 30 m from the fallen tower section, facing its middle.
    Fallen,
    /// On the nearest avenue 40 m from the leaning tower, looking up at its top.
    Leaning,
    /// On the nearest street 35 m from the burnt block centre, facing it.
    Burnt,
    /// 12 m before the first stacked-car wall on its street, facing it.
    Carwall,
    /// On the first rooftop camp, 6 m off the camp centre, looking across it.
    Camp,
    /// On an avenue 25 m ahead of an active street herd walking toward the camera (hour 10).
    Herd,
    /// 15 m from a body perch with pecking crows.
    Crows,
    /// Inside the startle radius (6 m) of a body perch so crows burst into the air.
    CrowsUp,
    /// 2 m in front of a Core army cache crate.
    Army,
    /// Core centre at hour 23 (night runners).
    Night,
    /// 35 m in front of the hospital ER canopy.
    Hospital,
    /// Inside the hospital reception hall looking toward the ward.
    HospitalIn,
    /// Front steps of the police station.
    Police,
    /// Inside the police station open office toward the cells.
    PoliceIn,
    /// Forecourt facing the fire station bays.
    Fire,
    /// Inside the fire station bay hall beside the fire truck.
    FireIn,
    /// 30 m outside the army line barrier looking at the breach and tank.
    Armyline,
    /// 60 m up pitched down over the whole army line.
    ArmylineTop,
}

impl MetroSpot {
    /// Parses a spot name (any case). None for an unknown name.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "edge" => Some(Self::Edge),
            "core" => Some(Self::Core),
            "highway" => Some(Self::Highway),
            "town_view" => Some(Self::TownView),
            "top" => Some(Self::Top),
            "lobby" => Some(Self::Lobby),
            "landmark" => Some(Self::Landmark),
            "fallen" => Some(Self::Fallen),
            "leaning" => Some(Self::Leaning),
            "burnt" => Some(Self::Burnt),
            "carwall" => Some(Self::Carwall),
            "camp" => Some(Self::Camp),
            "herd" => Some(Self::Herd),
            "crows" => Some(Self::Crows),
            "crows_up" => Some(Self::CrowsUp),
            "army" => Some(Self::Army),
            "night" => Some(Self::Night),
            "hospital" => Some(Self::Hospital),
            "hospital_in" => Some(Self::HospitalIn),
            "police" => Some(Self::Police),
            "police_in" => Some(Self::PoliceIn),
            "fire" => Some(Self::Fire),
            "fire_in" => Some(Self::FireIn),
            "armyline" => Some(Self::Armyline),
            "armyline_top" => Some(Self::ArmylineTop),
            _ => None,
        }
    }
}

/// Camera settings for the `top` spot: the rig's pitch (radians, negative = down) and spring length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopView {
    pub pitch: f32,
    pub spring: f32,
}

/// Where to teleport: world XZ, Godot yaw, optional camera override, and optional roof/lobby Y.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetroPose {
    pub pos: Vec2,
    pub yaw: f32,
    pub top: Option<TopView>,
    pub y: Option<f32>,
}

/// Yaw that faces along `dir` (world XZ): forward = (-sin yaw, -cos yaw).
pub fn yaw_facing(dir: Vec2) -> f32 {
    (-dir.x).atan2(-dir.y)
}

/// Rig spring length that puts the camera `height` metres above the feet when the camera looks
/// down at `pitch` (negative radians). The camera sits behind the pivot along `-forward`.
pub fn top_spring(height: f32, eye_height: f32, pitch: f32) -> f32 {
    (height - eye_height) / -pitch.sin()
}

/// The avenue whose centreline is closest to `p` (site-local). None when the city has no avenue.
pub fn nearest_avenue(streets: &[MetroStreet], p: Vec2) -> Option<&MetroStreet> {
    let dist = |s: &MetroStreet| s.closest_point(p).distance_squared(p);
    streets.iter().filter(|s| s.avenue).min_by(|a, b| dist(a).total_cmp(&dist(b)))
}

/// The street (or avenue) whose centreline is closest to `p` (site-local).
fn nearest_street(streets: &[MetroStreet], p: Vec2) -> Option<&MetroStreet> {
    let dist = |s: &MetroStreet| s.closest_point(p).distance_squared(p);
    streets.iter().min_by(|a, b| dist(a).total_cmp(&dist(b)))
}

/// `EDGE_INSET_M` along the avenue from `entrance` toward `far`, looking at `center`.
pub fn edge_pose(entrance: Vec2, far: Vec2, center: Vec2) -> MetroPose {
    let inward = (far - entrance).normalize_or_zero();
    let pos = entrance + inward * EDGE_INSET_M;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None, y: None }
}

/// The city centre, looking along the avenue direction `avenue_dir` (world XZ).
pub fn core_pose(center: Vec2, avenue_dir: Vec2) -> MetroPose {
    MetroPose { pos: center, yaw: yaw_facing(avenue_dir), top: None, y: None }
}

/// The point `back` metres before the polyline's end, and the travel direction there.
pub fn point_back_from_end(points: &[Vec2], back: f32) -> Option<(Vec2, Vec2)> {
    let mut left = back;
    for w in points.windows(2).rev() {
        let seg = w[0].distance(w[1]);
        let dir = (w[1] - w[0]).normalize_or_zero();
        if seg > 0.0 && seg >= left {
            return Some((w[1] - dir * left, dir));
        }
        left -= seg;
    }
    let first = *points.first()?;
    let dir = points.get(1).map_or(Vec2::X, |b| (*b - first).normalize_or(Vec2::X));
    Some((first, dir))
}

/// On the highway shoulder `HIGHWAY_BACK_M` before the city, looking along the road at the city.
pub fn highway_pose(points: &[Vec2], width: f32) -> Option<MetroPose> {
    let (on_road, dir) = point_back_from_end(points, HIGHWAY_BACK_M)?;
    let side = Vec2::new(-dir.y, dir.x);
    let pos = on_road + side * (width * 0.5 - SHOULDER_IN_M);
    let end = *points.last()?;
    Some(MetroPose { pos, yaw: yaw_facing(end - pos), top: None, y: None })
}

/// `distance` metres from the town centre on the side facing the city, looking at `center`.
pub fn town_view_pose(town_center: Vec2, center: Vec2, distance: f32) -> MetroPose {
    let toward_city = (center - town_center).normalize_or(Vec2::X);
    let pos = town_center + toward_city * distance;
    MetroPose { pos, yaw: yaw_facing(center - pos), top: None, y: None }
}

/// Above the Core centre, looking down `TOP_PITCH_DEG` toward the city from the highway side
/// (`from` is the highway's end on the city edge). The camera sits on the spring behind the pivot.
pub fn top_pose(center: Vec2, from: Vec2, eye_height: f32) -> MetroPose {
    let pitch = -TOP_PITCH_DEG.to_radians();
    let spring = top_spring(TOP_HEIGHT_M, eye_height, pitch);
    MetroPose { pos: center, yaw: yaw_facing(center - from), top: Some(TopView { pitch, spring }), y: None }
}

/// World XZ unit direction that a tower's front (its lobby door, Blender +Y / glTF -Z) faces.
pub fn tower_front_dir(yaw: f32) -> Vec2 {
    Vec2::new(-yaw.sin(), -yaw.cos())
}

/// Inside the ground-floor lobby at `tower.pos`, facing its street door.
pub fn lobby_pose(tower: &TowerSpot) -> MetroPose {
    let front_dir = tower_front_dir(tower.yaw);
    MetroPose { pos: tower.pos, yaw: yaw_facing(front_dir), top: None, y: None }
}

/// Point on segment `(seg_a, seg_b)` at `dist_m` from `target` (closest point if perpendicular >= `dist_m`).
fn point_on_segment_at_dist(target: Vec2, seg_a: Vec2, seg_b: Vec2, dist_m: f32) -> Vec2 {
    let ab = seg_b - seg_a;
    let len_sq = ab.length_squared();
    let closest = if len_sq < 1e-6 {
        seg_a
    } else {
        let t = ((target - seg_a).dot(ab) / len_sq).clamp(0.0, 1.0);
        seg_a + ab * t
    };
    let d_perp = closest.distance(target);
    if d_perp < dist_m && len_sq >= 1e-6 {
        let along = (dist_m * dist_m - d_perp * d_perp).sqrt();
        let dir = ab / len_sq.sqrt();
        let mid = (seg_a + seg_b) * 0.5;
        let p1 = closest + dir * along;
        let p2 = closest - dir * along;
        if p1.distance_squared(mid) <= p2.distance_squared(mid) { p1 } else { p2 }
    } else {
        closest
    }
}

/// On the avenue `(ave_a, ave_b)` `dist_m` from `tower_pos`, facing the tower and aiming at 85% of `tower_height`.
pub fn landmark_pose(
    tower_pos: Vec2,
    tower_height: f32,
    ave_a: Vec2,
    ave_b: Vec2,
    dist_m: f32,
    eye_height: f32,
) -> MetroPose {
    let pos = point_on_segment_at_dist(tower_pos, ave_a, ave_b, dist_m);
    let horiz = pos.distance(tower_pos).max(1.0);
    let pitch = ((tower_height * LANDMARK_AIM_SHARE - eye_height) / horiz).atan();
    MetroPose {
        pos,
        yaw: yaw_facing(tower_pos - pos),
        top: Some(TopView { pitch, spring: LANDMARK_SPRING_M }),
        y: None,
    }
}

/// Nearest tower of `style` to `center`. None when the city has no tower of that style.
pub fn nearest_tower_of_style(
    towers: &[TowerSpot],
    center: Vec2,
    style: TowerStyle,
) -> Option<&TowerSpot> {
    towers
        .iter()
        .filter(|t| t.style == style)
        .min_by(|a, b| a.pos.distance_squared(center).total_cmp(&b.pos.distance_squared(center)))
}

/// On the street in front of `tower`, facing its facade.
pub fn style_pose(tower: &TowerSpot) -> MetroPose {
    let front_dir = tower_front_dir(tower.yaw);
    let pos = tower.pos + front_dir * (tower.half.y + STYLE_STREET_DIST_M);
    MetroPose { pos, yaw: yaw_facing(-front_dir), top: None, y: None }
}

/// The pose for `spot` in the metro city. None when the city lacks what the spot needs
/// (no avenue, no highway, no jam prop, or no town for `town_view`).
pub fn metro_pose(
    spot: MetroSpot,
    metro: &MetroWorld,
    town_center: Option<Vec2>,
    town_distance: f32,
    eye_height: f32,
) -> Option<MetroPose> {
    let site = &metro.layout.site;
    let center = site.center;
    let streets = &metro.layout.streets;
    match spot {
        MetroSpot::Edge => {
            let end = site.to_local(*metro.highway.points.last()?);
            let avenue = nearest_avenue(streets, end)?;
            let (entrance, far) = if avenue.a.distance_squared(end) <= avenue.b.distance_squared(end) {
                (avenue.a, avenue.b)
            } else {
                (avenue.b, avenue.a)
            };
            Some(edge_pose(site.to_world(entrance), site.to_world(far), center))
        }
        MetroSpot::Core => {
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let dir = site.to_world(avenue.b) - site.to_world(avenue.a);
            Some(core_pose(center, dir))
        }
        MetroSpot::Highway => highway_pose(&metro.highway.points, metro.highway.width),
        MetroSpot::TownView => {
            // Where the highway leaves the town: the road cuts through the forest toward the skyline
            let _ = town_distance;
            let start = *metro.highway.points.first().or(town_center.as_ref())?;
            Some(town_view_pose(start, center, 0.0))
        }
        MetroSpot::Top => {
            // Look along the central avenue: the camera's wall check then finds open air, not a tower
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let back = center - (site.to_world(avenue.b) - site.to_world(avenue.a)).normalize_or(Vec2::X);
            Some(top_pose(center, back, eye_height))
        }
        MetroSpot::Lobby => {
            let tower = metro
                .towers
                .iter()
                .min_by(|a, b| a.pos.distance_squared(center).total_cmp(&b.pos.distance_squared(center)))?;
            Some(lobby_pose(tower))
        }
        MetroSpot::Landmark => {
            let tallest = crate::metro_render::parts::tallest_landmark_index(&metro.towers);
            let tower = tallest
                .and_then(|i| metro.towers.get(i))
                .or_else(|| metro.towers.iter().max_by(|a, b| a.height.total_cmp(&b.height)))?;
            let avenue = nearest_avenue(streets, site.to_local(tower.pos))?;
            Some(landmark_pose(
                tower.pos,
                tower.height,
                site.to_world(avenue.a),
                site.to_world(avenue.b),
                LANDMARK_VIEW_DIST_M,
                eye_height,
            ))
        }
        MetroSpot::Fallen => {
            let target = metro.damage.fallen.pos;
            let street = nearest_street(streets, site.to_local(target))?;
            let pos = point_on_segment_at_dist(
                target,
                site.to_world(street.a),
                site.to_world(street.b),
                FALLEN_VIEW_DIST_M,
            );
            // Raised camera so the whole broken section and its rubble read at once.
            let top = TopView { pitch: -18f32.to_radians(), spring: 14.0 }; // new: DECISIONS 2026-11-26
            Some(MetroPose { pos, yaw: yaw_facing(target - pos), top: Some(top), y: None })
        }
        MetroSpot::Leaning => {
            let target = metro.damage.leaning.pos;
            let avenue = nearest_avenue(streets, site.to_local(target))?;
            let height = metro
                .towers
                .iter()
                .filter(|t| metro.damage.tower_is_replaced(
                    metro.towers.iter().position(|x| std::ptr::eq(x, *t)).unwrap_or(usize::MAX)
                ))
                .map(|t| t.height)
                .fold(60.0f32, f32::max);
            Some(landmark_pose(
                target,
                height,
                site.to_world(avenue.a),
                site.to_world(avenue.b),
                LEANING_VIEW_DIST_M,
                eye_height,
            ))
        }
        MetroSpot::Burnt => {
            let (bx, bz) = metro.damage.burnt_block;
            let block = metro.layout.blocks.iter().find(|b| b.ix == bx && b.iz == bz)?;
            let local_center = (block.rect.0 + block.rect.1) * 0.5;
            let target = site.to_world(local_center);
            let street = nearest_street(streets, local_center)?;
            let pos = point_on_segment_at_dist(
                target,
                site.to_world(street.a),
                site.to_world(street.b),
                BURNT_VIEW_DIST_M,
            );
            Some(MetroPose { pos, yaw: yaw_facing(target - pos), top: None, y: None })
        }
        MetroSpot::Carwall => {
            let (cw_pos, _) = car_wall_sites(metro).into_iter().next()?;
            let street = nearest_street(streets, site.to_local(cw_pos))?;
            let pos = point_on_segment_at_dist(
                cw_pos,
                site.to_world(street.a),
                site.to_world(street.b),
                CARWALL_VIEW_DIST_M,
            );
            Some(MetroPose { pos, yaw: yaw_facing(cw_pos - pos), top: None, y: None })
        }
        MetroSpot::Camp => {
            let camp = camp_sites(metro).into_iter().next()?;
            let dir = -tower_front_dir(camp.yaw);
            let camp_xz = camp.pos.xz();
            let pos = camp_xz + dir * CAMP_VIEW_OFFSET_M;
            Some(MetroPose {
                pos,
                yaw: yaw_facing(camp_xz - pos),
                top: Some(TopView { pitch: -30f32.to_radians(), spring: 10.0 }),
                y: Some(camp.pos.y + 0.2), // new: DECISIONS 2026-11-26 (on the roof deck, the hull top)
            })
        }
        MetroSpot::Night => {
            let avenue = nearest_avenue(streets, Vec2::ZERO)?;
            let dir = site.to_world(avenue.b) - site.to_world(avenue.a);
            Some(core_pose(center, dir))
        }
        MetroSpot::Herd => herd_pose(metro),
        MetroSpot::Crows | MetroSpot::CrowsUp => crows_pose(metro, spot),
        MetroSpot::Army => army_pose(metro),
        MetroSpot::Hospital
        | MetroSpot::HospitalIn
        | MetroSpot::Police
        | MetroSpot::PoliceIn
        | MetroSpot::Fire
        | MetroSpot::FireIn
        | MetroSpot::Armyline
        | MetroSpot::ArmylineTop => r42_pose(spot, metro, eye_height),
    }
}

#[cfg(test)]
#[path = "metro_flags_tests.rs"]
mod tests;
