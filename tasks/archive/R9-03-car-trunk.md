# R9-03: Car trunk screen

Read `tasks/R9-common.md` first. Reference: `scripts/UI/TrunkScreen.cs`, `scripts/Vehicles/Car.cs` (Trunk, the `trunk` marker), `scripts/Player/PlayerInteractor.cs` (how E at the back of a car opens it).

Rust has the trunk inventory (`game::Car.trunk`), a trunk page in `rust/rotland/src/hub_ui/draw_trunk.rs` and trunk actions in `hub_ui/action.rs`. Read them first. Write `rust/rotland/src/trunk_play.rs`:
- `trunk_in_front(cars: &game::Cars, player: Vec3, facing: Vec3) -> Option<String car id>`: the parked car whose `trunk` marker (pack model markers, see `cargo run -p pack --example list -- car` output: exhaust, exit_L, headlight_L/R, seat_driver, trunk) is within reach behind the car, Godot's range and angle. The car's world matrix comes from `Car::draw_state().world_matrix`; take the marker's local position as a parameter (Claude reads it from the pack once).
- `apply_trunk_action(action: &HubAction, car: &mut game::Car, bag: &mut game::Inventory, items: &ItemRegistry) -> Option<String toast>`: move stacks between bag and trunk (weight/slot limits, what doesn't fit stays), as TrunkScreen.cs.
- The prompt text ("[E] Open trunk") as Godot.
- You may edit `rust/rotland/src/hub_ui/` trunk files if the page needs a fix; say what you changed.
Tests (`#[cfg(test)]` in trunk_play.rs): in front of the trunk finds the car, beside the door does not; moving a stack both ways; an over-weight move leaves the rest in the bag.

## Worker notes

### Files changed / created
- `rust/rotland/src/trunk_play.rs` (new):
  - Constants: `INTERACT_RANGE = 2.5`, `HALF_CONE_DEGREES = 35.0`, `DEFAULT_TRUNK_LOCAL = Vec3::new(0.0, 0.60, 2.20)`, `OPEN_TRUNK_PROMPT = "[E] Open trunk"`.
  - Detection: `trunk_in_front(cars, player, facing, trunk_local) -> Option<String>` and convenience `trunk_in_front_default(cars, player, facing) -> Option<String>`.
  - Interaction: `apply_trunk_action(action, car, bag, items) -> Option<String>`. Moves whole stack (`count == 0`) or partial (`count > 0`), respects weight/slot limits with leftovers staying in the source inventory, and returns exact Godot toasts.
  - Tests: unit tests covering in front vs beside door detection, driven cars, stack moves in both directions, over-weight moves leaving remaining items in bag, and shift-click single-item moves.
- `rust/rotland/src/hub_ui/draw_trunk.rs` (edited):
  - Added blocker check (`trunk.can_add` / `bag.can_add`) to both bag and trunk item buttons.
  - Formats button text with suffix ` ({blocker})` and disables button via `ui.add_enabled(blocker.is_none(), btn)` when the target inventory cannot accept even 1 item, matching Godot `TrunkScreen.cs` / `ShopRow.cs`.

### Public API in `trunk_play.rs`
- `pub const INTERACT_RANGE: f32 = 2.5;`
- `pub const HALF_CONE_DEGREES: f32 = 35.0;`
- `pub const DEFAULT_TRUNK_LOCAL: Vec3 = Vec3::new(0.0, 0.60, 2.20);`
- `pub const OPEN_TRUNK_PROMPT: &str = "[E] Open trunk";`
- `pub fn open_trunk_prompt() -> &'static str`
- `pub fn trunk_in_front(cars: &game::Cars, player: Vec3, facing: Vec3, trunk_local: Vec3) -> Option<String>`
- `pub fn trunk_in_front_default(cars: &game::Cars, player: Vec3, facing: Vec3) -> Option<String>`
- `pub fn apply_trunk_action(action: &crate::hub_ui::HubAction, car: &mut game::Car, bag: &mut game::Inventory, items: &game::ItemRegistry) -> Option<String>`

### Traced test numbers and sources
- `INTERACT_RANGE = 2.5`: `scripts/Player/PlayerInteractor.cs:16` (`Range = 2.5f`).
- `HALF_CONE_DEGREES = 35.0`: `scripts/Player/PlayerInteractor.cs:18` (`HalfConeDegrees = 35f`).
- `DEFAULT_TRUNK_LOCAL = (0.0, 0.60, 2.20)`: `tools/blender/vehicles/car_parts.py:33` empty `("trunk", (0.0, -2.20, 0.60))` in Blender coordinates `(x, y, z)` mapped to Godot/glTF `(x, z, -y) = (0.0, 0.60, 2.20)`.
- In `test_trunk_in_front_detection`:
  - Car spawned at `(0.0, 0.0, 0.0)` with yaw `0.0`. Trunk world position is `(0.0, 0.60, 2.20)`.
  - Player at `(0.0, 0.0, 3.5)` facing -Z `(0.0, 0.0, -1.0)`: `to_target = (0.0, 0.60, -1.30)`. Distance squared is `0.60^2 + (-1.30)^2 = 0.36 + 1.69 = 2.05 <= 2.5^2 (6.25)`. Angle is `0.0` degrees <= 35.0 degrees. Returns `Some("car_camp")`.
  - Player beside door at `(-1.8, 0.0, 0.0)` facing +X `(1.0, 0.0, 0.0)`: `to_target = (1.8, 0.60, 2.20)`. Distance squared is `1.8^2 + 0.6^2 + 2.2^2 = 3.24 + 0.36 + 4.84 = 8.44 > 6.25`. Exceeds 2.5m range. Returns `None`.
  - Player facing away `(0.0, 0.0, 1.0)`: dot product < 0. Returns `None`.
  - Driven car (`car.set_driven(true)`): skipped by `car.is_driven()`. Returns `None`.
- In `test_trunk_move_stack_both_ways`:
  - `scrap_metal`: `weight = 0.8` kg, `maxStack = 20` from `data/items.json:2`.
  - Bag starts with 5 `scrap_metal` (4.0 kg). Move whole stack (`count = 0`) to trunk gives toast `"Put 5 Scrap Metal in the trunk"`. Bag count = 0, trunk count = 5.
  - Move whole stack back gives toast `"Took 5 Scrap Metal out"`. Trunk count = 0, bag count = 5.
- In `test_overweight_move_leaves_rest_in_bag`:
  - `wood`: `weight = 1.0` kg, `maxStack = 20` from `data/items.json:44`.
  - Trunk max weight set to 10.0 kg. Pre-filled with 7 wood (7.0 kg). Remaining room is 3.0 kg.
  - Bag has 8 wood (8.0 kg) in slot 0. Moving whole stack to trunk: only 3 fit (3 * 1.0 = 3.0 kg), 5 stay in bag slot 0.
  - Toast is `"Put 3 Wood in the trunk"`. Bag slot 0 count = 5. Trunk count = 10 (10.0 kg).
  - Attempting another move when trunk is full returns `"The trunk is full"`.
- In `test_shift_move_single_item`:
  - `bandage`: `weight = 0.1` kg from `data/items.json:10`.
  - Shift-click (`count = 1`) moves 1 bandage: toast `"Put 1 Bandage in the trunk"`. Bag has 2 left, trunk has 1.
  - Move 1 back: toast `"Took 1 Bandage out"`. Trunk has 0, bag has 3.

### Wiring for Claude
1. In `rust/rotland/src/main.rs`:
   - Declare module: `mod trunk_play;`
   - In `App` struct, store `open_trunk_car_id: Option<String>` (initialized to `None`).
   - On pack load / startup, read trunk marker translation from `pack`:
     ```rust
     let trunk_local = self.pack.as_ref()
         .and_then(|p| p.find("car_hatchback_red").or_else(|| p.find("car_hatchback_blue")))
         .and_then(|m| m.markers.iter().find(|k| k.name == "trunk"))
         .map(|k| Vec3::from(k.translation))
         .unwrap_or(trunk_play::DEFAULT_TRUNK_LOCAL);
     ```
   - In interact tick (around line 1245):
     ```rust
     if let Some(cars) = self.cars.as_ref()
         && let Some(car_id) = trunk_play::trunk_in_front(cars, p.position, facing, trunk_local)
     {
         play.set_prompt(trunk_play::OPEN_TRUNK_PROMPT.to_string());
         if pressed {
             self.open_trunk_car_id = Some(car_id);
             self.hub = Some(hub_ui::HubUiState::new(hub_ui::HubScreen::Trunk));
         }
     }
     ```
   - In `hub_ui::draw` view construction (around line 1852):
     ```rust
     let trunk_car = self.open_trunk_car_id.as_deref().and_then(|id| self.cars.as_ref()?.get(id));
     ```
     Provide to `hub_ui::HubView`:
     ```rust
     trunk_inventory: trunk_car.map(|c| &c.trunk),
     trunk_max_weight: trunk_car.map(|c| c.trunk.max_weight()).unwrap_or(40.0),
     ```
   - In `hub_ui::HubAction` matching (around line 1900):
     ```rust
     Some(hub_ui::HubAction::TrunkMove { from_trunk, slot, count }) => {
         if let Some(car_id) = &self.open_trunk_car_id
             && let Some(cars) = self.cars.as_mut()
             && let Some(car) = cars.get_mut(car_id)
         {
             let action = hub_ui::HubAction::TrunkMove { from_trunk, slot, count };
             if let Some(toast) = trunk_play::apply_trunk_action(&action, car, &mut play.inventory, &play.items) {
                 state.set_status(toast);
             }
             play.hotbar.on_inventory_changed(&play.inventory);
         }
     }
     ```
   - Auto-close trunk if player moves beyond 3.5m or car is driven (matching `TrunkScreen.cs:72`):
     ```rust
     if let Some(car_id) = &self.open_trunk_car_id {
         let should_close = self.cars.as_ref().and_then(|c| c.get(car_id)).map_or(true, |c| {
             c.is_driven() || c.body.position.distance(self.player) > 3.5
         });
         if should_close {
             self.open_trunk_car_id = None;
             if self.hub.as_ref().is_some_and(|h| h.screen() == hub_ui::HubScreen::Trunk) {
                 self.hub = None;
             }
         }
     }
     ```
