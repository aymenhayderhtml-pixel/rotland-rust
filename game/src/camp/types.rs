//! Core types and enums for camp life (port of scripts/Settlement/Camp*.cs).

use serde::{Deserialize, Serialize};

/// What a camp resident is doing right now (Godot: CampState.cs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampState {
    Idle,
    Walk,
    Fire,
    Work,
    Chat,
    Hidden,
}

/// A resident's daytime job (Godot: CampJob.cs).
/// Only two residents have one; the rest wander.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampJob {
    None,
    Woodcutter,
    Carrier,
}

/// What a resident's own update reports back to the director (Godot: CampEvent.cs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CampEvent {
    None,
    Arrived,
    TimerDone,
}

/// What a resident does when a walk ends (Godot: CampNext.cs).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CampNext {
    Idle,
    Fire,
    Work,
    Chat,
    Hide,
}

/// The time of day as the camp rules read it (Godot: CampClock.cs).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CampClock {
    pub hour: f32,
    pub night: bool,
    pub fire_hours: bool,
    pub day: i32,
}

impl CampClock {
    pub fn new(hour: f32, night: bool, fire_hours: bool, day: i32) -> Self {
        Self {
            hour,
            night,
            fire_hours,
            day,
        }
    }

    /// Hours when people gather at the fire (19:00 to 06:00), from data/camp_life.json.
    // Godot: CampClock.cs From
    pub fn from_hour(hour: f32, night: bool, day: i32, fire_from: f32, fire_to: f32) -> Self {
        let fire_hours = hour >= fire_from || hour < fire_to;
        Self {
            hour,
            night,
            fire_hours,
            day,
        }
    }
}
