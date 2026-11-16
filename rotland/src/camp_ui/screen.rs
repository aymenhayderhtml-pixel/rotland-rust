//! Camp screen identifier enum.

/// The active camp screen: Shop (trader), Bank, Crafting, or Bounties (bounty board).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CampScreen {
    #[default]
    Shop,
    Bank,
    Crafting,
    Bounties,
}

impl CampScreen {
    /// Human-readable title for the screen header.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Shop => "Trader",
            Self::Bank => "Bank",
            Self::Crafting => "Crafting",
            Self::Bounties => "Bounty board",
        }
    }

    /// Tab name for tab switching buttons.
    pub fn tab_name(&self) -> &'static str {
        match self {
            Self::Shop => "Trader",
            Self::Bank => "Bank",
            Self::Crafting => "Crafting",
            Self::Bounties => "Bounties",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Shop => "shop",
            Self::Bank => "bank",
            Self::Crafting => "crafting",
            Self::Bounties => "bounties",
        }
    }
}
