//! Creation of the world pipelines and their bind group layouts (kept apart from the per-frame code).

use super::mesh::COLOR_VERTEX_LAYOUT;
use super::shadow::SHADOW_FORMAT;
use super::skinned::SkinInstance;
use crate::gpu::DEPTH_FORMAT;
use crate::target::COLOR_FORMAT;

pub struct Layouts {
    /// Scene uniforms + shadow map + shadow sampler (main pass).
    pub scene: wgpu::BindGroupLayout,
    /// Scene uniforms only (shadow pass: it cannot read the map it is drawing).
    pub scene_uniform: wgpu::BindGroupLayout,
    pub draw: wgpu::BindGroupLayout,
    pub bones: wgpu::BindGroupLayout,
}

pub struct Pipelines {
    pub sky: wgpu::RenderPipeline,
    pub mesh: wgpu::RenderPipeline,
    pub skinned: wgpu::RenderPipeline,
    pub shadow_mesh: wgpu::RenderPipeline,
    pub shadow_skinned: wgpu::RenderPipeline,
}

fn buffer_entry(binding: u32, stages: wgpu::ShaderStages, ty: wgpu::BufferBindingType, dynamic: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: stages,
        ty: wgpu::BindingType::Buffer { ty, has_dynamic_offset: dynamic, min_binding_size: None },
        count: None,
    }
}

impl Layouts {
    pub fn new(device: &wgpu::Device) -> Self {
        let vf = wgpu::ShaderStages::VERTEX_FRAGMENT;
        let uniform = buffer_entry(0, vf, wgpu::BufferBindingType::Uniform, false);
        let layout = |label, entries: &[wgpu::BindGroupLayoutEntry]| {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some(label), entries })
        };
        Self {
            scene: layout(
                "scene",
                &[
                    uniform,
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
            ),
            scene_uniform: layout("scene (shadow pass)", &[uniform]),
            draw: layout("draw", &[buffer_entry(0, vf, wgpu::BufferBindingType::Uniform, true)]),
            bones: layout(
                "bones",
                &[buffer_entry(0, wgpu::ShaderStages::VERTEX, wgpu::BufferBindingType::Storage { read_only: true }, false)],
            ),
        }
    }
}

pub fn depth_state(format: wgpu::TextureFormat, write: bool, compare: wgpu::CompareFunction, bias: wgpu::DepthBiasState) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState { format, depth_write_enabled: Some(write), depth_compare: Some(compare), stencil: Default::default(), bias }
}

impl Pipelines {
    pub fn new(device: &wgpu::Device, module: &wgpu::ShaderModule, l: &Layouts) -> Self {
        crate::timing::count_new_pipeline();
        let pl = |label, groups: &[Option<&wgpu::BindGroupLayout>]| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: groups, immediate_size: 0 })
        };
        let sky_layout = pl("sky", &[Some(&l.scene)]);
        let mesh_layout = pl("mesh", &[Some(&l.scene), Some(&l.draw)]);
        let skinned_layout = pl("skinned", &[Some(&l.scene), None, Some(&l.bones)]);
        let shadow_mesh_layout = pl("shadow mesh", &[Some(&l.scene_uniform), Some(&l.draw)]);
        let shadow_skinned_layout = pl("shadow skinned", &[Some(&l.scene_uniform), None, Some(&l.bones)]);

        // Reverse-Z for the camera (Greater wins), normal Z for the sun's orthographic box
        let main_depth = |write| depth_state(DEPTH_FORMAT, write, wgpu::CompareFunction::GreaterEqual, Default::default());
        let shadow_depth = depth_state(
            SHADOW_FORMAT,
            true,
            wgpu::CompareFunction::LessEqual,
            wgpu::DepthBiasState { constant: 2, slope_scale: 2.0, clamp: 0.0 },
        );
        // Terrain chunks keep Godot's clockwise order; glTF/Blender models and hand-built
        // prop meshes (campfire, wheel) are CCW-outward. Verified by
        // baker/tests/bake_test.rs::models_are_ccw_outward.
        let culled_cw = wgpu::PrimitiveState { front_face: wgpu::FrontFace::Cw, cull_mode: Some(wgpu::Face::Back), ..Default::default() };
        let culled_ccw = wgpu::PrimitiveState { front_face: wgpu::FrontFace::Ccw, cull_mode: Some(wgpu::Face::Back), ..Default::default() };
        let color_target = [Some(COLOR_FORMAT.into())];
        let fragment = |entry| Some(wgpu::FragmentState { module, entry_point: Some(entry), compilation_options: Default::default(), targets: &color_target });
        let vertex = |entry, buffers| wgpu::VertexState { module, entry_point: Some(entry), compilation_options: Default::default(), buffers };
        let mesh_buffers = [Some(COLOR_VERTEX_LAYOUT)];
        let skin_buffers = [Some(COLOR_VERTEX_LAYOUT), Some(SkinInstance::LAYOUT)];
        let make = |label, layout: &wgpu::PipelineLayout, vertex, primitive, depth, fragment| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(layout),
                vertex,
                primitive,
                depth_stencil: Some(depth),
                multisample: Default::default(),
                fragment,
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            // The sky never writes depth, so it only fills pixels nothing else covered
            sky: make("sky", &sky_layout, vertex("sky_vs", &[]), Default::default(), main_depth(false), fragment("sky_fs")),
            mesh: make("mesh", &mesh_layout, vertex("mesh_vs", &mesh_buffers), culled_cw, main_depth(true), fragment("mesh_fs")),
            skinned: make("skinned", &skinned_layout, vertex("skinned_vs", &skin_buffers), culled_ccw, main_depth(true), fragment("mesh_fs")),
            // Shadows drawn double-sided: thin parts (leaves, cloth) must cast from both sides
            shadow_mesh: make("shadow mesh", &shadow_mesh_layout, vertex("shadow_mesh_vs", &mesh_buffers), Default::default(), shadow_depth.clone(), None),
            shadow_skinned: make("shadow skinned", &shadow_skinned_layout, vertex("shadow_skinned_vs", &skin_buffers), Default::default(), shadow_depth, None),
        }
    }
}
