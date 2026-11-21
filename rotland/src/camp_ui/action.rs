//! Actions triggered by user interactions with camp screens.

use game::camp_upgrades::UpgradeLine;

/// Action chosen by the player on a camp screen.
/// The UI never modifies game state directly; caller matches and applies the action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CampAction {
    /// Buy `count` units of `item` from the trader stock.
    Buy { item: String, count: u32 },
    /// Sell `count` units of `item` from the backpack to the trader.
    Sell { item: String, count: u32 },
    /// Deposit `count` units of `item` from the backpack into the bank.
    Deposit { item: String, count: u32 },
    /// Deposit `amount` dollars into the bank.
    DepositDollars { amount: i32 },
    /// Withdraw `count` units of `item` from the bank into the backpack.
    Withdraw { item: String, count: u32 },
    /// Withdraw `amount` dollars from the bank into the wallet.
    WithdrawDollars { amount: i32 },
    /// Start crafting the given recipe.
    Craft { recipe: String },
    /// Cancel the currently running craft.
    CancelCraft,
    /// Accept an offered bounty from the bounty board.
    Accept { bounty: String },
    /// Turn in (claim) a finished bounty for rewards.
    TurnIn { bounty: String },
    /// Abandon an active bounty without penalty.
    Abandon { bounty: String },
    /// Start constructing the next level of the given camp upgrade line.
    StartUpgrade(UpgradeLine),
    /// Cancel the currently running camp upgrade (refund handled by caller).
    CancelUpgrade,
    /// Close the camp screen.
    Close,
}
