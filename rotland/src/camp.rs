//! The camp's systems around the player: wallet, bank, trader stock, skills, bounties and the world's
//! difficulty. Kept together so play.rs and the save code each touch one thing.

use std::sync::Arc;

use crate::camp_ui::CampAction;
use game::{Bank, BountyManager, CancelReason, CraftUpdate, CraftingSystem, RecipeRegistry, DeathRule, DifficultyConfigData, DifficultySettings, EconomyConfigData, Inventory, ItemRegistry, SkillSystem, TraderStock, Wallet};

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

    /// Writes every camp system under its Godot save id.
    pub fn save_into(&self, sys: &mut save::Systems) {
        sys.wallet = Some(save::WalletSaveData { dollars: self.wallet.dollars() });
        if let Ok(d) = serde_json::from_value(self.difficulty.to_save()) {
            sys.difficulty = Some(d);
        }
        sys.extra.insert("bank".into(), self.bank.to_save());
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

fn label(items: &ItemRegistry, id: &str) -> String {
    items.get(id).map_or_else(|| id.to_string(), |d| d.name.clone())
}
