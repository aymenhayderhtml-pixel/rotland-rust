use std::sync::Arc;

use crate::economy::wallet::Wallet;
use crate::items::{Inventory, InventorySaveData, ItemRegistry};

pub const BANK_SAVE_ID: &str = "bank";
/// Weight cap for the bank. Large but small enough that arithmetic in Inventory stays safe.
pub const BANK_MAX_WEIGHT: f32 = 100000.0;
pub const DEFAULT_BANK_SLOTS: usize = 40;

/// The bank: item slots with no practical weight limit and a dollar balance.
/// Port of `scripts/Economy/Bank.cs` (SaveId: "bank").
#[derive(Clone, Debug)]
pub struct Bank {
    items: Inventory,
    dollars: i32,
}

impl Default for Bank {
    fn default() -> Self {
        Self::new()
    }
}

impl Bank {
    /// Creates a bank with default 40 slots, BANK_MAX_WEIGHT, and repo item registry.
    pub fn new() -> Self {
        let registry = ItemRegistry::from_repo().expect("data/items.json must load for Bank::new");
        Self::with_registry(Arc::new(registry))
    }

    /// Creates a bank with the given ItemRegistry and default slot count.
    pub fn with_registry(registry: Arc<ItemRegistry>) -> Self {
        Self::with_slots_and_registry(DEFAULT_BANK_SLOTS, registry)
    }

    /// Creates a bank with custom slots and an explicit ItemRegistry.
    pub fn with_slots_and_registry(slots: usize, registry: Arc<ItemRegistry>) -> Self {
        let items = Inventory::with_capacity_and_registry(slots, BANK_MAX_WEIGHT, registry);
        Self { items, dollars: 0 }
    }

    pub fn items(&self) -> &Inventory {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Inventory {
        &mut self.items
    }

    pub fn dollars(&self) -> i32 {
        self.dollars
    }

    pub fn set_dollars(&mut self, dollars: i32) {
        self.dollars = dollars.max(0);
    }

    /// Moves items from a bag slot into the bank. Count 0 moves the whole stack. Returns how many moved.
    /// // Godot: Bank.cs MoveToBank
    pub fn move_to_bank(&mut self, bag: &mut Inventory, bag_slot: usize, count: u32) -> u32 {
        let Some(stack) = bag.slot(bag_slot).cloned() else {
            return 0;
        };

        let mut n = if count == 0 {
            stack.count
        } else {
            count.min(stack.count)
        };

        while n > 0 && !self.items.can_add(&stack.item_id, n) {
            n -= 1;
        }

        if n == 0 {
            return 0;
        }

        bag.remove_from_slot(bag_slot, n);
        self.items.add(&stack.item_id, n);
        n
    }

    /// Moves items from a bank slot back into the bag. Count 0 moves the whole stack. Returns how many moved.
    /// // Godot: Bank.cs MoveToBag
    pub fn move_to_bag(&mut self, bag: &mut Inventory, bank_slot: usize, count: u32) -> u32 {
        let Some(stack) = self.items.slot(bank_slot).cloned() else {
            return 0;
        };

        let mut n = if count == 0 {
            stack.count
        } else {
            count.min(stack.count)
        };

        while n > 0 && !bag.can_add(&stack.item_id, n) {
            n -= 1;
        }

        if n == 0 {
            return 0;
        }

        self.items.remove_from_slot(bank_slot, n);
        bag.add(&stack.item_id, n);
        n
    }

    /// Moves dollars from the wallet into the bank. Returns how much was deposited.
    /// // Godot: Bank.cs Deposit
    pub fn deposit(&mut self, wallet: &mut Wallet, amount: i32) -> i32 {
        let n = amount.min(wallet.dollars());
        if n <= 0 || !wallet.try_spend(n) {
            return 0;
        }
        self.dollars = self.dollars.saturating_add(n);
        n
    }

    /// Moves dollars from the bank to the wallet. Returns how much was withdrawn.
    /// // Godot: Bank.cs Withdraw
    pub fn withdraw(&mut self, wallet: &mut Wallet, amount: i32) -> i32 {
        let n = amount.min(self.dollars);
        if n <= 0 {
            return 0;
        }
        self.dollars -= n;
        wallet.add(n);
        n
    }

    /// Serializes bank state using Godot save keys ("items" and "dollars").
    /// // Godot: Bank.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({
            "items": serde_json::to_value(self.items.save()).unwrap_or_default(),
            "dollars": self.dollars
        })
    }

    /// Restores bank state from Godot save data.
    /// // Godot: Bank.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        if let Some(items_val) = data.get("items")
            && let Ok(save_data) = serde_json::from_value::<InventorySaveData>(items_val.clone())
        {
            self.items.load_save(&save_data);
        }
        self.dollars = data
            .get("dollars")
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
            .max(0) as i32;
    }

    /// Creates a Bank instance from Godot save data.
    pub fn from_save_value(
        data: &serde_json::Value,
        registry: Arc<ItemRegistry>,
        bank_slots: usize,
    ) -> Self {
        let mut bank = Self::with_slots_and_registry(bank_slots, registry);
        bank.from_save(data);
        bank
    }
}
