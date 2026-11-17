//! Car trunk gameplay interactions and inventory transfers.
//!
//! Port of scripts/UI/TrunkScreen.cs, scripts/Vehicles/Car.cs, scripts/Vehicles/CarPoint.cs,
//! and scripts/Player/PlayerInteractor.cs.

use glam::Vec3;

use game::{Car, Cars, Inventory, ItemRegistry};

use crate::hub_ui::HubAction;

/// Maximum reach distance in meters from player to car trunk marker (scripts/Player/PlayerInteractor.cs:16).
pub const INTERACT_RANGE: f32 = 2.5;

/// Half-cone angle in degrees for interaction targeting (scripts/Player/PlayerInteractor.cs:18).
pub const HALF_CONE_DEGREES: f32 = 35.0;

/// Default local position of the car's trunk marker in Godot coordinates (Blender empty: 0.0, -2.20, 0.60).
pub const DEFAULT_TRUNK_LOCAL: Vec3 = Vec3::new(0.0, 0.60, 2.20);

/// Prompt text for opening a car trunk (scripts/Vehicles/CarPoint.cs:33).
pub const OPEN_TRUNK_PROMPT: &str = "[E] Open trunk";

/// Returns the prompt text for opening a car trunk ("[E] Open trunk").
pub fn open_trunk_prompt() -> &'static str {
    OPEN_TRUNK_PROMPT
}

/// Finds the parked car whose trunk marker is within reach in front of the player.
///
/// Follows Godot PlayerInteractor.cs cone and range checks:
/// - Only parked (non-driven) cars are interactable.
/// - Range <= 2.5 m (3D distance).
/// - Within a 70 degree cone (35 degrees half-angle) on the flat XZ plane.
/// - If multiple cars qualify, the one closest to the center of view (smallest angle) is picked.
pub fn trunk_in_front(
    cars: &Cars,
    player: Vec3,
    facing: Vec3,
    trunk_local: Vec3,
) -> Option<String> {
    let forward = Vec3::new(facing.x, 0.0, facing.z);
    let forward_len2 = forward.length_squared();
    if forward_len2 < 0.0001 {
        return None;
    }
    let forward = forward / forward_len2.sqrt();
    let range2 = INTERACT_RANGE * INTERACT_RANGE;

    let mut best_car: Option<String> = None;
    let mut best_angle = f32::MAX;

    for car in cars.live_cars() {
        if car.is_driven() {
            continue;
        }
        let world_matrix = car.draw_state().world_matrix;
        let trunk_pos = world_matrix.transform_point3(trunk_local);
        let to_target = trunk_pos - player;
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
            best_car = Some(car.id.clone());
        }
    }

    best_car
}

/// Finds the parked car using the default trunk marker local position.
pub fn trunk_in_front_default(cars: &Cars, player: Vec3, facing: Vec3) -> Option<String> {
    trunk_in_front(cars, player, facing, DEFAULT_TRUNK_LOCAL)
}

/// Moves items between player bag and car trunk based on the action.
///
/// Port of scripts/UI/TrunkScreen.cs: MoveToTrunk, MoveToBag, and Move.
pub fn apply_trunk_action(
    action: &HubAction,
    car: &mut Car,
    bag: &mut Inventory,
    items: &ItemRegistry,
) -> Option<String> {
    let HubAction::TrunkMove { from_trunk, slot, count } = action else {
        return None;
    };

    let (from, to) = if *from_trunk {
        (&mut car.trunk, bag)
    } else {
        (bag, &mut car.trunk)
    };

    let stack = from.slot(*slot)?;
    let item_id = stack.item_id.clone();
    let item_count = stack.count;

    // In TrunkScreen.cs: int n = count <= 0 ? s.Count : Math.Min(count, s.Count);
    let mut n = if *count == 0 { item_count } else { (*count).min(item_count) };

    // In TrunkScreen.cs: while (n > 0 && !to.CanAdd(itemId, n)) { n--; }
    while n > 0 && !to.can_add_with(&item_id, n, items) {
        n -= 1;
    }

    let item_name = items.get(&item_id).map(|d| d.name.as_str()).unwrap_or(&item_id);

    if n == 0 {
        let msg = if *from_trunk { "Your bag is full" } else { "The trunk is full" };
        return Some(msg.to_string());
    }

    from.remove_from_slot(*slot, n);
    to.add_with_registry(&item_id, n, items);

    let toast = if *from_trunk {
        format!("Took {n} {item_name} out")
    } else {
        format!("Put {n} {item_name} in the trunk")
    };

    Some(toast)
}

#[cfg(test)]
mod tests {
    use super::*;
    use game::vehicles::VehicleConfigData;
    use game::Inventory;

    #[test]
    fn test_trunk_in_front_detection() {
        let config = VehicleConfigData::from_repo().expect("vehicles.json");
        let mut cars = Cars::new(config);
        cars.spawn_car("car_camp", "red", Vec3::ZERO, 0.0, 20.0, 300.0);

        // 1. In front of the trunk: behind the car at (0, 0, 3.5), facing -Z toward trunk at (0, 0.6, 2.2)
        // Distance = sqrt(0^2 + 0.6^2 + (-1.3)^2) = sqrt(2.05) ~= 1.432 m <= 2.5 m; angle = 0.0 deg <= 35.0 deg
        let player_at_trunk = Vec3::new(0.0, 0.0, 3.5);
        let facing_trunk = Vec3::new(0.0, 0.0, -1.0);
        let found = trunk_in_front(&cars, player_at_trunk, facing_trunk, DEFAULT_TRUNK_LOCAL);
        assert_eq!(found, Some("car_camp".to_string()));
        assert_eq!(trunk_in_front_default(&cars, player_at_trunk, facing_trunk), Some("car_camp".to_string()));

        // 2. Beside the door does not find the car (distance = sqrt(1.8^2 + 0.6^2 + 2.2^2) = sqrt(8.44) ~= 2.905 m > 2.5 m)
        let player_beside_door = Vec3::new(-1.8, 0.0, 0.0);
        let facing_door = Vec3::new(1.0, 0.0, 0.0);
        assert_eq!(trunk_in_front(&cars, player_beside_door, facing_door, DEFAULT_TRUNK_LOCAL), None);

        // 3. Facing away from the trunk does not find it
        let facing_away = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(trunk_in_front(&cars, player_at_trunk, facing_away, DEFAULT_TRUNK_LOCAL), None);

        // 4. Driven car is not interactable
        if let Some(car) = cars.get_mut("car_camp") {
            car.set_driven(true);
        }
        assert_eq!(trunk_in_front(&cars, player_at_trunk, facing_trunk, DEFAULT_TRUNK_LOCAL), None);
    }

    #[test]
    fn test_trunk_move_stack_both_ways() {
        let config = VehicleConfigData::from_repo().expect("vehicles.json");
        let items = ItemRegistry::from_repo().expect("items.json");
        let mut car = Car::new("test_car", "red", &config);
        let mut bag = Inventory::from_registry(items.clone());

        // Add 5 scrap_metal (weight 0.8 * 5 = 4.0 kg) to bag slot 0
        assert_eq!(bag.add_with_registry("scrap_metal", 5, &items), 0);
        assert_eq!(bag.count("scrap_metal"), 5);
        assert_eq!(car.trunk.count("scrap_metal"), 0);

        // Move whole stack from bag to trunk (count = 0)
        let to_trunk_act = HubAction::TrunkMove { from_trunk: false, slot: 0, count: 0 };
        let toast = apply_trunk_action(&to_trunk_act, &mut car, &mut bag, &items);
        assert_eq!(toast, Some("Put 5 Scrap Metal in the trunk".to_string()));
        assert_eq!(bag.count("scrap_metal"), 0);
        assert!(bag.slot(0).is_none());
        assert_eq!(car.trunk.count("scrap_metal"), 5);
        assert!(car.trunk.slot(0).is_some());

        // Move whole stack back from trunk to bag (count = 0)
        let to_bag_act = HubAction::TrunkMove { from_trunk: true, slot: 0, count: 0 };
        let toast_back = apply_trunk_action(&to_bag_act, &mut car, &mut bag, &items);
        assert_eq!(toast_back, Some("Took 5 Scrap Metal out".to_string()));
        assert_eq!(car.trunk.count("scrap_metal"), 0);
        assert!(car.trunk.slot(0).is_none());
        assert_eq!(bag.count("scrap_metal"), 5);
        assert!(bag.slot(0).is_some());
    }

    #[test]
    fn test_overweight_move_leaves_rest_in_bag() {
        let config = VehicleConfigData::from_repo().expect("vehicles.json");
        let items = ItemRegistry::from_repo().expect("items.json");
        let mut car = Car::new("test_car", "red", &config);
        let mut bag = Inventory::from_registry(items.clone());

        // Set trunk max weight to 10.0 kg; pre-fill trunk with 7 wood (1.0 kg each = 7.0 kg)
        car.trunk.set_max_weight(10.0);
        assert_eq!(car.trunk.add_with_registry("wood", 7, &items), 0);
        assert_eq!(car.trunk.count("wood"), 7);
        // Remaining weight room in trunk = 10.0 - 7.0 = 3.0 kg

        // Place 8 wood in bag slot 0 (8.0 kg)
        assert_eq!(bag.add_with_registry("wood", 8, &items), 0);
        assert_eq!(bag.count("wood"), 8);

        // Attempt to move all 8 wood from bag slot 0 into trunk: only 3 fit (3.0 kg), 5 stay in bag
        let act = HubAction::TrunkMove { from_trunk: false, slot: 0, count: 0 };
        let toast = apply_trunk_action(&act, &mut car, &mut bag, &items);
        assert_eq!(toast, Some("Put 3 Wood in the trunk".to_string()));
        assert_eq!(bag.count("wood"), 5);
        assert_eq!(bag.slot(0).map(|s| s.count), Some(5));
        assert_eq!(car.trunk.count("wood"), 10);
        assert!((car.trunk.total_weight_with(&items) - 10.0).abs() < 1e-4);

        // A second attempt when trunk is completely full returns "The trunk is full"
        let toast_full = apply_trunk_action(&act, &mut car, &mut bag, &items);
        assert_eq!(toast_full, Some("The trunk is full".to_string()));
        assert_eq!(bag.count("wood"), 5);
        assert_eq!(car.trunk.count("wood"), 10);
    }

    #[test]
    fn test_shift_move_single_item() {
        let config = VehicleConfigData::from_repo().expect("vehicles.json");
        let items = ItemRegistry::from_repo().expect("items.json");
        let mut car = Car::new("test_car", "red", &config);
        let mut bag = Inventory::from_registry(items.clone());

        // Place 3 bandages in bag slot 0
        assert_eq!(bag.add_with_registry("bandage", 3, &items), 0);

        // Shift+click moves 1 item
        let act = HubAction::TrunkMove { from_trunk: false, slot: 0, count: 1 };
        let toast = apply_trunk_action(&act, &mut car, &mut bag, &items);
        assert_eq!(toast, Some("Put 1 Bandage in the trunk".to_string()));
        assert_eq!(bag.count("bandage"), 2);
        assert_eq!(car.trunk.count("bandage"), 1);

        // Move 1 item back from trunk slot 0
        let act_back = HubAction::TrunkMove { from_trunk: true, slot: 0, count: 1 };
        let toast_back = apply_trunk_action(&act_back, &mut car, &mut bag, &items);
        assert_eq!(toast_back, Some("Took 1 Bandage out".to_string()));
        assert_eq!(car.trunk.count("bandage"), 0);
        assert_eq!(bag.count("bandage"), 3);
    }

    #[test]
    fn test_prompt_constant() {
        assert_eq!(OPEN_TRUNK_PROMPT, "[E] Open trunk");
        assert_eq!(open_trunk_prompt(), "[E] Open trunk");
    }
}
