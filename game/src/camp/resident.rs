//! Camp resident state and movement (port of scripts/Settlement/CampResident.cs).

use glam::{Vec2, Vec3};
use serde::{Deserialize, Serialize};

use crate::camp::chat::CampChatBubble;
use crate::camp::math::CampMath;
use crate::camp::types::{CampEvent, CampJob, CampNext, CampState};

/// Per-frame visual output for drawing a resident via humanoid animator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidentOutput {
    pub id: String,
    pub role: String,
    pub model_name: String,
    pub position: Vec3,
    pub yaw: f32,
    pub animation: String,
    pub chat_line: Option<String>,
    pub visible: bool,
}

/// One camp resident's own state (Godot: CampResident.cs).
#[derive(Debug, Clone, PartialEq)]
pub struct CampResident {
    pub index: usize,
    pub id: String,
    pub role: String,
    pub model_name: String,
    pub position: Vec3,
    pub yaw: f32,
    pub state: CampState,
    pub next: CampNext,
    pub goal: Vec3,
    pub timer: f32,
    pub job: CampJob,
    pub work_left: f32,
    pub carry_spot: usize,
    pub seat: Option<usize>,
    pub chat_id: Option<usize>,
    pub last_greet_day: i32,
    pub greets: u32,
    pub greet_target: Vec3,
    pub greet_left: f32,
    pub walk_speed: f32,
    pub tempo_scale: f32,
    pub animation: String,
    pub aim_pose: Option<String>,
    pub bubble: CampChatBubble,
    pub visible: bool,
}

impl CampResident {
    pub const ARRIVE_DISTANCE: f32 = 0.15;

    pub fn new(
        index: usize,
        id: impl Into<String>,
        role: impl Into<String>,
        model_name: impl Into<String>,
        position: Vec3,
        yaw: f32,
        walk_speed: f32,
        tempo_scale: f32,
    ) -> Self {
        Self {
            index,
            id: id.into(),
            role: role.into(),
            model_name: model_name.into(),
            position,
            yaw,
            state: CampState::Idle,
            next: CampNext::Idle,
            goal: position,
            timer: 0.0,
            job: CampJob::None,
            work_left: 0.0,
            carry_spot: 0,
            seat: None,
            chat_id: None,
            last_greet_day: -1,
            greets: 0,
            greet_target: Vec3::ZERO,
            greet_left: 0.0,
            walk_speed,
            tempo_scale,
            animation: "idle".to_string(),
            aim_pose: None,
            bubble: CampChatBubble::new(),
            visible: true,
        }
    }

    pub fn can_interact(&self) -> bool {
        self.role == "trader" || self.role == "banker"
    }

    pub fn is_greeting(&self) -> bool {
        self.greet_left > 0.0
    }

    /// Starts a walk to a goal on the ground.
    // Godot: CampResident.cs Walk
    pub fn walk(&mut self, goal: Vec3, next: CampNext, pose: Option<String>) {
        self.goal = goal;
        self.next = next;
        self.state = CampState::Walk;
        self.aim_pose = pose.clone();
        self.animation = pose.unwrap_or_else(|| "walk".to_string());
    }

    /// Stands still for the given seconds.
    // Godot: CampResident.cs Stand
    pub fn stand(
        &mut self,
        state: CampState,
        seconds: f32,
        pose: Option<String>,
        face: Option<Vec3>,
    ) {
        self.state = state;
        self.timer = seconds;
        self.aim_pose = pose.clone();
        if let Some(target) = face {
            self.face_world(target);
        }
        self.set_still(pose);
    }

    /// Sits on the ground facing the fire.
    // Godot: CampResident.cs SitAtFire
    pub fn sit_at_fire(&mut self, fire: Vec3, seconds: f32) {
        self.state = CampState::Fire;
        self.timer = seconds;
        self.aim_pose = None;
        self.face_world(fire);
        self.animation = "sit_ground".to_string();
    }

    /// Hides the body (a resident in bed).
    // Godot: CampResident.cs Hide
    pub fn hide(&mut self) {
        self.state = CampState::Hidden;
        self.aim_pose = None;
        self.visible = false;
        self.bubble.left = 0.0;
        self.animation = "hidden".to_string();
    }

    /// Shows the body again in the morning.
    // Godot: CampResident.cs Show
    pub fn show(&mut self) {
        self.visible = true;
        self.state = CampState::Idle;
        self.timer = 0.0;
        self.animation = "idle".to_string();
    }

    /// Waves once and turns to the player for a short time.
    // Godot: CampResident.cs Greet
    pub fn greet(&mut self, player_pos: Vec3, seconds: f32) {
        self.greet_target = player_pos;
        self.greet_left = seconds;
        self.greets += 1;
        self.animation = "wave".to_string();
    }

    /// Talker or listener pose for a chat.
    // Godot: CampResident.cs SetChatPose
    pub fn set_chat_pose(&mut self, talker: bool) {
        let pose = if talker { "talk" } else { "listen" };
        self.aim_pose = Some(pose.to_string());
        self.animation = pose.to_string();
    }

    /// Ends a fire sit.
    // Godot: CampResident.cs LeaveFire
    pub fn leave_fire(&mut self) {
        self.animation = "idle".to_string();
    }

    /// Speaks a line with speech bubble.
    pub fn say(&mut self, line: impl Into<String>, seconds: f32, fade_seconds: f32) {
        self.bubble.say(line, seconds, fade_seconds);
    }

    /// Advances resident by dt seconds and reports what finished.
    // Godot: CampResident.cs Update
    pub fn update(&mut self, dt: f32) -> CampEvent {
        if self.greet_left > 0.0 {
            self.greet_left -= dt;
            self.face_world(self.greet_target);
            self.animation = "wave".to_string();
            return CampEvent::None;
        }

        match self.state {
            CampState::Walk => {
                if self.move_to_goal(dt) {
                    CampEvent::Arrived
                } else {
                    CampEvent::None
                }
            }
            CampState::Idle | CampState::Fire | CampState::Work => {
                self.timer -= dt;
                if self.state == CampState::Work && self.job == CampJob::Carrier {
                    self.work_left -= dt;
                }
                if self.timer <= 0.0 {
                    CampEvent::TimerDone
                } else {
                    CampEvent::None
                }
            }
            _ => CampEvent::None,
        }
    }

    /// Turns the body to face a world point (yaw only).
    // Godot: CampResident.cs FaceWorld
    pub fn face_world(&mut self, target: Vec3) {
        self.yaw = CampMath::yaw_toward(self.position, target);
    }

    fn move_to_goal(&mut self, dt: f32) -> bool {
        let p = self.position;
        let d = Vec2::new(self.goal.x - p.x, self.goal.z - p.z);
        let dist = d.length();
        if dist <= Self::ARRIVE_DISTANCE {
            self.set_still(self.aim_pose.clone());
            return true;
        }
        let step = dist.min(self.walk_speed * dt);
        let dir = d / dist;
        self.position = Vec3::new(p.x + dir.x * step, p.y, p.z + dir.y * step);
        self.face_world(self.goal);
        self.animation = self.aim_pose.clone().unwrap_or_else(|| "walk".to_string());
        false
    }

    fn set_still(&mut self, pose: Option<String>) {
        self.animation = pose.unwrap_or_else(|| "idle".to_string());
    }

    /// Produces the render state for drawing this frame.
    pub fn render_output(&self, player_near: bool) -> ResidentOutput {
        ResidentOutput {
            id: self.id.clone(),
            role: self.role.clone(),
            model_name: self.model_name.clone(),
            position: self.position,
            yaw: self.yaw,
            animation: self.animation.clone(),
            chat_line: if player_near && self.bubble.is_saying() {
                Some(self.bubble.text.clone())
            } else {
                None
            },
            visible: self.visible,
        }
    }
}
