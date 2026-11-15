use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

use super::camera::Camera;
use super::frustum::Frustum;
use super::mesh::{GpuMesh, COLOR_VERTEX_LAYOUT};
use super::skinned::{self, GrowBuffer, SkinnedBatch};
use crate::gpu::{Gpu, DEPTH_FORMAT};
use crate::target::{FrameTarget, COLOR_FORMAT};

/// The light and fog of one moment of the day. Colours are sRGB like in the Godot scene files;
/// they are turned into linear values once, on the CPU.
#[derive(Clone, Copy, Debug)]
pub struct Lighting {
    pub sun_toward: Vec3,
    pub sun_energy: f32,
    pub sun_color: Vec3,
    pub sky_top: Vec3,
    pub sky_horizon: Vec3,
    pub ground_horizon: Vec3,
    pub ground_bottom: Vec3,
    pub ambient_energy: f32,
    pub fog_begin: f32,
    pub fog_end: f32,
    pub fog_density: f32,
    pub fog_color: Vec3,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    camera_pos: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    ground_horizon: [f32; 4],
    ground_bottom: [f32; 4],
    fog: [f32; 4],
    fog_color: [f32; 4],
}

/// Dynamic uniform offsets must be 256-byte aligned.
const DRAW_STRIDE: u64 = 256;
const MAX_DRAWS: u64 = 4096;

pub fn srgb_to_linear(c: Vec3) -> Vec3 {
    let f = |x: f32| if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
    Vec3::new(f(c.x), f(c.y), f(c.z))
}

/// Draws the sky and lit coloured meshes (terrain now, baked models later) into the frame target.
pub struct LitRenderer {
    scene_buf: wgpu::Buffer,
    scene_bind: wgpu::BindGroup,
    draw_buf: wgpu::Buffer,
    draw_bind: wgpu::BindGroup,
    sky: wgpu::RenderPipeline,
    mesh: wgpu::RenderPipeline,
    skinned: wgpu::RenderPipeline,
    bones: GrowBuffer,
    bones_layout: wgpu::BindGroupLayout,
    bones_bind: wgpu::BindGroup,
    instances: GrowBuffer,
    /// Draws and triangles of the last frame, for the stats JSON.
    pub last_draws: u32,
    pub last_triangles: u64,
}

/// One mesh to draw this frame, placed at `offset` (for chunks: the chunk origin).
pub struct DrawItem<'a> {
    pub mesh: &'a GpuMesh,
    pub offset: Vec3,
}

impl LitRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lit"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lit.wgsl").into()),
        });
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("scene"),
            entries: &[uniform_entry(false)],
        });
        let draw_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("draw"),
            entries: &[uniform_entry(true)],
        });
        let scene_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene"),
            size: std::mem::size_of::<SceneUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let draw_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("draws"),
            size: DRAW_STRIDE * MAX_DRAWS,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let scene_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene"),
            layout: &scene_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: scene_buf.as_entire_binding() }],
        });
        let draw_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("draws"),
            layout: &draw_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &draw_buf,
                    offset: 0,
                    size: wgpu::BufferSize::new(16),
                }),
            }],
        });

        let sky_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky"),
            bind_group_layouts: &[Some(&scene_layout)],
            immediate_size: 0,
        });
        let mesh_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("lit mesh"),
            bind_group_layouts: &[Some(&scene_layout), Some(&draw_layout)],
            immediate_size: 0,
        });
        let bones_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bones"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let skinned_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("skinned"),
            bind_group_layouts: &[Some(&scene_layout), None, Some(&bones_layout)],
            immediate_size: 0,
        });
        let skinned = skinned::pipeline(device, &module, &skinned_layout);
        let bones = GrowBuffer::new(device, "bones", wgpu::BufferUsages::STORAGE, 64 * 64 * 64);
        let bones_bind = Self::bones_bind(device, &bones_layout, &bones.buffer);
        let instances = GrowBuffer::new(device, "skin instances", wgpu::BufferUsages::VERTEX, 64 * 1024);
        let sky = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sky"),
            layout: Some(&sky_layout),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("sky_vs"), compilation_options: Default::default(), buffers: &[] },
            primitive: Default::default(),
            // The sky never writes depth, so the world drawn after it always wins
            depth_stencil: Some(depth_state(false)),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("sky_fs"),
                compilation_options: Default::default(),
                targets: &[Some(COLOR_FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        let mesh = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lit mesh"),
            layout: Some(&mesh_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("mesh_vs"),
                compilation_options: Default::default(),
                buffers: &[Some(COLOR_VERTEX_LAYOUT)],
            },
            primitive: wgpu::PrimitiveState {
                // Godot's front faces are clockwise; the meshes keep Godot's triangle order
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(depth_state(true)),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("mesh_fs"),
                compilation_options: Default::default(),
                targets: &[Some(COLOR_FORMAT.into())],
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            scene_buf,
            scene_bind,
            draw_buf,
            draw_bind,
            sky,
            mesh,
            skinned,
            bones,
            bones_layout,
            bones_bind,
            instances,
            last_draws: 0,
            last_triangles: 0,
        }
    }

    fn bones_bind(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bones"),
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }

    /// Draws the world: static items (culled against the view), skinned batches (one instanced draw
    /// per model; `bones` holds every instance's skin matrices), then the sky behind everything.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        light: &Lighting,
        items: &[DrawItem],
        skinned: &[SkinnedBatch],
        bones: &[glam::Mat4],
    ) {
        let view_proj = camera.view_proj(target.aspect());
        let lin = srgb_to_linear;
        let v4 = |v: Vec3, w: f32| Vec4::from((v, w)).to_array();
        let scene = SceneUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            camera_pos: v4(camera.position, 1.0),
            sun_dir: v4(light.sun_toward.normalize_or_zero(), light.sun_energy),
            sun_color: v4(lin(light.sun_color), 1.0),
            sky_top: v4(lin(light.sky_top), 1.0),
            sky_horizon: v4(lin(light.sky_horizon), 1.0),
            ground_horizon: v4(lin(light.ground_horizon), 1.0),
            ground_bottom: v4(lin(light.ground_bottom), 1.0),
            fog: [light.fog_begin, light.fog_end, light.fog_density, light.ambient_energy],
            fog_color: v4(lin(light.fog_color), 1.0),
        };
        gpu.queue.write_buffer(&self.scene_buf, 0, bytemuck::bytes_of(&scene));

        let frustum = Frustum::from_view_proj(&view_proj);
        let visible: Vec<&DrawItem> = items
            .iter()
            .filter(|d| frustum.intersects_aabb(d.mesh.min + d.offset, d.mesh.max + d.offset))
            .take(MAX_DRAWS as usize)
            .collect();
        let mut offsets = vec![0u8; visible.len() * DRAW_STRIDE as usize];
        for (i, d) in visible.iter().enumerate() {
            let at = i * DRAW_STRIDE as usize;
            offsets[at..at + 16].copy_from_slice(bytemuck::bytes_of(&[d.offset.x, d.offset.y, d.offset.z, 0.0f32]));
        }
        if !offsets.is_empty() {
            gpu.queue.write_buffer(&self.draw_buf, 0, &offsets);
        }

        if self.bones.write(&gpu.device, &gpu.queue, bytemuck::cast_slice(bones)) {
            self.bones_bind = Self::bones_bind(&gpu.device, &self.bones_layout, &self.bones.buffer);
        }
        let all_instances: Vec<skinned::SkinInstance> = skinned.iter().flat_map(|b| b.instances.iter().copied()).collect();
        self.instances.write(&gpu.device, &gpu.queue, bytemuck::cast_slice(&all_instances));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("world"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target.color,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &target.depth,
                // Reverse-Z: 0 is infinitely far
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Discard }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        // World first (front to back helps the GPU skip hidden pixels), sky last where nothing was drawn
        pass.set_pipeline(&self.mesh);
        pass.set_bind_group(0, &self.scene_bind, &[]);
        let mut triangles = 0u64;
        for (i, d) in visible.iter().enumerate() {
            pass.set_bind_group(1, &self.draw_bind, &[(i as u64 * DRAW_STRIDE) as u32]);
            pass.set_vertex_buffer(0, d.mesh.vertices.slice(..));
            pass.draw(0..d.mesh.vertex_count, 0..1);
            triangles += d.mesh.vertex_count as u64 / 3;
        }
        let mut draws = visible.len() as u32;
        if !all_instances.is_empty() {
            pass.set_pipeline(&self.skinned);
            pass.set_bind_group(0, &self.scene_bind, &[]);
            pass.set_bind_group(2, &self.bones_bind, &[]);
            pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
            let mut first = 0u32;
            for b in skinned {
                let count = b.instances.len() as u32;
                if count > 0 {
                    pass.set_vertex_buffer(0, b.mesh.vertices.slice(..));
                    pass.draw(0..b.mesh.vertex_count, first..first + count);
                    triangles += (b.mesh.vertex_count / 3) as u64 * count as u64;
                    draws += 1;
                }
                first += count;
            }
        }
        pass.set_pipeline(&self.sky);
        pass.set_bind_group(0, &self.scene_bind, &[]);
        pass.draw(0..3, 0..1);
        self.last_draws = draws + 1;
        self.last_triangles = triangles;
    }
}

fn uniform_entry(dynamic: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dynamic,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(crate) fn depth_state(write: bool) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: DEPTH_FORMAT,
        depth_write_enabled: Some(write),
        depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
        stencil: Default::default(),
        bias: Default::default(),
    }
}
