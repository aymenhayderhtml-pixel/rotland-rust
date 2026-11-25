use crate::survival::config::{NeedDef, SurvivalData};

/// One survival need (hunger or thirst), ported from scripts/Survival/NeedStat.cs.
/// Saved under "need_{need_id}".
#[derive(Clone, Debug, PartialEq)]
pub struct NeedStat {
    pub need_id: String,
    pub def: NeedDef,
    pub current: f32,
    pub rate_multiplier: f32,
}

impl NeedStat {
    pub fn new(need_id: impl Into<String>, def: NeedDef) -> Self {
        let current = def.start;
        Self {
            need_id: need_id.into(),
            def,
            current,
            rate_multiplier: 1.0,
        }
    }

    /// Initializes a need from SurvivalData.
    pub fn from_config(need_id: &str, config: &SurvivalData) -> Option<Self> {
        config.need(need_id).map(|def| Self::new(need_id, *def))
    }

    pub fn need_id(&self) -> &str {
        &self.need_id
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn max(&self) -> f32 {
        self.def.max
    }

    pub fn is_empty(&self) -> bool {
        self.current <= 0.0
    }

    pub fn is_low(&self) -> bool {
        self.current <= self.def.low_threshold
    }

    /// Sets the current value clamped between 0 and maximum.
    // Godot: NeedStat.cs Set
    pub fn set(&mut self, value: f32) {
        self.current = value.clamp(0.0, self.def.max);
    }

    /// Adds to the need (eating or drinking), clamped to maximum.
    // Godot: NeedStat.cs Restore
    pub fn restore(&mut self, amount: f32) {
        self.set(self.current + amount);
    }

    /// Advances need decay over time. Returns damage to apply to health when empty.
    // Godot: NeedStat.cs _PhysicsProcess
    pub fn update(&mut self, dt: f32, is_sprinting: bool, is_dead: bool) -> f32 {
        if is_dead || dt <= 0.0 {
            return 0.0;
        }

        let sprint_mult = if is_sprinting {
            self.def.sprint_drain_multiplier
        } else {
            1.0
        };
        let rate = self.def.drain_per_sec * self.rate_multiplier * sprint_mult;
        self.current = (self.current - rate * dt).max(0.0);

        if self.is_empty() {
            self.def.empty_damage_per_sec * dt
        } else {
            0.0
        }
    }

    /// Godot save id ("need_hunger" or "need_thirst").
    // Godot: NeedStat.cs SaveId
    pub fn save_id(&self) -> String {
        format!("need_{}", self.need_id)
    }

    /// Saves the current need value matching Godot dictionary format.
    // Godot: NeedStat.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({ "value": self.current })
    }

    /// Loads the saved value, clamping to [0, max].
    // Godot: NeedStat.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(val) = data.get("value").and_then(|v| v.as_f64()) {
            self.set(val as f32);
        }
    }
}
