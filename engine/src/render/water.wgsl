// Water surface shader: vertex waves, view-dependent color (shallow -> deep),
// shore foam from depth difference, directional river flow ripples vs gentle lake ripples,
// sun specular highlight, derivative flat-shaded normals, distance fog, and filmic tonemap.

struct WaterUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    sun_dir: vec4<f32>,
    sun_color: vec4<f32>,
    fog: vec4<f32>,            // begin, end, density, ambient energy
    fog_color: vec4<f32>,
    shallow_color: vec4<f32>,  // rgb: linear color, a: shallow alpha
    deep_color: vec4<f32>,     // rgb: linear color, a: deep alpha
    params: vec4<f32>,          // x: time, y: wave_amplitude, z: wave_frequency, w: wave_speed
    lighting_params: vec4<f32>, // x: specular_strength, y: specular_power, z: fresnel_power, w: camera_near
};

@group(0) @binding(0) var<uniform> u: WaterUniforms;
@group(0) @binding(1) var depth_tex: texture_depth_2d;

struct VsIn {
    @location(0) position: vec3<f32>,
    @location(1) flow: vec2<f32>,
};

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) flow: vec2<f32>,
};

fn wave_height(pos: vec2<f32>, time: f32) -> f32 {
    let amp = u.params.y;
    let freq = u.params.z;
    let speed = u.params.w;

    // 2-3 overlapping sine waves
    let w1 = sin(pos.x * freq + pos.y * (freq * 0.7) + time * speed);
    let w2 = sin(pos.x * (-freq * 0.6) + pos.y * (freq * 0.8) + time * (speed * 1.3));
    let w3 = sin(pos.x * (freq * 1.4) - pos.y * (freq * 1.1) + time * (speed * 0.9));

    return (w1 * 0.5 + w2 * 0.35 + w3 * 0.15) * amp;
}

@vertex
fn vs_main(v: VsIn) -> VsOut {
    var world = v.position;
    world.y += wave_height(world.xz, u.params.x);

    var o: VsOut;
    o.clip_pos = u.view_proj * vec4(world, 1.0);
    o.world_pos = world;
    o.flow = v.flow;
    return o;
}

// Godot 4 tonemap_filmic with white = 1 (matching lit.wgsl)
fn filmic(c: vec3<f32>) -> vec3<f32> {
    let A = 0.22 * 4.0; let B = 0.30 * 2.0; let C = 0.10; let D = 0.20; let E = 0.01; let F = 0.30;
    let t = ((c * (A * c + C * B) + D * E) / (c * (A * c + B) + D * F)) - E / F;
    let w = 1.0;
    let wt = ((w * (A * w + C * B) + D * E) / (w * (A * w + B) + D * F)) - E / F;
    return t / wt;
}

@fragment
fn fs_main(i: VsOut) -> @location(0) vec4<f32> {
    // Flat-shaded look to match the low-poly style: derivative normals
    let dx = dpdx(i.world_pos);
    let dy = dpdy(i.world_pos);
    var n = normalize(cross(dy, dx));
    if (n.y < 0.0) {
        n = -n;
    }

    let t = u.params.x;
    let flow_len = length(i.flow);
    var ripple_highlight = 0.0;
    if (flow_len > 0.01) {
        // Directional flowing ripple along the river's downstream tangent
        let flow_dir = i.flow / flow_len;
        let perp = vec2(-flow_dir.y, flow_dir.x);
        let along = dot(i.world_pos.xz, flow_dir);
        let across = dot(i.world_pos.xz, perp);
        let r1 = sin((along - t * 2.4) * 1.4 + sin(across * 0.9) * 1.2);
        let r2 = sin((along - t * 3.1) * 2.7 - across * 1.3);
        let ripple = r1 * 0.6 + r2 * 0.4;
        n = normalize(n + vec3(flow_dir.x, 0.0, flow_dir.y) * (ripple * 0.08));
        ripple_highlight = max(ripple, 0.0) * 0.08;
    } else {
        // Gentle isotropic ripple for lakes / still water
        let r = sin(i.world_pos.x * 1.1 + t * 0.9) * cos(i.world_pos.z * 1.1 - t * 0.7);
        n = normalize(n + vec3(r * 0.03, 0.0, -r * 0.03));
        ripple_highlight = max(r, 0.0) * 0.02;
    }

    let to_cam = normalize(u.camera_pos.xyz - i.world_pos);
    let ndv = clamp(dot(n, to_cam), 0.0, 1.0);

    // Fresnel view angle factor: shallow color -> deep color by view angle
    let fresnel_power = u.lighting_params.z;
    let fresnel = clamp(pow(1.0 - ndv, fresnel_power), 0.0, 1.0);

    let base_rgb = mix(u.shallow_color.rgb, u.deep_color.rgb, fresnel);
    var alpha = mix(u.shallow_color.a, u.deep_color.a, fresnel);

    // Sun diffuse & specular highlight
    let sun_dir = normalize(u.sun_dir.xyz);
    let sun_energy = u.sun_dir.w;
    let ndl = max(dot(n, sun_dir), 0.0);
    let diffuse = u.sun_color.rgb * (sun_energy * ndl * 0.25);

    let h_vec = normalize(sun_dir + to_cam);
    let ndh = max(dot(n, h_vec), 0.0);
    let spec_power = u.lighting_params.y;
    let spec_strength = u.lighting_params.x;
    let specular = pow(ndh, spec_power) * spec_strength * sun_energy;
    let sun_highlight = u.sun_color.rgb * specular;

    let ambient = base_rgb * (u.fog.w * 0.2);
    var color = base_rgb + ambient + diffuse + sun_highlight + vec3(ripple_highlight);

    // Thin white shore foam from reverse-Z scene depth difference
    let dims = vec2<i32>(textureDimensions(depth_tex));
    let pixel = clamp(vec2<i32>(i.clip_pos.xy), vec2<i32>(0), max(dims - vec2<i32>(1), vec2<i32>(0)));
    let scene_z = textureLoad(depth_tex, pixel, 0);
    let near = u.lighting_params.w;
    if (near > 0.0 && scene_z > 0.0001 && i.clip_pos.z > 0.0001) {
        let scene_dist = near / scene_z;
        let water_dist = near / i.clip_pos.z;
        let depth_diff = max(scene_dist - water_dist, 0.0);
        let foam_band = 1.0 - smoothstep(0.0, 0.45, depth_diff);
        let foam_break = sin(i.world_pos.x * 2.5 + i.world_pos.z * 2.5 + t * 2.0) * 0.15;
        let foam = clamp(foam_band + foam_break * foam_band, 0.0, 1.0) * step(depth_diff, 0.55);
        let foam_color = vec3(0.92, 0.96, 1.0);
        color = mix(color, foam_color, foam * 0.85);
        alpha = max(alpha, foam * 0.9);
    }

    // Distance fog (identical math to lit.wgsl)
    let dist = distance(i.world_pos, u.camera_pos.xyz);
    let fog = smoothstep(u.fog.x, u.fog.y, dist) * u.fog.z;
    color = mix(color, u.fog_color.rgb, fog);
    let final_alpha = mix(alpha, 1.0, fog);

    return vec4(filmic(color), final_alpha);
}
