use wgpu::util::DeviceExt;

/// One vertex of a coloured, flat-shaded mesh: the baked model-pack vertex (terrain uses it too).
/// Colour is sRGB bytes, as authored; the shader converts it to linear. `bone` is for rigid skinning.
pub use pack::PackVertex as ColorVertex;

/// GPU layout of ColorVertex (32 bytes: position, normal, sRGB colour, bone + padding).
pub const COLOR_VERTEX_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
    array_stride: std::mem::size_of::<ColorVertex>() as u64,
    step_mode: wgpu::VertexStepMode::Vertex,
    attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Unorm8x4, 3 => Uint8x4],
};

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
        crate::timing::count_new_mesh();
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
