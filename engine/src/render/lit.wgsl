// One cheap lighting model for the whole world: sun (Lambert) + sky ambient + depth fog + Godot's filmic curve.
// Matches the look of the Godot version (DayNightVisuals + World.tscn environment) at a fraction of the cost.

struct Scene {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    light_view_proj: mat4x4<f32>,
    shadow: vec4<f32>,         // x: 1 when sun shadows are on, y: shadow-map texel size, z: fade start, w: fade end
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,        // xyz: toward the sun, w: energy
    sun_color: vec4<f32>,
    sky_top: vec4<f32>,
    sky_horizon: vec4<f32>,
    ground_horizon: vec4<f32>,
    ground_bottom: vec4<f32>,
    fog: vec4<f32>,            // begin, end, density, ambient energy
    fog_color: vec4<f32>,
    moon_dir: vec4<f32>,       // xyz: toward the moon, w: energy
    moon_color: vec4<f32>,
    spot_pos: vec4<f32>,       // xyz: position, w: range (0 = no spot light)
    spot_dir: vec4<f32>,       // xyz: direction it points, w: cos of the cone half angle
    spot_color: vec4<f32>,     // rgb: linear colour, w: energy
};
@group(0) @binding(0) var<uniform> scene: Scene;

// Godot 4's spot light: range window times 1/d (attenuation 1), and a soft cone edge (spot attenuation 1)
fn spot_light(world: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    let range = scene.spot_pos.w;
    if (range <= 0.0) {
        return vec3(0.0);
    }
    let to_light = scene.spot_pos.xyz - world;
    let d = length(to_light);
    if (d >= range) {
        return vec3(0.0);
    }
    let l = to_light / max(d, 0.0001);
    var nd = d / range;
    nd = nd * nd;
    nd = nd * nd;
    nd = max(1.0 - nd, 0.0);
    let dist_att = nd * nd / max(d, 0.0001);
    let cos_cut = scene.spot_dir.w;
    let scos = max(dot(-l, normalize(scene.spot_dir.xyz)), cos_cut);
    let rim = max(0.0001, (1.0 - scos) / (1.0 - cos_cut));
    let cone = 1.0 - rim;
    return scene.spot_color.rgb * scene.spot_color.w * dist_att * cone * max(dot(n, l), 0.0);
}
@group(0) @binding(1) var shadow_map: texture_depth_2d;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;

// Sun shadow: 1 = lit, 0 = shadowed. 4 hardware-filtered taps (each tap already blends 2x2 texels),
// so edges are soft for the price of 4 samples. Fades out toward the shadow distance like Godot.
fn sun_shadow(world: vec3<f32>, n: vec3<f32>, dist: f32) -> f32 {
    if (scene.shadow.x < 0.5) {
        return 1.0;
    }
    // Normal offset: push the lookup off the surface a little, which removes most shadow acne
    let p = scene.light_view_proj * vec4(world + n * 0.06, 1.0);
    let ndc = p.xyz / p.w;
    let uv = vec2(ndc.x * 0.5 + 0.5, 0.5 - ndc.y * 0.5);
    if (any(uv < vec2(0.0)) || any(uv > vec2(1.0)) || ndc.z > 1.0) {
        return 1.0;
    }
    let t = scene.shadow.y;
    var lit = 0.0;
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2(-0.5, -0.5) * t, ndc.z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2(0.5, -0.5) * t, ndc.z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2(-0.5, 0.5) * t, ndc.z);
    lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + vec2(0.5, 0.5) * t, ndc.z);
    let fade = smoothstep(scene.shadow.z, scene.shadow.w, dist);
    return mix(lit * 0.25, 1.0, fade);
}

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
    let dist = distance(i.world, scene.camera_pos.xyz);
    let ndl = max(dot(n, scene.sun_dir.xyz), 0.0);
    var shade = 1.0;
    if (ndl > 0.0) {
        shade = sun_shadow(i.world, n, dist);
    }
    let sun = scene.sun_color.rgb * scene.sun_dir.w * ndl * shade;
    // Sky ambient: lerp ground to sky by how much the surface faces up
    let up = n.y * 0.5 + 0.5;
    let ambient = mix(scene.ground_horizon.rgb, scene.sky_top.rgb * 0.5 + scene.sky_horizon.rgb * 0.5, up) * scene.fog.w;
    // The moon lights the night without shadows (like Godot's moon light), the flashlight is the spot light
    let moon = scene.moon_color.rgb * scene.moon_dir.w * max(dot(n, scene.moon_dir.xyz), 0.0);
    var color = i.albedo * (sun + moon + ambient + spot_light(i.world, n));
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

// ---- shadow pass: depth only, from the sun
@vertex
fn shadow_mesh_vs(v: VsIn) -> @builtin(position) vec4<f32> {
    return scene.light_view_proj * vec4(v.position + draw.offset.xyz, 1.0);
}

@vertex
fn shadow_skinned_vs(v: SkinIn) -> @builtin(position) vec4<f32> {
    let model = mat4x4<f32>(v.m0, v.m1, v.m2, v.m3);
    return scene.light_view_proj * (model * bones[v.bone_base + v.bone.x] * vec4(v.position, 1.0));
}
