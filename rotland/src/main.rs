//! Rotland (Rust). R0 step 1: prove the engine draws on the owner's GPU (a test triangle and an FPS counter).

use engine::{EngineArgs, FrameTarget, Game, Gpu, Input};

struct Spike {
    pipeline: Option<wgpu::RenderPipeline>,
}

const SHADER: &str = r#"
@vertex
fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    let p = array(vec2(0.0, 0.6), vec2(-0.6, -0.5), vec2(0.6, -0.5));
    return vec4(p[i], 0.0, 1.0);
}
@fragment
fn fs() -> @location(0) vec4<f32> {
    return vec4(0.35, 0.6, 0.25, 1.0);
}
"#;

impl Game for Spike {
    fn init(&mut self, gpu: &Gpu, _args: &EngineArgs) {
        let module = gpu.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("spike"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        self.pipeline = Some(gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("spike"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(engine::target::COLOR_FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        }));
    }

    fn update(&mut self, _input: &Input, _dt: f32) {}

    fn render(&mut self, _gpu: &Gpu, target: &FrameTarget, encoder: &mut wgpu::CommandEncoder) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("spike"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.55, g: 0.7, b: 0.9, a: 1.0 }), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        if let Some(p) = &self.pipeline {
            pass.set_pipeline(p);
            pass.draw(0..3, 0..1);
        }
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn")).init();
    engine::run(Spike { pipeline: None }, "Rotland (Rust)", EngineArgs::from_env());
}
