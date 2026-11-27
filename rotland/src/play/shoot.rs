//! The gun in hand: firing, reloading and aiming each frame.

use super::PlayState;
use glam::Vec3;
use physics::PhysicsWorld;

use crate::shooting::{ShootEvents, ShootInput};

impl PlayState {
    /// The gun in hand fires, reloads and aims (shooting.rs). Returns this frame's events.
    #[allow(clippy::too_many_arguments)]
    pub fn shoot_tick(&mut self, dt: f32, input: ShootInput, eye: Vec3, aim: Vec3, targets: &mut impl crate::shooting::TargetWorld, physics: &PhysicsWorld) -> Option<ShootEvents> {
        let gun = self.gun.clone()?;
        let cond = self.equipped_condition();
        let ev = self.shooting.update_targets_with_wear(dt, input, &gun, eye, aim, &mut self.inventory, targets, physics, cond, &self.wear_config);
        if ev.broken && input.fire_pressed {
            self.show_toast("Broken - repair at a gunsmith".into());
        } else if ev.jammed {
            self.show_toast("Weapon jammed!".into());
        }
        self.apply_equipped_shot_wear(ev.shots_fired);
        self.last_shot = ev.clone();
        Some(ev)
    }
}
