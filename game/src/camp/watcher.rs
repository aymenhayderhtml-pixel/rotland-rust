//! Settlement visit tracking and respawn camp persistence (port of scripts/Settlement/SettlementWatcher.cs).

use std::collections::HashSet;
use glam::Vec3;
use serde_json::{json, Value};
use world::{SettlementConfigData, SettlementSite};

/// Watches player against safe zones and records visited settlements (Godot: SettlementWatcher.cs).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettlementWatcher {
    visited: HashSet<String>,
    respawn_id: Option<String>,
    current_settlement: Option<String>,
}

impl SettlementWatcher {
    /// Godot save ID.
    pub const SAVE_ID: &'static str = "settlements";

    pub fn new() -> Self {
        Self::default()
    }

    pub fn save_id(&self) -> &'static str {
        Self::SAVE_ID
    }

    pub fn visited(&self) -> &HashSet<String> {
        &self.visited
    }

    pub fn is_visited(&self, id: &str) -> bool {
        self.visited.contains(id)
    }

    pub fn respawn_id(&self) -> Option<&str> {
        self.respawn_id.as_deref()
    }

    pub fn current(&self) -> Option<&str> {
        self.current_settlement.as_deref()
    }

    pub fn mark_visited(&mut self, id: impl Into<String>) {
        let s = id.into();
        self.visited.insert(s.clone());
        self.respawn_id = Some(s);
    }

    pub fn set_respawn(&mut self, id: Option<String>) {
        self.respawn_id = id;
    }

    /// Godot: SettlementWatcher.cs _PhysicsProcess
    /// Updates the player against safe zones. Returns Some(inside) if safe zone state changed.
    pub fn update(
        &mut self,
        player_pos: Vec3,
        sites: &[SettlementSite],
        cfg: &SettlementConfigData,
    ) -> Option<bool> {
        let inside = sites.iter().find(|s| s.contains(player_pos, 0.0, cfg));
        let inside_id = inside.map(|s| s.id.clone());
        if inside_id == self.current_settlement {
            return None;
        }

        let entered = inside.is_some();
        self.current_settlement = inside_id;
        if let Some(site) = inside {
            self.visited.insert(site.id.clone());
            self.respawn_id = Some(site.id.clone());
        }
        Some(entered)
    }

    /// Where a respawn goes: the last settlement visited.
    // Godot: SettlementWatcher.cs TryRespawnPoint
    pub fn try_respawn_point(
        &self,
        sites: &[SettlementSite],
        cfg: &SettlementConfigData,
    ) -> Option<Vec3> {
        let id = self.respawn_id.as_ref()?;
        let site = sites.iter().find(|s| &s.id == id)?;
        Some(site.respawn_point(cfg))
    }

    /// Godot: SettlementWatcher.cs Save
    pub fn to_save(&self) -> Value {
        let mut visited: Vec<String> = self.visited.iter().cloned().collect();
        visited.sort();
        let mut map = serde_json::Map::new();
        map.insert("visited".to_string(), json!(visited));
        if let Some(ref r) = self.respawn_id {
            map.insert("respawn".to_string(), json!(r));
        }
        Value::Object(map)
    }

    /// Godot: SettlementWatcher.cs Load
    pub fn from_save(&mut self, data: &Value) {
        self.visited.clear();
        self.respawn_id = None;
        if let Some(arr) = data.get("visited").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(s) = item.as_str() {
                    self.visited.insert(s.to_string());
                }
            }
        }
        if let Some(r) = data.get("respawn").and_then(|v| v.as_str())
            && !r.is_empty()
        {
            self.respawn_id = Some(r.to_string());
        }
    }
}
