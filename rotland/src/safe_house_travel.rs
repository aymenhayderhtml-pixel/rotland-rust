//! Fast travel between the safe house roof and the home camp (R44-05).

use glam::Vec3;
use physics::PhysicsWorld;

use crate::props::PropLibrary;
use crate::streaming::Streamer;

/// Whether a travel goes through or is blocked by chasing zombies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TravelDecision {
    Blocked,
    Go,
}

/// Travel is only possible once the safe house is claimed, and never while zombies chase.
pub fn travel_decision(claimed: bool, chasing: usize) -> TravelDecision {
    if claimed && !game::safe_house::travel_blocked(chasing) {
        TravelDecision::Go
    } else {
        TravelDecision::Blocked
    }
}

/// Streams the chunks around `target` (blocking, like the test teleports) and returns the
/// position on the ground there. `target.y` is used when no ground is found.
pub fn stream_to(
    device: &wgpu::Device,
    pack: &pack::ModelPack,
    library: &mut PropLibrary,
    streamer: &mut Streamer,
    physics: &mut PhysicsWorld,
    target: Vec3,
) -> Vec3 {
    while !streamer.is_ready_around(target) {
        streamer.update(device, pack, library, physics, target);
        std::thread::yield_now();
    }
    let mut pos = target;
    pos.y = physics.ground_height(target.x, target.z).unwrap_or(target.y);
    pos
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn travel_needs_a_claim_and_no_chasers() {
        assert_eq!(travel_decision(true, 0), TravelDecision::Go);
        assert_eq!(travel_decision(true, 1), TravelDecision::Blocked);
        assert_eq!(travel_decision(false, 0), TravelDecision::Blocked);
    }
}
