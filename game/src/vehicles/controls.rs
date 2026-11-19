/// What the driver asks the car to do this physics step (port of scripts/Vehicles/CarControls.cs).
///
/// Throttle and Steer run from -1 to 1. Positive steer is right, positive throttle is forward.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarControls {
    pub throttle: f32,
    pub brake: f32,
    pub steer: f32,
    pub handbrake: bool,
}

impl CarControls {
    pub fn clear(&mut self) {
        self.throttle = 0.0;
        self.brake = 0.0;
        self.steer = 0.0;
        self.handbrake = false;
    }
}

/// Raw player input for driving a car from the game loop.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarInput {
    pub throttle: f32,
    pub brake: f32,
    pub steer: f32,
    pub handbrake: bool,
    pub horn: bool,
    pub lights_toggle: bool,
    /// Request to right an overturned vehicle in place ([R]).
    pub flip_recover: bool,
}

impl CarInput {
    pub fn to_controls(&self) -> CarControls {
        CarControls {
            throttle: self.throttle.clamp(-1.0, 1.0),
            brake: self.brake.clamp(0.0, 1.0),
            steer: self.steer.clamp(-1.0, 1.0),
            handbrake: self.handbrake,
        }
    }
}
