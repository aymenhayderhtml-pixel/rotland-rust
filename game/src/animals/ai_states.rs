//! State-specific think methods for AnimalAI.
// Godot: AnimalAI.cs

use std::f32::consts::TAU;
use glam::{Quat, Vec3};
use super::ai::{AnimalAI, AnimalAIOutput};
use super::config::{AnimalBehaviourData, AnimalSpeciesData};
use super::motion::AnimalMotion;
use super::state::AnimalState;

impl AnimalAI {
    // Godot: AnimalAI.cs ThinkCalm
    pub(crate) fn think_calm(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        if self.graze > 0.0 {
            self.graze -= dt;
            let clip = if self.rng.next_f32() < 0.6 { "graze" } else { "idle" };
            if self.graze <= 0.0 {
                self.has_wander = false;
            }
            return AnimalAIOutput {
                locomotion: clip,
                ..Default::default()
            };
        }
        if !self.has_wander {
            let angle = self.rng.gen_range_f32(0.0, TAU);
            let dist = self.rng.gen_range_f32(1.5, b.wander_radius);
            self.wander = self.home + Vec3::new(angle.cos() * dist, 0.0, angle.sin() * dist);
            self.has_wander = true;
        }
        let dist_to_wander = AnimalMotion::flat(self.wander - animal_pos).length();
        if dist_to_wander <= super::motion::STOP_DISTANCE {
            self.graze = self.rng.gen_range_f32(b.graze_min, b.graze_max);
            AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            }
        } else {
            AnimalAIOutput {
                wanted_target: Some((self.wander, sp.walk_speed)),
                locomotion: "walk",
                ..Default::default()
            }
        }
    }

    // Godot: AnimalAI.cs ThinkAlert
    pub(crate) fn think_alert(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        let to_threat = AnimalMotion::flat(self.threat - animal_pos);
        let wanted_yaw = if to_threat.length_squared() > 0.0001 {
            Some(AnimalMotion::face_direction(to_threat.normalize()))
        } else {
            None
        };
        self.timer -= dt;
        if self.timer <= 0.0 {
            let mut out = self.start_flee(false, sp, b);
            out.wanted_yaw = wanted_yaw;
            return out;
        }
        AnimalAIOutput {
            wanted_yaw,
            locomotion: "idle",
            one_shot: Some(("alert", self.timer)),
            ..Default::default()
        }
    }

    // Godot: AnimalAI.cs ThinkFlee
    pub(crate) fn think_flee(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        animal_yaw: f32,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        self.timer -= dt;
        if self.timer <= 0.0 {
            self.state = AnimalState::Calm;
            self.graze = 0.0;
            self.has_wander = false;
            return AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            };
        }
        self.zigzag_timer -= dt;
        if self.zigzag_timer <= 0.0 {
            self.zigzag_timer = b.zigzag_seconds;
            self.zigzag_side = if self.rng.next_f32() < 0.5 { -1.0 } else { 1.0 };
        }
        let mut away = AnimalMotion::flat(animal_pos - self.threat);
        if away.length_squared() < 0.0001 {
            away = Vec3::new(animal_yaw.cos(), 0.0, -animal_yaw.sin());
        }
        let turn = if sp.zigzag { b.zigzag_deg * self.zigzag_side } else { 0.0 };
        let dir = Quat::from_rotation_y(turn.to_radians()) * away.normalize();
        let target = animal_pos + dir * b.flee_distance;
        AnimalAIOutput {
            wanted_target: Some((target, sp.run_speed)),
            locomotion: "run",
            ..Default::default()
        }
    }

    // Godot: AnimalAI.cs ThinkCharge
    pub(crate) fn think_charge(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        player_pos: Option<Vec3>,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        if !self.hunting(animal_pos, player_pos, b) {
            self.give_up();
            return AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            };
        }
        let Some(p_pos) = player_pos else {
            return AnimalAIOutput {
                locomotion: "charge",
                ..Default::default()
            };
        };
        let dist = AnimalMotion::flat(p_pos - animal_pos).length();
        if dist <= sp.attack_range {
            self.state = AnimalState::Attack;
            return AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            };
        }
        self.timer -= dt;
        AnimalAIOutput {
            wanted_target: Some((p_pos, sp.charge_speed)),
            locomotion: "charge",
            ..Default::default()
        }
    }

    // Godot: AnimalAI.cs ThinkAttack
    pub(crate) fn think_attack(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        player_pos: Option<Vec3>,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        if !self.hunting(animal_pos, player_pos, b) {
            self.give_up();
            return AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            };
        }
        let Some(p_pos) = player_pos else {
            return AnimalAIOutput {
                locomotion: "idle",
                ..Default::default()
            };
        };
        let to_player = AnimalMotion::flat(p_pos - animal_pos);
        let dist = to_player.length();
        if dist > sp.attack_range + 0.6 {
            self.state = AnimalState::Charge;
            return AnimalAIOutput {
                locomotion: "charge",
                ..Default::default()
            };
        }
        let wanted_yaw = if to_player.length_squared() > 0.0001 {
            Some(AnimalMotion::face_direction(to_player.normalize()))
        } else {
            None
        };
        let attack_strike = if let Some(combat) = &self.combat {
            combat.attack_ready()
        } else {
            false
        };
        self.timer -= dt;
        AnimalAIOutput {
            wanted_yaw,
            locomotion: "idle",
            one_shot: if attack_strike { Some(("attack", 0.8)) } else { None },
            attack_strike,
            sound_id: if attack_strike { sp.attack_sound.clone() } else { None },
            ..Default::default()
        }
    }
}
