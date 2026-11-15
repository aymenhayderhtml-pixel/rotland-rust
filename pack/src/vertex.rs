use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

/// Vertex layout shared by baker, asset pack and GPU renderer.
///
/// 32 bytes in memory: 12-byte position, 12-byte normal, 4-byte sRGB color,
/// 1-byte bone index, and 3-byte padding. Derives Pod for direct zero-copy GPU
/// vertex buffer uploads.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct PackVertex {
    /// Model-space position.
    pub position: [f32; 3],
    /// Model-space normal vector (unit length).
    pub normal: [f32; 3],
    /// sRGB color bytes [R, G, B, A] sampled from the palette cell.
    pub color: [u8; 4],
    /// Rigid skinning bone index (joint with largest weight, or 0 for unskinned).
    pub bone: u8,
    /// Explicit padding to ensure 4-byte alignment and 32-byte stride.
    pub _pad: [u8; 3],
}

impl PackVertex {
    /// A default blank vertex for initialization.
    pub const ZERO: Self = Self {
        position: [0.0; 3],
        normal: [0.0, 1.0, 0.0],
        color: [255, 255, 255, 255],
        bone: 0,
        _pad: [0; 3],
    };
}
