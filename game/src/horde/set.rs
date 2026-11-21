//! Collection manager for active zombie hordes in the world.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::formation::Horde;
use crate::director::DirectorEvent;

/// Save state representation for active hordes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct HordeSaveState {
    #[serde(default)]
    pub hordes: Vec<Horde>,
}

/// Manages multiple zombie hordes, creating them from wave events,
/// removing dead members, and merging meeting hordes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HordeSet {
    pub hordes: Vec<Horde>,
}

impl HordeSet {
    pub fn new() -> Self {
        Self { hordes: Vec::new() }
    }

    /// Creates and registers a new horde with specified leader and members.
    pub fn create_horde(&mut self, leader: u64, members: Vec<u64>, leader_pos: Vec3) -> usize {
        let mut horde = Horde::new(leader, members);
        horde.set_leader_pos(leader_pos);
        self.hordes.push(horde);
        self.hordes.len() - 1
    }

    /// Creates a horde from a director wave or camp attack event and assigned zombie IDs.
    pub fn create_from_wave(
        &mut self,
        event: &DirectorEvent,
        ids: &[u64],
        leader_pos: Vec3,
    ) -> Option<usize> {
        let expected_size = match event {
            DirectorEvent::WaveStart { size, .. } => *size,
            DirectorEvent::CampAttack { size } => *size,
        };
        if ids.is_empty() {
            return None;
        }

        let leader = ids[0];
        let max_take = expected_size.min(ids.len());
        let members = if max_take > 1 {
            ids[1..max_take].to_vec()
        } else {
            Vec::new()
        };

        Some(self.create_horde(leader, members, leader_pos))
    }

    /// Removes a dead zombie from all hordes. Promotes a new leader if the leader died,
    /// or disbands the horde if no members remain.
    pub fn remove_dead(&mut self, dead_id: u64) {
        let mut to_remove = Vec::new();
        for (i, horde) in self.hordes.iter_mut().enumerate() {
            if horde.leader == dead_id {
                if !horde.members.is_empty() {
                    horde.leader = horde.members.remove(0);
                } else {
                    to_remove.push(i);
                }
            } else {
                horde.members.retain(|&id| id != dead_id);
            }
        }

        for &idx in to_remove.iter().rev() {
            self.hordes.remove(idx);
        }
    }

    /// Removes dead zombies using a liveness predicate.
    pub fn remove_dead_with(&mut self, is_alive: impl Fn(u64) -> bool) {
        let mut to_remove = Vec::new();
        for (i, horde) in self.hordes.iter_mut().enumerate() {
            horde.members.retain(|&id| is_alive(id));
            if !is_alive(horde.leader) {
                if !horde.members.is_empty() {
                    horde.leader = horde.members.remove(0);
                } else {
                    to_remove.push(i);
                }
            }
        }

        for &idx in to_remove.iter().rev() {
            self.hordes.remove(idx);
        }
    }

    /// Merges two hordes if their leaders come within `merge_distance`.
    pub fn merge_meeting(&mut self, merge_distance: f32) {
        let merge_dist_sq = merge_distance * merge_distance;
        let mut i = 0;
        while i < self.hordes.len() {
            let mut j = i + 1;
            while j < self.hordes.len() {
                let dist_sq = (self.hordes[i].leader_pos - self.hordes[j].leader_pos).length_squared();
                if dist_sq <= merge_dist_sq {
                    // Absorb horde j into horde i
                    let other = self.hordes.remove(j);
                    self.hordes[i].members.push(other.leader);
                    self.hordes[i].members.extend(other.members);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }

    /// Number of active hordes.
    pub fn count(&self) -> usize {
        self.hordes.len()
    }

    /// Total count of zombies across all hordes.
    pub fn total_zombies(&self) -> usize {
        self.hordes.iter().map(|h| h.member_count()).sum()
    }

    /// Access active hordes.
    pub fn hordes(&self) -> &[Horde] {
        &self.hordes
    }

    /// Mutable access to active hordes.
    pub fn hordes_mut(&mut self) -> &mut [Horde] {
        &mut self.hordes
    }

    /// Export active hordes into save state.
    pub fn to_save_state(&self) -> HordeSaveState {
        HordeSaveState {
            hordes: self.hordes.clone(),
        }
    }

    /// Restore hordes from save state.
    pub fn from_save_state(&mut self, save: &HordeSaveState) {
        self.hordes = save.hordes.clone();
    }
}
