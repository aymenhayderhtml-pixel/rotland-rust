// One cheap lighting model for the whole world: sun (Lambert) + sky ambient + depth fog + Godot's filmic curve.
// Matches the look of the Godot version (DayNightVisuals + World.tscn environment) at a fraction of the cost.

struct Scene {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,        // xyz: toward the sun, w: energy
    sun_color: vec4<f32>,
    sky_top: vec4<f32>,
    sky_horizon: vec4<f32>,
    ground_horizon: vec4<f32>,
    ground_bottom: vec4<f32>,
    fog: vec4<f32>,            // begin, end, density, ambient energy
    fog_color: vec4<f32>,
};
@group(0) @binding(0) var<uniform> scene: Scene;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    let lo = c / 12.92;
    let hi = pow((c + 0.055) / 1.055, vec3(2.4));
    return select(hi, lo, c <= vec3(0.04045));
}

// Godot 4 tonemap_filmic with white = 1
fn filmic(c: vec3<f32>) -> vec3<f32> {
    let A = 0.22 * 4.0; let B = 0.30 * 2.0; let C = 0.10; let D = 0.20; let E = 0.01; let F = 0.30;
    let t = ((c * (A * c + C * B) + D * E) / (c * (A * c + B) + D * F)) - E / F;
    let w = 1.0;
    let wt = ((w * (A * w + C * B) + D * E) / (w * (A * w + B) + D * F)) - E / F;
    return t / wt;
}

fn sky_color(dir: vec3<f32>) -> vec3<f32> {
    let PI = 3.14159265;
    let v_angle = acos(clamp(dir.y, -1.0, 1.0));
    let c_sky = 1.0 - v_angle / (PI * 0.5);
    let sky = mix(scene.sky_horizon.rgb, scene.sky_top.rgb, clamp(1.0 - pow(max(1.0 - c_sky, 0.0), 1.0 / 0.3), 0.0, 1.0));
    let c_ground = (v_angle - PI * 0.5) / (PI * 0.5);
    let ground = mix(scene.ground_horizon.rgb, scene.ground_bottom.rgb, clamp(1.0 - pow(max(1.0 - c_ground, 0.0), 1.0 / 0.02), 0.0, 1.0));
    return select(ground, sky, dir.y >= 0.0);
}

// ---- sky: one full-screen triangle drawn behind everything (depth 0 = infinitely far in reverse-Z)
struct SkyOut { @builtin(position) pos: vec4<f32>, @location(0) ndc: vec2<f32> };

@vertex
fn sky_vs(@builtin(vertex_index) i: u32) -> SkyOut {
    let uv = vec2(f32((i << 1u) & 2u), f32(i & 2u));
    let ndc = uv * 2.0 - 1.0;
    var o: SkyOut;
    o.pos = vec4(ndc, 0.0, 1.0);
    o.ndc = ndc;
    return o;
}

@fragment
fn sky_fs(i: SkyOut) -> @location(0) vec4<f32> {
    let far = scene.inv_view_proj * vec4(i.ndc, 0.0001, 1.0);
    let dir = normalize(far.xyz / far.w - scene.camera_pos.xyz);
    return vec4(filmic(sky_color(dir)), 1.0);
}

// ---- lit coloured meshes
struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
};
struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) albedo: vec3<f32>,
};

// Per-draw offset (chunk origin), so chunk meshes keep small local coordinates
struct Draw { offset: vec4<f32> };
@group(1) @binding(0) var<uniform> draw: Draw;

@vertex
fn mesh_vs(v: VsIn) -> VsOut {
    let world = v.position + draw.offset.xyz;
    var o: VsOut;
    o.pos = scene.view_proj * vec4(world, 1.0);
    o.world = world;
    o.normal = v.normal;
    o.albedo = srgb_to_linear(v.color.rgb);
    return o;
}

@fragment
fn mesh_fs(i: VsOut) -> @location(0) vec4<f32> {
    let n = normalize(i.normal);
    let ndl = max(dot(n, scene.sun_dir.xyz), 0.0);
    let sun = scene.sun_color.rgb * scene.sun_dir.w * ndl;
    // Sky ambient: lerp ground to sky by how much the surface faces up
    let up = n.y * 0.5 + 0.5;
    let ambient = mix(scene.ground_horizon.rgb, scene.sky_top.rgb * 0.5 + scene.sky_horizon.rgb * 0.5, up) * scene.fog.w;
    var color = i.albedo * (sun + ambient);
    let dist = distance(i.world, scene.camera_pos.xyz);
    let fog = smoothstep(scene.fog.x, scene.fog.y, dist) * scene.fog.z;
    color = mix(color, scene.fog_color.rgb, fog);
    return vec4(filmic(color), 1.0);
}

// ---- skinned meshes (characters): rigid skinning, many instances per draw.
// Each instance carries its world matrix and where its joints start in the shared bone list.
@group(2) @binding(0) var<storage, read> bones: array<mat4x4<f32>>;

struct SkinIn {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) bone: vec4<u32>,
    @location(4) m0: vec4<f32>,
    @location(5) m1: vec4<f32>,
    @location(6) m2: vec4<f32>,
    @location(7) m3: vec4<f32>,
    @location(8) bone_base: u32,
};

@vertex
fn skinned_vs(v: SkinIn) -> VsOut {
    let model = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);
    let skin = model * bones[v.bone_base + v.bone.x];
    let world = (skin * vec4(v.position, 1.0)).xyz;
    var o: VsOut;
    o.pos = scene.view_proj * vec4(world, 1.0);
    o.world = world;
    o.normal = (skin * vec4(v.normal, 0.0)).xyz;
    o.albedo = srgb_to_linear(v.color.rgb);
    return o;
}
