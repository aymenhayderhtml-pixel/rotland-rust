//! Butchering timer for one dead animal.
// Godot: AnimalButcher.cs

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AnimalButcher {
    pub started: bool,
    pub done: bool,
    pub left: f32,
}

impl AnimalButcher {
    pub fn new() -> Self {
        Self::default()
    }

    // Godot: AnimalButcher.cs Reset
    pub fn reset(&mut self) {
        self.started = false;
        self.done = false;
        self.left = 0.0;
    }

    // Godot: AnimalButcher.cs Start
    pub fn start(&mut self, seconds: f32) -> bool {
        if self.started || self.done {
            return false;
        }
        self.started = true;
        self.left = seconds;
        true
    }

    // Godot: AnimalButcher.cs Cancel
    pub fn cancel(&mut self) {
        self.started = false;
        self.left = 0.0;
    }

    // Godot: AnimalButcher.cs Tick
    pub fn tick(&mut self, dt: f32) -> bool {
        if !self.started || self.done {
            return false;
        }
        self.left -= dt;
        if self.left > 0.0 {
            return false;
        }
        self.started = false;
        self.done = true;
        self.left = 0.0;
        true
    }

    pub fn set_done(&mut self) {
        self.started = false;
        self.done = true;
        self.left = 0.0;
    }
}
