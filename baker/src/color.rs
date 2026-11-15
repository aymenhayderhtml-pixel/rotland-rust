//! Palette texture sampling and color space conversion.
//!
//! Rotland models use a shared palette image texture where each face's UV points
//! to the center of a color cell. During baking, we sample this texture at the
//! triangle's UV centroid and store the final sRGB color on each vertex.

/// Sample palette image at (u, v) with NEAREST interpolation and multiply by base color factor.
pub fn sample_palette_nearest(
    img: &gltf::image::Data,
    u: f32,
    v: f32,
    base_color_factor: [f32; 4],
) -> [u8; 4] {
    let w = img.width;
    let h = img.height;
    if w == 0 || h == 0 {
        return factor_to_srgb(base_color_factor);
    }

    let u_wrap = u.rem_euclid(1.0);
    let v_wrap = v.rem_euclid(1.0);
    let x = ((u_wrap * w as f32).floor() as u32).min(w - 1);
    let y = ((v_wrap * h as f32).floor() as u32).min(h - 1);
    let pixel_idx = (y * w + x) as usize;

    let (r, g, b, a) = match img.format {
        gltf::image::Format::R8G8B8 => {
            let offset = pixel_idx * 3;
            if offset + 2 < img.pixels.len() {
                (img.pixels[offset], img.pixels[offset + 1], img.pixels[offset + 2], 255)
            } else {
                (255, 255, 255, 255)
            }
        }
        gltf::image::Format::R8G8B8A8 => {
            let offset = pixel_idx * 4;
            if offset + 3 < img.pixels.len() {
                (img.pixels[offset], img.pixels[offset + 1], img.pixels[offset + 2], img.pixels[offset + 3])
            } else {
                (255, 255, 255, 255)
            }
        }
        gltf::image::Format::R8 => {
            if pixel_idx < img.pixels.len() {
                let v = img.pixels[pixel_idx];
                (v, v, v, 255)
            } else {
                (255, 255, 255, 255)
            }
        }
        gltf::image::Format::R8G8 => {
            let offset = pixel_idx * 2;
            if offset + 1 < img.pixels.len() {
                (img.pixels[offset], img.pixels[offset + 1], 0, 255)
            } else {
                (255, 255, 255, 255)
            }
        }
        _ => (255, 255, 255, 255),
    };

    let r_out = ((r as f32 * base_color_factor[0]).round().clamp(0.0, 255.0)) as u8;
    let g_out = ((g as f32 * base_color_factor[1]).round().clamp(0.0, 255.0)) as u8;
    let b_out = ((b as f32 * base_color_factor[2]).round().clamp(0.0, 255.0)) as u8;
    let a_out = ((a as f32 * base_color_factor[3]).round().clamp(0.0, 255.0)) as u8;

    [r_out, g_out, b_out, a_out]
}

/// Convert linear base color factor float to sRGB u8 bytes.
pub fn factor_to_srgb(factor: [f32; 4]) -> [u8; 4] {
    let to_srgb = |c: f32| -> u8 {
        let c = c.clamp(0.0, 1.0);
        let s = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (s * 255.0).round().clamp(0.0, 255.0) as u8
    };

    [
        to_srgb(factor[0]),
        to_srgb(factor[1]),
        to_srgb(factor[2]),
        (factor[3].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}
