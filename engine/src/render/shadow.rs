//! Sun shadows: one shadow map that follows the camera (like Godot's single-split directional shadow).

use glam::{Mat4, Vec3, Vec4Swizzles};

use super::camera::{ortho, view_matrix, Camera};

pub const SHADOW_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

/// How the sun shadow is set up. Godot's medium preset: 50 m, 2048 px.
#[derive(Clone, Copy, Debug)]
pub struct ShadowSettings {
    pub enabled: bool,
    pub size: u32,
    pub distance: f32,
}

impl Default for ShadowSettings {
    fn default() -> Self {
        Self { enabled: true, size: 2048, distance: 50.0 }
    }
}

pub struct ShadowMap {
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: u32,
}

impl ShadowMap {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let view = device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("sun shadow"),
                size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: 1 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: SHADOW_FORMAT,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
            .create_view(&Default::default());
        // Linear + compare = the GPU blends 4 depth tests per tap: soft edges for free
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sun shadow"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        Self { view, sampler, size }
    }
}

/// The sun's view-projection: a box `distance` wide around the area in front of the camera,
/// snapped to whole shadow texels so shadow edges do not crawl when the camera moves.
pub fn light_view_proj(camera: &Camera, sun_toward: Vec3, settings: &ShadowSettings) -> Mat4 {
    let f = camera.forward();
    let flat = Vec3::new(f.x, 0.0, f.z).normalize_or(Vec3::NEG_Z);
    let radius = settings.distance * 0.6;
    let center = camera.position + flat * settings.distance * 0.4;
    let depth = 200.0;
    let sun = sun_toward.normalize_or(Vec3::Y);
    let view = view_matrix(center + sun * depth * 0.5, -sun);
    let texel = 2.0 * radius / settings.size as f32;
    let c = (view * center.extend(1.0)).xy();
    let snapped = (c / texel).round() * texel;
    let shift = Mat4::from_translation((snapped - c).extend(0.0) * -1.0);
    ortho(radius, radius, 0.0, depth) * shift * view
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_ground_point_lands_inside_the_shadow_map() {
        let cam = Camera { position: Vec3::new(100.0, 5.0, -40.0), ..Default::default() };
        let m = light_view_proj(&cam, Vec3::new(-0.2, 0.9, 0.4), &ShadowSettings::default());
        let p = m * Vec3::new(100.0, 0.0, -50.0).extend(1.0);
        let ndc = p.xyz() / p.w;
        assert!(ndc.x.abs() < 1.0 && ndc.y.abs() < 1.0 && (0.0..1.0).contains(&ndc.z), "{ndc}");
    }
}
