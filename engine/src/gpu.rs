use std::sync::Arc;
use winit::window::Window;

/// The GPU connection: device, queue and the window's swapchain. The depth buffer lives in FrameTarget.
pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub adapter_info: wgpu::AdapterInfo,
}

pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

impl Gpu {
    /// Vulkan only: the owner's laptop (Intel UHD, Mesa) is the one target.
    pub fn new(window: Arc<Window>, vsync: bool) -> Self {
        let desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..wgpu::InstanceDescriptor::new_without_display_handle()
        }
        .with_env();
        let instance = wgpu::Instance::new(desc);
        let surface = instance.create_surface(window.clone()).expect("could not create the window surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("no Vulkan GPU found");
        let adapter_info = adapter.get_info();
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("rotland"),
            required_features: wgpu::Features::empty(),
            required_limits: adapter.limits(),
            ..Default::default()
        }))
        .expect("could not open the GPU");

        let size = window.inner_size();
        let mut config = surface
            .get_default_config(&adapter, size.width.max(1), size.height.max(1))
            .expect("the surface does not work with this GPU");
        let modes = surface.get_capabilities(&adapter).present_modes;
        log::info!("present modes: {modes:?}");
        // Uncapped: Mailbox (no tearing) first, then Immediate. Wayland may offer only Fifo (capped).
        config.present_mode = if vsync {
            wgpu::PresentMode::AutoVsync
        } else if modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else if modes.contains(&wgpu::PresentMode::Immediate) {
            wgpu::PresentMode::Immediate
        } else {
            wgpu::PresentMode::AutoNoVsync
        };
        config.desired_maximum_frame_latency = 2;
        surface.configure(&device, &config);
        Self { surface, device, queue, config, adapter_info }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }
        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

}
