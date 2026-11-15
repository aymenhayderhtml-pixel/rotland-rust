use glam::Vec3;

/// One hit on anything with health (port of scripts/Combat/DamageInfo.cs).
#[derive(Clone, Debug, PartialEq)]
pub struct DamageInfo {
    pub amount: f32,
    /// Id of whoever hit (a zombie, a player). None for falls, hunger and other non-entity damage.
    pub source: Option<u64>,
    pub hit_point: Vec3,
    pub direction: Vec3,
    /// Tag such as "melee", "bullet", "bite", "hunger", "thirst" or "fall".
    pub damage_type: String,
    /// True for a gun hit to a zombie's head, so XP rewards can read it on the killing blow.
    pub headshot: bool,
}

impl DamageInfo {
    /// A plain hit of the given amount with no source and the "generic" tag.
    pub fn new(amount: f32) -> Self {
        Self {
            amount,
            source: None,
            hit_point: Vec3::ZERO,
            direction: Vec3::ZERO,
            damage_type: "generic".to_string(),
            headshot: false,
        }
    }
}
