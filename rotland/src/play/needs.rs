//! Hunger and thirst over time, and the reset after a respawn.

use super::PlayState;

impl PlayState {
    /// Hunger and thirst drain; returns (damage from an empty need, healing from being well fed).
    pub fn needs_tick(&mut self, dt: f32, sprinting: bool, dead: bool, health: (f32, f32), stamina: &mut game::Stamina) -> (f32, f32) {
        if health.1 > 0.0 {
            self.player_health = (health.0 / health.1).clamp(0.0, 1.0);
        }
        self.use_cooldown = (self.use_cooldown - dt).max(0.0);
        let damage = self.hunger.update(dt, sprinting, dead) + self.thirst.update(dt, sprinting, dead);
        let heal = game::WellFedRegen::update(&self.hunger, &self.thirst, health.0, health.1, dead, &self.survival.well_fed, dt);
        game::update_stamina_regen_multiplier(stamina, &self.hunger, &self.thirst, self.survival.stamina.low_needs_regen_multiplier);
        (damage, heal)
    }

    /// After a respawn the needs start at Godot's respawn value.
    pub fn needs_respawn(&mut self) {
        let v = self.survival.respawn_needs_value;
        self.hunger.set(v);
        self.thirst.set(v);
    }
}
