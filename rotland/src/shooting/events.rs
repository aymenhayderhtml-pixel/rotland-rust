//! Input and per-frame event types for the player's shooting system.

use game::zombie::Noise;

/// Player shooting input for the current frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ShootInput {
    pub fire_pressed: bool,
    pub fire_held: bool,
    pub reload_pressed: bool,
    pub aiming: bool,
}

/// Events and outcome produced by one shooting update frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShootEvents {
    pub shots_fired: u32,
    pub hits: u32,
    pub kills: u32,
    pub noise: Option<Noise>,
    pub reload_started: bool,
    pub ammo: (u32, u32),
    pub empty_click: bool,
    pub jammed: bool,
    pub broken: bool,
    pub bash: bool,
    pub fired_weapon: Option<String>,
}
