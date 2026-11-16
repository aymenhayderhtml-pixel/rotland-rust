//! Actions triggered by user interactions with the player hub and related screens.

/// Dog-specific commands from the dog menu and dog pack screens.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DogCommand {
    /// Open the dog's pack screen.
    OpenPack,
    /// Feed one meat item to the dog.
    Feed,
    /// Toggle Stay / Follow mode.
    ToggleStay,
    /// Store items from player bag into dog pack. Count 0 means whole stack.
    StorePack { bag_slot: usize, count: u32 },
    /// Take items from dog pack back to player bag. Count 0 means whole stack.
    TakePack { pack_slot: usize, count: u32 },
    /// Close the dog menu or dog pack screen.
    Close,
}

/// Action chosen by the player on the hub or modal screens.
/// UI never modifies game state directly; caller matches and applies the action.
#[derive(Clone, Debug, PartialEq)]
pub enum HubAction {
    /// Move or swap slots within player bag (from, to).
    MoveSlot { from: usize, to: usize },
    /// Use item in bag slot (food, drink, medical, etc.).
    UseItem { slot: usize },
    /// Equip or unequip weapon in bag slot.
    EquipItem { slot: usize },
    /// Drop count items from bag slot (0 = drop whole stack).
    DropItem { slot: usize, count: u32 },
    /// Split stack in the given bag slot.
    SplitStack { slot: usize },
    /// Assign slot to hotbar key (1..6).
    AssignHotbar { slot: usize, key: usize },
    /// Craft the given recipe id at the current station.
    Craft { recipe: String },
    /// Cancel the active craft.
    CancelCraft,
    /// Move items between bag and car trunk.
    /// from_trunk = true: trunk slot -> bag; from_trunk = false: bag slot -> trunk.
    /// count 0 means whole stack.
    TrunkMove { from_trunk: bool, slot: usize, count: u32 },
    /// Dog interaction command.
    DogCommand(DogCommand),
    /// Rename dog to the given name.
    RenameDog { name: String },
    /// Adjust difficulty setting (zombies multiplier or loot multiplier).
    SetDifficulty { zombies: Option<f32>, loot: Option<f32> },
    /// Something done on the Map tab (set or clear the waypoint).
    Map(crate::map_ui::MapAction),
    /// Close the hub or open modal screen.
    Close,
}
