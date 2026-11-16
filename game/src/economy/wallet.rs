use serde::{Deserialize, Serialize};

pub const WALLET_SAVE_ID: &str = "wallet";

/// The player's cash wallet.
/// Port of `scripts/Economy/Wallet.cs` (SaveId: "wallet").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Wallet {
    dollars: i32,
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Wallet {
    /// Creates a wallet with the specified starting cash.
    pub fn new(start_dollars: i32) -> Self {
        Self {
            dollars: start_dollars.max(0),
        }
    }

    /// Current dollar balance.
    pub fn dollars(&self) -> i32 {
        self.dollars
    }

    /// Explicitly sets the dollar balance (clamped to at least 0).
    pub fn set(&mut self, value: i32) {
        self.dollars = value.max(0);
    }

    /// Adds cash to the wallet.
    /// // Godot: Wallet.cs Add
    pub fn add(&mut self, amount: i32) {
        if amount > 0 {
            self.set(self.dollars.saturating_add(amount));
        }
    }

    /// Attempts to spend cash (all-or-nothing). Returns false if balance is insufficient.
    /// // Godot: Wallet.cs TrySpend
    pub fn try_spend(&mut self, amount: i32) -> bool {
        if amount < 0 || amount > self.dollars {
            return false;
        }
        self.set(self.dollars - amount);
        true
    }

    /// Empties the wallet completely and returns the amount taken (death penalty hook).
    /// // Godot: Wallet.cs TakeAll
    pub fn take_all(&mut self) -> i32 {
        let all = self.dollars;
        self.set(0);
        all
    }

    /// Serializes wallet state using Godot save keys.
    /// // Godot: Wallet.cs Save
    pub fn to_save(&self) -> serde_json::Value {
        serde_json::json!({
            "dollars": self.dollars
        })
    }

    /// Restores wallet state from Godot save data.
    /// // Godot: Wallet.cs Load
    pub fn from_save(&mut self, data: &serde_json::Value) {
        let dollars = data
            .get("dollars")
            .and_then(|v| v.as_i64())
            .unwrap_or(20) as i32;
        self.set(dollars);
    }

    /// Creates a Wallet instance directly from Godot save data.
    pub fn from_save_value(data: &serde_json::Value) -> Self {
        let mut wallet = Self::default();
        wallet.from_save(data);
        wallet
    }
}
