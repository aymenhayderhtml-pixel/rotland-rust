use bytemuck::{Pod, Zeroable};
use glam::Mat4;

use super::mesh::GpuMesh;

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
        Self { model: model.to_cols_array_2d(), bone_base, _pad: [1.0_f32.to_bits(), 0, 0] }
    }

    pub fn with_fade(model: Mat4, bone_base: u32, fade: f32, complementary: bool) -> Self {
        let val = if complementary { -fade.abs() } else { fade.abs() };
        Self { model: model.to_cols_array_2d(), bone_base, _pad: [val.to_bits(), 0, 0] }
    }

    pub fn fade(&self) -> (f32, bool) {
        let bits = self._pad[0];
        let comp = (bits & 0x8000_0000) != 0;
        let fade = f32::from_bits(bits & 0x7FFF_FFFF);
        (fade, comp)
    }

    /// Returns true if this instance is inside an LOD cross-fade band (requires dither discard).
    pub fn is_fading(&self) -> bool {
        let (fade, comp) = self.fade();
        comp || fade < 1.0
    }

    const ATTRS: [wgpu::VertexAttribute; 6] =
        wgpu::vertex_attr_array![4 => Float32x4, 5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Uint32, 9 => Float32];

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

/// Pure helper: splits a list of instances into normal (fade 1.0, non-complementary) and fading.
pub fn split_instances(instances: &[SkinInstance]) -> (Vec<SkinInstance>, Vec<SkinInstance>) {
    let mut normal = Vec::new();
    let mut fading = Vec::new();
    for &inst in instances {
        if inst.is_fading() {
            fading.push(inst);
        } else {
            normal.push(inst);
        }
    }
    (normal, fading)
}

/// Pure helper: splits instanced batches into normal (fade 1.0) and fading batches.
/// Instances with fade 1.0 go to the normal batch, others to the fade batch.
/// Batches with zero instances after splitting are omitted.
pub fn split_batches<'a>(
    batches: &[SkinnedBatch<'a>],
) -> (Vec<SkinnedBatch<'a>>, Vec<SkinnedBatch<'a>>) {
    let mut normal = Vec::new();
    let mut fading = Vec::new();
    for b in batches {
        let (n_inst, f_inst) = split_instances(&b.instances);
        if !n_inst.is_empty() {
            normal.push(SkinnedBatch { mesh: b.mesh, instances: n_inst });
        }
        if !f_inst.is_empty() {
            fading.push(SkinnedBatch { mesh: b.mesh, instances: f_inst });
        }
    }
    (normal, fading)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skin_instance_lod_fade_encoding() {
        let inst = SkinInstance::new(Mat4::IDENTITY, 0);
        assert_eq!(inst.fade(), (1.0, false));

        let f1 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.75, false);
        assert_eq!(f1.fade(), (0.75, false));

        let f2 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.25, true);
        assert_eq!(f2.fade(), (0.25, true));

        let f3 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.0, true);
        assert_eq!(f3.fade(), (0.0, true));
    }

    #[test]
    fn test_skin_instance_is_fading() {
        let normal = SkinInstance::new(Mat4::IDENTITY, 0);
        assert!(!normal.is_fading());

        let normal_fade = SkinInstance::with_fade(Mat4::IDENTITY, 0, 1.0, false);
        assert!(!normal_fade.is_fading());

        let fading1 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.75, false);
        assert!(fading1.is_fading());

        let fading2 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 0.25, true);
        assert!(fading2.is_fading());

        let fading3 = SkinInstance::with_fade(Mat4::IDENTITY, 0, 1.0, true);
        assert!(fading3.is_fading());
    }

    #[test]
    fn test_lod_batch_split_instances() {
        let s1 = SkinInstance::new(Mat4::IDENTITY, 0);
        let s2 = SkinInstance::with_fade(Mat4::IDENTITY, 1, 1.0, false);
        let f1 = SkinInstance::with_fade(Mat4::IDENTITY, 2, 0.6, false);
        let f2 = SkinInstance::with_fade(Mat4::IDENTITY, 3, 0.4, true);

        let (normal, fading) = split_instances(&[s1, f1, s2, f2]);
        assert_eq!(normal.len(), 2);
        assert_eq!(fading.len(), 2);
        assert_eq!(normal[0].bone_base, 0);
        assert_eq!(normal[1].bone_base, 1);
        assert_eq!(fading[0].bone_base, 2);
        assert_eq!(fading[1].bone_base, 3);
    }

    #[test]
    fn test_lod_batch_split_batches() {
        let dummy = std::mem::MaybeUninit::<GpuMesh>::uninit();
        let mesh_ptr: &'static GpuMesh = unsafe { &*dummy.as_ptr() };

        let s1 = SkinInstance::new(Mat4::IDENTITY, 0);
        let f1 = SkinInstance::with_fade(Mat4::IDENTITY, 1, 0.5, false);
        let f2 = SkinInstance::with_fade(Mat4::IDENTITY, 2, 0.5, true);

        let batch1 = SkinnedBatch { mesh: mesh_ptr, instances: vec![s1, f1] };
        let batch2 = SkinnedBatch { mesh: mesh_ptr, instances: vec![s1, s1] };
        let batch3 = SkinnedBatch { mesh: mesh_ptr, instances: vec![f2] };
        let batch4 = SkinnedBatch { mesh: mesh_ptr, instances: vec![] };

        let (normal, fading) = split_batches(&[batch1, batch2, batch3, batch4]);

        assert_eq!(normal.len(), 2);
        assert_eq!(fading.len(), 2);

        assert_eq!(normal[0].instances.len(), 1);
        assert_eq!(normal[1].instances.len(), 2);
        assert_eq!(fading[0].instances.len(), 1);
        assert_eq!(fading[1].instances.len(), 1);
    }
}
