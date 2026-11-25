//! Weather particles: falling rain streaks, snow flakes and sandstorm dust.
//!
//! ONE static vertex buffer (a single shared quad) is made once in `new`; every particle is
//! one instance of it and the vertex shader in `weather.wgsl` hashes `@builtin(instance_index)`
//! to place each quad inside a 40 m box that follows the camera, wrapping fall and wind motion
//! with `fract()`. There is no per-particle CPU work and no per-frame buffer creation: only the
//! instance count changes with the weather intensity. Clear and fog draw nothing.
//!
//! Blending matches `water` (standard alpha, `BlendState::ALPHA_BLENDING`); depth tests against
//! the scene with reverse-Z (`GreaterEqual`) and never writes depth, so particles hide behind
//! walls and trees without affecting later passes. No roof check yet: particles draw indoors too
//! (R35 may add a roof check).
//!
//! Drawn after `LitRenderer` and water, into an already-begun pass that loads color and depth.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use super::lit::srgb_to_linear;
use crate::gpu::Gpu;

/// Weather kind indices for `WeatherParams::kind_index` (shared names: 0 clear .. 4 fog).
pub const KIND_CLEAR: u32 = 0; // new: DECISIONS 2026-11-25 (R32 shared names)
/// Rain: thin fast streaks slanted by the wind.
pub const KIND_RAIN: u32 = 1; // new: DECISIONS 2026-11-25 (R32 shared names)
/// Snow: soft round flakes with a little sideways sway.
pub const KIND_SNOW: u32 = 2; // new: DECISIONS 2026-11-25 (R32 shared names)
/// Sandstorm: larger faint almost-horizontal blobs hugging the box bottom.
pub const KIND_SANDSTORM: u32 = 3; // new: DECISIONS 2026-11-25 (R32 shared names)
/// Fog draws nothing (fog is done by the lit fog params, not particles).
pub const KIND_FOG: u32 = 4; // new: DECISIONS 2026-11-25 (R32 shared names)

/// Maximum particle instances (N), made once as instanced quads.
pub const MAX_PARTICLES: u32 = 6000; // new: DECISIONS 2026-11-25 (task R32-02: N = 6000)

// Share of N drawn at intensity 1 per kind. `data/weather.json` has `particles_per_m3` but
// `WeatherParams` does not carry it, so the factors are fixed here.
const RAIN_DENSITY: f32 = 1.0; // new: DECISIONS 2026-11-25
const SNOW_DENSITY: f32 = 0.6; // new: DECISIONS 2026-11-25 (big soft flakes: fewer look dense)
const SAND_DENSITY: f32 = 0.8; // new: DECISIONS 2026-11-25 (dust sits in a low 6 m band)

/// One shared quad, two CCW triangles covering [-1, 1]²: the only vertex data, made once.
const QUAD_CORNERS: [[f32; 2]; 6] = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [1.0, 1.0],
    [-1.0, -1.0],
    [1.0, 1.0],
    [-1.0, 1.0],
];

const QUAD_VERTICES: u32 = 6;

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x2,
        offset: 0,
        shader_location: 0,
    }],
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WeatherUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4], // xyz: camera position
    wind_time: [f32; 4],  // x: wind x (m/s), y: wind z (m/s), z: time (s)
    tint: [f32; 4],       // rgb: linear tint
    kind: [f32; 4],       // x: kind_index
}

/// Per-frame weather render input. `kind_index` is one of the `KIND_*` consts; `tint` is the
/// sRGB color from `data/weather.json` `kinds.<kind>.tint`; `wind` is `WeatherState.wind`
/// (m/s, xz); `intensity` is the eased 0..1 value from `WeatherState`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherParams {
    /// 0 clear, 1 rain, 2 snow, 3 sandstorm, 4 fog.
    pub kind_index: u32,
    /// Eased 0..1 weather strength.
    pub intensity: f32,
    /// Wind in m/s (xz).
    pub wind: [f32; 2],
    /// sRGB tint from `data/weather.json` `kinds.<kind>.tint`.
    pub tint: [f32; 3],
    /// Camera position; the 40 m particle box follows it.
    pub camera_pos: [f32; 3],
    /// Game time in seconds.
    pub time: f32,
}

/// Renders rain / snow / sandstorm particles over the lit scene.
pub struct WeatherRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    quad_buf: wgpu::Buffer,
}

impl WeatherRenderer {
    /// Builds the pipeline, the uniform buffer and the static quad buffer (all once).
    /// Pass the frame's formats, e.g. `WeatherRenderer::new(gpu, COLOR_FORMAT, DEPTH_FORMAT)`.
    pub fn new(gpu: &Gpu, format: wgpu::TextureFormat, depth_format: wgpu::TextureFormat) -> Self {
        crate::timing::count_new_pipeline();
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("weather"),
            source: wgpu::ShaderSource::Wgsl(include_str!("weather.wgsl").into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("weather uniforms"),
            size: std::mem::size_of::<WeatherUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let quad_buf = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("weather quad"),
            contents: bytemuck::cast_slice(&QUAD_CORNERS),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("weather layout"),
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
            label: Some("weather bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("weather pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("weather pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(VERTEX_LAYOUT)],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            // Reverse-Z like water: larger depth = closer. No depth write: particles never
            // occlude later passes, they only hide behind the scene.
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
                    // Same blend as water.
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
            quad_buf,
        }
    }

    /// Instances to draw this frame: `N x intensity x density_factor`, 0 for clear, fog and
    /// unknown kinds. Pure so tests and the stats JSON can call it.
    pub fn instance_count(kind_index: u32, intensity: f32) -> u32 {
        let density = match kind_index {
            KIND_RAIN => RAIN_DENSITY,
            KIND_SNOW => SNOW_DENSITY,
            KIND_SANDSTORM => SAND_DENSITY,
            _ => 0.0, // clear, fog and unknown kinds draw nothing
        };
        let intensity = intensity.clamp(0.0, 1.0);
        (MAX_PARTICLES as f32 * intensity * density).floor() as u32
    }

    /// Records the weather into an already-begun render pass. Call it every frame after the
    /// lit and water passes, with a pass that loads the color and depth of the frame:
    ///
    /// ```ignore
    /// let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    ///     label: Some("weather"),
    ///     color_attachments: &[Some(wgpu::RenderPassColorAttachment {
    ///         view: &target.color,
    ///         depth_slice: None,
    ///         resolve_target: None,
    ///         ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
    ///     })],
    ///     depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
    ///         view: &target.depth,
    ///         depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Discard }),
    ///         stencil_ops: None,
    ///     }),
    ///     timestamp_writes: None,
    ///     occlusion_query_set: None,
    ///     multiview_mask: None,
    /// });
    /// let view_proj = camera.view_proj(target.aspect());
    /// weather.draw(&mut pass, &gpu.queue, view_proj, &WeatherParams {
    ///     kind_index: WeatherRenderer::KIND_SNOW,
    ///     intensity: weather_state.intensity,
    ///     wind: [weather_state.wind.x, weather_state.wind.y],
    ///     tint: weather_data.kinds["snow"].tint,
    ///     camera_pos: camera.position.to_array(),
    ///     time: game_time_seconds,
    /// });
    /// ```
    ///
    /// `view_proj` is the same camera matrix the lit pass used. The call writes the uniform
    /// buffer (the only per-frame upload) and draws `instance_count` quad instances; it records
    /// nothing for clear, fog or intensity 0.
    pub fn draw(
        &mut self,
        pass: &mut wgpu::RenderPass<'_>,
        queue: &wgpu::Queue,
        view_proj: Mat4,
        params: &WeatherParams,
    ) {
        let instances = Self::instance_count(params.kind_index, params.intensity);
        if instances == 0 {
            return;
        }
        let tint = srgb_to_linear(Vec3::from_slice(&params.tint));
        let uniforms = WeatherUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [params.camera_pos[0], params.camera_pos[1], params.camera_pos[2], 0.0],
            wind_time: [params.wind[0], params.wind[1], params.time, 0.0],
            tint: [tint.x, tint.y, tint.z, 0.0],
            kind: [params.kind_index as f32, 0.0, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.quad_buf.slice(..));
        pass.draw(0..QUAD_VERTICES, 0..instances);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_count_zero_without_weather() {
        // Intensity 0 gives 0.
        assert_eq!(WeatherRenderer::instance_count(KIND_RAIN, 0.0), 0);
        assert_eq!(WeatherRenderer::instance_count(KIND_SNOW, 0.0), 0);
        assert_eq!(WeatherRenderer::instance_count(KIND_SANDSTORM, 0.0), 0);
        // Clear and fog give 0 even at full intensity.
        assert_eq!(WeatherRenderer::instance_count(KIND_CLEAR, 1.0), 0);
        assert_eq!(WeatherRenderer::instance_count(KIND_FOG, 1.0), 0);
    }

    #[test]
    fn sandstorm_at_full_intensity_is_capped_at_n() {
        let count = WeatherRenderer::instance_count(KIND_SANDSTORM, 1.0);
        assert!(count > 0, "a full sandstorm must draw particles");
        assert!(count <= MAX_PARTICLES, "draw count must never exceed N");
    }

    #[test]
    fn instance_count_grows_with_intensity_and_stays_capped() {
        let half = WeatherRenderer::instance_count(KIND_RAIN, 0.5);
        let full = WeatherRenderer::instance_count(KIND_RAIN, 1.0);
        assert!(half > 0 && half < full);
        assert!(full <= MAX_PARTICLES);
    }
}
