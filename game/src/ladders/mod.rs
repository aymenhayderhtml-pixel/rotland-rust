//! Ladders system: rails, ends, model marker pairing, and player climb state machine.
//! Port of `scripts/World/LadderBuilder.cs`, `LadderRail.cs`, `LadderEnd.cs`, and `scripts/Player/LadderClimb.cs`.

mod builder;
mod climb;
mod end;
mod rail;
mod tick;

pub use builder::ladders_from_markers;
pub use climb::{
    ClimbInput, ClimbPhase, LadderClimb, CLIMB_SPEED, GRAB_RANGE, SETTLE_LIFT, START_GRACE,
    TRANSITION_SECONDS,
};
pub use end::{ladder_prompt, nearest_ladder_end, LadderEnd};
pub use rail::LadderRail;
pub use tick::climb_tick;
