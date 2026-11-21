//! Bow draw charging, release, and aiming field-of-view calculation.

/// Tracks the draw amount of a bow from 0.0 to 1.0.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BowState {
    pub draw: f32,
}

impl BowState {
    pub fn new() -> Self {
        Self { draw: 0.0 }
    }

    /// Advances the bow draw while fire is held, or releases the arrow when released.
    /// Returns Some(draw) on the exact frame the bow is released with draw > 0.
    pub fn update(&mut self, dt: f32, fire_held: bool, draw_time: f32) -> Option<f32> {
        if fire_held {
            let dt_rate = if draw_time > 0.0 { dt / draw_time } else { 1.0 };
            self.draw = (self.draw + dt_rate).min(1.0);
            None
        } else if self.draw > 0.0 {
            let released_draw = self.draw;
            self.draw = 0.0;
            Some(released_draw)
        } else {
            None
        }
    }

    /// Cancels the bow draw (e.g. on weapon switch or death).
    pub fn cancel(&mut self) {
        self.draw = 0.0;
    }

    /// Computes the camera FOV narrowing while drawing the bow (slight zoom like aiming).
    /// Drops from base_fov down toward target aim fov proportionally to draw.
    pub fn aim_fov(base_fov: f32, draw: f32, aim_target_fov: f32) -> f32 {
        base_fov - (base_fov - aim_target_fov).max(0.0) * draw.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bow_draw_advances_and_clamps() {
        let mut state = BowState::new();
        assert_eq!(state.draw, 0.0);

        let res0 = state.update(0.5, true, 1.0);
        assert_eq!(res0, None);
        assert_eq!(state.draw, 0.5);

        let res1 = state.update(0.8, true, 1.0);
        assert_eq!(res1, None);
        assert_eq!(state.draw, 1.0); // clamped at 1.0
    }

    #[test]
    fn bow_release_returns_draw_and_resets() {
        let mut state = BowState::new();
        state.update(0.7, true, 1.0);
        assert_eq!(state.draw, 0.7);

        let release = state.update(0.1, false, 1.0);
        assert_eq!(release, Some(0.7));
        assert_eq!(state.draw, 0.0);

        // Next update without holding fire yields None
        let next = state.update(0.1, false, 1.0);
        assert_eq!(next, None);
    }

    #[test]
    fn bow_cancel_resets_draw() {
        let mut state = BowState::new();
        state.update(0.9, true, 1.0);
        assert_eq!(state.draw, 0.9);

        state.cancel();
        assert_eq!(state.draw, 0.0);
    }

    #[test]
    fn bow_aim_fov_narrows_with_draw() {
        let base_fov = 75.0;
        let aim_target_fov = 55.0;

        assert_eq!(BowState::aim_fov(base_fov, 0.0, aim_target_fov), 75.0);
        assert_eq!(BowState::aim_fov(base_fov, 0.5, aim_target_fov), 65.0);
        assert_eq!(BowState::aim_fov(base_fov, 1.0, aim_target_fov), 55.0);
    }
}
