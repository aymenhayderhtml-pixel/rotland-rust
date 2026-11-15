//! The in-game clock, ported from Godot's TimeOfDay.cs.
//! One in-game hour takes `minutes_per_game_hour` real minutes (1.0 by default), so a day is 24 minutes.

/// Records what transitions occurred when the clock advanced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClockChanges {
    /// Whole hours that were crossed during this advance step, in order.
    pub hours_passed: Vec<u32>,
    /// New day numbers that started (e.g., day 2 when crossing midnight).
    pub days_started: Vec<i32>,
    /// True if hour 20:00 (night start) was crossed.
    pub night_started: bool,
    /// True if hour 05:00 (day break) was crossed.
    pub day_break: bool,
}

impl ClockChanges {
    /// True if at least one new day started during this step.
    pub fn new_day(&self) -> bool {
        !self.days_started.is_empty()
    }
}

/// The in-game clock.
#[derive(Debug, Clone, PartialEq)]
pub struct Clock {
    /// Hour of the day, from 0.0 up to (not including) 24.0.
    pub hour: f32,
    /// Day number, starting at 1.
    pub day: i32,
    /// Real minutes per in-game hour (default 1.0).
    pub minutes_per_game_hour: f32,
    /// Total in-game hours that advance has moved the clock.
    pub elapsed_hours: f64,
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock {
    pub const START_HOUR: f32 = 8.0;
    pub const NIGHT_START_HOUR: i32 = 20;
    pub const DAY_BREAK_HOUR: i32 = 5;

    pub fn new() -> Self {
        Self {
            hour: Self::START_HOUR,
            day: 1,
            minutes_per_game_hour: 1.0,
            elapsed_hours: 0.0,
        }
    }

    pub fn with_minutes_per_game_hour(minutes: f32) -> Self {
        Self {
            minutes_per_game_hour: minutes,
            ..Self::new()
        }
    }

    /// Night runs from 20:00 to 05:00.
    pub fn is_night(&self) -> bool {
        self.hour >= Self::NIGHT_START_HOUR as f32 || self.hour < Self::DAY_BREAK_HOUR as f32
    }

    /// Day runs from 05:00 to 20:00.
    pub fn is_day(&self) -> bool {
        !self.is_night()
    }

    /// Jumps straight to an hour (wrapped to [0, 24)) without raising events.
    pub fn set_time(&mut self, hour: f32) {
        self.hour = hour.rem_euclid(24.0);
    }

    /// "21:30" for a fractional hour.
    pub fn format_clock(hour: f32) -> String {
        let minutes = ((hour.rem_euclid(24.0) * 60.0 + 0.5).floor() as i32) % (24 * 60);
        format!("{:02}:{:02}", minutes / 60, minutes % 60)
    }

    /// Formats the clock's current hour.
    pub fn format(&self) -> String {
        Self::format_clock(self.hour)
    }

    /// Advances the clock by `dt` real seconds (converted via `minutes_per_game_hour`).
    pub fn advance(&mut self, dt: f32) -> ClockChanges {
        if self.minutes_per_game_hour <= 0.0 || dt <= 0.0 {
            return ClockChanges::default();
        }
        let hours = dt / (60.0 * self.minutes_per_game_hour);
        self.advance_hours(hours)
    }

    /// Moves the clock forward by some in-game hours, crossing whole hours in order.
    pub fn advance_hours(&mut self, hours: f32) -> ClockChanges {
        if hours <= 0.0 {
            return ClockChanges::default();
        }
        self.elapsed_hours += hours as f64;
        let mut changes = ClockChanges::default();
        let mut remaining = hours;

        while remaining > 0.0 {
            let to_next_hour = self.hour.floor() + 1.0 - self.hour;
            if remaining < to_next_hour {
                self.hour += remaining;
                break;
            }
            remaining -= to_next_hour;
            self.hour = self.hour.floor() + 1.0;
            self.cross_whole_hour(&mut changes);
        }

        changes
    }

    fn cross_whole_hour(&mut self, changes: &mut ClockChanges) {
        if self.hour >= 24.0 {
            self.hour -= 24.0;
            self.day += 1;
            changes.days_started.push(self.day);
        }
        let h = self.hour.floor() as u32;
        changes.hours_passed.push(h);
        if h == Self::NIGHT_START_HOUR as u32 {
            changes.night_started = true;
        } else if h == Self::DAY_BREAK_HOUR as u32 {
            changes.day_break = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let clock = Clock::new();
        assert_eq!(clock.hour, 8.0);
        assert_eq!(clock.day, 1);
        assert_eq!(clock.elapsed_hours, 0.0);
        assert_eq!(clock.minutes_per_game_hour, 1.0);
        assert!(clock.is_day());
        assert!(!clock.is_night());
        assert_eq!(clock.format(), "08:00");
    }

    #[test]
    fn format_clock_examples() {
        assert_eq!(Clock::format_clock(0.0), "00:00");
        assert_eq!(Clock::format_clock(8.0), "08:00");
        assert_eq!(Clock::format_clock(21.5), "21:30");
        assert_eq!(Clock::format_clock(23.999), "00:00");
        assert_eq!(Clock::format_clock(-1.0), "23:00");
    }

    #[test]
    fn set_time_wraps_modulo_24() {
        let mut clock = Clock::new();
        clock.set_time(14.5);
        assert_eq!(clock.hour, 14.5);
        clock.set_time(25.0);
        assert_eq!(clock.hour, 1.0);
        clock.set_time(-2.0);
        assert_eq!(clock.hour, 22.0);
    }

    #[test]
    fn advance_fractional_hour_within_same_hour() {
        let mut clock = Clock::new();
        let changes = clock.advance_hours(0.5);
        assert_eq!(clock.hour, 8.5);
        assert_eq!(clock.elapsed_hours, 0.5);
        assert!(changes.hours_passed.is_empty());
        assert!(!changes.new_day());
    }

    #[test]
    fn advance_crosses_whole_hours() {
        let mut clock = Clock::new();
        let changes = clock.advance_hours(2.5);
        assert_eq!(clock.hour, 10.5);
        assert_eq!(changes.hours_passed, vec![9, 10]);
        assert!(!changes.night_started);
        assert!(!changes.day_break);
    }

    #[test]
    fn advance_night_start() {
        let mut clock = Clock::new();
        clock.set_time(19.0);
        let changes = clock.advance_hours(1.5);
        assert_eq!(clock.hour, 20.5);
        assert!(changes.night_started);
        assert!(clock.is_night());
        assert!(!clock.is_day());
    }

    #[test]
    fn advance_day_break() {
        let mut clock = Clock::new();
        clock.set_time(4.0);
        assert!(clock.is_night());
        let changes = clock.advance_hours(1.5);
        assert_eq!(clock.hour, 5.5);
        assert!(changes.day_break);
        assert!(clock.is_day());
        assert!(!clock.is_night());
    }

    #[test]
    fn advance_crosses_midnight_new_day() {
        let mut clock = Clock::new();
        clock.set_time(23.0);
        let changes = clock.advance_hours(2.0);
        assert_eq!(clock.hour, 1.0);
        assert_eq!(clock.day, 2);
        assert_eq!(changes.days_started, vec![2]);
        assert!(changes.new_day());
        assert_eq!(changes.hours_passed, vec![0, 1]);
    }

    #[test]
    fn advance_with_real_seconds() {
        let mut clock = Clock::with_minutes_per_game_hour(1.0);
        // 60 real seconds = 1 game hour (since minutes_per_game_hour = 1.0)
        let changes = clock.advance(60.0);
        assert_eq!(clock.hour, 9.0);
        assert_eq!(changes.hours_passed, vec![9]);
    }
}
