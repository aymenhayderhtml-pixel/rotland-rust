// Low-poly stylized drifting clouds on a flat sky plane (one full-screen triangle, no textures, no loops).

struct SkyUniforms {
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    sunlit_color: vec4<f32>, // rgb: linear sunlit color, w: max alpha
    shadow_color: vec4<f32>, // rgb: linear shaded color, w: edge softness
    cloud: vec4<f32>,        // x: cover (0..1), y: uv scale, zw: wind offset
};
@group(0) @binding(0) var<uniform> u: SkyUniforms;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) ndc: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VsOut {
    let uv = vec2(f32((i << 1u) & 2u), f32(i & 2u));
    let ndc = uv * 2.0 - 1.0;
    var o: VsOut;
    // Reverse-Z: 0.0 is infinitely far; only sky pixels (depth == 0.0) pass GreaterEqual.
    o.pos = vec4(ndc, 0.0, 1.0);
    o.ndc = ndc;
    return o;
}

fn hash2(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2(123.34, 456.21));
    let r = q + dot(q, q + 45.32);
    return fract(r.x * r.y);
}

fn noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let s = f * f * (3.0 - 2.0 * f);
    let a = hash2(i);
    let b = hash2(i + vec2(1.0, 0.0));
    let c = hash2(i + vec2(0.0, 1.0));
    let d = hash2(i + vec2(1.0, 1.0));
    return mix(mix(a, b, s.x), mix(c, d, s.x), s.y);
}

fn cloud_density(uv: vec2<f32>) -> f32 {
    let n0 = noise2(uv);
    let n1 = noise2(uv * 2.13 + vec2(17.3, 9.1));
    return n0 * 0.65 + n1 * 0.35;
}

// Godot 4 tonemap_filmic with white = 1 (matches lit.wgsl)
fn filmic(c: vec3<f32>) -> vec3<f32> {
    let A = 0.22 * 4.0; let B = 0.30 * 2.0; let C = 0.10; let D = 0.20; let E = 0.01; let F = 0.30;
    let t = ((c * (A * c + C * B) + D * E) / (c * (A * c + B) + D * F)) - E / F;
    let w = 1.0;
    let wt = ((w * (A * w + C * B) + D * E) / (w * (A * w + B) + D * F)) - E / F;
    return t / wt;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    let far = u.inv_view_proj * vec4(i.ndc, 0.0001, 1.0);
    let dir = normalize(far.xyz / far.w - u.camera_pos.xyz);
    if (dir.y <= 0.015) {
        return vec4(0.0);
    }

    let horizon_y = dir.y + 0.12;
    let plane_uv = (dir.xz / horizon_y) * u.cloud.y + u.cloud.zw;
    let density = cloud_density(plane_uv);

    let cover = clamp(u.cloud.x, 0.0, 1.0);
    let threshold = mix(0.82, 0.22, cover);
    let softness = max(u.shadow_color.w, 0.01);
    let raw_alpha = smoothstep(threshold, threshold + softness, density);
    if (raw_alpha <= 0.001) {
        return vec4(0.0);
    }

    // Flat-shaded 3-band low-poly bevel toward the sun direction.
    let sun_uv = plane_uv + u.sun_dir.xz * 0.14;
    let n_sun = cloud_density(sun_uv);
    let shade_raw = clamp((density - n_sun) * 3.5 + 0.5, 0.0, 1.0);
    let shade = clamp(floor(shade_raw * 3.0) / 2.0, 0.0, 1.0);
    let col = mix(u.shadow_color.rgb, u.sunlit_color.rgb, shade);

    let horizon_fade = smoothstep(0.015, 0.18, dir.y);
    let alpha = raw_alpha * horizon_fade * u.sunlit_color.w;
    return vec4(filmic(col), alpha);
}
