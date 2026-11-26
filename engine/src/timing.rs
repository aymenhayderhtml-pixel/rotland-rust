use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

/// Frame timer: delta time for the game, and an FPS figure refreshed once a second.
pub struct FrameTimer {
    last: Instant,
    window_start: Instant,
    frames: u32,
    pub fps: f32,
}

impl FrameTimer {
    pub fn new() -> Self {
        let now = Instant::now();
        Self { last: now, window_start: now, frames: 0, fps: 0.0 }
    }

    /// Seconds since the last call (capped so a long stall does not explode the simulation),
    /// and whether the FPS figure was just refreshed.
    pub fn tick(&mut self) -> (f32, bool) {
        let now = Instant::now();
        let dt = (now - self.last).as_secs_f32().min(0.1);
        self.last = now;
        self.frames += 1;
        let span = (now - self.window_start).as_secs_f32();
        if span >= 1.0 {
            self.fps = self.frames as f32 / span;
            self.frames = 0;
            self.window_start = now;
            return (dt, true);
        }
        (dt, false)
    }
}

impl Default for FrameTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(serde::Serialize, Default, Clone, Debug, PartialEq)]
pub struct FramePhases {
    pub input: f64,
    #[serde(rename = "world streaming")]
    pub world_streaming: f64,
    pub physics: f64,
    #[serde(rename = "zombies/actors")]
    pub zombies_actors: f64,
    pub camp: f64,
    #[serde(rename = "ui/egui")]
    pub ui_egui: f64,
    #[serde(rename = "render submit")]
    pub render_submit: f64,
    #[serde(rename = "GPU wait/present")]
    pub gpu_wait_present: f64,
    pub render: f64,
    /// The whole game update (simulation, streaming, actors); the named phases above are parts of it.
    #[serde(rename = "game update (all)")]
    pub update_all: f64,
    /// Waiting for the next screen image from the window system.
    #[serde(rename = "surface acquire")]
    pub surface_acquire: f64,
    #[serde(rename = "world_streaming")]
    pub world_streaming_alias: f64,
    #[serde(rename = "zombies_actors")]
    pub zombies_actors_alias: f64,
    #[serde(rename = "ui_egui")]
    pub ui_egui_alias: f64,
    #[serde(rename = "render_submit")]
    pub render_submit_alias: f64,
    #[serde(rename = "gpu_wait_present")]
    pub gpu_wait_present_alias: f64,
}

impl FramePhases {
    pub fn record_phase(&mut self, phase: &str, ms: f64) {
        match phase {
            "input" => self.input += ms,
            "world streaming" | "world_streaming" => {
                self.world_streaming += ms;
                self.world_streaming_alias += ms;
            }
            "physics" => self.physics += ms,
            "zombies/actors" | "zombies_actors" | "actors" => {
                self.zombies_actors += ms;
                self.zombies_actors_alias += ms;
            }
            "camp" => self.camp += ms,
            "ui/egui" | "ui_egui" | "ui" => {
                self.ui_egui += ms;
                self.ui_egui_alias += ms;
            }
            "render submit" | "render_submit" => {
                self.render_submit += ms;
                self.render_submit_alias += ms;
            }
            "GPU wait/present" | "gpu_wait_present" | "present" => {
                self.gpu_wait_present += ms;
                self.gpu_wait_present_alias += ms;
            }
            "render" => self.render += ms,
            "game update (all)" => self.update_all += ms,
            "surface acquire" => self.surface_acquire += ms,
            _ => {}
        }
    }
}

#[derive(serde::Serialize, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameEvents {
    #[serde(rename = "new chunk built")]
    pub new_chunks: u32,
    #[serde(rename = "new pipeline/shader created")]
    pub new_pipelines: u32,
    #[serde(rename = "new mesh uploaded")]
    pub new_meshes: u32,
    #[serde(rename = "save written")]
    pub saves_written: u32,
}

impl FrameEvents {
    pub fn count(&mut self, event: &str) {
        match event {
            "new chunk built" | "new_chunk" => self.new_chunks += 1,
            "new pipeline/shader created" | "new_pipeline" => self.new_pipelines += 1,
            "new mesh uploaded" | "new_mesh" => self.new_meshes += 1,
            "save written" | "save" => self.saves_written += 1,
            _ => {}
        }
    }
}

#[derive(serde::Serialize, Clone, Debug, PartialEq)]
pub struct SpikeRecord {
    pub frame: u64,
    /// The larger of `work_ms` and `since_last_ms`.
    pub total_ms: f64,
    /// Time spent inside this frame's own work (update, render, present).
    pub work_ms: f64,
    /// Time from the previous frame's start to this one's (includes everything between frames).
    pub since_last_ms: f64,
    #[serde(flatten)]
    pub phases: FramePhases,
    #[serde(flatten)]
    pub events: FrameEvents,
}

struct ActiveFrame {
    start: Instant,
    render_start: Option<Instant>,
    phases: FramePhases,
    events: FrameEvents,
}

struct SpikeRecorderState {
    file: Option<File>,
    active: Option<ActiveFrame>,
}

thread_local! {
    static SPIKE_RECORDER: RefCell<SpikeRecorderState> = const {
        RefCell::new(SpikeRecorderState {
            file: None,
            active: None,
        })
    };
}

pub fn init_spike_recorder(path: Option<&Path>) {
    SPIKE_RECORDER.with(|r| {
        let mut st = r.borrow_mut();
        st.file = path.and_then(|p| {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            File::create(p).ok()
        });
        st.active = None;
    });
}

pub fn start_frame() {
    SPIKE_RECORDER.with(|r| {
        let mut st = r.borrow_mut();
        st.active = Some(ActiveFrame {
            start: Instant::now(),
            render_start: None,
            phases: FramePhases::default(),
            events: FrameEvents::default(),
        });
    });
}

pub fn mark_render_start() {
    SPIKE_RECORDER.with(|r| {
        if let Some(act) = r.borrow_mut().active.as_mut() {
            act.render_start = Some(Instant::now());
        }
    });
}

pub fn mark_render_draw() {
    SPIKE_RECORDER.with(|r| {
        if let Some(act) = r.borrow_mut().active.as_mut()
            && let Some(start) = act.render_start.take()
        {
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            act.phases.record_phase("world streaming", ms);
        }
    });
}

pub fn record_phase(phase: &str, ms: f64) {
    SPIKE_RECORDER.with(|r| {
        if let Some(act) = r.borrow_mut().active.as_mut() {
            act.phases.record_phase(phase, ms);
        }
    });
}

pub fn count_event(event: &str) {
    SPIKE_RECORDER.with(|r| {
        if let Some(act) = r.borrow_mut().active.as_mut() {
            act.events.count(event);
        }
    });
}

pub fn count_new_chunk() {
    count_event("new chunk built");
}

pub fn count_new_pipeline() {
    count_event("new pipeline/shader created");
}

pub fn count_new_mesh() {
    count_event("new mesh uploaded");
}

pub fn count_save_written() {
    count_event("save written");
}

pub fn finish_frame(frame: u64, dt_ms: f64) -> Option<SpikeRecord> {
    SPIKE_RECORDER.with(|r| {
        let mut st = r.borrow_mut();
        let act = st.active.take()?;
        let work_ms = act.start.elapsed().as_secs_f64() * 1000.0;
        let total_ms = work_ms.max(dt_ms);
        if total_ms > 20.0 {
            let record = SpikeRecord {
                frame,
                total_ms,
                work_ms,
                since_last_ms: dt_ms,
                phases: act.phases,
                events: act.events,
            };
            if let Some(file) = st.file.as_mut()
                && let Ok(line) = serde_json::to_string(&record)
            {
                let _ = writeln!(file, "{line}");
                let _ = file.flush();
            }
            Some(record)
        } else {
            None
        }
    })
}

pub struct PhaseScope {
    phase: &'static str,
    start: Instant,
}

impl PhaseScope {
    pub fn new(phase: &'static str) -> Self {
        Self {
            phase,
            start: Instant::now(),
        }
    }
}

impl Drop for PhaseScope {
    fn drop(&mut self) {
        let ms = self.start.elapsed().as_secs_f64() * 1000.0;
        record_phase(self.phase, ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spike_phases_record() {
        let mut p = FramePhases::default();
        p.record_phase("input", 1.5);
        p.record_phase("world streaming", 2.5);
        assert!((p.input - 1.5).abs() < 1e-6);
        assert!((p.world_streaming - 2.5).abs() < 1e-6);
        assert!((p.world_streaming_alias - 2.5).abs() < 1e-6);
    }

    #[test]
    fn test_spike_events_count() {
        let mut ev = FrameEvents::default();
        ev.count("new chunk built");
        ev.count("new mesh uploaded");
        assert_eq!(ev.new_chunks, 1);
        assert_eq!(ev.new_meshes, 1);
    }
}
