use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// One vertex of a coloured, flat-shaded mesh (terrain, and later baked models).
/// Colour is sRGB bytes, as authored; the shader converts it to linear.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ColorVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [u8; 4],
}

impl ColorVertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<ColorVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Unorm8x4],
    };
}

/// A mesh living on the GPU. Non-indexed: flat shading needs its own vertices per triangle anyway.
pub struct GpuMesh {
    pub vertices: wgpu::Buffer,
    pub vertex_count: u32,
    /// World-space bounds, for culling.
    pub min: glam::Vec3,
    pub max: glam::Vec3,
}

impl GpuMesh {
    pub fn new(device: &wgpu::Device, label: &str, vertices: &[ColorVertex]) -> Self {
        let mut min = glam::Vec3::splat(f32::MAX);
        let mut max = glam::Vec3::splat(f32::MIN);
        for v in vertices {
            let p = glam::Vec3::from(v.position);
            min = min.min(p);
            max = max.max(p);
        }
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self { vertices: buffer, vertex_count: vertices.len() as u32, min, max }
    }
}
