//! Water surface rendering: grid generation, wave animation, view-dependent color, and culling.

use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::util::DeviceExt;

use super::camera::Camera;
use super::frustum::Frustum;
use super::lit::{srgb_to_linear, Lighting};
use crate::gpu::{Gpu, DEPTH_FORMAT};
use crate::target::{FrameTarget, COLOR_FORMAT};

/// One horizontal rectangular water surface patch.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterPatch {
    /// Minimum horizontal coordinates (x, z).
    pub min: Vec2,
    /// Maximum horizontal coordinates (x, z).
    pub max: Vec2,
    /// Vertical surface level (y).
    pub level: f32,
}

impl WaterPatch {
    pub fn new(min: Vec2, max: Vec2, level: f32) -> Self {
        Self { min, max, level }
    }
}

/// Visual and wave parameters for the water surface.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterStyle {
    /// Shallow water color in sRGB [r, g, b, alpha].
    pub shallow_color: [f32; 4],
    /// Deep water color in sRGB [r, g, b, alpha].
    pub deep_color: [f32; 4],
    /// Maximum wave displacement in meters (amplitude <= 0.08 m).
    pub wave_amplitude: f32,
    /// Wave spatial frequency.
    pub wave_frequency: f32,
    /// Wave animation speed.
    pub wave_speed: f32,
    /// Specular highlight multiplier.
    pub specular_strength: f32,
    /// Specular power (shininess).
    pub specular_power: f32,
    /// Fresnel power for view-dependent color transition.
    pub fresnel_power: f32,
}

impl Default for WaterStyle {
    fn default() -> Self {
        Self {
            shallow_color: [0.15, 0.45, 0.50, 0.75],
            deep_color: [0.05, 0.20, 0.35, 0.90],
            wave_amplitude: 0.05,
            wave_frequency: 0.5,
            wave_speed: 1.2,
            specular_strength: 0.8,
            specular_power: 32.0,
            fresnel_power: 3.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WaterUniforms {
    view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    fog: [f32; 4],
    fog_color: [f32; 4],
    shallow_color: [f32; 4],
    deep_color: [f32; 4],
    params: [f32; 4],          // x: time, y: wave_amplitude, z: wave_frequency, w: wave_speed
    lighting_params: [f32; 4], // x: specular_strength, y: specular_power, z: fresnel_power, w: unused
}

const VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<[f32; 3]>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &[wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float32x3,
        offset: 0,
        shader_location: 0,
    }],
};

/// Builds a non-indexed triangle list for a 4 m cell grid over the patch.
pub fn build_water_grid(patch: &WaterPatch) -> Vec<Vec3> {
    const CELL_SIZE: f32 = 4.0;
    let min_x = patch.min.x.min(patch.max.x);
    let max_x = patch.min.x.max(patch.max.x);
    let min_z = patch.min.y.min(patch.max.y);
    let max_z = patch.min.y.max(patch.max.y);
    let width = (max_x - min_x).max(CELL_SIZE);
    let height = (max_z - min_z).max(CELL_SIZE);
    let nx = ((width / CELL_SIZE).round() as u32).max(1);
    let nz = ((height / CELL_SIZE).round() as u32).max(1);
    let dx = (max_x - min_x) / nx as f32;
    let dz = (max_z - min_z) / nz as f32;

    let mut vertices = Vec::with_capacity((nx * nz * 6) as usize);
    let y = patch.level;
    for iz in 0..nz {
        let z0 = min_z + iz as f32 * dz;
        let z1 = if iz + 1 == nz { max_z } else { min_z + (iz + 1) as f32 * dz };
        for ix in 0..nx {
            let x0 = min_x + ix as f32 * dx;
            let x1 = if ix + 1 == nx { max_x } else { min_x + (ix + 1) as f32 * dx };

            let v0 = Vec3::new(x0, y, z0);
            let v1 = Vec3::new(x0, y, z1);
            let v2 = Vec3::new(x1, y, z1);
            let v3 = Vec3::new(x1, y, z0);

            // Triangle 1: CCW viewed from +Y
            vertices.push(v0);
            vertices.push(v1);
            vertices.push(v2);

            // Triangle 2: CCW viewed from +Y
            vertices.push(v0);
            vertices.push(v2);
            vertices.push(v3);
        }
    }
    vertices
}

/// Returns true if the water patch intersects the camera frustum.
pub fn water_patch_visible(frustum: &Frustum, patch: &WaterPatch, wave_amplitude: f32) -> bool {
    let margin = wave_amplitude.max(0.1);
    let min_x = patch.min.x.min(patch.max.x);
    let max_x = patch.min.x.max(patch.max.x);
    let min_z = patch.min.y.min(patch.max.y);
    let max_z = patch.min.y.max(patch.max.y);
    let aabb_min = Vec3::new(min_x, patch.level - margin, min_z);
    let aabb_max = Vec3::new(max_x, patch.level + margin, max_z);
    frustum.intersects_aabb(aabb_min, aabb_max)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PatchKey {
    min_x: u32,
    min_z: u32,
    max_x: u32,
    max_z: u32,
    level: u32,
}

impl PatchKey {
    fn from_patch(p: &WaterPatch) -> Self {
        Self {
            min_x: p.min.x.to_bits(),
            min_z: p.min.y.to_bits(),
            max_x: p.max.x.to_bits(),
            max_z: p.max.y.to_bits(),
            level: p.level.to_bits(),
        }
    }
}

struct CachedMesh {
    buffer: wgpu::Buffer,
    vertex_count: u32,
}

/// Renders water surfaces with alpha blending and depth testing (depth write disabled).
pub struct WaterRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    cache: HashMap<PatchKey, CachedMesh>,
    pub style: WaterStyle,
}

impl WaterRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        Self::with_style(gpu, WaterStyle::default())
    }

    pub fn with_style(gpu: &Gpu, style: WaterStyle) -> Self {
        crate::timing::count_new_pipeline();
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("water"),
            source: wgpu::ShaderSource::Wgsl(include_str!("water.wgsl").into()),
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water uniforms"),
            size: std::mem::size_of::<WaterUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("water layout"),
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
            label: Some("water bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buf.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("water pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("water pipeline"),
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
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
                    format: COLOR_FORMAT,
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
            cache: HashMap::new(),
            style,
        }
    }

    /// Draws all visible water patches in a separate pass after the opaque scene.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        light: &Lighting,
        time: f32,
        patches: &[WaterPatch],
    ) {
        if patches.is_empty() {
            return;
        }

        let view_proj = camera.view_proj(target.aspect());
        let frustum = Frustum::from_view_proj(&view_proj);
        let visible_indices: Vec<usize> = (0..patches.len())
            .filter(|&i| water_patch_visible(&frustum, &patches[i], self.style.wave_amplitude))
            .collect();

        if visible_indices.is_empty() {
            return;
        }

        self.write_uniforms(gpu, camera, light, view_proj, time);

        // Ensure meshes for all visible patches are cached on GPU
        for &i in &visible_indices {
            let p = &patches[i];
            let key = PatchKey::from_patch(p);
            if !self.cache.contains_key(&key) {
                let verts = build_water_grid(p);
                let raw: Vec<[f32; 3]> = verts.iter().map(|v| v.to_array()).collect();
                let buf = gpu.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("water patch"),
                    contents: bytemuck::cast_slice(&raw),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                self.cache.insert(
                    key,
                    CachedMesh {
                        buffer: buf,
                        vertex_count: verts.len() as u32,
                    },
                );
            }
        }

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("water"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        for &i in &visible_indices {
            let key = PatchKey::from_patch(&patches[i]);
            if let Some(mesh) = self.cache.get(&key) {
                pass.set_vertex_buffer(0, mesh.buffer.slice(..));
                pass.draw(0..mesh.vertex_count, 0..1);
            }
        }
    }

    fn write_uniforms(&self, gpu: &Gpu, camera: &Camera, light: &Lighting, view_proj: Mat4, time: f32) {
        let lin = srgb_to_linear;
        let v4 = |v: Vec3, w: f32| Vec4::from((v, w)).to_array();

        let shallow_rgb = lin(Vec3::from_slice(&self.style.shallow_color[..3]));
        let deep_rgb = lin(Vec3::from_slice(&self.style.deep_color[..3]));

        let uniforms = WaterUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: v4(camera.position, 1.0),
            sun_dir: v4(light.sun_toward.normalize_or_zero(), light.sun_energy),
            sun_color: v4(lin(light.sun_color), 1.0),
            fog: [light.fog_begin, light.fog_end, light.fog_density, light.ambient_energy],
            fog_color: v4(lin(light.fog_color), 1.0),
            shallow_color: [shallow_rgb.x, shallow_rgb.y, shallow_rgb.z, self.style.shallow_color[3]],
            deep_color: [deep_rgb.x, deep_rgb.y, deep_rgb.z, self.style.deep_color[3]],
            params: [time, self.style.wave_amplitude, self.style.wave_frequency, self.style.wave_speed],
            lighting_params: [
                self.style.specular_strength,
                self.style.specular_power,
                self.style.fresnel_power,
                0.0,
            ],
        };
        gpu.queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&uniforms));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_grid_vertex_count_and_bounds() {
        let patch = WaterPatch::new(Vec2::new(10.0, 20.0), Vec2::new(210.0, 220.0), 5.0);
        let vertices = build_water_grid(&patch);
        // 200m / 4m = 50 cells across X, 50 across Z.
        // 50 * 50 = 2500 quads * 6 vertices = 15000 vertices.
        assert_eq!(vertices.len(), 50 * 50 * 6);
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_z = f32::MAX;
        let mut max_z = f32::MIN;
        for v in &vertices {
            assert_eq!(v.y, 5.0);
            min_x = min_x.min(v.x);
            max_x = max_x.max(v.x);
            min_z = min_z.min(v.z);
            max_z = max_z.max(v.z);
        }
        assert!((min_x - 10.0).abs() < 1e-4);
        assert!((max_x - 210.0).abs() < 1e-4);
        assert!((min_z - 20.0).abs() < 1e-4);
        assert!((max_z - 220.0).abs() < 1e-4);
    }

    #[test]
    fn water_culling_skips_patch_behind_camera() {
        let cam = Camera { position: Vec3::ZERO, ..Default::default() };
        let frustum = Frustum::from_view_proj(&cam.view_proj(16.0 / 9.0));
        // In front of camera (forward is -Z in default view):
        let in_front = WaterPatch::new(Vec2::new(-10.0, -50.0), Vec2::new(10.0, -10.0), 0.0);
        assert!(water_patch_visible(&frustum, &in_front, 0.05));

        // Behind camera (+Z):
        let behind = WaterPatch::new(Vec2::new(-10.0, 10.0), Vec2::new(10.0, 50.0), 0.0);
        assert!(!water_patch_visible(&frustum, &behind, 0.05));
    }

    #[test]
    fn water_style_default_in_range() {
        let s = WaterStyle::default();
        assert!(s.wave_amplitude > 0.0 && s.wave_amplitude <= 0.08, "amplitude must be <= 0.08 m");
        assert!(s.wave_frequency > 0.0);
        assert!(s.wave_speed > 0.0);
        assert!(s.specular_strength >= 0.0 && s.specular_strength <= 10.0);
        assert!(s.specular_power >= 1.0);
        assert!(s.fresnel_power >= 0.5);
        for c in s.shallow_color {
            assert!((0.0..=1.0).contains(&c));
        }
        for c in s.deep_color {
            assert!((0.0..=1.0).contains(&c));
        }
    }
}
