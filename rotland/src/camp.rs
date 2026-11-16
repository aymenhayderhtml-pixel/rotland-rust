//! The camp's systems around the player: wallet, bank, trader stock, skills, bounties and the world's
//! difficulty. Kept together so play.rs and the save code each touch one thing.

use std::sync::Arc;

use game::{Bank, BountyManager, DeathRule, DifficultyConfigData, DifficultySettings, EconomyConfigData, Inventory, ItemRegistry, SkillSystem, TraderStock, Wallet};

pub struct Camp {
    pub economy: EconomyConfigData,
    pub wallet: Wallet,
    pub bank: Bank,
    pub trader: TraderStock,
    pub skills: SkillSystem,
    pub bounties: BountyManager,
    pub difficulty_config: DifficultyConfigData,
    pub difficulty: DifficultySettings,
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
