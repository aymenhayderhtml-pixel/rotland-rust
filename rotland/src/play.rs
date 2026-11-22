//! The play-mode systems that sit around the player: the backpack and hotbar (what is in hand), the
//! zombie spawner, and the HUD numbers. Kept out of main.rs so that file stays small.

use std::sync::Arc;

use engine::Input;
use game::director::{Director, DirectorConfig, DirectorInput, HordeEvent};
use game::items::{start_inventory, Hotbar, HotbarAction, Inventory, ItemRegistry};
use game::spawner::{SpawnContext, Spawner};
use game::spawner_config::SpawnConfig;
use game::{WeaponDef, WeaponRegistry};
use glam::{Vec2, Vec3};
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


pub struct PlayState {
    pub items: Arc<ItemRegistry>,
    pub weapons: WeaponRegistry,
    pub inventory: Inventory,
    pub hotbar: Hotbar,
    spawner: Spawner,
    loot_tables: LootTableRegistry,
    loot_state: LootState,
    pub world_items: WorldItems,
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
    /// Something was picked up or a crate opened this frame (pickup sound).
    pub picked_up: bool,
    /// What the player has explored (Godot WorldMap, saved under "map").
    pub map: game::MapState,
    /// Lifetime numbers for the profile page (Godot PlayerStats, save id "stats").
    pub stats: game::ProfileStats,
    /// The tips for new players (Godot HintSystem, saved under "hints"); None if data/hints.json is broken.
    pub hints: Option<game::HintSystem>,
    /// Items collected this frame for HUD pickup popups.
    pub pending_pickups: Vec<(String, u32)>,
    /// AI director pacing pressure and wave hordes.
    pub director: Director,
    /// Player health fraction (0.0..1.0) for director input.
    pub player_health: f32,
    /// Cumulative kills count seen from zombies.
    pub last_total_kills: u64,
    /// Events emitted for HUD and camp defense.
    pub pending_horde_events: Vec<HordeEvent>,
    horde_test_spawned: bool,
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
    /// The campfire kit was used: place a fire in front (needs the ground, so main.rs does it).
    PlaceCampfire,
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
        let director_cfg = DirectorConfig::load_default()
            .unwrap_or_else(|e| {
                log::warn!("data/director.json: {e}; using default");
                DirectorConfig::default()
            });
        let director = Director::new(director_cfg);
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
            picked_up: false,
            map: game::MapState::new(),
            stats: game::ProfileStats::default(),
            hints: game::HintSystem::from_repo().map_err(|e| log::warn!("data/hints.json: {e}")).ok(),
            pending_pickups: Vec::new(),
            director,
            player_health: 1.0,
            last_total_kills: 0,
            pending_horde_events: Vec::new(),
            horde_test_spawned: false,
        })
    }

    /// The interaction hint at the bottom of the screen ("[E] Trade").
    pub fn set_prompt(&mut self, text: String) {
        self.prompt = Some(text);
    }

    pub fn has_prompt(&self) -> bool {
        self.prompt.as_ref().is_some_and(|p| !p.is_empty())
    }

    /// A message in the middle of the screen for a few seconds.
    /// Notes the POI buildings within reveal range on the map, dimmed once all their crates are looted.
    pub fn note_buildings<'a>(
        &mut self,
        player: Vec3,
        chunks: impl Iterator<Item = (&'a [world::PoiInstance], &'a [world::LootSpot])>,
        config: &world::PoiConfigData,
    ) {
        let looted = self.loot_state.all_looted();
        for (pois, crates) in chunks {
            crate::poi_links::note_nearby_pois(&mut self.map, player, pois, crates, config, |id| looted.contains(id));
        }
    }

    /// Puts items on the ground as a world item (felled wood).
    pub fn drop_on_ground(&mut self, item_id: &str, count: u32, at: Vec3) {
        self.world_items.add(item_id, count, at);
    }

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

    /// A button on the bag screen (hub_ui). Returns a hand change and a line for the screen's status.
    /// Dropped items land at `feet` and can be picked up again with E.
    pub fn bag_action(&mut self, action: &crate::hub_ui::HubAction, split: u32, feet: Vec3) -> (Option<HandChange>, Option<String>) {
        use crate::hub_ui::HubAction;
        let name = |s: &Self, slot: usize| s.inventory.slot(slot).map(|st| s.items.get(&st.item_id).map_or(st.item_id.clone(), |d| d.name.clone()));
        let out = match action {
            HubAction::MoveSlot { from, to } => {
                self.inventory.swap_slots(*from, *to);
                (None, None)
            }
            HubAction::AssignHotbar { slot, key } if (1..=self.hotbar.slot_count()).contains(key) => {
                self.inventory.swap_slots(*slot, key - 1);
                (None, Some(format!("On key {key}")))
            }
            HubAction::UseItem { slot } => {
                let Some(id) = self.inventory.slot(*slot).map(|s| s.item_id.clone()) else { return (None, None) };
                let label = name(self, *slot);
                if id == "flashlight" {
                    (Some(HandChange::ToggleFlashlight), None)
                } else if id == "campfire_kit" {
                    (Some(HandChange::PlaceCampfire), None)
                } else {
                    let before = self.inventory.count(&id);
                    let change = self.use_item(&id);
                    let used = self.inventory.count(&id) < before;
                    (change, Some(if used { format!("Used {}", label.unwrap_or(id)) } else { "Can't use that now".into() }))
                }
            }
            HubAction::EquipItem { slot } => {
                let Some(id) = self.inventory.slot(*slot).map(|s| s.item_id.clone()) else { return (None, None) };
                if self.hotbar.toggle_equip_item(&id, &self.inventory, &self.items) {
                    let weapon = self.hotbar.equipped_item_id().and_then(|w| self.weapons.get(w)).cloned();
                    (Some(HandChange::Weapon(weapon)), None)
                } else {
                    (None, Some("Not a weapon".into()))
                }
            }
            HubAction::DropItem { slot, count } => {
                let Some(stack) = self.inventory.slot(*slot).cloned() else { return (None, None) };
                let n = if *count == 0 { stack.count } else { (*count).min(stack.count) };
                let label = name(self, *slot).unwrap_or_default();
                let removed = self.inventory.remove_from_slot(*slot, n);
                if removed > 0 {
                    self.world_items.add(stack.item_id, removed, feet);
                }
                (None, Some(format!("Dropped {removed} {label}")))
            }
            HubAction::SplitStack { slot } => {
                let Some(stack) = self.inventory.slot(*slot).cloned() else { return (None, None) };
                let n = split.clamp(1, stack.count.saturating_sub(1).max(1));
                let free = self.inventory.slots().iter().position(Option::is_none);
                match free {
                    Some(free) if stack.count > 1 => {
                        self.inventory.remove_from_slot(*slot, n);
                        self.inventory.slots_mut()[free] = Some(game::items::ItemStack { item_id: stack.item_id, count: n });
                        self.inventory.set_changed();
                        (None, Some(format!("Split off {n}")))
                    }
                    _ => (None, Some("No free slot".into())),
                }
            }
            _ => (None, None),
        };
        self.hotbar.on_inventory_changed(&self.inventory);
        out
    }

    /// Hunger and thirst drain; returns (damage from an empty need, healing from being well fed).
    pub fn needs_tick(&mut self, dt: f32, sprinting: bool, dead: bool, health: (f32, f32), stamina: &mut game::Stamina) -> (f32, f32) {
        if health.1 > 0.0 {
            self.player_health = (health.0 / health.1).clamp(0.0, 1.0);
        }
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
    pub fn hotbar_keys(&mut self, input: &Input, keys: &[KeyCode; 6]) -> Option<HandChange> {
        let key = keys.iter().position(|k| input.pressed(*k))? + 1;
        self.press(key)
    }

    pub fn press(&mut self, key: usize) -> Option<HandChange> {
        match self.hotbar.press(key, &self.inventory, &self.items) {
            HotbarAction::EquipWeapon { weapon_id, .. } => Some(HandChange::Weapon(self.weapons.get(&weapon_id).cloned())),
            HotbarAction::UnequipWeapon { .. } => Some(HandChange::Weapon(None)),
            // Godot's ItemUse: using the flashlight item switches it on and off
            HotbarAction::UseItem { item_id } if item_id == "flashlight" => Some(HandChange::ToggleFlashlight),
            HotbarAction::UseItem { item_id } if item_id == "campfire_kit" => Some(HandChange::PlaceCampfire),
            HotbarAction::UseItem { item_id } => self.use_item(&item_id),
            _ => None,
        }
    }

    /// Zombies appear and leave around the player by Godot's spawner rules and director waves.
    #[allow(clippy::too_many_arguments)]
    pub fn spawn_tick(
        &mut self,
        dt: f32,
        player: Vec3,
        forward: Vec3,
        night: bool,
        zombies: &mut Zombies,
        physics: &PhysicsWorld,
        world_gen: &WorldGenerator,
    ) -> Vec<HordeEvent> {
        if !self.horde_test_spawned && std::env::args().any(|a| a == "--horde-test") {
            self.horde_test_spawned = true;
            self.spawn_test_horde(player, forward, zombies, physics);
        }

        let alive = zombies.active();
        let p = Vec2::new(player.x, player.z);
        let in_city = world_gen.city().is_some_and(|c| c.contains(p));
        let in_city_core = world_gen.city().is_some_and(|c| c.district_at(p) == Some(world::District::Core));
        let player_in_camp = world_gen.settlements().first().is_some_and(|s| s.contains(player, 0.0, world_gen.config()));
        let zombies_near = alive.iter().filter(|z| (z.position - player).length_squared() < 900.0).count();
        let kills_since = (zombies.total_kills - self.last_total_kills) as u32;
        self.last_total_kills = zombies.total_kills;

        let dir_input = DirectorInput {
            dt,
            is_night: night,
            player_health: self.player_health,
            recent_kills: kills_since,
            zombies_near,
            in_city,
            player_in_camp,
        };
        let prev_dir_state = self.director.state();
        let dir_out = self.director.update(&dir_input);
        let cur_dir_state = self.director.state();

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
            in_city,
            in_city_core,
            spawn_rate_multiplier: dir_out.spawn_rate_multiplier,
            max_active_multiplier: dir_out.max_active_multiplier,
            special_chance: dir_out.special_chance,
        };
        let orders = self.spawner.update(dt, &ctx);
        zombies.despawn(&orders.despawns);
        for order in orders.spawns {
            zombies.spawn_kind(&order.type_id, order.position);
        }

        for horde in zombies.hordes.hordes_mut() {
            if horde.target_pos.is_some() {
                let near_camp = world_gen.settlements().iter().any(|s| {
                    horde.target_pos.is_some_and(|t| (t - s.center).length_squared() < 900.0)
                });
                if !near_camp {
                    horde.set_target_pos(Some(player));
                }
            }
        }

        let mut frame_events = Vec::new();
        if prev_dir_state != cur_dir_state {
            let phase_evt = HordeEvent::PhaseChange { from: prev_dir_state, to: cur_dir_state };
            frame_events.push(phase_evt);
            self.pending_horde_events.push(phase_evt);
        }
        for event in dir_out.events {
            let (event_size, is_camp_attack, hud_event) = match event {
                game::director::DirectorEvent::WaveStart { size, .. } => (size, false, HordeEvent::HordeIncoming { size }),
                game::director::DirectorEvent::CampAttack { size } => (size, true, HordeEvent::CampAttack { size }),
            };
            frame_events.push(hud_event);
            self.pending_horde_events.push(hud_event);

            let cap = self.spawner.calculate_cap(&ctx);
            let current_alive = zombies.alive();
            let available_slots = cap.saturating_sub(current_alive);
            let count = event_size.min(available_slots);

            if count > 0 {
                let target = if is_camp_attack {
                    let camp_gate = world_gen.settlements().first().map(|site| {
                        let half = world_gen.config().safe_zone_half;
                        site.center + site.local(0.0, 0.0, -half)
                    }).unwrap_or(player);
                    Some(camp_gate)
                } else {
                    Some(player)
                };

                let angle_mag = self.spawner.rng.range(self.spawner.config.out_of_view_angle_deg, 180.0);
                let sign = if self.spawner.rng.next_bool() { 1.0 } else { -1.0 };
                let angle = angle_mag * sign;
                let dist = self.spawner.rng.range(70.0, 90.0);
                let mut flat_fwd = Vec3::new(forward.x, 0.0, forward.z);
                if flat_fwd.length_squared() < 1e-4 { flat_fwd = Vec3::NEG_Z; }
                let rot = glam::Quat::from_rotation_y(angle.to_radians());
                let spawn_center = player + (rot * flat_fwd.normalize()) * dist;

                let mut spawned_ids = Vec::with_capacity(count);
                for i in 0..count {
                    let offset_angle = i as f32 * 2.4;
                    let offset_dist = (i as f32).sqrt() * 1.5;
                    let pt = spawn_center + Vec3::new(offset_angle.cos() * offset_dist, 0.0, offset_angle.sin() * offset_dist);
                    let ground_pt = physics.ground_height(pt.x, pt.z).map(|y| Vec3::new(pt.x, y + 0.1, pt.z)).unwrap_or(pt);

                    let kind = if (night || in_city)
                        && dir_out.special_chance > 0.0
                        && self.spawner.rng.range(0.0, 1.0) < dir_out.special_chance
                    {
                        let specials = ["brute", "screamer", "crawler"];
                        let idx = (self.spawner.rng.next_f32() * specials.len() as f32).min(specials.len() as f32 - 1.0) as usize;
                        specials[idx]
                    } else {
                        self.spawner.config.pick_type_id_core(&mut self.spawner.rng, night, in_city_core)
                    };

                    if let Some(id) = zombies.spawn_kind(kind, ground_pt) {
                        spawned_ids.push(id);
                    }
                }

                if !spawned_ids.is_empty() {
                    let leader_pos = spawn_center;
                    if let Some(h_idx) = zombies.hordes.create_from_wave(&event, &spawned_ids, leader_pos)
                        && let Some(h) = zombies.hordes.hordes_mut().get_mut(h_idx)
                    {
                        h.set_target_pos(target);
                    }
                    for &id in &spawned_ids {
                        if let Some(z) = zombies.list.iter_mut().find(|z| z.id == id)
                            && let Some(t) = target
                        {
                            z.brain.alert_chase(t);
                        }
                    }
                }
            }
        }

        frame_events
    }

    /// Spawns a test horde of 12 zombies 60m away for testing/benchmarking.
    pub fn spawn_test_horde(&mut self, player: Vec3, forward: Vec3, zombies: &mut Zombies, physics: &PhysicsWorld) {
        let count = 12;
        let dist = 60.0;
        let mut flat_fwd = Vec3::new(forward.x, 0.0, forward.z);
        if flat_fwd.length_squared() < 1e-4 { flat_fwd = Vec3::NEG_Z; }
        let rot = glam::Quat::from_rotation_y(std::f32::consts::PI);
        let center = player + (rot * flat_fwd.normalize()) * dist;
        let mut spawned = Vec::with_capacity(count);
        for i in 0..count {
            let offset_angle = i as f32 * 2.4;
            let offset_dist = (i as f32).sqrt() * 1.5;
            let pt = center + Vec3::new(offset_angle.cos() * offset_dist, 0.0, offset_angle.sin() * offset_dist);
            let ground_pt = physics.ground_height(pt.x, pt.z).map(|y| Vec3::new(pt.x, y + 0.1, pt.z)).unwrap_or(pt);
            if let Some(id) = zombies.spawn_kind("walker", ground_pt) {
                spawned.push(id);
            }
        }
        if !spawned.is_empty() {
            let leader_id = spawned[0];
            let members = spawned[1..].to_vec();
            let mut horde = game::horde::Horde::new(leader_id, members);
            horde.set_leader_pos(center);
            horde.set_target_pos(Some(player));
            zombies.hordes.hordes.push(horde);
            for &id in &spawned {
                if let Some(z) = zombies.list.iter_mut().find(|z| z.id == id) {
                    z.brain.alert_chase(player);
                }
            }
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
        self.picked_up = false;
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
                InteractResult::OpenedCrate { toast, cash, items, leftovers, .. } => {
                    // Crate cash goes to the wallet; opening one counts for loot bounties
                    self.camp.wallet.add(cash);
                    self.camp.bounties.on_crate_looted();
                    self.picked_up = true;
                    for stack in items {
                        let leftover = leftovers.iter().find(|l| l.item_id == stack.item_id).map_or(0, |l| l.count);
                        let added = stack.count.saturating_sub(leftover);
                        if added > 0 {
                            self.pending_pickups.push((stack.item_id, added));
                        }
                    }
                    Some(toast)
                }
                InteractResult::PickedUpItem { toast, item_id, count, .. } => {
                    self.picked_up = true;
                    if count > 0 {
                        self.pending_pickups.push((item_id, count));
                    }
                    toast
                }
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
    pub fn shoot_tick(&mut self, dt: f32, input: ShootInput, eye: Vec3, aim: Vec3, targets: &mut impl crate::shooting::TargetWorld, physics: &PhysicsWorld) -> Option<ShootEvents> {
        let gun = self.gun.clone()?;
        let ev = self.shooting.update_targets(dt, input, &gun, eye, aim, &mut self.inventory, targets, physics);
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
        let hotbar_item_ids = (1..=self.hotbar.slot_count())
            .map(|key| self.hotbar.slot_entry(key, &self.inventory).map(|st| st.item_id.clone()))
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
            hotbar_item_ids: Some(hotbar_item_ids),
            ..Default::default()
        }
    }
}
