use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

use super::camera::Camera;
use super::frustum::Frustum;
use super::mesh::GpuMesh;
use super::pipelines::{Layouts, Pipelines};
use super::shadow::{light_view_proj, ShadowMap, ShadowSettings};
use super::skinned::{GrowBuffer, SkinInstance, SkinnedBatch};
use crate::gpu::Gpu;
use crate::target::FrameTarget;

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
    pub moon_toward: Vec3,
    pub moon_energy: f32,
    pub moon_color: Vec3,
    /// Up to 3 spot lights (e.g. index 0: flashlight, indices 1..2: vehicle headlights).
    pub spots: [Option<SpotLight>; 3],
}

/// A cone of light from a point, like Godot's SpotLight3D. Colour is sRGB.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpotLight {
    pub position: Vec3,
    pub direction: Vec3,
    pub range: f32,
    /// The cone's half angle in degrees (Godot's SpotAngle).
    pub angle_deg: f32,
    pub color: Vec3,
    pub energy: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable, PartialEq)]
pub struct SpotLightUniform {
    pub pos: [f32; 4],
    pub dir: [f32; 4],
    pub color: [f32; 4],
}

impl SpotLightUniform {
    pub const ZERO: Self = Self {
        pos: [0.0; 4],
        dir: [0.0, -1.0, 0.0, 1.0],
        color: [0.0; 4],
    };
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct SceneUniforms {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    light_view_proj: [[f32; 4]; 4],
    shadow: [f32; 4],
    camera_pos: [f32; 4],
    sun_dir: [f32; 4],
    sun_color: [f32; 4],
    sky_top: [f32; 4],
    sky_horizon: [f32; 4],
    ground_horizon: [f32; 4],
    ground_bottom: [f32; 4],
    fog: [f32; 4],
    fog_color: [f32; 4],
    moon_dir: [f32; 4],
    moon_color: [f32; 4],
    spots: [SpotLightUniform; 3],
}

/// Dynamic uniform offsets must be 256-byte aligned.
const DRAW_STRIDE: u64 = 256;
const MAX_DRAWS: usize = 4096;

pub fn srgb_to_linear(c: Vec3) -> Vec3 {
    let f = |x: f32| if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) };
    Vec3::new(f(c.x), f(c.y), f(c.z))
}

/// One mesh to draw this frame, placed at `offset` (for chunks: the chunk origin).
pub struct DrawItem<'a> {
    pub mesh: &'a GpuMesh,
    pub offset: Vec3,
    /// 1.0 = normal. Above 1.0 = a far item (the city skyline): its fog distance is divided by this
    /// and it casts no sun shadow.
    pub fog_scale: f32,
}

/// Counts of the last frame, for the stats JSON.
#[derive(Default, Clone, Copy, Debug)]
pub struct FrameCounts {
    pub draws: u32,
    pub triangles: u64,
    pub shadow_draws: u32,
    pub shadow_triangles: u64,
}

/// Draws the world into the frame target: sun shadow map, lit meshes, skinned instances, sky.
pub struct LitRenderer {
    layouts: Layouts,
    pipes: Pipelines,
    scene_buf: wgpu::Buffer,
    scene_bind: wgpu::BindGroup,
    scene_shadow_bind: wgpu::BindGroup,
    draw_buf: wgpu::Buffer,
    draw_bind: wgpu::BindGroup,
    bones: GrowBuffer,
    bones_bind: wgpu::BindGroup,
    instances: GrowBuffer,
    shadow: ShadowMap,
    offsets_scratch: Vec<u8>,       // new: PERF R30.5
    visible_scratch: Vec<usize>,    // new: PERF R30.5
    instances_scratch: Vec<SkinInstance>, // new: PERF R30.5
    normal_ranges: Vec<(usize, u32, u32)>, // new: PERF R34
    fade_ranges: Vec<(usize, u32, u32)>,   // new: PERF R34
    pub shadows: ShadowSettings,
    pub last: FrameCounts,
}

impl LitRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        Self::with_shadows(gpu, ShadowSettings::default())
    }

    pub fn with_shadows(gpu: &Gpu, shadows: ShadowSettings) -> Self {
        let device = &gpu.device;
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("lit"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lit.wgsl").into()),
        });
        let layouts = Layouts::new(device);
        let pipes = Pipelines::new(device, &module, &layouts);
        let buffer = |label, size, usage| device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false });
        let scene_buf = buffer("scene", std::mem::size_of::<SceneUniforms>() as u64, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let draw_buf = buffer("draws", DRAW_STRIDE * MAX_DRAWS as u64, wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST);
        let shadow = ShadowMap::new(device, shadows.size);
        let scene_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene"),
            layout: &layouts.scene,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: scene_buf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow.sampler) },
            ],
        });
        let scene_shadow_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("scene (shadow pass)"),
            layout: &layouts.scene_uniform,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: scene_buf.as_entire_binding() }],
        });
        let draw_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("draws"),
            layout: &layouts.draw,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &draw_buf, offset: 0, size: wgpu::BufferSize::new(16) }),
            }],
        });
        let bones = GrowBuffer::new(device, "bones", wgpu::BufferUsages::STORAGE, 64 * 64 * 64);
        let bones_bind = Self::bones_bind(device, &layouts.bones, &bones.buffer);
        let instances = GrowBuffer::new(device, "instances", wgpu::BufferUsages::VERTEX, 64 * 1024);
        Self {
            layouts,
            pipes,
            scene_buf,
            scene_bind,
            scene_shadow_bind,
            draw_buf,
            draw_bind,
            bones,
            bones_bind,
            instances,
            shadow,
            offsets_scratch: Vec::new(),
            visible_scratch: Vec::new(),
            instances_scratch: Vec::new(),
            normal_ranges: Vec::new(),
            fade_ranges: Vec::new(),
            shadows,
            last: FrameCounts::default(),
        }
    }

    fn bones_bind(device: &wgpu::Device, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bones"),
            layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: buffer.as_entire_binding() }],
        })
    }

    /// Draws one frame of the world. `items` are static meshes (culled here). `instanced` are the batches
    /// for the camera and `shadow_instanced` the batches for the sun's shadow (they differ: things behind
    /// the camera still cast shadows into view). Skin matrices for all of them are in `bones`; index 0
    /// must be the identity, used by static instanced models (trees, rocks).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        gpu: &Gpu,
        target: &FrameTarget,
        encoder: &mut wgpu::CommandEncoder,
        camera: &Camera,
        light: &Lighting,
        items: &[DrawItem],
        instanced: &[SkinnedBatch],
        shadow_instanced: &[SkinnedBatch],
        bones: &[Mat4],
    ) {
        crate::timing::mark_render_draw();
        let t_draw_start = std::time::Instant::now();
        let items = &items[..items.len().min(MAX_DRAWS)];
        let view_proj = camera.view_proj(target.aspect());
        let shadows_on = self.shadows.enabled && light.sun_energy > 0.01;
        let light_vp = light_view_proj(camera, light.sun_toward, &self.shadows);
        self.write_scene(gpu, camera, light, view_proj, light_vp, shadows_on);

        // Every item's offset at its own slot, shared by the shadow and main passes
        self.offsets_scratch.clear();
        self.offsets_scratch.resize(items.len() * DRAW_STRIDE as usize, 0);
        for (i, d) in items.iter().enumerate() {
            let at = i * DRAW_STRIDE as usize;
            self.offsets_scratch[at..at + 16].copy_from_slice(bytemuck::bytes_of(&[d.offset.x, d.offset.y, d.offset.z, d.fog_scale]));
        }
        if !self.offsets_scratch.is_empty() {
            gpu.queue.write_buffer(&self.draw_buf, 0, &self.offsets_scratch);
        }
        if self.bones.write(&gpu.device, &gpu.queue, bytemuck::cast_slice(bones)) {
            self.bones_bind = Self::bones_bind(&gpu.device, &self.layouts.bones, &self.bones.buffer);
        }
        self.instances_scratch.clear();
        self.normal_ranges.clear();
        self.fade_ranges.clear();
        for (idx, b) in instanced.iter().enumerate() {
            let first = self.instances_scratch.len() as u32;
            self.instances_scratch.extend(b.instances.iter().copied().filter(|inst| !inst.is_fading()));
            let count = self.instances_scratch.len() as u32 - first;
            if count > 0 {
                self.normal_ranges.push((idx, first, count));
            }
        }
        for (idx, b) in instanced.iter().enumerate() {
            let first = self.instances_scratch.len() as u32;
            self.instances_scratch.extend(b.instances.iter().copied().filter(|inst| inst.is_fading()));
            let count = self.instances_scratch.len() as u32 - first;
            if count > 0 {
                self.fade_ranges.push((idx, first, count));
            }
        }
        let main_count = self.instances_scratch.len() as u32;
        if shadows_on {
            self.instances_scratch.extend(
                shadow_instanced
                    .iter()
                    .flat_map(|b| b.instances.iter().copied()),
            );
        }
        self.instances.write(&gpu.device, &gpu.queue, bytemuck::cast_slice(&self.instances_scratch));

        let mut counts = FrameCounts::default();
        if shadows_on {
            let sun_frustum = Frustum::from_view_proj(&light_vp);
            self.visible_scratch.clear();
            self.visible_scratch.extend((0..items.len()).filter(|&i| items[i].fog_scale <= 1.0 && in_view(&sun_frustum, &items[i])));
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sun shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow.view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            let (d0, t0) = self.record_items(&mut pass, items, &self.visible_scratch, &self.pipes.shadow_mesh, &self.scene_shadow_bind);
            let (d1, t1) = self.record_skinned(&mut pass, shadow_instanced, main_count, &self.pipes.shadow_skinned, &self.scene_shadow_bind);
            counts.shadow_draws = d0 + d1;
            counts.shadow_triangles = t0 + t1;
        }

        let frustum = Frustum::from_view_proj(&view_proj);
        self.visible_scratch.clear();
        self.visible_scratch.extend((0..items.len()).filter(|&i| in_view(&frustum, &items[i])));
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
                // Reverse-Z: 0 is infinitely far. Stored: the water pass after this one tests against it
                // (with Discard the water saw garbage depth and drew over hills and the player).
                depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        let (d0, t0) = self.record_items(&mut pass, items, &self.visible_scratch, &self.pipes.mesh, &self.scene_bind);
        let (d1, t1) = self.record_skinned_ranges(&mut pass, instanced, &self.normal_ranges, &self.pipes.skinned, &self.scene_bind);
        let (d2, t2) = self.record_skinned_ranges(&mut pass, instanced, &self.fade_ranges, &self.pipes.skinned_fade, &self.scene_bind);
        // Sky last: it only fills pixels nothing else covered
        pass.set_pipeline(&self.pipes.sky);
        pass.set_bind_group(0, &self.scene_bind, &[]);
        pass.draw(0..3, 0..1);
        counts.draws = d0 + d1 + d2 + 1;
        counts.triangles = t0 + t1 + t2;
        self.last = counts;
        crate::timing::record_phase("render", t_draw_start.elapsed().as_secs_f64() * 1000.0);
    }

    /// Records static items into a pass. Returns (draw calls, triangles).
    fn record_items(
        &self,
        pass: &mut wgpu::RenderPass,
        items: &[DrawItem],
        visible: &[usize],
        mesh_pipe: &wgpu::RenderPipeline,
        scene: &wgpu::BindGroup,
    ) -> (u32, u64) {
        let (mut draws, mut triangles) = (0u32, 0u64);
        pass.set_pipeline(mesh_pipe);
        pass.set_bind_group(0, scene, &[]);
        for &i in visible {
            let d = &items[i];
            pass.set_bind_group(1, &self.draw_bind, &[(i as u64 * DRAW_STRIDE) as u32]);
            pass.set_vertex_buffer(0, d.mesh.vertices.slice(..));
            pass.draw(0..d.mesh.vertex_count, 0..1);
            draws += 1;
            triangles += d.mesh.vertex_count as u64 / 3;
        }
        (draws, triangles)
    }

    /// Records instanced ranges into a pass without per-frame batch splitting allocations.
    fn record_skinned_ranges(
        &self,
        pass: &mut wgpu::RenderPass,
        instanced: &[SkinnedBatch],
        ranges: &[(usize, u32, u32)],
        skin_pipe: &wgpu::RenderPipeline,
        scene: &wgpu::BindGroup,
    ) -> (u32, u64) {
        let (mut draws, mut triangles) = (0u32, 0u64);
        if !ranges.is_empty() {
            pass.set_pipeline(skin_pipe);
            pass.set_bind_group(0, scene, &[]);
            pass.set_bind_group(2, &self.bones_bind, &[]);
            pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
            for &(idx, first, count) in ranges {
                let b = &instanced[idx];
                pass.set_vertex_buffer(0, b.mesh.vertices.slice(..));
                pass.draw(0..b.mesh.vertex_count, first..first + count);
                draws += 1;
                triangles += (b.mesh.vertex_count / 3) as u64 * count as u64;
            }
        }
        (draws, triangles)
    }

    /// Records instanced batches into a pass. Returns (draw calls, triangles).
    fn record_skinned(
        &self,
        pass: &mut wgpu::RenderPass,
        skinned: &[SkinnedBatch],
        first_instance: u32,
        skin_pipe: &wgpu::RenderPipeline,
        scene: &wgpu::BindGroup,
    ) -> (u32, u64) {
        let (mut draws, mut triangles) = (0u32, 0u64);
        if skinned.iter().any(|b| !b.instances.is_empty()) {
            pass.set_pipeline(skin_pipe);
            pass.set_bind_group(0, scene, &[]);
            pass.set_bind_group(2, &self.bones_bind, &[]);
            pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
            let mut first = first_instance;
            for b in skinned {
                let count = b.instances.len() as u32;
                if count > 0 {
                    pass.set_vertex_buffer(0, b.mesh.vertices.slice(..));
                    pass.draw(0..b.mesh.vertex_count, first..first + count);
                    draws += 1;
                    triangles += (b.mesh.vertex_count / 3) as u64 * count as u64;
                }
                first += count;
            }
        }
        (draws, triangles)
    }

    fn write_scene(&self, gpu: &Gpu, camera: &Camera, light: &Lighting, view_proj: Mat4, light_vp: Mat4, shadows_on: bool) {
        let lin = srgb_to_linear;
        let v4 = |v: Vec3, w: f32| Vec4::from((v, w)).to_array();
        let s = &self.shadows;
        let mut spots = [SpotLightUniform::ZERO; 3];
        for (dst, src) in spots.iter_mut().zip(light.spots.iter()) {
            if let Some(sp) = src {
                *dst = SpotLightUniform {
                    pos: v4(sp.position, sp.range),
                    dir: v4(sp.direction.normalize_or_zero(), sp.angle_deg.to_radians().cos()),
                    color: v4(lin(sp.color), sp.energy),
                };
            }
        }
        let scene = SceneUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: view_proj.inverse().to_cols_array_2d(),
            light_view_proj: light_vp.to_cols_array_2d(),
            shadow: [if shadows_on { 1.0 } else { 0.0 }, 1.0 / s.size as f32, s.distance * 0.8, s.distance],
            camera_pos: v4(camera.position, 1.0),
            sun_dir: v4(light.sun_toward.normalize_or_zero(), light.sun_energy),
            sun_color: v4(lin(light.sun_color), 1.0),
            sky_top: v4(lin(light.sky_top), 1.0),
            sky_horizon: v4(lin(light.sky_horizon), 1.0),
            ground_horizon: v4(lin(light.ground_horizon), 1.0),
            ground_bottom: v4(lin(light.ground_bottom), 1.0),
            fog: [light.fog_begin, light.fog_end, light.fog_density, light.ambient_energy],
            fog_color: v4(lin(light.fog_color), 1.0),
            moon_dir: v4(light.moon_toward.normalize_or_zero(), light.moon_energy),
            moon_color: v4(lin(light.moon_color), 1.0),
            spots,
        };
        gpu.queue.write_buffer(&self.scene_buf, 0, bytemuck::bytes_of(&scene));
    }
}

fn in_view(frustum: &Frustum, d: &DrawItem) -> bool {
    frustum.intersects_aabb(d.mesh.min + d.offset, d.mesh.max + d.offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_size_and_alignment() {
        assert_eq!(std::mem::size_of::<SpotLightUniform>(), 48);
        assert_eq!(std::mem::size_of::<SpotLightUniform>() % 16, 0);

        assert_eq!(std::mem::size_of::<SceneUniforms>(), 528);
        assert_eq!(std::mem::size_of::<SceneUniforms>() % 16, 0);

        assert_eq!(std::mem::offset_of!(SceneUniforms, spots), 384);
    }
}
