//! The camp's systems around the player: wallet, bank, trader stock, skills, bounties and the world's
//! difficulty. Kept together so play.rs and the save code each touch one thing.

use std::sync::Arc;

use glam::Vec3;

/// How far in front of the player a bed can be used.
const BED_REACH: f32 = 2.5;

use crate::camp_ui::CampAction;
use game::camp_upgrades::{CampEffects, CampUpgradeState, CampUpgradesConfig, UpgradeLine};
use game::{FarmManager, FarmRules, Bank, BountyManager, CancelReason, CraftUpdate, CraftingSystem, RecipeRegistry, DeathRule, DifficultyConfigData, DifficultySettings, EconomyConfigData, Inventory, ItemRegistry, SkillSystem, TraderStock, Wallet};

#[path = "camp_upgrade_view.rs"]
pub mod camp_upgrade_view;
pub use camp_upgrade_view::CampUpgradeView;

pub struct Camp {
    pub economy: EconomyConfigData,
    pub wallet: Wallet,
    pub bank: Bank,
    pub trader: TraderStock,
    pub skills: SkillSystem,
    pub bounties: BountyManager,
    pub difficulty_config: DifficultyConfigData,
    pub difficulty: DifficultySettings,
    pub recipes: RecipeRegistry,
    pub crafting: CraftingSystem,
    /// Beds in the camp and the player's fields.
    pub farm: FarmManager,
    /// The camp's fire and the ones the player placed (Godot WorldState "campfires").
    pub campfires: game::CampfireManager,
    /// Settlements visited and respawn camp tracking (Godot: SettlementWatcher).
    pub settlements: game::SettlementWatcher,
    /// Camp upgrade progression (walls, guards, workshop, traders).
    pub upgrades: CampUpgradeState,
    pub upgrades_config: CampUpgradesConfig,
    /// Visual and collider representation of camp upgrades in the 3D world.
    pub upgrade_view: CampUpgradeView,
    /// Where the player stands (set each frame): campfire recipes need a lit fire near.
    pub player_at: Vec3,
}

impl Camp {
    pub fn new(seed: i32, items: Arc<ItemRegistry>) -> Result<Self, String> {
        let economy = EconomyConfigData::from_repo().map_err(|e| format!("data/economy.json: {e:?}"))?;
        let difficulty_config = DifficultyConfigData::from_repo().map_err(|e| format!("data/difficulty.json: {e}"))?;
        let upgrades_config = CampUpgradesConfig::load_default().map_err(|e| format!("data/camp_upgrades.json: {e}"))?;
        let mut upgrades = CampUpgradeState::default();
        if let Some(levels) = camp_upgrade_view::override_levels() {
            upgrades.walls_level = levels.walls;
            upgrades.guards_level = levels.guards;
            upgrades.workshop_level = levels.workshop;
            upgrades.traders_level = levels.traders;
        }
        Ok(Self {
            wallet: Wallet::new(economy.start_dollars),
            bank: Bank::with_slots_and_registry(economy.bank_slots, items),
            trader: TraderStock::new(&economy),
            skills: SkillSystem::from_repo().map_err(|e| format!("data/skills.json: {e}"))?,
            bounties: BountyManager::from_repo(seed).map_err(|e| format!("data/bounties.json: {e:?}"))?,
            difficulty: DifficultySettings::default_preset(&difficulty_config),
            recipes: RecipeRegistry::from_repo().map_err(|e| format!("data/recipes.json: {e:?}"))?,
            crafting: CraftingSystem::new(),
            farm: FarmManager::from_repo().map_err(|e| format!("data/farming.json: {e:?}"))?,
            campfires: game::CampfireManager::new(),
            settlements: game::SettlementWatcher::new(),
            upgrades,
            upgrades_config,
            upgrade_view: CampUpgradeView::default(),
            player_at: Vec3::ZERO,
            difficulty_config,
            economy,
        })
    }

    /// Once a frame: the trader restocks and the bounty board refreshes on new days.
    pub fn tick(&mut self, day: i32, hour: f32) {
        self.trader.tick_with_config(day, hour as u32, &self.economy);
        self.bounties.on_hour_changed(day, hour);
    }

    /// A zombie of this type died by the player's hand.
    pub fn on_zombie_killed(&mut self, kind: &str, night: bool) {
        self.bounties.on_zombie_killed_night(kind, night);
    }

    /// The death penalty (Godot DeathPenalty): the wallet empties and the backpack follows the
    /// difficulty's death rule. The bank keeps everything.
    pub fn on_death(&mut self, bag: &mut Inventory, seed: u64) {
        self.wallet.take_all();
        if self.difficulty.death != DeathRule::KeepAll {
            self.difficulty.death.apply_to_inventory_with_seed(bag, seed);
        }
    }

    /// Applies a button press from a camp screen. Returns a line for the screen's status bar.
    pub fn apply(&mut self, action: CampAction, bag: &mut Inventory, items: &ItemRegistry, dead: bool) -> Option<String> {
        match action {
            CampAction::Buy { item, count } => {
                let got = game::buy(&mut self.wallet, bag, &mut self.trader, &item, count, self.economy.buy_rate, items);
                Some(if got > 0 { format!("Bought {got} {}", label(items, &item)) } else { "Can't buy that".into() })
            }
            CampAction::Sell { item, count } => {
                let sold = game::sell(&mut self.wallet, bag, &item, count, self.economy.sell_rate, items);
                Some(if sold > 0 { format!("Sold {sold} {}", label(items, &item)) } else { "Can't sell that".into() })
            }
            CampAction::Deposit { item, count } => {
                let slot = bag.slots().iter().position(|s| s.as_ref().is_some_and(|s| s.item_id == item))?;
                let moved = self.bank.move_to_bank(bag, slot, count);
                Some(format!("Stored {moved} {}", label(items, &item)))
            }
            CampAction::Withdraw { item, count } => {
                let slot = self.bank.items().slots().iter().position(|s| s.as_ref().is_some_and(|s| s.item_id == item))?;
                let moved = self.bank.move_to_bag(bag, slot, count);
                Some(if moved > 0 { format!("Took {moved} {}", label(items, &item)) } else { "No room in the bag".into() })
            }
            CampAction::DepositDollars { amount } => Some(format!("Deposited ${}", self.bank.deposit(&mut self.wallet, amount))),
            CampAction::WithdrawDollars { amount } => Some(format!("Withdrew ${}", self.bank.withdraw(&mut self.wallet, amount))),
            CampAction::Craft { recipe } => {
                let station = self.recipes.get(&recipe).map(|r| r.station.clone()).unwrap_or_default();
                let near = self.campfires.near_campfire(self.player_at);
                let ok = self.crafting.start_craft(&recipe, &station, bag, near, dead, &self.recipes);
                Some(if ok { "Crafting...".into() } else { "Missing materials".into() })
            }
            CampAction::CancelCraft => self.crafting.cancel_craft(bag, CancelReason::Manual).map(|_| "Crafting cancelled".into()),
            CampAction::Accept { bounty } => Some(match self.bounties.accept(&bounty) {
                Ok(_) => "Bounty accepted".into(),
                Err(e) => format!("Can't accept: {e:?}"),
            }),
            CampAction::TurnIn { bounty } => Some(match self.bounties.claim(&bounty) {
                Ok(r) => {
                    self.wallet.add(r.dollars);
                    if r.xp > 0 && !r.skill.is_empty() {
                        self.skills.add_xp(&r.skill, r.xp as f32);
                    }
                    format!("Reward: ${}", r.dollars)
                }
                Err(e) => format!("Not done yet: {e:?}"),
            }),
            CampAction::Abandon { bounty } => self.bounties.abandon(&bounty).then(|| "Bounty dropped".into()),
            CampAction::StartUpgrade(line) => {
                match game::camp_upgrades::can_start(&self.upgrades, &self.upgrades_config, line, self.wallet.dollars(), |id, count| bag.has(id, count)) {
                    Ok(cost) => {
                        self.wallet.try_spend(cost.cash);
                        for mat in &cost.materials {
                            bag.remove(&mat.item_id, mat.count);
                        }
                        game::camp_upgrades::start(&mut self.upgrades, &self.upgrades_config, line);
                        Some(format!("Started upgrade: {}", line.as_str()))
                    }
                    Err(e) => Some(format!("Cannot start: {e}")),
                }
            }
            CampAction::CancelUpgrade => {
                if let Some(refund) = game::camp_upgrades::cancel(&mut self.upgrades, &self.upgrades_config) {
                    self.wallet.add(refund.cash);
                    for mat in &refund.materials {
                        bag.add(&mat.item_id, mat.count);
                    }
                    Some(format!("Upgrade cancelled (refunded ${})", refund.cash))
                } else {
                    Some("No upgrade in progress".into())
                }
            }
            CampAction::DoctorHeal => None,
            CampAction::MechanicRepair => None,
            CampAction::Close => None,
        }
    }

    /// A running craft moves on; finished crafts give their items and skill XP. Returns a toast.
    pub fn craft_tick(&mut self, dt: f32, bag: &mut Inventory, dead: bool) -> Option<String> {
        let near = self.campfires.near_campfire(self.player_at);
        match self.crafting.update(dt, bag, near, dead) {
            CraftUpdate::Finished(r) => {
                if let Some(xp) = &r.xp {
                    self.skills.add_xp(&xp.skill, xp.amount);
                }
                Some(format!("Crafted {}", r.recipe_id.replace('_', " ")))
            }
            CraftUpdate::Cancelled(_) => Some("Crafting stopped".into()),
            _ => None,
        }
    }

    /// The bed in front of the player within reach, if any (Godot aims a ray at the bed's collider).
    pub fn bed_in_front(&self, player: Vec3, facing: Vec3) -> Option<String> {
        let flat = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
        self.farm
            .plots()
            .iter()
            .filter(|p| {
                let to = p.position - player;
                let d = Vec3::new(to.x, 0.0, to.z);
                d.length() < BED_REACH && d.normalize_or_zero().dot(flat) > 0.5
            })
            .min_by(|a, b| a.position.distance_squared(player).total_cmp(&b.position.distance_squared(player)))
            .map(|p| p.id.clone())
    }

    /// What E does for farming this frame: use the bed in front, or till the ground in front with a hoe
    /// (Godot FarmManager.Actions). `ground` gives the surface (point, normal) under a spot. Returns the
    /// prompt and, when pressed, a toast; None when farming has nothing to offer here.
    pub fn farm_interact(
        &mut self,
        player: Vec3,
        facing: Vec3,
        bag: &mut Inventory,
        pressed: bool,
        ground: impl Fn(Vec3) -> Option<(Vec3, Vec3)>,
    ) -> Option<(String, Option<String>)> {
        if let Some(id) = self.bed_in_front(player, facing) {
            let plot = self.farm.find(&id)?;
            let prompt = self.farm.prompt_for(plot, bag);
            if prompt.is_empty() {
                return None;
            }
            let toast = pressed.then(|| match self.farm.act(&id, bag, Some(&mut self.skills)) {
                Ok(()) => done_line(&prompt),
                Err(e) => e,
            });
            return Some((format!("[E] {prompt}"), toast));
        }
        if !bag.has("hoe", 1) {
            return None;
        }
        let flat = Vec3::new(facing.x, 0.0, facing.z).normalize_or_zero();
        if flat == Vec3::ZERO {
            return None;
        }
        let spot = player + flat * self.farm.config().till_range;
        let (point, normal) = ground(spot)?;
        let free = FarmRules::spot_is_free(point, self.farm.plots().iter().map(|p| &p.position), self.farm.config().till_spacing);
        if !FarmRules::is_flat(normal) || !free {
            // Godot only offers tilling where it would work; pressing E still says why not
            let why = if free { "The ground there is too steep" } else { "Too close to another bed" };
            return pressed.then(|| (String::new(), Some(why.to_string())));
        }
        let toast = pressed.then(|| match self.farm.till_with_surface(point, normal, Some(bag), Some(&mut self.skills)) {
            Ok(_) => "Soil tilled".to_string(),
            Err(e) => e.to_string(),
        });
        Some(("[E] Till soil".into(), toast))
    }

    /// Writes every camp system under its Godot save id.
    pub fn save_into(&self, sys: &mut save::Systems) {
        sys.wallet = Some(save::WalletSaveData { dollars: self.wallet.dollars() });
        if let Ok(d) = serde_json::from_value(self.difficulty.to_save()) {
            sys.difficulty = Some(d);
        }
        sys.extra.insert("bank".into(), self.bank.to_save());
        sys.extra.insert("farm".into(), self.farm.to_save());
        // NOTE(R10-09): ws.campfires is now captured in session::capture (SAVE-001 / ARCH-002)
        sys.extra.insert("trader".into(), self.trader.to_save());
        sys.extra.insert("skills".into(), self.skills.to_save());
        sys.extra.insert("bounties".into(), self.bounties.to_save());
        let mut visited: Vec<String> = self.settlements.visited().iter().cloned().collect();
        visited.sort();
        sys.settlements = Some(save::SettlementsSaveData {
            visited,
            respawn: self.settlements.respawn_id().map(|s| s.to_string()),
            camp_upgrades: self.upgrades_to_save(),
        });
    }

    /// Reads back whatever a save has (Godot saves have all of them).
    /// Returns warning messages for any damaged data that was reset (SAVE-004).
    pub fn load_from(&mut self, sys: &save::Systems) -> Vec<String> {
        let mut warnings = Vec::new();
        if let Some(w) = &sys.wallet {
            self.wallet.set(w.dollars);
        }
        if let Some(d) = &sys.difficulty
            && let Ok(v) = serde_json::to_value(d)
        {
            self.difficulty = DifficultySettings::from_save(&v, &self.difficulty_config);
        }
        if let Some(ws) = &sys.worldstate
            && let Ok(v) = serde_json::to_value(&ws.campfires)
            && let Err(e) = self.campfires.from_save(&v)
        {
            log::warn!("campfires not loaded: {e}");
            warnings.push("Campfire data was damaged and was reset".to_string());
        }
        if let Some(v) = sys.extra.get("farm")
            && let Err(e) = self.farm.from_save(v)
        {
            log::warn!("farm not loaded: {e}");
            warnings.push("Farm data was damaged and was reset".to_string());
        }
        if let Some(v) = sys.extra.get("bank") {
            self.bank.from_save(v);
        }
        if let Some(v) = sys.extra.get("trader") {
            self.trader.from_save(v);
        }
        if let Some(v) = sys.extra.get("skills") {
            self.skills.from_save(v);
        }
        if let Some(v) = sys.extra.get("bounties")
            && let Err(e) = self.bounties.from_save(v)
        {
            log::warn!("bounties not loaded: {e:?}");
            warnings.push("Bounty data was damaged and was reset".to_string());
        }
        if let Some(st) = &sys.settlements {
            self.load_upgrades_from_save(&st.camp_upgrades);
            if let Ok(v) = serde_json::to_value(st) {
                self.settlements.from_save(&v);
            }
        } else if let Some(v) = sys.extra.get("settlements") {
            self.settlements.from_save(v);
        }
        if let Some(levels) = camp_upgrade_view::override_levels() {
            self.upgrades.walls_level = levels.walls;
            self.upgrades.guards_level = levels.guards;
            self.upgrades.workshop_level = levels.workshop;
            self.upgrades.traders_level = levels.traders;
        }
        warnings
    }

    pub fn effects(&self) -> CampEffects {
        self.upgrades_config.effects(&self.upgrades)
    }

    pub fn camp_effects(&self) -> CampEffects {
        self.effects()
    }

    pub fn tick_upgrades(&mut self, game_hours: f32) -> Option<(UpgradeLine, u8)> {
        game::camp_upgrades::tick(&mut self.upgrades, &self.upgrades_config, game_hours)
    }

    pub fn levels(&self) -> world::CampLevels {
        world::CampLevels {
            walls: self.upgrades.walls_level,
            guards: self.upgrades.guards_level,
            workshop: self.upgrades.workshop_level,
            traders: self.upgrades.traders_level,
        }
    }

    pub fn upgrade_name(&self, line: UpgradeLine, level: u8) -> String {
        match line {
            UpgradeLine::Walls => self.upgrades_config.walls.get(level as usize).map_or_else(|| "Walls".into(), |d| d.name.clone()),
            UpgradeLine::Guards => self.upgrades_config.guards.get(level as usize).map_or_else(|| "Guards".into(), |d| d.name.clone()),
            UpgradeLine::Workshop => self.upgrades_config.workshop.get(level as usize).map_or_else(|| "Workshop".into(), |d| d.name.clone()),
            UpgradeLine::Traders => self.upgrades_config.traders.get(level as usize).map_or_else(|| "Traders".into(), |d| d.name.clone()),
        }
    }

    pub fn upgrades_to_save(&self) -> save::CampUpgradesSaveData {
        save::CampUpgradesSaveData {
            walls_level: self.upgrades.walls_level,
            guards_level: self.upgrades.guards_level,
            workshop_level: self.upgrades.workshop_level,
            traders_level: self.upgrades.traders_level,
            in_progress: self.upgrades.in_progress.as_ref().map(|p| save::UpgradeInProgressSaveData {
                line: p.line.as_str().to_string(),
                target_level: p.target_level,
                hours_left: p.hours_left,
            }),
            wall_hp: self.upgrades.wall_hp,
        }
    }

    pub fn load_upgrades_from_save(&mut self, data: &save::CampUpgradesSaveData) {
        self.upgrades.walls_level = data.walls_level;
        self.upgrades.guards_level = data.guards_level;
        self.upgrades.workshop_level = data.workshop_level;
        self.upgrades.traders_level = data.traders_level;
        self.upgrades.wall_hp = data.wall_hp;
        self.upgrades.in_progress = data.in_progress.as_ref().and_then(|p| {
            let line = match p.line.as_str() {
                "walls" => UpgradeLine::Walls,
                "guards" => UpgradeLine::Guards,
                "workshop" => UpgradeLine::Workshop,
                "traders" => UpgradeLine::Traders,
                _ => return None,
            };
            Some(game::camp_upgrades::UpgradeInProgress {
                line,
                target_level: p.target_level,
                hours_left: p.hours_left,
            })
        });
    }

    pub fn upgrades_view(&self, bag: &Inventory, items: &ItemRegistry) -> Option<crate::camp_ui::CampUpgradesView> {
        use crate::camp_ui::{CampUpgradesView, NextUpgradeView, UpgradeCostItem, UpgradeInProgressView, UpgradeLineView};

        let eff = self.effects();
        let make_line = |line: UpgradeLine| -> UpgradeLineView {
            let cur = self.upgrades.level(line);
            let max = self.upgrades_config.max_level(line);
            let summary = if cur == 0 {
                match line {
                    UpgradeLine::Walls => "No perimeter walls".into(),
                    UpgradeLine::Guards => "2 basic camp guards".into(),
                    UpgradeLine::Workshop => "Basic hand crafting".into(),
                    UpgradeLine::Traders => "Single trader stock".into(),
                }
            } else {
                self.upgrade_name(line, cur)
            };

            let next = if cur >= max {
                None
            } else {
                let target = cur + 1;
                let (name, desc, cost) = match line {
                    UpgradeLine::Walls => {
                        let d = &self.upgrades_config.walls[target as usize];
                        (d.name.clone(), format!("+{:.0} Wall HP, +{:.1}m safe zone", d.wall_hp, d.safe_zone_half_bonus), &d.cost)
                    }
                    UpgradeLine::Guards => {
                        let d = &self.upgrades_config.guards[target as usize];
                        (d.name.clone(), format!("{} guards ({:.0} dmg, {:.0}m)", d.guard_count, d.guard_damage, d.guard_range), &d.cost)
                    }
                    UpgradeLine::Workshop => {
                        let d = &self.upgrades_config.workshop[target as usize];
                        (d.name.clone(), format!("Unlocks tier {} crafting recipes", d.recipe_tier), &d.cost)
                    }
                    UpgradeLine::Traders => {
                        let d = &self.upgrades_config.traders[target as usize];
                        let mut perks = Vec::new();
                        if d.has_doctor { perks.push("Doctor"); }
                        if d.has_mechanic { perks.push("Mechanic"); }
                        let perk_str = if perks.is_empty() { String::new() } else { format!(" ({})", perks.join(", ")) };
                        (d.name.clone(), format!("+{} stock slots{}", d.trader_slots_bonus, perk_str), &d.cost)
                    }
                };
                let cost_items = cost.materials.iter().map(|m| {
                    UpgradeCostItem {
                        name: label(items, &m.item_id),
                        have: bag.count(&m.item_id),
                        need: m.count,
                    }
                }).collect();
                let cant_start_reason = game::camp_upgrades::can_start(
                    &self.upgrades,
                    &self.upgrades_config,
                    line,
                    self.wallet.dollars(),
                    |id, count| bag.has(id, count),
                ).err().map(|e| e.to_string());

                Some(NextUpgradeView {
                    name,
                    description: desc,
                    cash_cost: cost.cash,
                    items: cost_items,
                    build_hours: cost.build_hours,
                    cant_start_reason,
                })
            };

            UpgradeLineView {
                line,
                current_level: cur,
                max_level: max,
                current_summary: summary,
                next,
            }
        };

        let in_progress = self.upgrades.in_progress.as_ref().map(|p| {
            let total_hours = self.upgrades_config.level_cost(p.line, p.target_level).map_or(0.0, |c| c.build_hours);
            UpgradeInProgressView {
                line: p.line,
                target_level: p.target_level,
                target_name: self.upgrade_name(p.line, p.target_level),
                hours_left: p.hours_left,
                total_hours,
            }
        });

        Some(CampUpgradesView {
            lines: [
                make_line(UpgradeLine::Walls),
                make_line(UpgradeLine::Guards),
                make_line(UpgradeLine::Workshop),
                make_line(UpgradeLine::Traders),
            ],
            in_progress,
            wall_hp: self.upgrades.wall_hp,
            wall_hp_max: eff.wall_hp_max,
            player_cash: self.wallet.dollars(),
            refund_ratio: self.upgrades_config.refund_ratio,
            has_doctor: eff.has_doctor,
            has_mechanic: eff.has_mechanic,
        })
    }
}

/// "Plant Wheat seeds" -> "Planted Wheat seeds" and so on, for the toast after E.
fn done_line(prompt: &str) -> String {
    for (from, to) in [("Plant ", "Planted "), ("Water ", "Watered "), ("Harvest ", "Harvested "), ("Clear ", "Cleared ")] {
        if let Some(rest) = prompt.strip_prefix(from) {
            return format!("{to}{rest}");
        }
    }
    prompt.to_string()
}

fn label(items: &ItemRegistry, id: &str) -> String {
    items.get(id).map_or_else(|| id.to_string(), |d| d.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(spot: Vec3) -> Option<(Vec3, Vec3)> {
        Some((Vec3::new(spot.x, 0.0, spot.z), Vec3::Y))
    }

    #[test]
    fn hoe_tills_then_e_plants_waters_and_spacing_holds() {
        let items = Arc::new(ItemRegistry::from_repo().unwrap());
        let mut camp = Camp::new(7, items.clone()).unwrap();
        let mut bag = Inventory::with_registry(items);
        let me = Vec3::ZERO;
        let ahead = Vec3::NEG_Z;
        // No hoe: farming offers nothing on bare ground
        assert!(camp.farm_interact(me, ahead, &mut bag, true, flat).is_none());
        bag.add("hoe", 1);
        let (prompt, toast) = camp.farm_interact(me, ahead, &mut bag, false, flat).unwrap();
        assert_eq!(prompt, "[E] Till soil");
        assert!(toast.is_none());
        let (_, toast) = camp.farm_interact(me, ahead, &mut bag, true, flat).unwrap();
        assert_eq!(toast.as_deref(), Some("Soil tilled"));
        assert_eq!(camp.farm.plots().len(), 1);
        // The new bed is now in front: with seeds, E plants
        bag.add("seeds_wheat", 2);
        let (prompt, toast) = camp.farm_interact(me, ahead, &mut bag, true, flat).unwrap();
        assert_eq!(prompt, "[E] Plant Wheat seeds");
        assert_eq!(toast.as_deref(), Some("Planted Wheat seeds"));
        assert_eq!(camp.farm.plots()[0].crop.as_deref(), Some("wheat"));
        // Walking up to the bed from beyond reach, the spot ahead is too close for a second one
        let (prompt, toast) = camp.farm_interact(Vec3::new(0.0, 0.0, -4.5), Vec3::Z, &mut bag, true, flat).unwrap();
        assert!(prompt.is_empty());
        assert_eq!(toast.as_deref(), Some("Too close to another bed"));
        // Steep ground is refused
        let steep = |s: Vec3| Some((s, Vec3::new(0.7, 0.7, 0.0).normalize()));
        let (_, toast) = camp.farm_interact(Vec3::new(20.0, 0.0, 0.0), ahead, &mut bag, true, steep).unwrap();
        assert_eq!(toast.as_deref(), Some("The ground there is too steep"));
        // The farm survives a save
        let mut sys = save::Systems::default();
        camp.save_into(&mut sys);
        let mut again = Camp::new(7, Arc::new(ItemRegistry::from_repo().unwrap())).unwrap();
        again.load_from(&sys);
        assert_eq!(again.farm.plots().len(), 1);
        assert_eq!(again.farm.plots()[0].crop.as_deref(), Some("wheat"));
    }

    #[test]
    fn upgrades_start_tick_save_roundtrip() {
        let items = Arc::new(ItemRegistry::from_repo().unwrap());
        let mut camp = Camp::new(42, items.clone()).unwrap();
        let mut bag = Inventory::with_registry(items.clone());

        // Fund wallet and bag with required items for walls level 1
        camp.wallet.add(200);
        bag.add("wood", 20);

        let view = camp.upgrades_view(&bag, &items).expect("upgrades view");
        assert_eq!(view.player_cash, 220);
        assert_eq!(view.wall_hp, 0.0);

        // Start walls upgrade
        let res = camp.apply(CampAction::StartUpgrade(UpgradeLine::Walls), &mut bag, &items, false);
        assert_eq!(res.as_deref(), Some("Started upgrade: walls"));
        assert!(camp.upgrades.in_progress.is_some());

        // Save roundtrip
        let mut sys = save::Systems::default();
        camp.save_into(&mut sys);

        let mut loaded_camp = Camp::new(42, items.clone()).unwrap();
        loaded_camp.load_from(&sys);
        assert!(loaded_camp.upgrades.in_progress.is_some());

        // Tick to completion
        let finished = loaded_camp.tick_upgrades(10.0);
        assert_eq!(finished, Some((UpgradeLine::Walls, 1)));
        assert_eq!(loaded_camp.upgrades.walls_level, 1);
        assert!(loaded_camp.upgrades.wall_hp > 0.0);
    }

    #[test]
    fn camp_levels_matches_upgrades() {
        let items = Arc::new(ItemRegistry::from_repo().unwrap());
        let mut camp = Camp::new(42, items).unwrap();
        assert_eq!(camp.levels(), world::CampLevels::default());
        camp.upgrades.walls_level = 2;
        camp.upgrades.guards_level = 1;
        camp.upgrades.workshop_level = 3;
        camp.upgrades.traders_level = 2;
        assert_eq!(
            camp.levels(),
            world::CampLevels {
                walls: 2,
                guards: 1,
                workshop: 3,
                traders: 2,
            }
        );
    }
}
