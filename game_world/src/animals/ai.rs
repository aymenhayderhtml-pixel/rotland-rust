//! Wild animal mind and state machine.
// Godot: AnimalAI.cs

use glam::Vec3;
use super::combat::AnimalCombat;
use super::config::{AnimalBehaviourData, AnimalButcherData, AnimalSpeciesData};
use super::motion::AnimalMotion;
use super::rng::AnimalRng;
use super::senses::AnimalSenses;
use super::state::AnimalState;

const SENSE_INTERVAL: f32 = 0.2;
const HIT_CHARGE_SECONDS: f32 = 12.0;

/// Output decisions from one think tick.
#[derive(Clone, Debug, Default)]
pub struct AnimalAIOutput {
    pub wanted_target: Option<(Vec3, f32)>, // (target_position, speed)
    pub wanted_yaw: Option<f32>,
    pub locomotion: &'static str,
    pub one_shot: Option<(&'static str, f32)>, // (clip_name, duration)
    pub attack_strike: bool,
    pub herd_alarm: Option<Vec3>,
    pub sound_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AnimalAI {
    pub state: AnimalState,
    pub home: Vec3,
    pub body_left: f32,
    pub combat: Option<AnimalCombat>,
    pub(crate) threat: Vec3,
    pub(crate) wander: Vec3,
    pub(crate) has_wander: bool,
    pub(crate) hit_charge: bool,
    pub(crate) timer: f32,
    pub(crate) graze: f32,
    pub(crate) sense_timer: f32,
    pub(crate) zigzag_timer: f32,
    pub(crate) zigzag_side: f32,
    pub(crate) rng: AnimalRng,
}

impl AnimalAI {
    pub fn new(sp: &AnimalSpeciesData, seed: u64) -> Self {
        let combat = if sp.danger {
            Some(AnimalCombat::new())
        } else {
            None
        };
        Self {
            state: AnimalState::Calm,
            home: Vec3::ZERO,
            body_left: 0.0,
            combat,
            threat: Vec3::ZERO,
            wander: Vec3::ZERO,
            has_wander: false,
            hit_charge: false,
            timer: 0.0,
            graze: 0.0,
            sense_timer: 0.0,
            zigzag_timer: 0.0,
            zigzag_side: 1.0,
            rng: AnimalRng::new(seed),
        }
    }

    // Godot: AnimalAI.cs Reset
    pub fn reset(&mut self, home: Vec3) {
        self.home = home;
        self.state = AnimalState::Calm;
        self.body_left = 0.0;
        self.timer = 0.0;
        self.graze = 0.0;
        self.sense_timer = 0.0;
        self.has_wander = false;
        self.hit_charge = false;
        if let Some(c) = &mut self.combat {
            c.reset();
        }
    }

    // Godot: AnimalAI.cs Die
    pub fn die(&mut self, butcher: &AnimalButcherData) {
        self.state = AnimalState::Dead;
        self.body_left = butcher.body_seconds;
    }

    // Godot: AnimalAI.cs React
    pub fn react(&mut self, threat: Vec3, from_herd: bool, sp: &AnimalSpeciesData, b: &AnimalBehaviourData) -> Option<AnimalAIOutput> {
        if sp.danger {
            return self.begin_charge(false, sp);
        }
        if self.state != AnimalState::Calm {
            return None;
        }
        Some(self.start_alert(threat, from_herd, sp, b))
    }

    // Godot: AnimalAI.cs OnHurt
    pub fn on_hurt(&mut self, from: Vec3, sp: &AnimalSpeciesData, b: &AnimalBehaviourData) -> Option<AnimalAIOutput> {
        if self.state == AnimalState::Dead {
            return None;
        }
        if sp.danger {
            return self.begin_charge(true, sp);
        }
        self.threat = from;
        Some(self.start_flee(true, sp, b))
    }

    // Godot: AnimalAI.cs OnNoise
    pub fn on_noise(
        &mut self,
        pos: Vec3,
        radius: f32,
        animal_pos: Vec3,
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> Option<AnimalAIOutput> {
        if sp.danger || self.state != AnimalState::Calm || radius < b.gunshot_noise {
            return None;
        }
        if AnimalMotion::flat(pos - animal_pos).length() > b.gunshot_range {
            return None;
        }
        Some(self.start_alert(pos, false, sp, b))
    }

    pub(crate) fn start_alert(&mut self, threat: Vec3, from_herd: bool, sp: &AnimalSpeciesData, b: &AnimalBehaviourData) -> AnimalAIOutput {
        self.state = AnimalState::Alert;
        self.threat = threat;
        self.timer = b.alert_seconds;
        let herd_alarm = if !from_herd { Some(threat) } else { None };
        AnimalAIOutput {
            locomotion: "idle",
            one_shot: Some(("alert", b.alert_seconds)),
            herd_alarm,
            sound_id: sp.alert_sound.clone(),
            ..Default::default()
        }
    }

    pub(crate) fn start_flee(&mut self, alarm_herd: bool, sp: &AnimalSpeciesData, b: &AnimalBehaviourData) -> AnimalAIOutput {
        self.state = AnimalState::Flee;
        self.timer = b.flee_seconds;
        self.zigzag_timer = 0.0;
        let herd_alarm = if alarm_herd { Some(self.threat) } else { None };
        AnimalAIOutput {
            locomotion: "run",
            herd_alarm,
            sound_id: sp.alert_sound.clone(),
            ..Default::default()
        }
    }

    pub(crate) fn begin_charge(&mut self, hit: bool, sp: &AnimalSpeciesData) -> Option<AnimalAIOutput> {
        if matches!(self.state, AnimalState::Charge | AnimalState::Attack) {
            return None;
        }
        self.state = AnimalState::Charge;
        self.hit_charge = hit;
        self.timer = HIT_CHARGE_SECONDS;
        Some(AnimalAIOutput {
            locomotion: "charge",
            sound_id: sp.alert_sound.clone(),
            ..Default::default()
        })
    }

    // Godot: AnimalAI.cs Think
    pub fn think(
        &mut self,
        dt: f32,
        animal_pos: Vec3,
        animal_yaw: f32,
        player_info: Option<(Vec3, bool, bool, bool, f32)>, // (pos, crouch, sprint, dead, stealth_bonus)
        sp: &AnimalSpeciesData,
        b: &AnimalBehaviourData,
    ) -> AnimalAIOutput {
        if self.state == AnimalState::Dead {
            self.body_left -= dt;
            return AnimalAIOutput {
                locomotion: "idle",
                one_shot: Some(("die", f32::INFINITY)),
                ..Default::default()
            };
        }

        if let Some(c) = &mut self.combat {
            c.tick(dt);
        }

        let living_player_pos = match player_info {
            Some((pos, _, _, false, _)) => Some(pos),
            _ => None,
        };

        // Sense the player every 0.2s
        self.sense_timer -= dt;
        if self.sense_timer <= 0.0 {
            self.sense_timer = SENSE_INTERVAL;
            if let Some((p_pos, p_crouch, p_sprint, false, p_stealth)) = player_info
                && AnimalSenses::notices(sp, b, animal_pos, animal_yaw, p_pos, p_crouch, p_sprint, p_stealth) {
                    if sp.danger {
                        if self.state == AnimalState::Calm
                            && let Some(out) = self.begin_charge(false, sp) {
                                return out;
                            }
                    } else if self.state == AnimalState::Calm {
                        return self.start_alert(p_pos, false, sp, b);
                    }
                }
        }

        match self.state {
            AnimalState::Calm => self.think_calm(dt, animal_pos, sp, b),
            AnimalState::Alert => self.think_alert(dt, animal_pos, sp, b),
            AnimalState::Flee => self.think_flee(dt, animal_pos, animal_yaw, sp, b),
            AnimalState::Charge => self.think_charge(dt, animal_pos, living_player_pos, sp, b),
            AnimalState::Attack => self.think_attack(dt, animal_pos, living_player_pos, sp, b),
            AnimalState::Dead => AnimalAIOutput::default(),
        }
    }

    pub(crate) fn hunting(&self, animal_pos: Vec3, player_pos: Option<Vec3>, b: &AnimalBehaviourData) -> bool {
        let Some(p_pos) = player_pos else {
            return false;
        };
        if self.hit_charge {
            return self.timer > 0.0;
        }
        AnimalMotion::flat(p_pos - animal_pos).length() <= b.boar_leash
    }

    pub(crate) fn give_up(&mut self) {
        self.state = AnimalState::Calm;
        self.hit_charge = false;
        self.graze = 0.0;
        self.has_wander = false;
    }
}
