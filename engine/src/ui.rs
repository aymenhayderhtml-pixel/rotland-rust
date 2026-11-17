//! egui UI integration on top of wgpu and winit for Rotland.
//! Draws into FrameTarget's color texture so that screenshots and the blit include the UI.

use winit::raw_window_handle::HasDisplayHandle;
use winit::window::Window;

use crate::gpu::Gpu;
use crate::target::FrameTarget;

/// Owns the egui context, winit event state, and wgpu renderer.
pub struct UiLayer {
    pub ctx: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,
}

impl UiLayer {
    pub fn new(
        device: &wgpu::Device,
        output_color_format: wgpu::TextureFormat,
        window: &Window,
    ) -> Self {
        let ctx = egui::Context::default();
        let viewport_id = ctx.viewport_id();
        let state = egui_winit::State::new(
            ctx.clone(),
            viewport_id,
            window as &dyn HasDisplayHandle,
            Some(window.scale_factor() as f32),
            window.theme(),
            Some(device.limits().max_texture_dimension_2d as usize),
        );
        let renderer = egui_wgpu::Renderer::new(
            device,
            output_color_format,
            egui_wgpu::RendererOptions::default(),
        );
        Self { ctx, state, renderer }
    }

    /// Forward a window event from winit into egui.
    pub fn on_window_event(
        &mut self,
        window: &Window,
        event: &winit::event::WindowEvent,
    ) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    /// Run UI building and render shapes into `target.color` if any UI was drawn.
    /// Returns any extra user command buffers to submit to the GPU queue.
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &FrameTarget,
        window: &Window,
        encoder: &mut wgpu::CommandEncoder,
        cursor_grabbed: bool,
        mut build_ui: impl FnMut(&egui::Context),
    ) -> Vec<wgpu::CommandBuffer> {
        let raw_input = self.state.take_egui_input(window);
        // egui 0.37: run_pass (it may call the UI code more than once in a frame for layout passes)
        let mut output = self.ctx.run_pass(raw_input, |ctx| build_ui(ctx));

        // When the game holds the mouse cursor, don't let egui modify cursor visibility or icon
        if !cursor_grabbed {
            self.state.handle_platform_output(window, output.platform_output);
        }

        // Texture uploads (font atlas) must be applied even on frames with no shapes, or later text is blank
        for (id, image_deltas) in &output.textures_delta.set {
            for image_delta in image_deltas {
                self.renderer
                    .update_texture(&gpu.device, &gpu.queue, *id, image_delta);
            }
        }

        // Keep 3D path unchanged when no UI was drawn
        if output.shapes.is_empty() {
            for id in &output.textures_delta.free {
                self.renderer.free_texture(id);
            }
            output.textures_delta.clear();
            return Vec::new();
        }

        let win_size = window.inner_size();
        let scale = if win_size.width > 0 {
            target.width as f32 / win_size.width as f32
        } else {
            1.0
        };
        let pixels_per_point = output.pixels_per_point * scale;

        let clipped_primitives = self.ctx.tessellate(output.shapes, pixels_per_point);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [target.width, target.height],
            pixels_per_point,
        };

        let user_cmd_bufs = self.renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            encoder,
            &clipped_primitives,
            &screen_descriptor,
        );

        {
            let mut render_pass = encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &target.color,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                })
                .forget_lifetime();

            self.renderer
                .render(&mut render_pass, &clipped_primitives, &screen_descriptor);
        }

        for id in &output.textures_delta.free {
            self.renderer.free_texture(id);
        }
        // Handled above; clear so epaint's drop check doesn't fire
        output.textures_delta.clear();

        user_cmd_bufs
    }
}
