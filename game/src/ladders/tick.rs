use glam::Vec3;

use super::climb::{ClimbInput, LadderClimb};
use super::end::nearest_ladder_end;
use super::rail::LadderRail;
use crate::player::Player;

/// Updates ladder interaction and climbing each frame before normal player move.
/// Returns `true` if the player is currently on a ladder.
pub fn climb_tick(
    climb: &mut LadderClimb,
    player: &mut Player,
    input: &ClimbInput,
    rails: &[LadderRail],
    dt: f32,
) -> bool {
    if climb.is_active() {
        let on_ladder = climb.step(
            dt,
            input,
            &mut player.position,
            &mut player.velocity,
            &mut player.yaw,
        );
        if on_ladder {
            player.on_floor = false;
        }
        return on_ladder;
    }

    if input.interact
        && let Some((end, _)) = nearest_ladder_end(player.position, rails, false)
    {
        climb.start(end.rail.clone(), end.is_top, player.position);
        let facing = end.rail.facing();
        player.yaw = (-facing.x).atan2(-facing.z);
        if !end.is_top {
            player.position = end.rail.foot;
        }
        player.velocity = Vec3::ZERO;
        player.on_floor = false;
        return true;
    }

    false
}
