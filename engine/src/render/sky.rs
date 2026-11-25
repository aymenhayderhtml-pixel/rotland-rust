//! Drifting low-poly clouds rendered in one full-screen pass behind terrain and meshes.
//!
//! Draws a single full-screen triangle at reverse-Z depth `0.0` (infinitely far) with
//! `GreaterEqual` depth testing and no depth writes, so clouds only touch uncovered sky pixels.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};

use super::lit::srgb_to_linear;
use crate::gpu::Gpu;

const MIN_VISIBLE_COVER: f32 = 1e-3; // new: DECISIONS 2026-11-25
const MIN_VISIBLE_ALPHA: f32 = 1e-3; // new: DECISIONS 2026-11-25
const MIN_SOFTNESS: f32 = 0.01; // new: DECISIONS 2026-11-25
const MIN_SCALE: f32 = 0.01; // new: DECISIONS 2026-11-25

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SkyUniforms {
    inv_view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    sun_dir: [f32; 4],
    sunlit_color: [f32; 4],
    shadow_color: [f32; 4],
    cloud: [f32; 4],
}

/// Per-frame parameters for [`SkyRenderer`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyParams {
    /// Cloud coverage in `0.0..=1.0` (clear -> low, rain/snow/fog -> high).
    pub cover: f32,
    /// Edge softness in noise space (e.g. `0.09`).
    pub softness: f32,
    /// UV scale on the sky plane.
    pub scale: f32,
    /// Maximum cloud opacity (`0.0..=1.0`).
    pub max_alpha: f32,
    /// Accumulated wind drift offset in cloud UV space.
    pub wind_offset: [f32; 2],
    /// sRGB colour of sunlit cloud tops.
    pub sunlit_color: [f32; 3],
    /// sRGB colour of shaded cloud undersides.
    pub shadow_color: [f32; 3],
    /// Normalized world-space direction toward the sun.
    pub sun_dir: [f32; 3],
    /// Camera world position.
    pub camera_pos: [f32; 3],
}

/// Renders procedural low-poly clouds onto sky pixels in one draw call.
pub struct SkyRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
}

impl SkyRenderer {
    /// Creates the cloud pipeline and uniform buffer for `format` and `depth_format`.
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat, depth_format: wgpu::TextureFormat) -> Self {
        crate::timing::count_new_pipeline();
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sky_clouds"),
            source: wgpu::ShaderSource::Wgsl(include_str!("sky.wgsl").into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sky cloud uniforms"),
            size: std::mem::size_of::<SkyUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky cloud layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky cloud bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky cloud pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky cloud pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline,
            bind_group,
            uniform_buf,
        }
    }

    /// Draws clouds in a single full-screen triangle draw call into an active render pass.
    pub fn draw(
        &self,
        pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        params: &SkyParams,
    ) {
        if params.cover <= MIN_VISIBLE_COVER || params.max_alpha <= MIN_VISIBLE_ALPHA {
            return;
        }
        let sunlit = srgb_to_linear(Vec3::from(params.sunlit_color));
        let shadow = srgb_to_linear(Vec3::from(params.shadow_color));
        let uniforms = SkyUniforms {
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            camera_pos: [params.camera_pos[0], params.camera_pos[1], params.camera_pos[2], 1.0],
            sun_dir: [params.sun_dir[0], params.sun_dir[1], params.sun_dir[2], 0.0],
            sunlit_color: [sunlit.x, sunlit.y, sunlit.z, params.max_alpha.clamp(0.0, 1.0)],
            shadow_color: [shadow.x, shadow.y, shadow.z, params.softness.max(MIN_SOFTNESS)],
            cloud: [
                params.cover.clamp(0.0, 1.0),
                params.scale.max(MIN_SCALE),
                params.wind_offset[0],
                params.wind_offset[1],
            ],
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_uniform_size_and_alignment() {
        assert_eq!(std::mem::size_of::<SkyUniforms>(), 144);
        assert_eq!(std::mem::size_of::<SkyUniforms>() % 16, 0);
    }
}
