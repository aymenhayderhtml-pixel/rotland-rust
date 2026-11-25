# R32-02: falling rain, snow and sand (engine)

Owns: engine/src/render/weather.rs (new), engine/src/render/weather.wgsl (new), engine/src/render/mod.rs (only the `pub mod weather;` line)
Files: engine/src/render/weather.rs, engine/src/render/weather.wgsl, engine/src/render/mod.rs
Read: tasks/R32-round.md, tasks/LEAD.md, engine/src/render/mod.rs, engine/src/render/water.rs, engine/src/render/water.wgsl, engine/src/render/lit.rs
API: engine/src/lib.rs
Rules: `tasks/LEAD.md` subagent rules. Use the "R32 shared names" in `tasks/R32-round.md` EXACTLY. Copy the structure of `engine/src/render/water.rs` + `water.wgsl` (pipeline creation, bind groups, camera uniform, how it is drawn after the lit pass), and read `engine/src/render/lit.rs` for the camera/fog uniform layout.

**Goal:** cheap particles that look good on Intel UHD. Budget: **< 1.5 ms per frame at 1080p**.
- **No per-particle CPU work and no per-frame buffer creation.** Use ONE static vertex buffer of N quads (N = 6000, made once). The vertex shader places each quad inside a 40 m box that follows the camera:
  - position = hash(instance id) inside the box;
  - fall = `fract((seed_y - time × fall_speed) / box_h)` so particles wrap around forever;
  - wind offset = `wind × time`, also wrapped.
  Draw only `N × intensity × density_factor` instances (the draw count changes per frame, not the buffer).
- **Kinds** (`kind_index`):
  - rain: thin stretched streaks (0.02 × 0.6 m), fast (9 m/s), slanted by the wind, alpha 0.35, tint.
  - snow: soft round flakes (0.06 m, round alpha made in the fragment shader, no texture), slow (1.2 m/s), with a little sideways sway (sin of time + id).
  - sandstorm: larger, faint, almost horizontal blobs (0.15 m, alpha 0.18), fast along the wind, low (in the bottom 6 m of the box).
  - fog / clear: draw nothing.
- **Depth:** test against the scene depth (no depth write), so particles hide behind walls and trees. Additive or premultiplied alpha blend, whichever matches how `water` blends. Fade particles out near the camera (< 0.5 m) and at the box edge, so there is no popping.
- Particles under a roof: skip it this round (say so in notes). R35 may add a roof check.
- **API exactly** as in the shared names. Write a short doc comment on how the lead calls it each frame.
- **Tests:** a pure-Rust unit test for the instance-count function (intensity 0 gives 0; clear and fog give 0; a sandstorm at intensity 1 gives ≤ N). Shader code can't be unit-tested: keep it simple and say what you are unsure about.

## Worker notes
