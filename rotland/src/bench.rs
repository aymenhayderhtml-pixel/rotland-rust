//! The fixed walk of Godot's forest25 benchmark (BenchmarkPath): forward 5 s, turn 90 degrees over 1 s, repeat.

use std::f32::consts::PI;

pub const SIDE_SECONDS: f32 = 5.0;
pub const TURN_SECONDS: f32 = 1.0;
pub const WALK_SPEED: f32 = 4.0;

/// Camera yaw at time `t` after the walk starts (the player always walks forward).
pub fn yaw_at(t: f32) -> f32 {
    let side = (t / SIDE_SECONDS).floor();
    let into = t - side * SIDE_SECONDS;
    let yaw = side * PI * 0.5;
    if side > 0.0 && into < TURN_SECONDS {
        let from = (side - 1.0) * PI * 0.5;
        let k = (into / TURN_SECONDS).clamp(0.0, 1.0);
        return from + (yaw - from) * (k * k * (3.0 - 2.0 * k));
    }
    yaw
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_a_quarter_each_side() {
        assert_eq!(yaw_at(2.0), 0.0);
        assert!((yaw_at(5.0 + 1.0) - PI * 0.5).abs() < 1e-5);
        assert!((yaw_at(10.5) - (PI * 0.5 + PI * 0.25)).abs() < 1e-5);
    }
}
