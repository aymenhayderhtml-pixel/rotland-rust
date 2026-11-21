//! Represents an individual zombie horde moving in loose formation.

use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::boids::{steer_follower, steer_leader};

/// State of a single zombie horde with a designated leader and followers.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Horde {
    pub leader: u64,
    pub members: Vec<u64>,
    pub leader_pos: Vec3,
    pub target_pos: Option<Vec3>,
    pub speed: f32,
    pub spacing: f32,
    pub max_distance: f32,
    pub cohesion_weight: f32,
    pub separation_weight: f32,
    pub alignment_weight: f32,
}

impl Horde {
    /// Create a new horde with a leader and follower members.
    pub fn new(leader: u64, members: Vec<u64>) -> Self {
        Self {
            leader,
            members,
            leader_pos: Vec3::ZERO,
            target_pos: None,
            speed: 2.2, // data: director.json speed
            spacing: 1.5, // data: director.json spacing
            max_distance: 12.0, // data: director.json maxDistance
            cohesion_weight: 1.0,
            separation_weight: 1.5, // data: director.json separationWeight
            alignment_weight: 1.2, // data: director.json alignmentWeight
        }
    }

    /// Set leader's current position.
    pub fn set_leader_pos(&mut self, pos: Vec3) {
        self.leader_pos = pos;
    }

    /// Set navigation fallback target when flow field cell is unavailable.
    pub fn set_target_pos(&mut self, target: Option<Vec3>) {
        self.target_pos = target;
    }

    /// Total count including leader and members.
    pub fn member_count(&self) -> usize {
        1 + self.members.len()
    }

    /// Checks if a zombie id belongs to this horde.
    pub fn contains(&self, id: u64) -> bool {
        self.leader == id || self.members.contains(&id)
    }

    /// Steers a member based on position and flow field query.
    /// Matches task signature `Horde::steer(&self, member_pos, flow_dir_at)`.
    pub fn steer(&self, member_pos: Vec3, flow_dir_at: impl Fn(Vec3) -> Option<Vec3>) -> Vec3 {
        let is_at_leader = (member_pos - self.leader_pos).length_squared() < 0.25;
        if is_at_leader {
            steer_leader(self.leader_pos, self.target_pos, self.speed, flow_dir_at)
        } else {
            let leader_dir = steer_leader(self.leader_pos, self.target_pos, 1.0, &flow_dir_at);
            steer_follower(
                member_pos,
                self.leader_pos,
                leader_dir,
                self.spacing,
                self.max_distance,
                self.speed,
                self.cohesion_weight,
                self.separation_weight,
                self.alignment_weight,
            )
        }
    }

    /// Steers a specific member by ID.
    pub fn steer_member(
        &self,
        member_id: u64,
        member_pos: Vec3,
        flow_dir_at: impl Fn(Vec3) -> Option<Vec3>,
    ) -> Vec3 {
        if member_id == self.leader {
            steer_leader(self.leader_pos, self.target_pos, self.speed, flow_dir_at)
        } else {
            let leader_dir = steer_leader(self.leader_pos, self.target_pos, 1.0, &flow_dir_at);
            steer_follower(
                member_pos,
                self.leader_pos,
                leader_dir,
                self.spacing,
                self.max_distance,
                self.speed,
                self.cohesion_weight,
                self.separation_weight,
                self.alignment_weight,
            )
        }
    }
}
