//! Benchmark mode for measuring rendering throughput (triangles and draw calls at 60 FPS).

use std::path::{Path, PathBuf};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use crate::app::state::WorldState;
use crate::props;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchStepResult {
    pub step: usize,
    pub copies: usize,
    pub triangles: u64,
    pub shadow_triangles: u64,
    pub draw_calls: u64,
    pub median_frame_ms: f64,
    pub p95_frame_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchOutput {
    pub steps: Vec<BenchStepResult>,
    pub triangles_at_60: u64,
    pub triangles_at_30: u64,
    pub extrapolated: bool,
}

pub struct BenchState {
    pub output_path: PathBuf,
    pub step: usize,
    pub step_time: f32,
    pub copies: usize,
    pub step_samples: Vec<f64>,
    pub step_triangles: Vec<u64>,
    pub step_shadow_triangles: Vec<u64>,
    pub step_draw_calls: Vec<u64>,
    pub results: Vec<BenchStepResult>,
    pub fixed_camera: Option<engine::render::Camera>,
    pub fixed_player_pos: Option<Vec3>,
    pub placed_instances: Vec<props::Placed>,
}

impl BenchState {
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            output_path,
            step: 0,
            step_time: 0.0,
            copies: 50,
            step_samples: Vec::new(),
            step_triangles: Vec::new(),
            step_shadow_triangles: Vec::new(),
            step_draw_calls: Vec::new(),
            results: Vec::new(),
            fixed_camera: None,
            fixed_player_pos: None,
            placed_instances: Vec::new(),
        }
    }

    pub fn init_fixed(&mut self, camera: engine::render::Camera, player_pos: Vec3) {
        self.fixed_camera = Some(camera);
        self.fixed_player_pos = Some(player_pos);
    }

    pub fn update_instances(
        &mut self,
        gpu: &engine::Gpu,
        library: &mut props::PropLibrary,
        pack: Option<&pack::ModelPack>,
        world: &WorldState,
    ) {
        if self.placed_instances.len() == self.copies {
            return;
        }
        let Some(pack) = pack else { return };
        let Some(cam) = self.fixed_camera else { return };

        let fwd_3d = cam.forward();
        let fwd = Vec3::new(fwd_3d.x, 0.0, fwd_3d.z).normalize_or(Vec3::NEG_Z);
        let right = fwd.cross(Vec3::Y).normalize_or(Vec3::X);

        const BENCH_MODELS: &[(&str, bool)] = &[
            ("tree_pine_01", false),
            ("tree_oak_big", false),
            ("tree_birch", false),
            ("shed", true),
            ("human_player", false),
            ("car_hatchback_red", false),
        ];

        let mut slots = Vec::new();
        for (name, is_building) in BENCH_MODELS {
            if let Some(slot) = library.get(&gpu.device, pack, name, *is_building) {
                slots.push(slot);
            }
        }
        if slots.is_empty() {
            return;
        }

        self.placed_instances.clear();
        self.placed_instances.reserve(self.copies);

        let n = self.copies;
        let rows = ((n as f32).sqrt().round() as usize).max(1);
        let cols = n.div_ceil(rows);

        for i in 0..n {
            let r = i / cols;
            let c = i % cols;
            let depth_t = if rows > 1 { r as f32 / (rows - 1) as f32 } else { 0.5 };
            let dist = 15.0 + (140.0 - 15.0) * depth_t;

            let half_w = dist * 0.40;
            let lat_t = if cols > 1 { (c as f32 / (cols - 1) as f32) * 2.0 - 1.0 } else { 0.0 };
            let lat = half_w * lat_t;

            let pos_xz = cam.position + fwd * dist + right * lat;
            let y = world.height(pos_xz.x, pos_xz.z);
            let pos = Vec3::new(pos_xz.x, y, pos_xz.z);

            let slot = slots[i % slots.len()];
            let yaw = (i as f32) * 1.618;
            self.placed_instances.push(library.place(slot, pos, yaw, 1.0));
        }
    }

    pub fn step_frame(&mut self, dt: f32, counts: &engine::render::FrameCounts, quit: &mut bool) {
        if dt <= 0.0 {
            return;
        }
        self.step_time += dt;

        // Skip the first 0.5 s of each step for warmup
        if self.step_time >= 0.5 {
            let ms = dt as f64 * 1000.0;
            self.step_samples.push(ms);
            self.step_triangles.push(counts.triangles);
            self.step_shadow_triangles.push(counts.shadow_triangles);
            self.step_draw_calls.push(counts.draws as u64);
        }

        // Each step lasts 3.0 s
        if self.step_time >= 3.0 {
            let triangles = median_u64(&self.step_triangles);
            let shadow_triangles = median_u64(&self.step_shadow_triangles);
            let draw_calls = median_u64(&self.step_draw_calls);

            let res = bench_step_summary(
                self.step,
                self.copies,
                std::mem::take(&mut self.step_samples),
                triangles,
                shadow_triangles,
                draw_calls,
            );

            println!(
                "BENCH step={:>2} copies={:>5} tris={:>8} shadowTris={:>8} draws={:>4} medianMs={:>6.2} p95Ms={:>6.2}",
                res.step, res.copies, res.triangles, res.shadow_triangles, res.draw_calls, res.median_frame_ms, res.p95_frame_ms
            );

            self.results.push(res);
            self.step_triangles.clear();
            self.step_shadow_triangles.clear();
            self.step_draw_calls.clear();

            if bench_should_stop(&self.results, 12, 16.7) {
                let output = bench_compute_output(self.results.clone());
                println!(
                    "BENCH COMPLETE: trianglesAt60={} trianglesAt30={} (extrapolated={})",
                    output.triangles_at_60, output.triangles_at_30, output.extrapolated
                );
                write_bench_json(&self.output_path, &output);
                *quit = true;
            } else {
                self.step += 1;
                self.step_time = 0.0;
                self.copies *= 2;
            }
        }
    }
}

fn write_bench_json(path: &Path, output: &BenchOutput) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string_pretty(output) {
        Ok(json) => {
            if let Err(e) = std::fs::write(path, json) {
                eprintln!("Failed to write bench output to {}: {e}", path.display());
            } else {
                println!("Wrote benchmark results to {}", path.display());
            }
        }
        Err(e) => eprintln!("Failed to serialize bench results: {e}"),
    }
}

fn median_u64(vals: &[u64]) -> u64 {
    if vals.is_empty() {
        return 0;
    }
    let mut sorted = vals.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

pub fn bench_step_summary(
    step: usize,
    copies: usize,
    mut samples: Vec<f64>,
    triangles: u64,
    shadow_triangles: u64,
    draw_calls: u64,
) -> BenchStepResult {
    let n = samples.len();
    if n == 0 {
        return BenchStepResult {
            step,
            copies,
            triangles,
            shadow_triangles,
            draw_calls,
            median_frame_ms: 0.0,
            p95_frame_ms: 0.0,
        };
    }
    samples.sort_by(|a, b| a.total_cmp(b));
    let pct = |p: f64| samples[((n as f64 - 1.0) * p).round() as usize];
    BenchStepResult {
        step,
        copies,
        triangles,
        shadow_triangles,
        draw_calls,
        median_frame_ms: pct(0.5),
        p95_frame_ms: pct(0.95),
    }
}

pub fn bench_should_stop(steps: &[BenchStepResult], max_steps: usize, threshold_ms: f64) -> bool {
    if steps.len() >= max_steps {
        return true;
    }
    if let Some(last) = steps.last()
        && last.median_frame_ms > threshold_ms
    {
        return true;
    }
    false
}

pub fn bench_interpolate_or_extrapolate(steps: &[BenchStepResult], target_ms: f64) -> (u64, bool) {
    if steps.is_empty() {
        return (0, true);
    }
    // Check if target_ms is bracketed by any two adjacent steps
    for i in 1..steps.len() {
        let prev = &steps[i - 1];
        let curr = &steps[i];
        if (prev.median_frame_ms <= target_ms && curr.median_frame_ms >= target_ms)
            || (prev.median_frame_ms >= target_ms && curr.median_frame_ms <= target_ms)
        {
            let diff_m = curr.median_frame_ms - prev.median_frame_ms;
            if diff_m.abs() < 1e-6 {
                return (curr.triangles, false);
            }
            let t = (target_ms - prev.median_frame_ms) / diff_m;
            let tris = prev.triangles as f64 + t * (curr.triangles as f64 - prev.triangles as f64);
            return (tris.round().max(0.0) as u64, false);
        }
    }
    // Target is outside the range: extrapolate from last two steps
    if steps.len() >= 2 {
        let s1 = &steps[steps.len() - 2];
        let s2 = &steps[steps.len() - 1];
        let diff_m = s2.median_frame_ms - s1.median_frame_ms;
        if diff_m.abs() > 1e-6 {
            let slope = (s2.triangles as f64 - s1.triangles as f64) / diff_m;
            let tris = s2.triangles as f64 + (target_ms - s2.median_frame_ms) * slope;
            return (tris.round().max(0.0) as u64, true);
        }
        return (s2.triangles, true);
    }
    // Single step fallback
    let s0 = &steps[0];
    if s0.median_frame_ms > 0.0 {
        let tris = (s0.triangles as f64 * (target_ms / s0.median_frame_ms)).round().max(0.0) as u64;
        return (tris, true);
    }
    (s0.triangles, true)
}

pub fn bench_compute_output(steps: Vec<BenchStepResult>) -> BenchOutput {
    let (triangles_at_60, _) = bench_interpolate_or_extrapolate(&steps, 16.7);
    let (triangles_at_30, extrapolated) = bench_interpolate_or_extrapolate(&steps, 1000.0 / 30.0);
    BenchOutput {
        steps,
        triangles_at_60,
        triangles_at_30,
        extrapolated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bench_step_percentiles() {
        let samples = vec![10.0, 12.0, 14.0, 16.0, 18.0];
        let res = bench_step_summary(0, 50, samples, 1000, 500, 10);
        assert_eq!(res.step, 0);
        assert_eq!(res.copies, 50);
        assert_eq!(res.median_frame_ms, 14.0);
    }

    #[test]
    fn bench_stops_when_median_exceeds_16_7() {
        let steps = vec![
            BenchStepResult {
                step: 0,
                copies: 50,
                triangles: 50000,
                shadow_triangles: 20000,
                draw_calls: 30,
                median_frame_ms: 12.0,
                p95_frame_ms: 13.0,
            },
            BenchStepResult {
                step: 1,
                copies: 100,
                triangles: 100000,
                shadow_triangles: 40000,
                draw_calls: 40,
                median_frame_ms: 18.5,
                p95_frame_ms: 20.0,
            },
        ];
        assert!(bench_should_stop(&steps, 12, 16.7));
    }

    #[test]
    fn bench_stops_at_12_steps() {
        let mut steps = Vec::new();
        for i in 0..12 {
            steps.push(BenchStepResult {
                step: i,
                copies: 50 * (i + 1),
                triangles: 10000 * (i as u64 + 1),
                shadow_triangles: 5000 * (i as u64 + 1),
                draw_calls: 20,
                median_frame_ms: 10.0,
                p95_frame_ms: 11.0,
            });
        }
        assert!(bench_should_stop(&steps, 12, 16.7));
    }

    #[test]
    fn bench_interpolates_target_within_range() {
        let steps = vec![
            BenchStepResult {
                step: 0,
                copies: 50,
                triangles: 100_000,
                shadow_triangles: 50_000,
                draw_calls: 30,
                median_frame_ms: 10.0,
                p95_frame_ms: 11.0,
            },
            BenchStepResult {
                step: 1,
                copies: 100,
                triangles: 200_000,
                shadow_triangles: 100_000,
                draw_calls: 40,
                median_frame_ms: 20.0,
                p95_frame_ms: 22.0,
            },
        ];
        let (tris, extrapolated) = bench_interpolate_or_extrapolate(&steps, 15.0);
        assert_eq!(tris, 150_000);
        assert!(!extrapolated);
    }

    #[test]
    fn bench_extrapolates_target_above_range() {
        let steps = vec![
            BenchStepResult {
                step: 0,
                copies: 50,
                triangles: 100_000,
                shadow_triangles: 50_000,
                draw_calls: 30,
                median_frame_ms: 10.0,
                p95_frame_ms: 11.0,
            },
            BenchStepResult {
                step: 1,
                copies: 100,
                triangles: 200_000,
                shadow_triangles: 100_000,
                draw_calls: 40,
                median_frame_ms: 20.0,
                p95_frame_ms: 22.0,
            },
        ];
        let (tris, extrapolated) = bench_interpolate_or_extrapolate(&steps, 30.0);
        assert_eq!(tris, 300_000);
        assert!(extrapolated);
    }
}
