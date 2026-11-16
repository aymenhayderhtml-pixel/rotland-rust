//! Camp activity scheduler and job assigner (port of scripts/Settlement/CampPlanner.cs).

use std::f32::consts::TAU;
use glam::Vec3;
use world::{DotnetRandom, SettlementSite};

use crate::camp::chat::CampChat;
use crate::camp::config::CampLifeData;
use crate::camp::math::CampMath;
use crate::camp::resident::CampResident;
use crate::camp::types::{CampClock, CampJob, CampNext, CampState};

/// Chooses what an idle resident does next, from the clock and the dice (Godot: CampPlanner.cs).
pub struct CampPlanner {
    pub site_center: Vec3,
    pub site_yaw: f32,
    pub wander_points: Vec<Vec3>,
    pub bed_points: Vec<Vec3>,
    pub carry_spots: Vec<Vec3>,
    pub chop_stand: Vec3,
    pub chop_face: Vec3,
    pub seats: Vec<Vec3>,
    pub seat_owners: Vec<Option<usize>>,
    pub bed_count: usize,
    next_chat_id: usize,
}

impl CampPlanner {
    pub const FIRE_HOUR_CHANCE: f32 = 0.7;

    pub fn new(site: &SettlementSite, data: &CampLifeData) -> Self {
        let wander_points = data.wander_points.iter().map(|p| site.center + site.local(p[0], 0.0, p[1])).collect();
        let bed_points = data.bed_points.iter().map(|p| site.center + site.local(p[0], 0.0, p[1])).collect();
        let carry_spots = data.carry_spots.iter().map(|p| site.center + site.local(p[0], 0.0, p[1])).collect();
        let chop_stand = site.center + site.local(data.chop_stand[0], 0.0, data.chop_stand[1]);
        let chop_face = site.center + site.local(data.chop_face[0], 0.0, data.chop_face[1]);
        let bed_count = data.resident_count / 2;

        let mut seats = Vec::with_capacity(data.fire_seats);
        let mut seat_owners = Vec::with_capacity(data.fire_seats);
        for i in 0..data.fire_seats {
            let angle = (i as f32 + 0.5) * TAU / data.fire_seats as f32;
            let offset = site.local(angle.cos() * data.seat_radius, 0.0, angle.sin() * data.seat_radius);
            seats.push(site.center + offset);
            seat_owners.push(None);
        }

        Self {
            site_center: site.center,
            site_yaw: site.yaw,
            wander_points,
            bed_points,
            carry_spots,
            chop_stand,
            chop_face,
            seats,
            seat_owners,
            bed_count,
            next_chat_id: 1,
        }
    }

    pub fn world_point(&self, p: [f32; 2]) -> Vec3 {
        let c = self.site_yaw.cos();
        let s = self.site_yaw.sin();
        self.site_center + Vec3::new(c * p[0] + s * p[1], 0.0, -s * p[0] + c * p[1])
    }

    pub fn is_bedtime(&self, resident_idx: usize) -> bool {
        resident_idx < self.bed_count
    }

    /// Picks the next activity for an idle resident (Godot: CampPlanner.cs Plan).
    pub fn plan(
        &mut self,
        r_idx: usize,
        residents: &mut [CampResident],
        chats: &mut Vec<CampChat>,
        clock: CampClock,
        data: &CampLifeData,
        rng: &mut DotnetRandom,
    ) {
        if clock.night {
            if self.is_bedtime(r_idx) {
                let dest = self.bed_points[r_idx % self.bed_points.len()];
                self.walk_to(r_idx, residents, dest, CampNext::Hide, None);
            } else if !self.try_seat(r_idx, residents) {
                self.start_idle(r_idx, residents, data, rng);
            }
            return;
        }

        let roll = rng.next_double() as f32;
        let wants_fire = if clock.fire_hours { roll < Self::FIRE_HOUR_CHANCE } else { roll < data.day_sit_chance };
        if wants_fire && self.try_seat(r_idx, residents) {
            return;
        }

        let roll = rng.next_double() as f32;
        if roll < data.day_chat_chance && self.try_chat(r_idx, residents, chats, data, rng) {
            return;
        }

        if residents[r_idx].job != CampJob::None && (rng.next_double() as f32) < data.day_work_chance {
            self.start_work(r_idx, residents, data, rng);
            return;
        }

        if !self.wander_points.is_empty() {
            let idx = rng.next(self.wander_points.len() as i32) as usize;
            self.walk_to(r_idx, residents, self.wander_points[idx], CampNext::Idle, None);
        } else {
            self.start_idle(r_idx, residents, data, rng);
        }
    }

    /// A walk ended: start what the walk was for (Godot: CampPlanner.cs Arrive).
    pub fn arrive(&mut self, r_idx: usize, residents: &mut [CampResident], data: &CampLifeData, rng: &mut DotnetRandom) {
        match residents[r_idx].next {
            CampNext::Idle => self.start_idle(r_idx, residents, data, rng),
            CampNext::Fire => {
                let dur = CampMath::range(rng, data.fire_seconds[0], data.fire_seconds[1]);
                residents[r_idx].sit_at_fire(self.site_center, dur);
            }
            CampNext::Work => {
                if residents[r_idx].job == CampJob::Woodcutter {
                    let left = residents[r_idx].work_left;
                    residents[r_idx].stand(CampState::Work, left, Some("chop".to_string()), Some(self.chop_face));
                } else {
                    residents[r_idx].stand(CampState::Work, 3.0, None, None);
                }
            }
            CampNext::Chat => residents[r_idx].stand(CampState::Chat, 0.0, None, None),
            CampNext::Hide => residents[r_idx].hide(),
        }
    }

    /// A timed state ran out (Godot: CampPlanner.cs TimerDone).
    pub fn timer_done(
        &mut self,
        r_idx: usize,
        residents: &mut [CampResident],
        chats: &mut Vec<CampChat>,
        clock: CampClock,
        data: &CampLifeData,
        rng: &mut DotnetRandom,
    ) {
        match residents[r_idx].state {
            CampState::Idle => self.plan(r_idx, residents, chats, clock, data, rng),
            CampState::Fire => {
                if clock.night && !self.is_bedtime(r_idx) {
                    let dur = CampMath::range(rng, data.fire_seconds[0], data.fire_seconds[1]);
                    residents[r_idx].sit_at_fire(self.site_center, dur);
                    return;
                }
                residents[r_idx].leave_fire();
                self.free_seat(r_idx, residents);
                if clock.night {
                    self.plan(r_idx, residents, chats, clock, data, rng);
                } else {
                    self.start_idle(r_idx, residents, data, rng);
                }
            }
            CampState::Work => {
                if residents[r_idx].job == CampJob::Carrier && residents[r_idx].work_left > 0.0 && !self.carry_spots.is_empty() {
                    residents[r_idx].carry_spot = (residents[r_idx].carry_spot + 1) % self.carry_spots.len();
                    let dest = self.carry_spots[residents[r_idx].carry_spot];
                    self.walk_to(r_idx, residents, dest, CampNext::Work, Some("carry".to_string()));
                } else {
                    self.start_idle(r_idx, residents, data, rng);
                }
            }
            _ => {}
        }
    }

    pub fn end_chat(&mut self, chat: &CampChat, residents: &mut [CampResident], data: &CampLifeData, rng: &mut DotnetRandom) {
        residents[chat.a_index].chat_id = None;
        residents[chat.b_index].chat_id = None;
        self.start_idle(chat.a_index, residents, data, rng);
        self.start_idle(chat.b_index, residents, data, rng);
    }

    pub fn wake_up(&mut self, r_idx: usize, residents: &mut [CampResident], data: &CampLifeData, rng: &mut DotnetRandom) {
        residents[r_idx].show();
        self.start_idle(r_idx, residents, data, rng);
    }

    pub fn start_idle(&mut self, r_idx: usize, residents: &mut [CampResident], data: &CampLifeData, rng: &mut DotnetRandom) {
        let sec = CampMath::range(rng, data.idle_min_seconds, data.idle_max_seconds);
        residents[r_idx].stand(CampState::Idle, sec, None, None);
    }

    fn start_work(&mut self, r_idx: usize, residents: &mut [CampResident], data: &CampLifeData, rng: &mut DotnetRandom) {
        residents[r_idx].work_left = CampMath::range(rng, data.work_seconds[0], data.work_seconds[1]);
        if residents[r_idx].job == CampJob::Woodcutter {
            self.walk_to(r_idx, residents, self.chop_stand, CampNext::Work, None);
        } else if !self.carry_spots.is_empty() {
            let spot = residents[r_idx].carry_spot % self.carry_spots.len();
            let dest = self.carry_spots[spot];
            self.walk_to(r_idx, residents, dest, CampNext::Work, Some("carry".to_string()));
        }
    }

    fn try_seat(&mut self, r_idx: usize, residents: &mut [CampResident]) -> bool {
        for (i, owner) in self.seat_owners.iter_mut().enumerate() {
            if owner.is_none() {
                *owner = Some(r_idx);
                residents[r_idx].seat = Some(i);
                self.walk_to(r_idx, residents, self.seats[i], CampNext::Fire, None);
                return true;
            }
        }
        false
    }

    pub fn free_seat(&mut self, r_idx: usize, residents: &mut [CampResident]) {
        if let Some(seat) = residents[r_idx].seat
            && seat < self.seat_owners.len()
            && self.seat_owners[seat] == Some(r_idx)
        {
            self.seat_owners[seat] = None;
        }
        residents[r_idx].seat = None;
    }

    fn try_chat(&mut self, r_idx: usize, residents: &mut [CampResident], chats: &mut Vec<CampChat>, data: &CampLifeData, rng: &mut DotnetRandom) -> bool {
        let r_pos = residents[r_idx].position;
        let mut partner: Option<usize> = None;
        let mut best = data.chat_range;

        for (other_idx, other) in residents.iter().enumerate() {
            if other_idx != r_idx && other.state == CampState::Idle && other.chat_id.is_none() {
                let d = CampMath::flat(r_pos, other.position);
                if d <= best {
                    best = d;
                    partner = Some(other_idx);
                }
            }
        }

        let Some(partner_idx) = partner else { return false };
        let middle = (r_pos + residents[partner_idx].position) * 0.5;
        let meet = self.nearest_wander(middle);

        let chat_timer = CampMath::range(rng, data.chat_min_seconds, data.chat_max_seconds);
        let chat_id = self.next_chat_id;
        self.next_chat_id += 1;
        chats.push(CampChat::new(chat_id, r_idx, partner_idx, chat_timer));

        residents[r_idx].chat_id = Some(chat_id);
        residents[partner_idx].chat_id = Some(chat_id);

        self.walk_to(r_idx, residents, meet + Vec3::new(-data.chat_half_gap, 0.0, 0.0), CampNext::Chat, None);
        self.walk_to(partner_idx, residents, meet + Vec3::new(data.chat_half_gap, 0.0, 0.0), CampNext::Chat, None);
        true
    }

    fn nearest_wander(&self, point: Vec3) -> Vec3 {
        let mut best = self.wander_points.first().copied().unwrap_or(point);
        let mut best_dist = f32::MAX;
        for &spot in &self.wander_points {
            let d = CampMath::flat(spot, point);
            if d < best_dist {
                best_dist = d;
                best = spot;
            }
        }
        best
    }

    fn walk_to(&mut self, r_idx: usize, residents: &mut [CampResident], goal: Vec3, next: CampNext, pose: Option<String>) {
        if residents[r_idx].seat.is_some() && next != CampNext::Fire {
            self.free_seat(r_idx, residents);
        }
        residents[r_idx].walk(goal, next, pose);
    }
}
