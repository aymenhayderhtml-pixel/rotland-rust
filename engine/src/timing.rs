use std::time::Instant;

/// Frame timer: delta time for the game, and an FPS figure refreshed once a second.
pub struct FrameTimer {
    last: Instant,
    window_start: Instant,
    frames: u32,
    pub fps: f32,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self { last: now, window_start: now, frames: 0, fps: 0.0 }
    }

    /// Seconds since the last call (capped so a long stall does not explode the simulation),
    /// and whether the FPS figure was just refreshed.
    pub fn tick(&mut self) -> (f32, bool) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        self.frames += 1;
        let span = (now - self.window_start).as_secs_f32();
        if span >= 1.0 {
            self.fps = self.frames as f32 / span;
            self.frames = 0;
            self.window_start = now;
            return (dt, true);
        }
        (dt, false)
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}
