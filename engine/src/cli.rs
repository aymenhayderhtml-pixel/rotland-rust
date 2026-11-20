//! Engine command-line options. Made for AI-driven work: every run can be scripted, timed and photographed.
//!
//! | Option | Meaning |
//! |---|---|
//! | `--shot=PATH` | save a PNG of frame `--shot-frame` (default 120) plus `PATH.json` stats, then quit |
//! | `--shot-frame=N` | which frame `--shot` takes |
//! | `--frames=N` / `--seconds=S` | quit after N frames / S seconds |
//! | `--warmup=S` | frames in the first S seconds are left out of the stats (default 2) |
//! | `--stats=PATH` | write frame-time stats as JSON when the run ends |
//! | `--size=WxH` | window size (default 1920x1080) |
//! | `--scale=F` | render scale 0.5 to 1.0 (default 1.0) |
//! | `--vsync` | cap to the screen rate (default: uncapped) |
//! Anything else (for example `--scene=forest`) is left for the game to read.

use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct EngineArgs {
    pub shot: Option<PathBuf>,
    pub shot_frame: u64,
    pub frames: Option<u64>,
    pub seconds: Option<f32>,
    pub warmup: f32,
    pub stats: Option<PathBuf>,
    pub bench: Option<PathBuf>,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
    pub vsync: bool,
    /// Every argument, so the game can read its own options.
    pub all: Vec<String>,
}

impl EngineArgs {
    pub fn from_env() -> Self {
        Self::parse(std::env::args().skip(1).collect())
    }

    pub fn parse(all: Vec<String>) -> Self {
        let mut a = Self {
            shot: None,
            shot_frame: 120,
            frames: None,
            seconds: None,
            warmup: 2.0,
            stats: None,
            bench: None,
            width: 1920,
            height: 1080,
            scale: 1.0,
            vsync: false,
            all: all.clone(),
        };
        for arg in &all {
            let (key, value) = arg.split_once('=').unwrap_or((arg.as_str(), ""));
            match key {
                "--shot" => a.shot = Some(value.into()),
                "--shot-frame" => a.shot_frame = value.parse().unwrap_or(a.shot_frame),
                "--frames" => a.frames = value.parse().ok(),
                "--seconds" => a.seconds = value.parse().ok(),
                "--warmup" => a.warmup = value.parse().unwrap_or(a.warmup),
                "--stats" => a.stats = Some(value.into()),
                "--bench" => a.bench = Some(value.into()),
                "--scale" => a.scale = value.parse::<f32>().unwrap_or(1.0).clamp(0.5, 1.0),
                "--vsync" => a.vsync = true,
                "--size" => {
                    if let Some((w, h)) = value.split_once('x') {
                        a.width = w.parse().unwrap_or(a.width);
                        a.height = h.parse().unwrap_or(a.height);
                    }
                }
                _ => {}
            }
        }
        a
    }

    /// The value of a game option such as `--scene=forest`, if given.
    pub fn value(&self, key: &str) -> Option<&str> {
        let prefix = format!("{key}=");
        self.all.iter().find_map(|a| a.strip_prefix(prefix.as_str()))
    }

    pub fn flag(&self, key: &str) -> bool {
        self.all.iter().any(|a| a == key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_engine_and_game_options() {
        let a = EngineArgs::parse(vec![
            "--shot=out/a.png".into(),
            "--size=1280x720".into(),
            "--scene=forest".into(),
            "--scale=0.3".into(),
            "--bench=out/bench.json".into(),
        ]);
        assert_eq!(a.shot, Some("out/a.png".into()));
        assert_eq!(a.bench, Some("out/bench.json".into()));
        assert_eq!((a.width, a.height), (1280, 720));
        assert_eq!(a.scale, 0.5);
        assert_eq!(a.value("--scene"), Some("forest"));
        assert!(!a.vsync);
    }
}
