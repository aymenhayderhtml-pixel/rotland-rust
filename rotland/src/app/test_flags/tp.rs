//! Teleports for the dev flags (`--biome`, `--town`, `--trailhead`, `--bridge`, `--river`, `--tp-rescue`).

use engine::Gpu;
use glam::{Vec2, Vec3};
use world::BiomeId;

use crate::app::state::{ActorsState, PlaySide, RenderState, SessionState, WorldState};

use super::biome_pose::find_biome_point;
use super::pose::{river_tp_pose, town_tp_pose, trailhead_tp_pose, tp_rescue_pose};

pub(super) fn teleport_player_and_stream(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    xz: Vec2,
    yaw: f32,
) {
    let mut pos = Vec3::new(xz.x, world.height(xz.x, xz.y), xz.y);
    if let (Some(pack), Some(streamer)) = (&render.pack, world.streamer.as_mut()) {
        while !streamer.is_ready_around(pos) {
            streamer.update(
                &gpu.device,
                pack,
                &mut render.library,
                &mut world.physics,
                pos,
            );
            std::thread::yield_now();
        }
    }
    pos.y = world
        .physics
        .ground_height(pos.x, pos.z)
        .unwrap_or_else(|| world.height(pos.x, pos.z));

    actors.player = pos;
    if let Some(p) = actors.player_ctrl.as_mut() {
        p.position = pos;
        p.yaw = yaw;
        p.velocity = Vec3::ZERO;
    }
    if let Some(rig) = actors.camera_rig.as_mut() {
        rig.yaw = yaw;
        if session.scene == "play" && !session.free_cam {
            render.camera = rig.camera(&pos, &world.physics);
        }
    }
}

pub(super) fn apply_biome_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    target: BiomeId,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let camp = world_gen
        .settlements()
        .first()
        .map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
    let Some((xz, yaw)) = find_biome_point(camp, target, |x, z| world_gen.biome_sample(x, z))
    else {
        log::warn!("--biome={}: no matching biome point found", target.key());
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

pub(super) fn apply_town_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    key: &str,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(town) = world_gen
        .towns()
        .iter()
        .find(|t| t.key.eq_ignore_ascii_case(key))
    else {
        log::warn!("--town={key}: no matching town found");
        return;
    };
    let (xz, yaw) = town_tp_pose(town);
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

pub(super) fn apply_trailhead_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(&th) = world_gen.roads().trailheads().get(index) else {
        log::warn!("--trailhead={index}: trailhead index out of range");
        return;
    };
    let camp = world_gen
        .settlements()
        .first()
        .map_or(Vec2::ZERO, |s| Vec2::new(s.center.x, s.center.z));
    let (xz, yaw) = trailhead_tp_pose(th, world_gen.roads().roads(), camp);
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

pub(super) fn apply_bridge_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let crossings = world_gen.rivers().crossings(world_gen.roads());
    let Some(&(xz, road_yaw, _span)) = crossings.get(index) else {
        log::warn!("--bridge={index}: crossing index out of range");
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, road_yaw);
}

pub(super) fn apply_river_tp(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    session: &SessionState,
    index: usize,
) {
    let Some(world_gen) = world.world_gen.clone() else {
        return;
    };
    let Some(river) = world_gen.rivers().rivers().get(index) else {
        log::warn!("--river={index}: river index out of range");
        return;
    };
    let Some((xz, yaw)) = river_tp_pose(river) else {
        log::warn!("--river={index}: river has too few points");
        return;
    };
    teleport_player_and_stream(gpu, render, world, actors, session, xz, yaw);
}

pub(super) fn apply_tp_rescue(
    gpu: &Gpu,
    render: &mut RenderState,
    world: &mut WorldState,
    actors: &mut ActorsState,
    play: &PlaySide,
    session: &SessionState,
) {
    let Some(spots) = play.rescue.as_ref().map(|r| r.active_spots()) else {
        return;
    };
    let Some((pos, yaw)) = tp_rescue_pose(actors.player, spots) else {
        log::warn!("--tp-rescue: no unrescued survivor spots found");
        return;
    };
    teleport_player_and_stream(
        gpu,
        render,
        world,
        actors,
        session,
        Vec2::new(pos.x, pos.z),
        yaw,
    );
}
