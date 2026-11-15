use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use super::mesh::{GpuMesh, COLOR_VERTEX_LAYOUT};

/// Per-instance data of a skinned draw: world matrix and the index of this instance's first joint.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct SkinInstance {
    pub model: [[f32; 4]; 4],
    pub bone_base: u32,
    pub _pad: [u32; 3],
}

impl SkinInstance {
    pub fn new(model: Mat4, bone_base: u32) -> Self {
        Self { model: model.to_cols_array_2d(), bone_base, _pad: [0; 3] }
    }

    const ATTRS: [wgpu::VertexAttribute; 5] =
        wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Uint32];

    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SkinInstance>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &Self::ATTRS,
    };
}

/// All instances of one skinned model for this frame: drawn with one instanced draw call.
pub struct SkinnedBatch<'a> {
    pub mesh: &'a GpuMesh,
    pub instances: Vec<SkinInstance>,
}

pub(crate) fn pipeline(device: &wgpu::Device, module: &wgpu::ShaderModule, layout: &wgpu::PipelineLayout) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("skinned"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some("skinned_vs"),
            compilation_options: Default::default(),
            buffers: &[Some(COLOR_VERTEX_LAYOUT), Some(SkinInstance::LAYOUT)],
        },
        primitive: wgpu::PrimitiveState { front_face: wgpu::FrontFace::Cw, cull_mode: Some(wgpu::Face::Back), ..Default::default() },
        depth_stencil: Some(super::lit::depth_state(true)),
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some("mesh_fs"),
            compilation_options: Default::default(),
            targets: &[Some(crate::target::COLOR_FORMAT.into())],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// A GPU buffer that grows (doubling) when a frame needs more room than it has.
pub(crate) struct GrowBuffer {
    pub buffer: wgpu::Buffer,
    usage: wgpu::BufferUsages,
    label: &'static str,
}

impl GrowBuffer {
    pub fn new(device: &wgpu::Device, label: &'static str, usage: wgpu::BufferUsages, size: u64) -> Self {
        let usage = usage | wgpu::BufferUsages::COPY_DST;
        Self { buffer: device.create_buffer(&wgpu::BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false }), usage, label }
    }

    /// Uploads `data`; returns true when the buffer was replaced (bind groups using it must be rebuilt).
    pub fn write(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, data: &[u8]) -> bool {
        let mut grew = false;
        if data.len() as u64 > self.buffer.size() {
            let size = (data.len() as u64).next_power_of_two();
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor { label: Some(self.label), size, usage: self.usage, mapped_at_creation: false });
            grew = true;
        }
        if !data.is_empty() {
            queue.write_buffer(&self.buffer, 0, data);
        }
        grew
    }
}
