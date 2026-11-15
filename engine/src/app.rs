use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::capture;
use crate::cli::EngineArgs;
use crate::gpu::Gpu;
use crate::input::Input;
use crate::stats::FrameStats;
use crate::target::{Blit, FrameTarget};
use crate::timing::FrameTimer;

/// Extra numbers a game adds to the stats JSON (draw calls, objects, triangles, ...).
pub type GameStats = serde_json::Map<String, serde_json::Value>;

/// What the game crate plugs into the engine. The engine owns the window, GPU, frame image,
/// screenshots and stats, and calls these each frame.
pub trait Game: 'static {
    /// Once, after the GPU is ready: create pipelines and upload meshes here.
    fn init(&mut self, gpu: &Gpu, args: &EngineArgs);
    /// Game rules for one frame. `dt` is seconds since the last frame.
    fn update(&mut self, input: &Input, dt: f32);
    /// Draw the frame into `target` (its color and depth images).
    fn render(&mut self, gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder);
    /// Extra numbers for the stats JSON and screenshot JSON.
    fn stats(&self) -> GameStats {
        GameStats::new()
    }
    /// A scripted run (snapshot preset, test) returns a path here to have this frame saved as a PNG.
    fn screenshot_request(&mut self) -> Option<PathBuf> {
        None
    }
    /// Return true to end the run (a scripted scene finished, the quit button).
    fn should_quit(&self) -> bool {
        false
    }
    /// Desired mouse cursor capture: Some(true) = capture and hide, Some(false) = release and show, None = no management.
    fn cursor_grab(&self) -> Option<bool> {
        None
    }
}

/// Opens the window and runs the game until it quits, the window is closed, or a command-line limit is hit.
pub fn run<G: Game>(game: G, title: &str, args: EngineArgs) {
    let event_loop = EventLoop::new().expect("could not start the event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App {
        game,
        title: title.to_string(),
        args,
        window: None,
        gpu: None,
        target: None,
        blit: None,
        input: Input::default(),
        timer: FrameTimer::new(),
        stats: FrameStats::default(),
        frame: 0,
        started: Instant::now(),
        f12_count: 0,
        last_cursor_grab: None,
    };
    event_loop.run_app(&mut app).expect("event loop failed");
}

struct App<G: Game> {
    game: G,
    title: String,
    args: EngineArgs,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    target: Option<FrameTarget>,
    blit: Option<Blit>,
    input: Input,
    timer: FrameTimer,
    stats: FrameStats,
    frame: u64,
    started: Instant,
    f12_count: u32,
    last_cursor_grab: Option<bool>,
}

impl<G: Game> App<G> {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let (Some(gpu), Some(window)) = (self.gpu.as_mut(), self.window.as_ref()) else {
            return;
        };
        let (dt, fps_ready) = self.timer.tick();
        self.frame += 1;
        let elapsed = self.started.elapsed().as_secs_f32();
        if elapsed >= self.args.warmup {
            self.stats.push(dt as f64 * 1000.0);
        }
        if fps_ready {
            window.set_title(&format!("{} - {:.0} FPS", self.title, self.timer.fps));
        }

        let f12 = self.input.pressed(KeyCode::F12);
        self.game.update(&self.input, dt);
        self.input.end_frame();

        if let Some(grab) = self.game.cursor_grab()
            && self.last_cursor_grab != Some(grab)
        {
            self.last_cursor_grab = Some(grab);
            Self::apply_cursor_grab(window, grab);
        }

        let texture = match gpu.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t)
            | wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                let (w, h) = (gpu.config.width, gpu.config.height);
                gpu.resize(w, h);
                return;
            }
            _ => return,
        };
        let (Some(target), Some(blit)) = (self.target.as_ref(), self.blit.as_ref()) else {
            return;
        };
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame"),
            });
        self.game.render(gpu, target, &mut encoder);
        blit.draw(&mut encoder, &view);
        gpu.queue.submit([encoder.finish()]);
        window.pre_present_notify();
        gpu.queue.present(texture);

        // Screenshots: scripted (--shot or the game asks) and F12
        let mut shot: Option<(PathBuf, bool)> = self.game.screenshot_request().map(|p| (p, false));
        if self.args.shot.is_some() && self.frame == self.args.shot_frame {
            shot = self.args.shot.clone().map(|p| (p, true));
        }
        if f12 {
            self.f12_count += 1;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            shot = Some((
                PathBuf::from(format!("shots/shot_{stamp}_{}.png", self.f12_count)),
                false,
            ));
        }
        if let Some((path, quit_after)) = shot {
            match capture::save_png(&gpu.device, &gpu.queue, target, &path) {
                Ok(()) => {
                    self.write_json(&path.with_extension("json"));
                    println!("ROTLAND SHOT {}", path.display());
                }
                Err(e) => eprintln!("ROTLAND SHOT FAILED {}: {e}", path.display()),
            }
            if quit_after {
                self.quit(event_loop);
                return;
            }
        }

        let over_frames = self.args.frames.is_some_and(|n| self.frame >= n);
        let over_time = self.args.seconds.is_some_and(|s| elapsed >= s);
        if self.game.should_quit() || over_frames || over_time {
            self.quit(event_loop);
        }
    }

    fn quit(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(path) = self.args.stats.clone() {
            self.write_json(&path);
            println!("ROTLAND STATS {}", path.display());
        }
        let s = self.stats.summary();
        println!(
            "ROTLAND DONE frames={} avgFps={:.1} p95FrameMs={:.2} worst1PercentFps={:.1}",
            self.frame, s.avg_fps, s.p95_frame_ms, s.worst1_percent_fps
        );
        event_loop.exit();
    }

    /// One JSON with everything an AI needs to judge a run: frame times, the setup, and the game's own numbers.
    fn write_json(&self, path: &Path) {
        let mut json = serde_json::to_value(self.stats.summary()).unwrap_or_default();
        if let (Some(obj), Some(gpu), Some(target)) = (
            json.as_object_mut(),
            self.gpu.as_ref(),
            self.target.as_ref(),
        ) {
            obj.insert("gpu".into(), gpu.adapter_info.name.clone().into());
            obj.insert(
                "window".into(),
                format!("{}x{}", gpu.config.width, gpu.config.height).into(),
            );
            obj.insert(
                "render".into(),
                format!("{}x{}", target.width, target.height).into(),
            );
            obj.insert(
                "presentMode".into(),
                format!("{:?}", gpu.config.present_mode).into(),
            );
            obj.insert("frame".into(), self.frame.into());
            obj.insert(
                "seconds".into(),
                self.started.elapsed().as_secs_f64().into(),
            );
            obj.insert("args".into(), self.args.all.clone().into());
            obj.insert("game".into(), serde_json::Value::Object(self.game.stats()));
        }
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(
            path,
            serde_json::to_string_pretty(&json).unwrap_or_default(),
        ) {
            eprintln!("could not write {}: {e}", path.display());
        }
    }

    fn rebuild_target(&mut self) {
        let Some(gpu) = self.gpu.as_ref() else { return };
        let w = (gpu.config.width as f32 * self.args.scale).round() as u32;
        let h = (gpu.config.height as f32 * self.args.scale).round() as u32;
        let target = FrameTarget::new(&gpu.device, w, h);
        match self.blit.as_mut() {
            Some(blit) => blit.retarget(&gpu.device, &target),
            None => self.blit = Some(Blit::new(gpu, &target)),
        }
        self.target = Some(target);
    }

    fn apply_cursor_grab(window: &Window, grab: bool) {
        if grab {
            let _ = window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(winit::window::CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(winit::window::CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }
}

impl<G: Game> ApplicationHandler for App<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title(self.title.clone())
            .with_inner_size(winit::dpi::PhysicalSize::new(
                self.args.width,
                self.args.height,
            ));
        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("could not open the window"),
        );
        let gpu = Gpu::new(window.clone(), self.args.vsync);
        log::info!(
            "GPU: {} ({:?}, {})",
            gpu.adapter_info.name,
            gpu.adapter_info.backend,
            gpu.adapter_info.driver_info
        );
        self.game.init(&gpu, &self.args);
        self.gpu = Some(gpu);
        self.window = Some(window);
        self.rebuild_target();
        self.started = Instant::now();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.quit(event_loop),
            WindowEvent::Resized(size) => {
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(size.width, size.height);
                }
                self.rebuild_target();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    self.input.key(code, event.state == ElementState::Pressed);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.button(button, state == ElementState::Pressed)
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.input.wheel += match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
                };
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if let DeviceEvent::MouseMotion { delta } = event {
            self.input.mouse_delta.0 += delta.0 as f32;
            self.input.mouse_delta.1 += delta.1 as f32;
        }
    }

    /// Frames are drawn here in a loop instead of waiting for the desktop to ask (Wayland asks only
    /// once per screen refresh, which capped every run at 60 FPS).
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.frame(event_loop);
    }
}
