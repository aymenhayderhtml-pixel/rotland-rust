//! Tab and screen identifiers for player hub and overlay screens.

/// The tabs of the player hub, in tab-bar order (matches scripts/UI/HubTab.cs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HubTab {
    Profile,
    #[default]
    Bag,
    Skills,
    Bounties,
    Map,
    Settings,
}

impl HubTab {
    pub const ALL: [HubTab; 6] = [
        HubTab::Profile,
        HubTab::Bag,
        HubTab::Skills,
        HubTab::Bounties,
        HubTab::Map,
        HubTab::Settings,
    ];

    /// Tab label displayed in the top bar.
    pub fn tab_name(&self) -> &'static str {
        match self {
            Self::Profile => "Profile (P)",
            Self::Bag => "Bag (Tab)",
            Self::Skills => "Skills (L)",
            Self::Bounties => "Bounties (J)",
            Self::Map => "Map (M)",
            Self::Settings => "Settings",
        }
    }

    /// Screen title in header.
    pub fn title(&self) -> &'static str {
        match self {
            Self::Profile => "Profile",
            Self::Bag => "Bag",
            Self::Skills => "Skills",
            Self::Bounties => "Bounties",
            Self::Map => "Map",
            Self::Settings => "Settings",
        }
    }

    /// Next tab with wrapping.
    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|t| t == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    /// Previous tab with wrapping.
    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|t| t == self).unwrap_or(0);
        let len = Self::ALL.len();
        Self::ALL[(idx + len - 1) % len]
    }
}

/// The currently active modal screen managed by the hub UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HubScreen {
    /// The standard tabbed player hub screen.
    Tab(HubTab),
    /// Car trunk storage screen (two inventories side by side).
    Trunk,
    /// Dog interaction radial/quick menu.
    DogMenu,
    /// Dog backpack storage screen.
    DogPack,
    /// "Name your puppy" dialog.
    DogNameDialog,
}

impl Default for HubScreen {
    fn default() -> Self {
        Self::Tab(HubTab::Bag)
    }
}

impl HubScreen {
    pub fn tab(&self) -> Option<HubTab> {
        match self {
            Self::Tab(t) => Some(*t),
            _ => None,
        }
    }
}
