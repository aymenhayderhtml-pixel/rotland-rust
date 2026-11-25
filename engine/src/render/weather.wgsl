// Weather particles: one instanced quad per particle. The vertex shader hashes the instance
// index to place each quad inside a 40 m box that follows the camera; fall and wind offsets
// wrap with fract() so particles cycle forever and the CPU only changes the draw count.
// Kinds: 1 rain (thin fast streaks slanted by the wind), 2 snow (soft round flakes with a
// little sway), 3 sandstorm (larger, faint, almost horizontal dust hugging the bottom of the
// box). Clear (0) and fog (4) never reach the GPU: the draw count is 0.
// Depth-tested against the lit frame with reverse-Z (GreaterEqual, no depth write) like the
// water pass, so particles hide behind walls and trees; alpha-blended exactly like water.

struct WeatherUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,  // xyz: camera position (the particle box follows it)
    wind_time: vec4<f32>,   // x: wind x (m/s), y: wind z (m/s), z: time (s)
    tint: vec4<f32>,        // rgb: linear tint (data/weather.json kinds.<kind>.tint)
    kind: vec4<f32>,        // x: kind_index (0 clear, 1 rain, 2 snow, 3 sandstorm, 4 fog)
};

@group(0) @binding(0) var<uniform> u: WeatherUniforms;

struct VsIn {
    @location(0) corner: vec2<f32>,  // quad corner in [-1, 1]
};

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) local: vec2<f32>,  // corner in [-1, 1], for the soft round alpha
    @location(1) alpha: f32,        // per-particle alpha: base x near fade x box-edge fades
    @location(2) soft: f32,         // 1.0: apply the round alpha falloff in the fragment shader
};

// Kind sizes, speeds and alphas (owner decisions in tasks/R32-02-weather-render.md).
const BOX_SIZE: f32 = 40.0;         // new: DECISIONS 2026-11-25 (task: a 40 m box)
const BOX_BOTTOM_DROP: f32 = 2.0;   // new: DECISIONS 2026-11-25 (box bottom just below the eye)
const SAND_BAND: f32 = 6.0;         // new: DECISIONS 2026-11-25 (task: bottom 6 m of the box)
const RAIN_FALL: f32 = 9.0;         // new: DECISIONS 2026-11-25 (task: 9 m/s)
const SNOW_FALL: f32 = 1.2;         // new: DECISIONS 2026-11-25 (task: 1.2 m/s)
const SAND_FALL: f32 = 0.4;         // new: DECISIONS 2026-11-25 (dust settles only slightly)
const RAIN_HALF_W: f32 = 0.01;      // new: DECISIONS 2026-11-25 (task: 0.02 m wide streak)
const RAIN_HALF_H: f32 = 0.3;       // new: DECISIONS 2026-11-25 (task: 0.6 m long streak)
const SNOW_SIZE: f32 = 0.03;        // new: DECISIONS 2026-11-25 (task: 0.06 m flake)
const SAND_HALF_L: f32 = 0.075;     // new: DECISIONS 2026-11-25 (task: 0.15 m blob)
const SAND_HALF_V: f32 = 0.023;     // new: DECISIONS 2026-11-25 (squashed: almost horizontal)
const RAIN_ALPHA: f32 = 0.35;       // new: DECISIONS 2026-11-25 (task: alpha 0.35)
const SNOW_ALPHA: f32 = 0.8;        // new: DECISIONS 2026-11-25 (soft flake core)
const SAND_ALPHA: f32 = 0.18;       // new: DECISIONS 2026-11-25 (task: alpha 0.18)
const NEAR_FADE: f32 = 0.5;         // new: DECISIONS 2026-11-25 (task: fade in < 0.5 m)
const EDGE_FADE: f32 = 2.0;         // new: DECISIONS 2026-11-25 (fade band at the box edges)
const SNOW_SWAY: f32 = 0.3;         // new: DECISIONS 2026-11-25 (sway amplitude in m)
const SNOW_SWAY_SPEED: f32 = 1.3;   // new: DECISIONS 2026-11-25 (sway speed in rad/s)
const SOFT_INNER: f32 = 0.3;        // new: DECISIONS 2026-11-25 (disc radius where soft fades start)

// Standard PCG hash constants (not game numbers).
fn pcg_hash(v: u32) -> u32 {
    let state = v * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return (word >> 22u) ^ word;
}

// Random in [0, 1) from one u32 seed.
fn rand(seed: u32) -> f32 {
    return f32(pcg_hash(seed)) / 4294967296.0;
}

// Camera basis from the view rows of the view-projection matrix (billboarding without
// extra uniforms): row 0 is the camera right, row 1 the camera up, both world-space.
fn cam_right() -> vec3<f32> {
    return normalize(vec3<f32>(u.view_proj[0].x, u.view_proj[1].x, u.view_proj[2].x));
}

fn cam_up() -> vec3<f32> {
    return normalize(vec3<f32>(u.view_proj[0].y, u.view_proj[1].y, u.view_proj[2].y));
}

@vertex
fn vs_main(v: VsIn, @builtin(instance_index) id: u32) -> VsOut {
    let t = u.wind_time.z;
    let kind = u.kind.x;

    // Three stable seeds per particle: hash(instance id).
    let seed = id * 3u;
    let sx = rand(seed);
    let sz = rand(seed + 1u);
    let sy = rand(seed + 2u);

    // The 40 m box follows the camera: xz centered on it, bottom BOX_BOTTOM_DROP below it.
    let box_min = vec3<f32>(
        u.camera_pos.x - BOX_SIZE * 0.5,
        u.camera_pos.y - BOX_BOTTOM_DROP,
        u.camera_pos.z - BOX_SIZE * 0.5,
    );

    // Wind drift, wrapped by fract() so particles cycle inside the box forever.
    let wx = u.camera_pos.x + (fract(sx + u.wind_time.x * t / BOX_SIZE) - 0.5) * BOX_SIZE;
    let wz = u.camera_pos.z + (fract(sz + u.wind_time.y * t / BOX_SIZE) - 0.5) * BOX_SIZE;

    var center: vec3<f32>;
    var axis_long: vec3<f32>;
    var axis_short: vec3<f32>;
    var half_long: f32;
    var half_short: f32;
    var base_alpha: f32;
    var soft: f32;
    var vert_fade: f32 = 1.0;

    if (kind == 2.0) {
        // Snow: slow round flakes with a little sideways sway (sin of time + id).
        let fy = fract(sy - t * SNOW_FALL / BOX_SIZE);
        let cy = box_min.y + fy * BOX_SIZE;
        center = vec3<f32>(
            wx + sin(t * SNOW_SWAY_SPEED + f32(id)) * SNOW_SWAY,
            cy,
            wz + cos(t * SNOW_SWAY_SPEED + f32(id)) * SNOW_SWAY,
        );
        axis_long = cam_up();
        axis_short = cam_right();
        half_long = SNOW_SIZE;
        half_short = SNOW_SIZE;
        base_alpha = SNOW_ALPHA;
        soft = 1.0;
        // Snow wraps vertically, so also fade near the box top and bottom.
        let fy_m = cy - box_min.y;
        vert_fade = smoothstep(0.0, EDGE_FADE, min(fy_m, BOX_SIZE - fy_m));
    } else if (kind == 3.0) {
        // Sandstorm: larger faint blobs, blown fast along the wind (wind x time), low in the
        // bottom 6 m of the box, squashed almost horizontal.
        var dir = vec3<f32>(u.wind_time.x, 0.0, u.wind_time.y);
        if (dot(dir.xz, dir.xz) < 0.0001) {
            dir = vec3<f32>(1.0, 0.0, 0.0);
        }
        center = vec3<f32>(wx, box_min.y + sy * SAND_BAND, wz);
        axis_long = normalize(dir);
        axis_short = vec3<f32>(0.0, 1.0, 0.0);
        half_long = SAND_HALF_L;
        half_short = SAND_HALF_V;
        base_alpha = SAND_ALPHA;
        soft = 1.0;
    } else {
        // Rain (kind 1): thin streaks falling fast, slanted by the wind; they stretch along
        // the motion vector and are seen side-on from the camera.
        let fy = fract(sy - t * RAIN_FALL / BOX_SIZE);
        let cy = box_min.y + fy * BOX_SIZE;
        center = vec3<f32>(wx, cy, wz);
        let fall_dir = normalize(vec3<f32>(u.wind_time.x, -RAIN_FALL, u.wind_time.y));
        let to_cam = normalize(u.camera_pos.xyz - center);
        var side = cross(fall_dir, to_cam);
        if (dot(side, side) < 0.000001) {
            side = vec3<f32>(u.view_proj[0].x, u.view_proj[1].x, u.view_proj[2].x);
        }
        axis_long = fall_dir;
        axis_short = normalize(side);
        half_long = RAIN_HALF_H;
        half_short = RAIN_HALF_W;
        base_alpha = RAIN_ALPHA;
        soft = 0.0;
        let fy_m = cy - box_min.y;
        vert_fade = smoothstep(0.0, EDGE_FADE, min(fy_m, BOX_SIZE - fy_m));
    }

    let offset = v.corner.x * half_short * axis_short + v.corner.y * half_long * axis_long;
    let world = center + offset;

    // Fade near the camera and at the box edges, so wrapping never pops.
    let dist = distance(center, u.camera_pos.xyz);
    let near_fade = smoothstep(0.0, NEAR_FADE, dist);
    let edge_x = min(world.x - box_min.x, box_min.x + BOX_SIZE - world.x);
    let edge_z = min(world.z - box_min.z, box_min.z + BOX_SIZE - world.z);
    let edge_fade = smoothstep(0.0, EDGE_FADE, min(edge_x, edge_z));

    var o: VsOut;
    o.local = v.corner;
    o.alpha = base_alpha * near_fade * edge_fade * vert_fade;
    o.soft = soft;
    o.clip_pos = u.view_proj * vec4<f32>(world, 1.0);
    return o;
}

// Godot 4 tonemap_filmic with white = 1 (copied from water.wgsl, matching lit.wgsl)
fn filmic(c: vec3<f32>) -> vec3<f32> {
    let A = 0.22 * 4.0; let B = 0.30 * 2.0; let C = 0.10; let D = 0.20; let E = 0.01; let F = 0.30;
    let t = ((c * (A * c + C * B) + D * E) / (c * (A * c + B) + D * F)) - E / F;
    let w = 1.0;
    let wt = ((w * (A * w + C * B) + D * E) / (w * (A * w + B) + D * F)) - E / F;
    return t / wt;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    var alpha = i.alpha;
    if (i.soft > 0.5) {
        // Soft round flakes and blobs: the alpha falls off from the disc center to the rim.
        alpha *= 1.0 - smoothstep(SOFT_INNER, 1.0, length(i.local));
    }
    return vec4<f32>(filmic(u.tint.rgb), alpha);
}
