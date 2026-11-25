//! Actions triggered by user interactions with camp screens.

use game::camp_upgrades::UpgradeLine;
use game::guild::RecruitOrder;

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
    /// Heal to full health for cash via the unlocked camp doctor.
    DoctorHeal,
    /// Repair car to full for cash via the unlocked camp mechanic.
    MechanicRepair,
    /// Close the camp screen.
    Close,
    /// A button on the Guild tab.
    Guild(GuildCampAction),
}

/// Guild action variants triggered by interactions on the Guild camp board screen.
/// Ready to be merged into `CampAction` when Claude wires `camp.rs` and `frame_ui.rs`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuildCampAction {
    /// Accept an offered task on the guild board.
    AcceptTask { task_id: String },
    /// Abandon an active guild task without penalty.
    AbandonTask { task_id: String },
    /// Claim rewards for a completed guild task.
    ClaimTask { task_id: String },
    /// Hand the items a fetch task needs from the bag in at the guild (R24-N6; handled by the frame, which owns the hooks).
    HandIn { task_id: String },
    /// Hire an available teammate recruit.
    HireRecruit { recruit_id: String },
    /// Dismiss a hired teammate recruit.
    DismissRecruit { recruit_id: String },
    /// Set a standing order (Stay, Follow, Guard) for a hired teammate.
    SetRecruitOrder { recruit_id: String, order: RecruitOrder },
    /// Buy `count` units of `item_id` from the guild shop.
    BuyItem { item_id: String, count: u32 },
}
