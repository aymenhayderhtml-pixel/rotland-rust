//! Rotland's sound (port of scripts/Audio): one-shot sounds at a point or in the ears, the day and night
//! ambience loops, and the master/music/sfx volumes. Uses kira. If there is no sound device, or the game
//! runs muted (tests, screenshots), everything is a silent no-op.

mod library;
mod mix;

use std::collections::HashMap;
use std::time::Duration;

use glam::Vec3;
use kira::sound::static_sound::{StaticSoundData, StaticSoundHandle};
use kira::{AudioManager, AudioManagerSettings, Decibels, DefaultBackend, Panning, PlaybackRate, Tween};

pub use library::{EventDef, SoundData, SoundDef};
pub use mix::{distance_db, linear_to_db, pan};

/// Seconds for the day/night ambience to cross over (AmbienceMixer.FadeSeconds).
const AMBIENCE_FADE: f32 = 3.0;

pub struct Audio {
    manager: Option<AudioManager<DefaultBackend>>,
    pub data: SoundData,
    /// Decoded files per sound id (one entry per file; a random one plays).
    clips: HashMap<String, Vec<StaticSoundData>>,
    listener: Vec3,
    forward: Vec3,
    master: f32,
    music: f32,
    sfx: f32,
    day: Option<StaticSoundHandle>,
    night: Option<StaticSoundHandle>,
    /// 0 = full day, 1 = full night.
    night_mix: Option<f32>,
    /// Continuous RPM-pitched engine loop (StaticSoundHandle + load-aware volume/rate).
    car_engine: Option<StaticSoundHandle>,
    car_engine_active: bool,
    rng: u64,
    /// How many sounds were started (stats for tests).
    pub played: u32,
    pub last_played: String,
}

impl Audio {
    /// Opens the sound device and loads every sound. `muted` (tests, screenshots) opens nothing.
    pub fn new(muted: bool) -> Self {
        let data = SoundData::from_repo().unwrap_or_else(|e| {
            log::warn!("{e}");
            SoundData::default()
        });
        let manager = if muted {
            None
        } else {
            match AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()) {
                Ok(m) => Some(m),
                Err(e) => {
                    log::warn!("no sound device ({e}); playing silent");
                    None
                }
            }
        };
        let mut clips = HashMap::new();
        if manager.is_some() {
            for (id, def) in &data.sounds {
                let loaded: Vec<StaticSoundData> = def
                    .files
                    .iter()
                    .filter_map(|f| match StaticSoundData::from_file(SoundData::file_path(f)) {
                        Ok(s) => Some(s),
                        Err(e) => {
                            log::warn!("sound {f}: {e}");
                            None
                        }
                    })
                    .collect();
                clips.insert(id.clone(), loaded);
            }
        }
        Self {
            manager,
            data,
            clips,
            listener: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            master: 1.0,
            music: 1.0,
            sfx: 1.0,
            day: None,
            night: None,
            night_mix: None,
            car_engine: None,
            car_engine_active: false,
            rng: 0x2545_F491_4F6C_DD1D,
            played: 0,
            last_played: String::new(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.manager.is_some()
    }

    fn rand(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng >> 40) as f32 / (1u64 << 24) as f32
    }

    /// The camera: where sounds are heard from.
    pub fn set_listener(&mut self, position: Vec3, forward: Vec3) {
        self.listener = position;
        self.forward = forward;
    }

    /// Settings sliders (0..1).
    pub fn set_volumes(&mut self, master: f32, music: f32, sfx: f32) {
        self.master = master;
        self.music = music;
        self.sfx = sfx;
    }

    fn bus_db(&self, bus: &str) -> f32 {
        let bus_gain = if bus == "Ambient" { self.music } else { self.sfx };
        linear_to_db(self.master) + linear_to_db(bus_gain)
    }

    /// A sound at a point in the world (AudioManager.PlayAt). Non-spatial sounds play in the ears.
    pub fn play_at(&mut self, id: &str, at: Vec3) {
        let Some(def) = self.data.sounds.get(id).cloned() else { return };
        let (dist_db, panning) = if def.spatial {
            let Some(db) = distance_db(at.distance(self.listener), def.max_distance) else { return };
            (db, pan(self.listener, self.forward, at))
        } else {
            (0.0, 0.0)
        };
        self.start(id, &def, dist_db, panning);
    }

    /// A sound in the ears (AudioManager.PlayUi).
    pub fn play_ui(&mut self, id: &str) {
        let Some(def) = self.data.sounds.get(id).cloned() else { return };
        self.start(id, &def, 0.0, 0.0);
    }

    /// A named game event from sounds.json "events" (reload, pickup, levelUp...).
    pub fn event(&mut self, name: &str, at: Option<Vec3>) {
        let Some(sound) = self.data.events.get(name).map(|e| e.sound.clone()) else { return };
        match at {
            Some(p) => self.play_at(&sound, p),
            None => self.play_ui(&sound),
        }
    }

    fn start(&mut self, id: &str, def: &SoundDef, extra_db: f32, panning: f32) {
        self.played += 1;
        self.last_played = id.to_string();
        let count = self.clips.get(id).map_or(0, Vec::len);
        if count == 0 || self.manager.is_none() {
            return;
        }
        let pick = ((self.rand() * count as f32) as usize).min(count - 1);
        let jitter = 1.0 + (self.rand() * 2.0 - 1.0) * def.pitch_jitter;
        let volume = def.volume_db + extra_db + self.bus_db(&def.bus);
        let sound = self.clips[id][pick]
            .volume(Decibels(volume))
            .panning(Panning(panning))
            .playback_rate(PlaybackRate(jitter as f64));
        if let Some(m) = self.manager.as_mut()
            && let Err(e) = m.play(sound)
        {
            log::debug!("sound {id} not played: {e}");
        }
    }

    /// Once a frame: the day and night loops cross over with the clock.
    pub fn tick(&mut self, dt: f32, night: bool) {
        if self.manager.is_none() {
            return;
        }
        if self.day.is_none() {
            self.day = self.start_loop("day_loop");
            self.night = self.start_loop("night_loop");
        }
        let target = if night { 1.0 } else { 0.0 };
        // Start at the right mix straight away, without a fade from day (AmbienceMixer)
        let mix = match self.night_mix {
            None => target,
            Some(m) => {
                let step = dt / AMBIENCE_FADE;
                if m < target { (m + step).min(target) } else { (m - step).max(target) }
            }
        };
        self.night_mix = Some(mix);
        for (handle, gain, id) in [(self.day.as_mut(), 1.0 - mix, "day_loop"), (self.night.as_mut(), mix, "night_loop")] {
            let Some(h) = handle else { continue };
            let base = self.data.sounds.get(id).map_or(0.0, |d| d.volume_db);
            let bus = linear_to_db(self.master) + linear_to_db(self.music);
            h.set_volume(Decibels(base + bus + linear_to_db(gain)), Tween { duration: Duration::from_millis(100), ..Default::default() });
        }
    }

    /// Drives the continuous RPM-pitched engine loop. `active = false` fades it out.
    pub fn set_car_engine(&mut self, active: bool, rpm: f32) {
        self.car_engine_active = active;
        if self.manager.is_none() {
            return;
        }
        if !active {
            if let Some(mut h) = self.car_engine.take() {
                h.stop(Tween {
                    duration: Duration::from_millis(150),
                    ..Default::default()
                });
            }
            return;
        }
        let bus = self.bus_db("Sfx");
        if self.car_engine.is_none() {
            self.car_engine = self.start_loop("car_engine_loop");
        }
        if let Some(h) = self.car_engine.as_mut() {
            let rpm = rpm.clamp(0.0, 1.0);
            let base = self.data.sounds.get("car_engine_loop").map_or(0.0, |d| d.volume_db);
            let load_db = -7.0 + rpm * 6.5;
            let rate = 0.72 + rpm * 1.18;
            let tween = Tween {
                duration: Duration::from_millis(80),
                ..Default::default()
            };
            h.set_volume(Decibels(base + bus + load_db), tween);
            h.set_playback_rate(PlaybackRate(rate as f64), tween);
        }
    }

    pub fn is_car_engine_active(&self) -> bool {
        self.car_engine_active
    }

    fn start_loop(&mut self, id: &str) -> Option<StaticSoundHandle> {
        let data = self.clips.get(id)?.first()?.loop_region(..).volume(Decibels::SILENCE);
        self.manager.as_mut()?.play(data).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn muted_car_engine_toggle_is_safe() {
        let mut audio = Audio::new(true);
        audio.set_car_engine(true, 0.5);
        assert!(audio.is_car_engine_active());
        audio.set_car_engine(true, 1.0);
        audio.set_car_engine(false, 0.0);
        assert!(!audio.is_car_engine_active());
    }
}
