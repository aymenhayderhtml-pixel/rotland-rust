//! Chat and speech bubble logic for camp residents (port of scripts/Settlement/CampChat.cs and CampChatBubble.cs).

use world::DotnetRandom;
use crate::camp::config::CampLifeData;
use crate::camp::math::CampMath;
use crate::camp::resident::CampResident;
use crate::camp::types::CampState;

/// Speech bubble state for a speaking resident (Godot: CampChatBubble.cs).
#[derive(Debug, Clone, PartialEq)]
pub struct CampChatBubble {
    pub text: String,
    pub total: f32,
    pub fade: f32,
    pub left: f32,
}

impl Default for CampChatBubble {
    fn default() -> Self {
        Self::new()
    }
}

impl CampChatBubble {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            total: 4.0,
            fade: 0.6,
            left: 0.0,
        }
    }

    /// Shows a line for the given seconds, with the fade taken from the end of that time.
    // Godot: CampChatBubble.cs Say
    pub fn say(&mut self, line: impl Into<String>, seconds: f32, fade_seconds: f32) {
        self.text = line.into();
        self.total = seconds;
        self.fade = fade_seconds;
        self.left = seconds;
    }

    /// Counts the line down.
    // Godot: CampChatBubble.cs Tick
    pub fn tick(&mut self, dt: f32) {
        self.left = (self.left - dt).max(0.0);
    }

    /// True while a line is still active.
    // Godot: CampChatBubble.cs IsSaying
    pub fn is_saying(&self) -> bool {
        self.left > 0.0
    }

    /// Opacity calculation matching Godot fade.
    pub fn alpha(&self) -> f32 {
        if self.left <= 0.0 {
            0.0
        } else {
            (self.left / self.fade.max(0.01)).clamp(0.0, 1.0)
        }
    }

    /// Active text line if still displaying.
    pub fn text(&self) -> Option<&str> {
        if self.left > 0.0 {
            Some(&self.text)
        } else {
            None
        }
    }
}

/// Two residents conversing (Godot: CampChat.cs).
#[derive(Debug, Clone, PartialEq)]
pub struct CampChat {
    pub id: usize,
    pub a_index: usize,
    pub b_index: usize,
    pub timer: f32,
    pub swap: f32,
    pub wait: f32,
    pub started: bool,
    pub talker: Option<usize>,
    pub lines_spoken: u32,
}

impl CampChat {
    /// A partner that has not arrived after this long means the chat is dropped.
    pub const MAX_WAIT_SECONDS: f32 = 25.0;

    pub fn new(id: usize, a_index: usize, b_index: usize, timer: f32) -> Self {
        Self {
            id,
            a_index,
            b_index,
            timer,
            swap: 0.0,
            wait: 0.0,
            started: false,
            talker: None,
            lines_spoken: 0,
        }
    }

    /// Advances the talk. Returns true when the chat is over and the planner should end it.
    // Godot: CampChat.cs Tick
    pub fn tick(
        &mut self,
        dt: f32,
        residents: &mut [CampResident],
        data: &CampLifeData,
        rng: &mut DotnetRandom,
    ) -> bool {
        if !self.started {
            if residents[self.a_index].state == CampState::Chat
                && residents[self.b_index].state == CampState::Chat
            {
                self.start(residents, data, rng);
            } else {
                self.wait += dt;
                return self.wait > Self::MAX_WAIT_SECONDS;
            }
        }

        self.timer -= dt;
        if self.timer <= 0.0 {
            return true;
        }

        self.swap -= dt;
        if self.swap <= 0.0 {
            let next_talker = if self.talker == Some(self.a_index) {
                self.b_index
            } else {
                self.a_index
            };
            self.swap_talker(next_talker, residents, data, rng);
        }

        false
    }

    fn start(
        &mut self,
        residents: &mut [CampResident],
        data: &CampLifeData,
        rng: &mut DotnetRandom,
    ) {
        self.started = true;
        let pos_a = residents[self.a_index].position;
        let pos_b = residents[self.b_index].position;
        residents[self.a_index].face_world(pos_b);
        residents[self.b_index].face_world(pos_a);

        let initial_talker = if rng.next(2) == 0 {
            self.a_index
        } else {
            self.b_index
        };
        self.swap_talker(initial_talker, residents, data, rng);
    }

    fn swap_talker(
        &mut self,
        talker_idx: usize,
        residents: &mut [CampResident],
        data: &CampLifeData,
        rng: &mut DotnetRandom,
    ) {
        self.talker = Some(talker_idx);
        let listener_idx = if talker_idx == self.a_index {
            self.b_index
        } else {
            self.a_index
        };

        residents[talker_idx].set_chat_pose(true);
        residents[listener_idx].set_chat_pose(false);

        if !data.chat_lines.is_empty() {
            let line_idx = rng.next(data.chat_lines.len() as i32) as usize;
            let line = &data.chat_lines[line_idx];
            residents[talker_idx].say(line, data.bubble_seconds, data.bubble_fade_seconds);
        }

        self.lines_spoken += 1;
        self.swap = CampMath::range(rng, data.chat_swap_min_seconds, data.chat_swap_max_seconds);
    }
}
