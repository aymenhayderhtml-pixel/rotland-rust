//! R44 spot poses for `--metro=<spot>`: the safe house roof, the rooftop camp, the rooftop trader
//! and a stop-floor rescue. Geometry comes from `data/safe_house.json` (through `safe_house_layout`)
//! and the R44-03 spot functions. Pure maths.

use glam::{Vec2, Vec3Swizzles};
use world::metro::safehouse::{rescue_floor_seeds, rooftop_camp_tower, safe_house_layout, safe_house_spots};
use world::metro::{MetroWorld, TowerSpot, default_styles, roof_deck_y, style_for};

use super::metro_flags::{MetroPose, MetroSpot, TopView, yaw_facing};

const ROOF_STAND_CLEAR_M: f32 = 0.1; // new: DECISIONS 2026-11-27 (clearance above roof deck, as R43)
const CAMP_VIEW_DIST_M: f32 = 6.0; // new: DECISIONS 2026-11-28 (6 m from the trader stall)
const TRADER_SIDE_DIST_M: f32 = 1.2; // new: DECISIONS 2026-11-28 (beside the trader, inside useRadiusM)
const SAFE_STAND_TO_SHACK_M: f32 = 0.5; // new: DECISIONS 2026-11-28 (0.5 m off travel_point toward the shack: shack and sign both within useRadiusM 2.0)
const BED_SIDE_DIST_M: f32 = 1.0; // new: DECISIONS 2026-11-28 (beside the bed on the side away from the chest, inside useRadiusM)
const CLAIMED_PITCH_DEG: f32 = 35.0; // new: DECISIONS 2026-11-28 (camera looks down the back face to the rope)
const CLAIMED_SPRING_M: f32 = 6.0; // new: DECISIONS 2026-11-28 (camera spring behind the player)
const CAMP_PITCH_DEG: f32 = 30.0; // new: DECISIONS 2026-11-28 (camera looks down over the tents)
const CAMP_SPRING_M: f32 = 8.0; // new: DECISIONS 2026-11-28 (camera spring over the stall)

/// A camera looking down at `deg` degrees with a spring of `spring` metres.
fn look_down(deg: f32, spring: f32) -> TopView {
    TopView { pitch: -deg.to_radians(), spring }
}

/// Tower-local `(lx, lz)` to world XZ (same frame as `metro_flags_r43`).
fn tower_local_xz(t: &TowerSpot, lx: f32, lz: f32) -> Vec2 {
    let (sin_y, cos_y) = t.yaw.sin_cos();
    Vec2::new(
        t.pos.x + lx * cos_y + lz * sin_y,
        t.pos.y - lx * sin_y + lz * cos_y,
    )
}

/// The bed's tower-local `(x, z)` from the safe house data.
fn safe_house_bed_local(layout: &world::metro::safehouse::SafeHouseLayout) -> [f32; 2] {
    layout.shack.bed_local
}

/// Tower index from a stop-floor POI id `metro_floor_<tower>_<floor>`.
fn tower_of_floor_poi(poi_id: &str) -> Option<usize> {
    poi_id.strip_prefix("metro_floor_")?.split('_').next()?.parse().ok()
}

/// Resolves an R44 `--metro=<spot>` pose, or `None` if not an R44 spot or the city lacks it.
pub(super) fn r44_pose(spot: MetroSpot, metro: &MetroWorld) -> Option<MetroPose> {
    let layout = safe_house_layout()?;
    let ground_y = metro.layout.site.ground_y;
    match spot {
        MetroSpot::SafeHouse | MetroSpot::SafeHouseClaimed | MetroSpot::SafeHouseBed => {
            // The height closure only feeds the ladder foot; the poses use the roof plane and XZ.
            let spots = safe_house_spots(metro, layout, &|_, _| ground_y)?;
            let y = Some(spots.deck_y + ROOF_STAND_CLEAR_M);
            if spot == MetroSpot::SafeHouse {
                let travel = spots.travel_point.xz();
                let toward_shack = (spots.shack.xz() - travel).normalize_or(Vec2::NEG_Y);
                let pos = travel + toward_shack * SAFE_STAND_TO_SHACK_M;
                Some(MetroPose { pos, yaw: yaw_facing(spots.sign.xz() - pos), top: None, y })
            } else if spot == MetroSpot::SafeHouseBed {
                let bed = safe_house_bed_local(layout);
                let pos = tower_local_xz(&metro.towers[spots.tower], bed[0] - BED_SIDE_DIST_M, bed[1]);
                Some(MetroPose { pos, yaw: yaw_facing(spots.bed.xz() - pos), top: None, y })
            } else {
                let pos = spots.ladder_deck.xz();
                let top = Some(look_down(CLAIMED_PITCH_DEG, CLAIMED_SPRING_M));
                Some(MetroPose { pos, yaw: yaw_facing(spots.ladder_foot.xz() - pos), top, y })
            }
        }
        MetroSpot::RooftopCamp => {
            let camp = &layout.rooftop_camp;
            let t = &metro.towers[rooftop_camp_tower(metro)?];
            let style = style_for(default_styles(), t.style)?;
            let stall = camp.props.iter().find(|p| p.model == "rooftop_trader_stall")?.local;
            let tents: Vec<[f32; 2]> = camp.props.iter().filter(|p| p.model == "evac_tent").map(|p| p.local).collect();
            if tents.is_empty() {
                return None;
            }
            let n = tents.len() as f32;
            let tent_mid = tents.iter().fold(Vec2::ZERO, |a, p| a + Vec2::new(p[0], p[1])) / n;
            let stall = Vec2::new(stall[0], stall[1]);
            let dir = (tent_mid - stall).normalize_or(Vec2::NEG_X);
            let local = stall + dir * CAMP_VIEW_DIST_M;
            let pos = tower_local_xz(t, local.x, local.y);
            let aim = tower_local_xz(t, tent_mid.x, tent_mid.y);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(aim - pos),
                top: Some(look_down(CAMP_PITCH_DEG, CAMP_SPRING_M)),
                y: Some(roof_deck_y(t, style, ground_y) + ROOF_STAND_CLEAR_M),
            })
        }
        MetroSpot::RooftopTrader => {
            let camp = &layout.rooftop_camp;
            let t = &metro.towers[rooftop_camp_tower(metro)?];
            let style = style_for(default_styles(), t.style)?;
            let trader = camp.npcs.iter().find(|n| n.role.as_deref() == Some("trader"))?.local;
            let pos = tower_local_xz(t, trader[0] - TRADER_SIDE_DIST_M, trader[1]);
            let aim = tower_local_xz(t, trader[0], trader[1]);
            Some(MetroPose {
                pos,
                yaw: yaw_facing(aim - pos),
                top: None,
                y: Some(roof_deck_y(t, style, ground_y) + ROOF_STAND_CLEAR_M),
            })
        }
        MetroSpot::RescueFloor => {
            let seed = rescue_floor_seeds(metro).into_iter().next()?;
            let tower = &metro.towers[tower_of_floor_poi(&seed.poi_id)?];
            // The shaft door of that tower; the seed stands 3 m in from it, so the door is ahead.
            let door_lz = metro.config.climb.shaft_centre_z_share * tower.size as f32
                - metro.config.climb.shaft_size[1] * 0.5;
            let door = tower_local_xz(tower, 0.0, door_lz);
            let pos = seed.position.xz();
            Some(MetroPose { pos, yaw: yaw_facing(door - pos), top: None, y: Some(seed.position.y) })
        }
        _ => None,
    }
}
