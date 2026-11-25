use glam::Vec3;

use game_core::items::Inventory;
use super::fire::Campfire;

/// Distance in front of the player where the campfire is placed.
// Godot: CampfirePlacer.cs PlaceDistance
pub const PLACE_DISTANCE: f32 = 1.5;
/// Minimum horizontal spacing between campfires in metres.
pub const CAMPFIRE_SPACING: f32 = 2.0;
/// Item ID of the campfire kit.
pub const CAMPFIRE_KIT_ID: &str = "campfire_kit";

/// Reason a campfire could not be placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceCampfireError {
    NoGround,
    GroundTooSteep,
    TooCloseToAnotherFire,
    NoCampfireKit,
}

impl PlaceCampfireError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::NoGround => "There is no ground there",
            Self::GroundTooSteep => "The ground there is too steep",
            Self::TooCloseToAnotherFire => "Too close to another campfire",
            Self::NoCampfireKit => "No campfire kit",
        }
    }
}

/// Checks if ground normal is flat enough for placing (~23 degrees max, normal.y >= 0.92).
pub fn is_flat(normal: Vec3) -> bool {
    normal.y >= 0.92
}

/// Places a campfire 1.5 m in front of the player on flat ground.
/// Checks spacing against `existing_fires`, ground slope via `ground` closure,
/// and consumes 1 `campfire_kit` if `inventory` is provided.
// Godot: CampfirePlacer.cs Place
pub fn place_campfire(
    player_pos: Vec3,
    facing: Vec3,
    existing_fires: &[Campfire],
    mut inventory: Option<&mut Inventory>,
    ground: impl Fn(Vec3) -> Option<(Vec3, Vec3)>,
) -> Result<Campfire, PlaceCampfireError> {
    if let Some(ref bag) = inventory
        && !bag.has(CAMPFIRE_KIT_ID, 1)
    {
        return Err(PlaceCampfireError::NoCampfireKit);
    }

    let flat = Vec3::new(facing.x, 0.0, facing.z);
    let forward = if flat.length_squared() > 1e-4 {
        flat.normalize()
    } else {
        Vec3::NEG_Z
    };

    let target_feet = player_pos + forward * PLACE_DISTANCE;
    let Some((ground_pt, normal)) = ground(target_feet) else {
        return Err(PlaceCampfireError::NoGround);
    };

    if !is_flat(normal) {
        return Err(PlaceCampfireError::GroundTooSteep);
    }

    let spacing_sq = CAMPFIRE_SPACING * CAMPFIRE_SPACING;
    for fire in existing_fires {
        let dx = ground_pt.x - fire.position.x;
        let dz = ground_pt.z - fire.position.z;
        if dx * dx + dz * dz < spacing_sq {
            return Err(PlaceCampfireError::TooCloseToAnotherFire);
        }
    }

    if let Some(ref mut bag) = inventory {
        bag.remove(CAMPFIRE_KIT_ID, 1);
    }

    Ok(Campfire::create(ground_pt))
}
