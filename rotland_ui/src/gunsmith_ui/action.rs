//! Actions and attachment slot kinds for the Gunsmith UI.

/// The five attachment slots supported by the gunsmith upgrade screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GunsmithSlot {
    #[default]
    Optic,
    Muzzle,
    Magazine,
    Grip,
    Stock,
}

impl GunsmithSlot {
    pub const ALL: [Self; 5] = [
        Self::Optic,
        Self::Muzzle,
        Self::Magazine,
        Self::Grip,
        Self::Stock,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Optic => "Optic",
            Self::Muzzle => "Muzzle",
            Self::Magazine => "Magazine",
            Self::Grip => "Grip",
            Self::Stock => "Stock",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Optic => "optic",
            Self::Muzzle => "muzzle",
            Self::Magazine => "magazine",
            Self::Grip => "grip",
            Self::Stock => "stock",
        }
    }
}

/// Action emitted by the gunsmith screen; caller applies it to game state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GunsmithAction {
    /// Buy `count` units of `item_id` using scrap metal.
    Buy { item_id: String, count: u32 },
    /// Sell `count` units from bag `slot` for scrap metal.
    Sell {
        slot: usize,
        item_id: String,
        count: u32,
    },
    /// Repair the worn weapon in bag `slot` to full condition.
    Repair { slot: usize },
    /// Repair all worn weapons in the bag.
    RepairAll,
    /// Fit `attachment_id` onto the weapon in `weapon_slot`.
    FitMod {
        weapon_slot: usize,
        attachment_id: String,
    },
    /// Remove `attachment_id` from the weapon in `weapon_slot` and return it to bag.
    RemoveMod {
        weapon_slot: usize,
        attachment_id: String,
    },
    /// Close the gunsmith screen.
    Close,
}
