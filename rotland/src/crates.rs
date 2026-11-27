//! Loot crates and world items: placement and player interaction.
//!
//! Port of scripts/World/LootCrate.cs, scripts/Items/WorldItem.cs, and scripts/Player/PlayerInteractor.cs.

use glam::{Mat4, Vec3};
use world::LootSpot;

use game::{
    Inventory, ItemRegistry, ItemStack, LootRoller, LootState, LootTableRegistry, WorldItem,
    WorldItems,
};


/// The model name for loot crates in the asset pack.
pub const CRATE_MODEL: &str = "crate_loot";

/// Model id to draw for `spot`: `spot.model` when set, `CRATE_MODEL` when `None`, or `None` when `Some("")`.
pub fn crate_model(spot: &LootSpot) -> Option<&str> {
    match spot.model.as_deref() {
        Some("") => None,
        Some(m) => Some(m),
        None => Some(CRATE_MODEL),
    }
}

/// Loot table to roll for `spot`: `spot.table` when set and present, else the biome table.
pub fn crate_table<'a>(spot: &LootSpot, loot_tables: &'a LootTableRegistry) -> &'a game::LootTableData {
    spot.table
        .as_deref()
        .and_then(|name| loot_tables.get(name))
        .unwrap_or_else(|| loot_tables.for_biome(spot.biome))
}

/// Maximum reach distance in meters from player to interactable (scripts/Player/PlayerInteractor.cs:16).
pub const INTERACT_RANGE: f32 = 2.5;

/// Half-cone angle in degrees for interaction targeting (scripts/Player/PlayerInteractor.cs:18).
pub const HALF_CONE_DEGREES: f32 = 35.0;


/// Distance beside the crate where non-fitting items drop (scripts/World/LootCrate.cs:21).
pub const DROP_DISTANCE: f32 = 0.9;

/// Half a world item box height, so the dropped box sits on the ground (scripts/World/LootCrate.cs:23).
pub const DROP_LIFT: f32 = 0.125;

pub const SEARCH_CRATE_PROMPT: &str = "[E] Search crate";
pub const EMPTY_CRATE_PROMPT: &str = "Empty crate";

/// Result of player interaction with a crate or world item.
#[derive(Clone, Debug, PartialEq)]
pub enum InteractResult {
    None,
    OpenedCrate {
        crate_id: String,
        items: Vec<ItemStack>,
        cash: i32,
        leftovers: Vec<ItemStack>,
        toast: String,
    },
    PickedUpItem {
        item_id: String,
        count: u32,
        leftover: u32,
        toast: Option<String>,
    },
}

/// Identifies an interactable in reach of the player.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InteractTarget<'a> {
    Crate(&'a LootSpot),
    WorldItem(usize, &'a WorldItem),
}



/// Prompt text for a crate HUD interaction.
pub fn crate_prompt(looted: bool) -> &'static str {
    if looted {
        EMPTY_CRATE_PROMPT
    } else {
        SEARCH_CRATE_PROMPT
    }
}


/// Prompt text for picking up a world item.
pub fn world_item_prompt(item_name: &str, count: u32) -> String {
    format!("[E] Pick up {item_name} \u{00D7}{count}")
}

/// Finds the best interactable target in front of the player within cone and range.
pub fn find_target<'a>(
    player_pos: Vec3,
    facing: Vec3,
    crates: &'a [LootSpot],
    world_items: &'a WorldItems,
) -> Option<InteractTarget<'a>> {
    let forward = Vec3::new(facing.x, 0.0, facing.z);
    let forward_len2 = forward.length_squared();
    if forward_len2 < 0.0001 {
        return None;
    }
    let forward = forward / forward_len2.sqrt();
    let range2 = INTERACT_RANGE * INTERACT_RANGE;

    let mut best_target: Option<InteractTarget<'a>> = None;
    let mut best_angle = f32::MAX;

    for spot in crates {
        let to_target = spot.position - player_pos;
        if to_target.length_squared() > range2 {
            continue;
        }
        let flat_to_target = Vec3::new(to_target.x, 0.0, to_target.z);
        if flat_to_target.dot(forward) < 0.0 {
            continue;
        }
        let flat_len2 = flat_to_target.length_squared();
        let angle = if flat_len2 < 0.0001 {
            0.0
        } else {
            let cos_angle = (flat_to_target.dot(forward) / flat_len2.sqrt()).clamp(-1.0, 1.0);
            cos_angle.acos().to_degrees()
        };
        if angle <= HALF_CONE_DEGREES && angle < best_angle {
            best_angle = angle;
            best_target = Some(InteractTarget::Crate(spot));
        }
    }

    for (index, item) in world_items.items().iter().enumerate() {
        let to_target = item.position - player_pos;
        if to_target.length_squared() > range2 {
            continue;
        }
        let flat_to_target = Vec3::new(to_target.x, 0.0, to_target.z);
        if flat_to_target.dot(forward) < 0.0 {
            continue;
        }
        let flat_len2 = flat_to_target.length_squared();
        let angle = if flat_len2 < 0.0001 {
            0.0
        } else {
            let cos_angle = (flat_to_target.dot(forward) / flat_len2.sqrt()).clamp(-1.0, 1.0);
            cos_angle.acos().to_degrees()
        };
        if angle <= HALF_CONE_DEGREES && angle < best_angle {
            best_angle = angle;
            best_target = Some(InteractTarget::WorldItem(index, item));
        }
    }

    best_target
}

/// Prompt text for the best interactable in front of the player, or None if nothing in reach.
pub fn interact_prompt(
    player_pos: Vec3,
    facing: Vec3,
    crates: &[LootSpot],
    world_items: &WorldItems,
    loot_state: &LootState,
    items: Option<&ItemRegistry>,
) -> Option<String> {
    match find_target(player_pos, facing, crates, world_items) {
        None => None,
        Some(InteractTarget::Crate(spot)) => {
            Some(crate_prompt(loot_state.is_looted(&spot.crate_id)).to_string())
        }
        Some(InteractTarget::WorldItem(_, item)) => {
            let name = items
                .and_then(|reg| reg.get(&item.item_id))
                .map(|def| def.name.as_str())
                .unwrap_or(item.item_id.as_str());
            Some(world_item_prompt(name, item.count))
        }
    }
}

/// Opens the nearest unlooted crate or picks up the nearest world item in reach.
#[allow(clippy::too_many_arguments)]
pub fn interact(
    player_pos: Vec3,
    facing: Vec3,
    inventory: &mut Inventory,
    loot_state: &mut LootState,
    crates: &[LootSpot],
    world_items: &mut WorldItems,
    world_seed: i32,
    loot_tables: &LootTableRegistry,
    items: &ItemRegistry,
    loot_multiplier: f32,
) -> InteractResult {
    // Copy what is needed out of the target first, so the world item list can be changed below
    enum Picked {
        Crate(LootSpot),
        Item(usize, String, u32),
    }
    let picked = match find_target(player_pos, facing, crates, world_items) {
        None => return InteractResult::None,
        Some(InteractTarget::Crate(spot)) => Picked::Crate(spot.clone()),
        Some(InteractTarget::WorldItem(index, item)) => Picked::Item(index, item.item_id.clone(), item.count),
    };

    match picked {
        Picked::Crate(spot) => {
            if loot_state.is_looted(&spot.crate_id) {
                return InteractResult::None;
            }
            loot_state.mark_looted(&spot.crate_id);

            let table = crate_table(&spot, loot_tables);
            let rolled = LootRoller::roll(table, world_seed, &spot.crate_id, loot_multiplier);
            let dollars = LootRoller::roll_cash(table, world_seed, &spot.crate_id);

            let mut parts = Vec::new();
            let mut leftovers = Vec::new();

            // Right-hand vector of the crate (GlobalBasis.X in Godot)
            let basis_x = Mat4::from_rotation_y(spot.yaw).transform_vector3(Vec3::X);
            let drop_pos = spot.position + basis_x * DROP_DISTANCE + Vec3::Y * DROP_LIFT;

            for stack in rolled.iter() {
                let not_fitting = inventory.add_with_registry(&stack.item_id, stack.count, items);
                if not_fitting > 0 {
                    world_items.add(&stack.item_id, not_fitting, drop_pos);
                    leftovers.push(ItemStack::new(&stack.item_id, not_fitting));
                }
                let name = items
                    .get(&stack.item_id)
                    .map(|d| d.name.as_str())
                    .unwrap_or(stack.item_id.as_str());
                if stack.count > 1 {
                    parts.push(format!("{name} \u{00D7}{}", stack.count));
                } else {
                    parts.push(name.to_string());
                }
            }

            if dollars > 0 {
                parts.push(format!("${dollars}"));
            }

            let text = format!("Found: {}", parts.join(", "));
            let toast = if !leftovers.is_empty() {
                format!("{text} (backpack full, rest on the ground)")
            } else {
                text
            };

            InteractResult::OpenedCrate {
                crate_id: spot.crate_id.clone(),
                items: rolled,
                cash: dollars,
                leftovers,
                toast,
            }
        }
        Picked::Item(index, item_id, count) => {
            let not_fitting = inventory.add_with_registry(&item_id, count, items);
            let picked = count - not_fitting;

            if not_fitting == 0 {
                world_items.remove(index);
            } else if let Some(target_item) = world_items.get_mut(index) {
                target_item.count = not_fitting;
            }

            let toast = if picked == 0 {
                Some("Backpack full".to_string())
            } else {
                None
            };

            InteractResult::PickedUpItem {
                item_id,
                count: picked,
                leftover: not_fitting,
                toast,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world::BiomeId;

    #[test]
    fn crate_model_and_table_respect_overrides() {
        let mut spot = LootSpot::new("c1", Vec3::ZERO, 0.0, BiomeId::Plains);
        assert_eq!(crate_model(&spot), Some(CRATE_MODEL));
        spot.model = Some("army_crate".into());
        assert_eq!(crate_model(&spot), Some("army_crate"));
        spot.model = Some(String::new());
        assert_eq!(crate_model(&spot), None);

        let reg = LootTableRegistry::from_repo().expect("loot_tables.json must load");
        assert_eq!(crate_table(&spot, &reg), reg.for_biome(BiomeId::Plains));
        spot.table = Some("metro_army".into());
        assert_eq!(crate_table(&spot, &reg), reg.get("metro_army").unwrap());
    }
}
