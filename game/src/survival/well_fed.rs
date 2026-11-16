use crate::items::ItemEffects;
use crate::player::Stamina;
use crate::survival::config::WellFedDef;
use crate::survival::need::NeedStat;

/// Well-fed regeneration logic (port of scripts/Survival/WellFedRegen.cs).
pub struct WellFedRegen;

impl WellFedRegen {
    /// Checks whether well-fed health regeneration conditions are met.
    // Godot: WellFedRegen.cs _PhysicsProcess
    pub fn can_heal(
        hunger: &NeedStat,
        thirst: &NeedStat,
        health_current: f32,
        health_max: f32,
        is_dead: bool,
        threshold: f32,
    ) -> bool {
        if is_dead || health_current >= health_max {
            return false;
        }
        hunger.current() >= threshold && thirst.current() >= threshold
    }

    /// Calculates health points regenerated this frame.
    // Godot: WellFedRegen.cs _PhysicsProcess
    pub fn update(
        hunger: &NeedStat,
        thirst: &NeedStat,
        health_current: f32,
        health_max: f32,
        is_dead: bool,
        def: &WellFedDef,
        dt: f32,
    ) -> f32 {
        if Self::can_heal(hunger, thirst, health_current, health_max, is_dead, def.threshold) {
            def.heal_per_sec * dt
        } else {
            0.0
        }
    }
}

/// Applies food/drink item effects to hunger and thirst stats, scaling positive restoration
/// by the cooking skill food multiplier (port of scripts/Items/ItemUse.cs ApplyEffects).
/// Returns health change (positive for healing, negative for damage/sickness).
// Godot: ItemUse.cs ApplyEffects
pub fn apply_item_effects(
    effects: &ItemEffects,
    hunger: &mut NeedStat,
    thirst: &mut NeedStat,
    food_multiplier: f32,
) -> f32 {
    let scale = |amount: f32| {
        if amount > 0.0 {
            amount * food_multiplier
        } else {
            amount
        }
    };

    if effects.hunger != 0.0 {
        hunger.restore(scale(effects.hunger));
    }
    if effects.thirst != 0.0 {
        thirst.restore(scale(effects.thirst));
    }

    effects.health
}

/// Slows stamina regen while hunger or thirst is low (port of scripts/Player/PlayerStaminaUse.cs).
// Godot: PlayerStaminaUse.cs _PhysicsProcess
pub fn update_stamina_regen_multiplier(
    stamina: &mut Stamina,
    hunger: &NeedStat,
    thirst: &NeedStat,
    low_needs_regen_multiplier: f32,
) {
    let low_needs = hunger.is_low() || thirst.is_low();
    stamina.regen_multiplier = if low_needs {
        low_needs_regen_multiplier
    } else {
        1.0
    };
}
