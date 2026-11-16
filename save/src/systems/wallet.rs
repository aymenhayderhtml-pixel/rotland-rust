use serde::{Deserialize, Serialize};

fn default_dollars() -> i32 {
    20
}

/// Player cash on hand.
/// Port of `scripts/Economy/Wallet.cs` (SaveId: "wallet").
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletSaveData {
    #[serde(default = "default_dollars")]
    pub dollars: i32,
}

impl Default for WalletSaveData {
    fn default() -> Self {
        Self { dollars: 20 }
    }
}

impl WalletSaveData {
    pub fn new(dollars: i32) -> Self {
        Self { dollars }
    }
}
