use serde::{Deserialize, Serialize};

use game_core::items::Inventory;

use crate::survival::boosts::NightVisionConfig;

/// Item ID consumed by night vision goggles (`data/items.json`).
pub const BATTERY_ITEM: &str = "battery";

/// Night vision goggles state: on/off and remaining battery charge in seconds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NightVision {
    pub on: bool,
    pub charge: f32,
}

impl NightVision {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggles night vision. Turning on requires existing charge or 1 `battery` in `inventory`.
    /// Returns the resulting `on` state.
    pub fn toggle(&mut self, inventory: &mut Inventory, cfg: &NightVisionConfig) -> bool {
        if self.on {
            self.on = false;
            return false;
        }
        if self.charge > 0.0 {
            self.on = true;
            return true;
        }
        if self.consume_battery(inventory, cfg) {
            self.on = true;
        }
        self.on
    }

    /// Drains charge while `on`. At 0 it consumes the next battery or turns off.
    pub fn update(&mut self, dt: f32, inventory: &mut Inventory, cfg: &NightVisionConfig) {
        if !self.on {
            return;
        }
        let mut remaining = if dt > 0.0 && dt.is_finite() { dt } else { 0.0 };
        while remaining > 0.0 {
            if self.charge > remaining {
                self.charge -= remaining;
                return;
            }
            remaining -= self.charge;
            self.charge = 0.0;
            if !self.consume_battery(inventory, cfg) {
                self.on = false;
                return;
            }
        }
    }

    /// Visual strength: `1.0` when on, `0.0` when off.
    pub fn strength(&self) -> f32 {
        if self.on && self.charge > 0.0 {
            1.0
        } else {
            0.0
        }
    }

    fn consume_battery(&mut self, inventory: &mut Inventory, cfg: &NightVisionConfig) -> bool {
        let cap = cfg.max_charge.max(0.0);
        let add = cfg.seconds_per_battery.max(0.0).min(cap);
        if add <= 0.0 {
            return false;
        }
        if inventory.remove(BATTERY_ITEM, 1) {
            self.charge = add;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_and_drain_batteries() {
        let cfg = NightVisionConfig::default();
        let mut inv = Inventory::new();
        let mut nv = NightVision::new();

        assert!(!nv.toggle(&mut inv, &cfg));
        assert_eq!(nv.strength(), 0.0);

        inv.add(BATTERY_ITEM, 2);
        assert!(nv.toggle(&mut inv, &cfg));
        assert_eq!(nv.strength(), 1.0);
        assert_eq!(inv.count(BATTERY_ITEM), 1);

        nv.update(300.0, &mut inv, &cfg);
        assert!(nv.on);
        assert_eq!(inv.count(BATTERY_ITEM), 0);

        nv.update(300.0, &mut inv, &cfg);
        assert!(!nv.on);
        assert_eq!(nv.strength(), 0.0);
    }
}
