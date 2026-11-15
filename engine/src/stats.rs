use serde::Serialize;

/// Frame times of one run, summarised the same way as the Godot benchmark JSON (so the two can be compared).
#[derive(Default)]
pub struct FrameStats {
    pub frame_ms: Vec<f64>,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatsSummary {
    pub frames: usize,
    pub avg_fps: f64,
    pub p50_frame_ms: f64,
    pub p95_frame_ms: f64,
    pub p99_frame_ms: f64,
    pub worst1_percent_fps: f64,
}

impl FrameStats {
    pub fn push(&mut self, ms: f64) {
        self.frame_ms.push(ms);
    }

    pub fn summary(&self) -> StatsSummary {
        let n = self.frame_ms.len();
        if n == 0 {
            return StatsSummary { frames: 0, avg_fps: 0.0, p50_frame_ms: 0.0, p95_frame_ms: 0.0, p99_frame_ms: 0.0, worst1_percent_fps: 0.0 };
        }
        let mut sorted = self.frame_ms.clone();
        sorted.sort_by(|a, b| a.total_cmp(b));
        let pct = |p: f64| sorted[((n as f64 - 1.0) * p).round() as usize];
        let total: f64 = sorted.iter().sum();
        let worst_count = (n / 100).max(1);
        let worst_avg = sorted[n - worst_count..].iter().sum::<f64>() / worst_count as f64;
        StatsSummary {
            frames: n,
            avg_fps: 1000.0 * n as f64 / total,
            p50_frame_ms: pct(0.5),
            p95_frame_ms: pct(0.95),
            p99_frame_ms: pct(0.99),
            worst1_percent_fps: 1000.0 / worst_avg,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steady_10ms_frames_are_100_fps() {
        let mut s = FrameStats::default();
        for _ in 0..200 {
            s.push(10.0);
        }
        let sum = s.summary();
        assert!((sum.avg_fps - 100.0).abs() < 1e-6);
        assert_eq!(sum.p95_frame_ms, 10.0);
    }
}
