//! Camp screen identifier enum.

/// The active camp screen: Shop (trader), Bank, Crafting, Bounties, or Upgrades (camp board).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CampScreen {
    #[default]
    Shop,
    Bank,
    Crafting,
    Bounties,
    Upgrades,
}

impl CampScreen {
    /// Human-readable title for the screen header.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Shop => "Trader",
            Self::Bank => "Bank",
            Self::Crafting => "Crafting",
            Self::Bounties => "Bounty board",
            Self::Upgrades => "Camp Board",
        }
    }

    /// Tab name for tab switching buttons.
    pub fn tab_name(&self) -> &'static str {
        match self {
            Self::Shop => "Trader",
            Self::Bank => "Bank",
            Self::Crafting => "Crafting",
            Self::Bounties => "Bounties",
            Self::Upgrades => "Upgrades",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shop => "shop",
            Self::Bank => "bank",
            Self::Crafting => "crafting",
            Self::Bounties => "bounties",
            Self::Upgrades => "upgrades",
        }
    }
}
