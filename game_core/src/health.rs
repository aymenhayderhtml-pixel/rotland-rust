use crate::damage::DamageInfo;

/// Why a hit did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IgnoredReason {
    Dead,
    /// Still inside the invulnerability window after the last hit.
    Invulnerable,
}

/// What one `Health::take_damage` call did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageResult {
    /// Health actually removed. Never more than was left, and 0 when the hit was ignored.
    pub dealt: f32,
    /// True only on the hit that brought health to 0.
    pub died_now: bool,
    pub ignored: Option<IgnoredReason>,
}

/// The Godot signals as data. The caller drains them with `Health::drain_events`, so no callbacks.
#[derive(Clone, Debug, PartialEq)]
pub enum HealthEvent {
    Changed { current: f32, max: f32 },
    Damaged { amount: f32, source: Option<u64> },
    Died,
}

/// Hit points for anything that can be damaged (port of scripts/Combat/Health.cs).
#[derive(Clone, Debug)]
pub struct Health {
    max: f32,
    current: f32,
    /// Seconds after a hit during which further hits are ignored. 0 turns this off.
    invulnerability_time: f32,
    dead: bool,
    /// Seconds since the last accepted hit. None until the first hit (and after a revive), so a fresh
    /// body is never invulnerable.
    since_hit: Option<f32>,
    events: Vec<HealthEvent>,
}

impl Health {
    /// Full health, no invulnerability window.
    pub fn new(max: f32) -> Self {
        Self {
            max,
            current: max,
            invulnerability_time: 0.0,
            dead: false,
            since_hit: None,
            events: Vec::new(),
        }
    }

    /// Sets health directly (loading a save). 0 or less counts as full health, like a revive.
    pub fn set_current(&mut self, value: f32) {
        self.dead = false;
        self.current = if value <= 0.0 { self.max } else { value.min(self.max) };
    }

    /// Sets the invulnerability window in seconds (0 = off).
    pub fn with_invulnerability(mut self, seconds: f32) -> Self {
        self.invulnerability_time = seconds;
        self
    }

    pub fn max(&self) -> f32 {
        self.max
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn is_dead(&self) -> bool {
        self.dead
    }

    pub fn invulnerability_time(&self) -> f32 {
        self.invulnerability_time
    }

    /// Advances the invulnerability clock. Call once per game step with the step length in seconds, so
    /// the window is game time and not wall time.
    pub fn tick(&mut self, dt: f32) {
        if let Some(t) = self.since_hit {
            self.since_hit = Some(t + dt);
        }
    }

    /// Applies one hit. Dead bodies and hits inside the invulnerability window are ignored.
    pub fn take_damage(&mut self, info: DamageInfo) -> DamageResult {
        if self.dead {
            return ignored(IgnoredReason::Dead);
        }
        if self.is_invulnerable() {
            return ignored(IgnoredReason::Invulnerable);
        }
        // Negative damage would heal through the back door; treat it as zero.
        let amount = info.amount.max(0.0);
        let dealt = amount.min(self.current);
        self.current = (self.current - amount).max(0.0);
        self.since_hit = Some(0.0);

        self.events.push(HealthEvent::Damaged { amount: dealt, source: info.source });
        self.push_changed();

        let died_now = self.current <= 0.0;
        if died_now {
            self.die();
        }
        DamageResult { dealt, died_now, ignored: None }
    }

    /// Adds health up to max. Does nothing on a dead body.
    pub fn heal(&mut self, amount: f32) {
        if self.dead {
            return;
        }
        self.current = (self.current + amount.max(0.0)).min(self.max);
        self.push_changed();
    }

    /// Brings the body back. Full health unless an amount is given (clamped to 0..=max). Clears the
    /// invulnerability window too, as in the C# Revive.
    pub fn revive(&mut self, amount: Option<f32>) {
        self.dead = false;
        self.since_hit = None;
        self.current = amount.unwrap_or(self.max).max(0.0).min(self.max);
        self.push_changed();
    }

    /// Kills instantly, ignoring invulnerability. Does nothing if already dead.
    pub fn kill(&mut self) {
        if self.dead {
            return;
        }
        self.current = 0.0;
        self.push_changed();
        self.die();
    }

    /// Returns the events since the last call, oldest first.
    pub fn drain_events(&mut self) -> Vec<HealthEvent> {
        std::mem::take(&mut self.events)
    }

    fn is_invulnerable(&self) -> bool {
        match self.since_hit {
            Some(t) if self.invulnerability_time > 0.0 => t < self.invulnerability_time,
            _ => false,
        }
    }

    fn push_changed(&mut self) {
        self.events.push(HealthEvent::Changed { current: self.current, max: self.max });
    }

    fn die(&mut self) {
        self.dead = true;
        self.events.push(HealthEvent::Died);
    }
}

fn ignored(reason: IgnoredReason) -> DamageResult {
    DamageResult { dealt: 0.0, died_now: false, ignored: Some(reason) }
}
