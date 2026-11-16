//! The camp's systems around the player: wallet, bank, trader stock, skills, bounties and the world's
//! difficulty. Kept together so play.rs and the save code each touch one thing.

use std::sync::Arc;

use glam::Vec3;

/// How far in front of the player a bed can be used.
const BED_REACH: f32 = 2.5;

use crate::camp_ui::CampAction;
use game::{FarmManager, FarmRules, Bank, BountyManager, CancelReason, CraftUpdate, CraftingSystem, RecipeRegistry, DeathRule, DifficultyConfigData, DifficultySettings, EconomyConfigData, Inventory, ItemRegistry, SkillSystem, TraderStock, Wallet};

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
}

impl Camp {
    pub fn new(seed: i32, items: Arc<ItemRegistry>) -> Result<Self, String> {
        let economy = EconomyConfigData::from_repo().map_err(|e| format!("data/economy.json: {e:?}"))?;
        let difficulty_config = DifficultyConfigData::from_repo().map_err(|e| format!("data/difficulty.json: {e}"))?;
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
                // No campfires in the Rust world yet: campfire recipes stay locked
                let ok = self.crafting.start_craft(&recipe, &station, bag, false, dead, &self.recipes);
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
            CampAction::Close => None,
        }
    }

    /// A running craft moves on; finished crafts give their items and skill XP. Returns a toast.
    pub fn craft_tick(&mut self, dt: f32, bag: &mut Inventory, dead: bool) -> Option<String> {
        match self.crafting.update(dt, bag, false, dead) {
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
        sys.extra.insert("trader".into(), self.trader.to_save());
        sys.extra.insert("skills".into(), self.skills.to_save());
        sys.extra.insert("bounties".into(), self.bounties.to_save());
    }

    /// Reads back whatever a save has (Godot saves have all of them).
    pub fn load_from(&mut self, sys: &save::Systems) {
        if let Some(w) = &sys.wallet {
            self.wallet.set(w.dollars);
        }
        if let Some(d) = &sys.difficulty
            && let Ok(v) = serde_json::to_value(d)
        {
            self.difficulty = DifficultySettings::from_save(&v, &self.difficulty_config);
        }
        if let Some(v) = sys.extra.get("farm")
            && let Err(e) = self.farm.from_save(v)
        {
            log::warn!("farm not loaded: {e}");
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
        }
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
}
