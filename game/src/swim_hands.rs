//! Remembers the held item when entering deep water and restores it upon leaving.

/// Action the caller should apply to the player's equipped hand item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwimHandsChange {
    /// Stow the currently held item while swimming in deep water.
    Stow,
    /// Restore the previously stowed item when leaving deep water.
    Restore(String),
}

/// Tracks a stowed hand item across entering and leaving deep water.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SwimHands {
    was_in_deep: bool,
    stowed_item: Option<String>,
}

impl SwimHands {
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates hand stow state for the current frame.
    ///
    /// Fires [`SwimHandsChange::Stow`] once when entering deep water while holding an item,
    /// and [`SwimHandsChange::Restore`] once when leaving deep water if an item was stowed.
    /// Entering with empty hands produces no change.
    pub fn update(&mut self, in_deep: bool, held_item: Option<&str>) -> Option<SwimHandsChange> {
        if in_deep && !self.was_in_deep {
            self.was_in_deep = true;
            return held_item.filter(|s| !s.is_empty()).map(|item| {
                self.stowed_item = Some(item.to_owned());
                SwimHandsChange::Stow
            });
        }

        if !in_deep && self.was_in_deep {
            self.was_in_deep = false;
            return self.stowed_item.take().map(SwimHandsChange::Restore);
        }

        None
    }

    /// Clears any remembered item and deep-water state (e.g., when the player dies).
    pub fn clear(&mut self) {
        self.was_in_deep = false;
        self.stowed_item = None;
    }

    /// Returns the item currently stowed while swimming, if any.
    pub fn stowed_item(&self) -> Option<&str> {
        self.stowed_item.as_deref()
    }
}
