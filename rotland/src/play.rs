//! The play-mode systems that sit around the player: the backpack and hotbar (what is in hand), the
//! zombie spawner, and the HUD numbers. Kept out of main.rs so that file stays small.

use std::sync::Arc;

use engine::Input;
use game::items::{start_inventory, Hotbar, HotbarAction, Inventory, ItemRegistry};
use game::spawner::{SpawnContext, Spawner};
use game::spawner_config::SpawnConfig;
use game::{WeaponDef, WeaponRegistry};
use glam::Vec3;
use physics::PhysicsWorld;
use winit::keyboard::KeyCode;
use world::WorldGenerator;

use crate::crates::{interact, interact_prompt, InteractResult};
use crate::hud::{HotbarSlot, HudState};
use crate::shooting::{ShootEvents, ShootInput, Shooting};
use crate::zombies::Zombies;
use game::{LootState, LootTableRegistry, WorldItems};
use world::LootSpot;

/// How long a pickup message stays on screen (Godot's toast).
const TOAST_SECONDS: f32 = 3.0;
/// Seconds between two uses of food, drink or bandages (ItemUse.cs UseCooldown).
const USE_COOLDOWN: f32 = 0.8;

const HOTBAR_KEYS: [KeyCode; 6] = [KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4, KeyCode::Digit5, KeyCode::Digit6];

pub struct PlayState {
    pub items: Arc<ItemRegistry>,
    pub weapons: WeaponRegistry,
    pub inventory: Inventory,
    pub hotbar: Hotbar,
    spawner: Spawner,
    loot_tables: LootTableRegistry,
    loot_state: LootState,
    world_items: WorldItems,
    loot_multiplier: f32,
    world_seed: i32,
    shooting: Shooting,
    /// The gun in hand, if any (melee weapons go to combat.rs instead).
    pub gun: Option<WeaponDef>,
    /// The last shooting frame (ammo for the HUD).
    pub last_shot: ShootEvents,
    toast: Option<(String, f32)>,
    prompt: Option<String>,
    pub hunger: game::NeedStat,
    pub thirst: game::NeedStat,
    pub survival: game::SurvivalData,
    use_cooldown: f32,
    /// Health is full this frame (bandages are refused then, like Godot).
    pub health_full: bool,
    /// Wallet, bank, trader, skills, bounties and difficulty.
    pub camp: crate::camp::Camp,
}

/// What changed in hand this frame.
#[allow(clippy::large_enum_variant)] // made a few times a minute at most; boxing would only add noise
pub enum HandChange {
    /// A weapon was taken out (its definition) or put away (None).
    Weapon(Option<WeaponDef>),
    /// The flashlight was switched on or off.
    ToggleFlashlight,
    /// Food, drink or a bandage changed health (heal if above 0, sickness if below).
    Health(f32),
}

impl PlayState {
    pub fn new(seed: u64) -> Result<Self, String> {
        let items = ItemRegistry::from_repo().map_err(|e| format!("data/items.json: {e:?}"))?;
        let weapons = WeaponRegistry::load(&paths::repo_path("data/weapons.json")).map_err(|e| format!("data/weapons.json: {e:?}"))?;
        let config = SpawnConfig::load_default().map_err(|e| format!("data/spawning.json: {e}"))?;
        let loot_tables = LootTableRegistry::from_repo().map_err(|e| format!("data/loot_tables.json: {e:?}"))?;
        let loot_multiplier = game::DifficultyConfigData::from_repo().map(|d| d.default_loot_multiplier()).unwrap_or(1.0);
        let survival = game::SurvivalData::from_repo().map_err(|e| format!("data/survival.json: {e}"))?;
        let hunger = game::NeedStat::from_config("hunger", &survival).ok_or("survival.json has no hunger")?;
        let thirst = game::NeedStat::from_config("thirst", &survival).ok_or("survival.json has no thirst")?;
        let mut inventory = start_inventory(&items);
        let items = Arc::new(items);
        let camp = crate::camp::Camp::new(seed as i32, items.clone())?;
        inventory.set_registry(items.clone());
        Ok(Self {
            items,
            weapons,
            inventory,
            hotbar: Hotbar::new(),
            spawner: Spawner::new(config, seed),
            loot_tables,
            loot_state: LootState::default(),
            world_items: WorldItems::default(),
            loot_multiplier,
            world_seed: seed as i32,
            shooting: Shooting::new(seed ^ 0x5157),
            gun: None,
            last_shot: ShootEvents::default(),
            toast: None,
            prompt: None,
            hunger,
            thirst,
            survival,
            use_cooldown: 0.0,
            health_full: true,
            camp,
        })
    }

    /// A message in the middle of the screen for a few seconds.
    pub fn show_toast(&mut self, text: String) {
        self.toast = Some((text, TOAST_SECONDS));
    }

    /// Takes out the first hotbar item at the start of a game, like Godot's starting bat.
    /// The backpack's stacks and the looted crates, for a save file.
    pub fn save_parts(&self) -> (game::items::InventorySaveData, Vec<String>) {
        let mut looted: Vec<String> = self.loot_state.all_looted().iter().cloned().collect();
        looted.sort();
        (game::items::InventorySaveData::write_from_slots(self.inventory.slots()), looted)
    }

    /// Puts a save file's backpack and looted crates back.
    pub fn load_parts(&mut self, stacks: &game::items::InventorySaveData, looted: &[String]) {
        stacks.read_into_inventory(&mut self.inventory);
        self.hotbar.on_inventory_changed(&self.inventory);
        self.loot_state.clear();
        for id in looted {
            self.loot_state.mark_looted(id.clone());
        }
    }

    /// Eats, drinks or uses a bandage: one item is used up (Godot ItemUse.TryUse).
    fn use_item(&mut self, item_id: &str) -> Option<HandChange> {
        let def = self.items.get(item_id)?.clone();
        use game::items::ItemCategory;
        if self.use_cooldown > 0.0 || !matches!(def.category, ItemCategory::Food | ItemCategory::Drink | ItemCategory::Medical) {
            return None;
        }
        if def.category == ItemCategory::Medical && self.health_full {
            return None;
        }
        if self.inventory.count(item_id) == 0 {
            return None;
        }
        // Cooking skill bonus comes with the skills wiring; 1.0 until then
        let health = game::apply_item_effects(&def.effects, &mut self.hunger, &mut self.thirst, 1.0);
        self.inventory.remove(item_id, 1);
        self.hotbar.on_inventory_changed(&self.inventory);
        self.use_cooldown = USE_COOLDOWN;
        (health != 0.0).then_some(HandChange::Health(health))
    }

    /// Hunger and thirst drain; returns (damage from an empty need, healing from being well fed).
    pub fn needs_tick(&mut self, dt: f32, sprinting: bool, dead: bool, health: (f32, f32), stamina: &mut game::Stamina) -> (f32, f32) {
        self.use_cooldown = (self.use_cooldown - dt).max(0.0);
        let damage = self.hunger.update(dt, sprinting, dead) + self.thirst.update(dt, sprinting, dead);
        let heal = game::WellFedRegen::update(&self.hunger, &self.thirst, health.0, health.1, dead, &self.survival.well_fed, dt);
        game::update_stamina_regen_multiplier(stamina, &self.hunger, &self.thirst, self.survival.stamina.low_needs_regen_multiplier);
        (damage, heal)
    }

    /// After a respawn the needs start at Godot's respawn value.
    pub fn needs_respawn(&mut self) {
        let v = self.survival.respawn_needs_value;
        self.hunger.set(v);
        self.thirst.set(v);
    }

    pub fn equip_first(&mut self) -> Option<HandChange> {
        self.press(1)
    }

    /// Hotbar keys 1-6: take out or put away the item in that slot.
    pub fn hotbar_keys(&mut self, input: &Input) -> Option<HandChange> {
        let key = HOTBAR_KEYS.iter().position(|k| input.pressed(*k))? + 1;
        self.press(key)
    }

    pub fn press(&mut self, key: usize) -> Option<HandChange> {
        match self.hotbar.press(key, &self.inventory, &self.items) {
            HotbarAction::EquipWeapon { weapon_id, .. } => Some(HandChange::Weapon(self.weapons.get(&weapon_id).cloned())),
            HotbarAction::UnequipWeapon { .. } => Some(HandChange::Weapon(None)),
            // Godot's ItemUse: using the flashlight item switches it on and off
            HotbarAction::UseItem { item_id } if item_id == "flashlight" => Some(HandChange::ToggleFlashlight),
            HotbarAction::UseItem { item_id } => self.use_item(&item_id),
            _ => None,
        }
    }

    /// Zombies appear and leave around the player by Godot's spawner rules.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_tick(&mut self, dt: f32, player: Vec3, forward: Vec3, night: bool, zombies: &mut Zombies, physics: &PhysicsWorld, world_gen: &WorldGenerator) {
        let alive = zombies.active();
        let ctx = SpawnContext {
            player_position: player,
            player_forward: forward,
            is_night: night,
            alive_zombies: &alive,
            settlements: world_gen.settlements(),
            settlement_config: Some(world_gen.config()),
            physics,
            ground_check: None,
            zombie_amount: 1.0,
            biome_name: Some(if world_gen.forest_weight(player.x, player.z) > 0.5 { "forest" } else { "plains" }),
        };
        let orders = self.spawner.update(dt, &ctx);
        zombies.despawn(&orders.despawns);
        for order in orders.spawns {
            zombies.spawn_kind(&order.type_id, order.position);
        }
    }

    /// E: open the crate or pick up the item in front of the player; also refreshes the HUD prompt.
    pub fn interact_tick(&mut self, dt: f32, pressed: bool, player: Vec3, facing: Vec3, crates: &[LootSpot]) {
        if let Some((_, t)) = self.toast.as_mut() {
            *t -= dt;
        }
        if self.toast.as_ref().is_some_and(|(_, t)| *t <= 0.0) {
            self.toast = None;
        }
        if pressed {
            let result = interact(
                player,
                facing,
                &mut self.inventory,
                &mut self.loot_state,
                crates,
                &mut self.world_items,
                self.world_seed,
                &self.loot_tables,
                &self.items,
                self.loot_multiplier,
            );
            let text = match result {
                InteractResult::OpenedCrate { toast, cash, .. } => {
                    // Crate cash goes to the wallet; opening one counts for loot bounties
                    self.camp.wallet.add(cash);
                    self.camp.bounties.on_crate_looted();
                    Some(toast)
                }
                InteractResult::PickedUpItem { toast, .. } => toast,
                InteractResult::None => None,
            };
            if let Some(t) = text {
                self.toast = Some((t, TOAST_SECONDS));
            }
        }
        self.prompt = interact_prompt(player, facing, crates, &self.world_items, &self.loot_state, Some(&self.items));
    }

    /// The gun in hand fires, reloads and aims (shooting.rs). Returns this frame's events.
    #[allow(clippy::too_many_arguments)]
    pub fn shoot_tick(&mut self, dt: f32, input: ShootInput, eye: Vec3, aim: Vec3, zombies: &mut Zombies, physics: &PhysicsWorld) -> Option<ShootEvents> {
        let gun = self.gun.clone()?;
        let ev = self.shooting.update(dt, input, &gun, eye, aim, &mut self.inventory, zombies, physics);
        self.last_shot = ev.clone();
        Some(ev)
    }

    /// The HUD's numbers for this frame.
    #[allow(clippy::too_many_arguments)]
    pub fn hud(&self, health: f32, max_health: f32, stamina: f32, max_stamina: f32, hour: f32, yaw: f32, fps: f32, dead: bool) -> HudState {
        let selected = self.hotbar.selected_item();
        let hotbar = (1..=self.hotbar.slot_count())
            .map(|key| match self.hotbar.slot_entry(key, &self.inventory) {
                Some(stack) => HotbarSlot {
                    label: self.items.get(&stack.item_id).map_or(stack.item_id.clone(), |d| d.hotbar_label().to_string()),
                    count: stack.count,
                    selected: selected == Some(stack.item_id.as_str()),
                },
                None => HotbarSlot { label: String::new(), count: 0, selected: false },
            })
            .collect();
        HudState {
            health,
            max_health,
            stamina,
            max_stamina,
            hour,
            day: 1,
            heading_yaw: yaw,
            hotbar,
            ammo: self.gun.as_ref().map(|_| self.last_shot.ammo),
            fps,
            food: Some((self.hunger.current() / self.hunger.max().max(1.0), self.hunger.is_low())),
            water: Some((self.thirst.current() / self.thirst.max().max(1.0), self.thirst.is_low())),
            message: self.toast.as_ref().map(|(t, _)| t.clone()).or_else(|| self.prompt.clone()),
            dead,
        }
    }
}
