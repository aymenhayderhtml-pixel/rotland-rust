/// The car's hit points (port of scripts/Vehicles/CarHealth.cs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarHealth {
    pub(crate) max: f32,
    pub(crate) current: f32,
}

impl CarHealth {
    pub fn new(max: f32) -> Self {
        Self { max, current: max }
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn is_wrecked(&self) -> bool {
        self.current <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            self.current / self.max
        }
    }

    pub fn damage(&mut self, amount: f32) {
        if amount > 0.0 {
            self.current = (self.current - amount).max(0.0);
        }
    }

    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.max);
    }
}
