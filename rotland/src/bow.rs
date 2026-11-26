//! Bow draw charging, crossbow fire/reload state, and aiming field-of-view calculation.

/// Result of updating the crossbow for one frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CrossbowStep {
    /// True on the exact frame the crossbow fires a loaded bolt.
    pub fired: bool,
    /// True on the exact frame a reload begins (caller should play `crossbow_reload`).
    pub reload_started: bool,
    /// True on the exact frame a reload completes (caller should consume 1 bolt from inventory).
    pub reload_finished: bool,
}

/// Tracks the draw amount of a bow from 0.0 to 1.0, and crossbow loaded/reload state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BowState {
    pub draw: f32,
    /// Stays loaded while carried across weapon switches.
    pub crossbow_loaded: bool,
    /// Remaining reload time in seconds when reloading the crossbow.
    pub crossbow_reload_timer: Option<f32>,
}

impl Default for BowState {
    fn default() -> Self {
        Self::new()
    }
}

impl BowState {
    pub fn new() -> Self {
        Self {
            draw: 0.0,
            crossbow_loaded: true,
            crossbow_reload_timer: None,
        }
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

    /// Updates crossbow firing (instant when loaded, no draw) and 2.0 s reload state.
    pub fn update_crossbow(
        &mut self,
        dt: f32,
        fire_pressed: bool,
        reload_pressed: bool,
        has_ammo_in_bag: bool,
        reload_time: f32,
    ) -> CrossbowStep {
        let mut step = CrossbowStep::default();
        if let Some(t) = self.crossbow_reload_timer.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                self.crossbow_reload_timer = None;
                if has_ammo_in_bag {
                    self.crossbow_loaded = true;
                    step.reload_finished = true;
                }
            }
            return step;
        }
        if fire_pressed && self.crossbow_loaded {
            self.crossbow_loaded = false;
            step.fired = true;
            if has_ammo_in_bag {
                self.crossbow_reload_timer = Some(reload_time.max(0.0));
                step.reload_started = true;
            }
            return step;
        }
        if (reload_pressed || fire_pressed) && !self.crossbow_loaded && has_ammo_in_bag {
            self.crossbow_reload_timer = Some(reload_time.max(0.0));
            step.reload_started = true;
        }
        step
    }

    /// True while the crossbow is actively reloading.
    pub fn is_crossbow_reloading(&self) -> bool {
        self.crossbow_reload_timer.is_some()
    }

    /// Cancels the bow draw and any in-progress crossbow reload (e.g. on weapon switch or death).
    /// Preserves `crossbow_loaded` so the crossbow stays loaded across weapon switches.
    pub fn cancel(&mut self) {
        self.draw = 0.0;
        self.crossbow_reload_timer = None;
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

    #[test]
    fn crossbow_fires_instantly_and_reloads() {
        let mut state = BowState::new();
        assert!(state.crossbow_loaded);

        let fired = state.update_crossbow(0.016, true, false, true, 2.0);
        assert!(fired.fired);
        assert!(fired.reload_started);
        assert!(!fired.reload_finished);
        assert!(!state.crossbow_loaded);
        assert!(state.is_crossbow_reloading());

        // Mid-reload: cannot fire, not finished yet
        let mid = state.update_crossbow(1.0, true, false, true, 2.0);
        assert!(!mid.fired);
        assert!(!mid.reload_finished);
        assert!(!state.crossbow_loaded);

        // Finish reload
        let done = state.update_crossbow(1.0, false, false, true, 2.0);
        assert!(!done.fired);
        assert!(done.reload_finished);
        assert!(state.crossbow_loaded);
        assert!(!state.is_crossbow_reloading());
    }

    #[test]
    fn crossbow_stays_loaded_across_cancel() {
        let mut state = BowState::new();
        assert!(state.crossbow_loaded);
        state.cancel();
        assert!(state.crossbow_loaded);
    }
}
