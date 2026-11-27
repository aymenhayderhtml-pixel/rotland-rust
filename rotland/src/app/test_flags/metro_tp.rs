//! `--metro=<spot>` teleport into New Halden, including the `top` camera pose.

use engine::Gpu;

use crate::app::metro_flags::{self, MetroSpot};
use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState, WorldState};

use super::pose::WORLD_TP_DISTANCE_M;
use super::tp::teleport_player_and_stream;

/// `--metro=<spot>`: teleports to a New Halden pose (see `metro_flags`). `top` also sets the camera
/// pitch and the rig's spring length, so the camera hangs above the Core.
pub(super) fn apply_metro_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &mut PlaySide,
    session: &SessionState,
    key: &str,
) {
    let Some(spot) = MetroSpot::parse(key) else {
        log::warn!(
            "--metro={key}: unknown spot (expected edge|core|highway|town_view|top|lobby|landmark|fallen|leaning|burnt|carwall|camp|herd|crows|crows_up|army|night)"
        );
        return;
    };
    if spot == MetroSpot::Herd && !play.clock_frozen {
        play.hour = 10.0; // new: DECISIONS 2026-11-27
        play.clock.set_time(10.0);
        play.clock_frozen = true;
    } else if spot == MetroSpot::Night && !play.clock_frozen {
        play.hour = 23.0; // new: DECISIONS 2026-11-27
        play.clock.set_time(23.0);
        play.clock_frozen = true;
    }
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(metro) = world_gen.metro_world() else {
        log::warn!("--metro={key}: this world has no New Halden");
        return;
    };
    // The town on the highway is the nearest one to the city
    let town_center = world_gen
        .towns()
        .iter()
        .find(|t| t.key.eq_ignore_ascii_case(&metro.highway.from_town))
        .map(|t| t.center);
    let eye_height = actors.camera_rig.as_ref().map_or(0.0, |r| r.eye_height);
    let pose = if spot == MetroSpot::Core
        && let Some(style) = session.test_flags.tower_style
    {
        if let Some(tower) =
            metro_flags::nearest_tower_of_style(&metro.towers, metro.layout.site.center, style)
        {
            Some(metro_flags::style_pose(tower))
        } else {
            log::warn!(
                "--tower-style={}: no tower of that style in the city; falling back to --metro=core",
                style.key()
            );
            metro_flags::metro_pose(spot, metro, town_center, WORLD_TP_DISTANCE_M, eye_height)
        }
    } else {
        metro_flags::metro_pose(spot, metro, town_center, WORLD_TP_DISTANCE_M, eye_height)
    };
    let Some(pose) = pose else {
        log::warn!("--metro={key}: the city has nothing for this spot");
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, pose.pos, pose.yaw);
    let custom_y = if spot == MetroSpot::Lobby {
        // Place inside the ground-floor lobby (above the 0.1 m slab), not on the roof collider.
        Some(metro.layout.site.ground_y + 0.15) // new: DECISIONS 2026-11-26
    } else {
        pose.y
    };
    if let Some(y) = custom_y {
        actors.player.y = y;
        if let Some(p) = actors.player_ctrl.as_mut() {
            p.position.y = y;
        }
        if let Some(rig) = actors.camera_rig.as_mut()
            && session.scene == "play"
            && !session.free_cam
        {
            render.camera = rig.camera(&actors.player, &world.physics);
        }
    }
    if let (Some(top), Some(rig)) = (pose.top, actors.camera_rig.as_mut()) {
        rig.pitch = top.pitch;
        rig.stored_zoom = top.spring;
        rig.current_spring_length = top.spring;
        if session.scene == "play" && !session.free_cam {
            render.camera = rig.camera(&actors.player, &world.physics);
        }
    }
}
