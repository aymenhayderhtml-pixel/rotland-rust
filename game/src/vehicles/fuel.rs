/// The petrol tank: litres in, litres out (port of scripts/Vehicles/CarFuel.cs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CarFuel {
    pub(crate) capacity: f32,
    pub(crate) litres: f32,
}

impl CarFuel {
    pub fn new(capacity: f32) -> Self {
        Self {
            capacity,
            litres: capacity,
        }
    }

    pub fn capacity(&self) -> f32 {
        self.capacity
    }

    pub fn litres(&self) -> f32 {
        self.litres
    }

    pub fn is_empty(&self) -> bool {
        self.litres <= 0.0
    }

    pub fn fraction(&self) -> f32 {
        if self.capacity <= 0.0 {
            0.0
        } else {
            self.litres / self.capacity
        }
    }

    pub fn add(&mut self, litres: f32) -> f32 {
        if litres <= 0.0 {
            return 0.0;
        }
        let before = self.litres;
        self.litres = (self.litres + litres).min(self.capacity);
        self.litres - before
    }

    pub fn burn(&mut self, litres: f32) {
        self.litres = (self.litres - litres.max(0.0)).max(0.0);
    }

    pub fn set(&mut self, litres: f32) {
        self.litres = litres.clamp(0.0, self.capacity);
    }
}
