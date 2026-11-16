use glam::Vec3;

use crate::player::Player;
use super::climb::{ClimbInput, LadderClimb, GRAB_RANGE};
use super::rail::LadderRail;

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

    if input.interact {
        let mut best: Option<(&LadderRail, bool, f32)> = None;
        for rail in rails {
            let foot_pt = rail.foot_interaction_point();
            let d_foot = player
                .position
                .distance(foot_pt)
                .min(player.position.distance(rail.foot));
            let foot_closer = match best {
                None => true,
                Some((_, _, dist)) => d_foot < dist,
            };
            if d_foot <= GRAB_RANGE && foot_closer {
                best = Some((rail, false, d_foot));
            }

            let deck_pt = rail.deck_interaction_point();
            let d_deck = player
                .position
                .distance(deck_pt)
                .min(player.position.distance(rail.deck));
            let deck_closer = match best {
                None => true,
                Some((_, _, dist)) => d_deck < dist,
            };
            if d_deck <= GRAB_RANGE && deck_closer {
                best = Some((rail, true, d_deck));
            }
        }

        if let Some((rail, from_deck, _)) = best {
            climb.start(rail.clone(), from_deck, player.position);
            let facing = rail.facing();
            player.yaw = (-facing.x).atan2(-facing.z);
            if !from_deck {
                player.position = rail.foot;
            }
            player.velocity = Vec3::ZERO;
            player.on_floor = false;
            return true;
        }
    }

    false
}
