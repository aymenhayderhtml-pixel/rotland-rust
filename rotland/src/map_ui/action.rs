//! User actions resulting from interacting with the map UI.

use glam::Vec2;

/// Actions returned from the map screen when buttons are pressed or mouse clicks occur.
#[derive(Debug, Clone, PartialEq)]
pub enum MapAction {
    /// Place or move a waypoint marker at the given world XZ position.
    AddMarker(Vec2),
    /// Remove or clear the current waypoint marker.
    RemoveMarker,
    /// Close the map UI.
    Close,
}
