use crate::color::Color;
use crate::math::smooth_step;
use crate::noise::GodotNoise;

/// Ground colour variety: lays dry-grass patches, leaf litter, bare dirt,
/// worn trails, trampled dirt inside camps, and rock on slopes.
pub struct GroundPaint {
    patch: GodotNoise,
    trail: GodotNoise,
    warp: GodotNoise,
}

impl GroundPaint {
    // godot: World/GroundPaint.cs:12
    pub const DIRT: Color = Color::new(110.0 / 255.0, 86.0 / 255.0, 56.0 / 255.0, 1.0);
    // godot: World/GroundPaint.cs:13
    pub const DRY_GRASS: Color = Color::new(138.0 / 255.0, 138.0 / 255.0, 62.0 / 255.0, 1.0);
    // godot: World/GroundPaint.cs:14
    pub const LEAF_LITTER: Color = Color::new(90.0 / 255.0, 74.0 / 255.0, 42.0 / 255.0, 1.0);
    // godot: World/GroundPaint.cs:15
    pub const TRAIL: Color = Color::new(125.0 / 255.0, 102.0 / 255.0, 70.0 / 255.0, 1.0);
    // godot: World/GroundPaint.cs:16
    pub const ROCK: Color = Color::new(107.0 / 255.0, 82.0 / 255.0, 54.0 / 255.0, 1.0);
    // godot: World/GroundPaint.cs:17
    pub const ROCK_LIGHT: Color = Color::new(127.0 / 255.0, 116.0 / 255.0, 104.0 / 255.0, 1.0);

    // godot: World/GroundPaint.cs:19
    pub const STEEP_START: f32 = 0.75;
    // godot: World/GroundPaint.cs:20
    pub const STEEP_FULL: f32 = 0.6;
    // godot: World/GroundPaint.cs:21
    const TRAIL_WIDTH: f32 = 0.035;
    // godot: World/GroundPaint.cs:22
    const CAMP_DIRT_RADIUS: f32 = 18.0;
    // godot: World/GroundPaint.cs:23
    const CAMP_DIRT_FADE: f32 = 26.0;

    pub fn new(seed: i32) -> Self {
        Self {
            // godot: World/GroundPaint.cs:27
            patch: GodotNoise::fbm(seed + 11, 0.03, 2),
            // godot: World/GroundPaint.cs:28
            trail: GodotNoise::simplex(seed + 12, 0.004),
            // godot: World/GroundPaint.cs:29
            warp: GodotNoise::simplex(seed + 13, 0.02),
        }
    }

    /// Slope blend weight shared by core rock painting and outer biome slope painting.
    pub fn steepness(normal_y: f32) -> f32 {
        smooth_step(Self::STEEP_START, Self::STEEP_FULL, normal_y)
    }

    /// The final colour of one ground triangle at world (x, z).
    pub fn paint(
        &self,
        grass: Color,
        x: f32,
        z: f32,
        forest: f32,
        normal_y: f32,
        camp_distance: f32,
    ) -> Color {
        let p = self.patch.get_noise_2d(x, z);
        let mut c = grass;

        // Dry grass in open plains, leaf litter under the forest
        // godot: World/GroundPaint.cs:37
        c = c.lerp(Self::DRY_GRASS, (1.0 - forest) * smooth_step(0.3, 0.5, p) * 0.45);
        // godot: World/GroundPaint.cs:38
        c = c.lerp(Self::LEAF_LITTER, forest * smooth_step(0.15, 0.4, p) * 0.75);

        // Bare dirt where the patch noise dips low
        // godot: World/GroundPaint.cs:41
        c = c.lerp(Self::DIRT, smooth_step(-0.45, -0.65, p) * 0.75);

        // Worn trails: a warped band, mostly in the open
        // godot: World/GroundPaint.cs:44
        let w = self.warp.get_noise_2d(x, z) * 0.06;
        let band = (self.trail.get_noise_2d(x, z) + w).abs();
        let trail = 1.0 - smooth_step(Self::TRAIL_WIDTH * 0.5, Self::TRAIL_WIDTH, band);
        // godot: World/GroundPaint.cs:47
        c = c.lerp(Self::TRAIL, trail * (1.0 - 0.6 * forest) * 0.85);

        // Trampled camp ground, broken up by the patch noise
        if camp_distance < Self::CAMP_DIRT_FADE {
            let camp = 1.0 - smooth_step(Self::CAMP_DIRT_RADIUS, Self::CAMP_DIRT_FADE, camp_distance);
            // godot: World/GroundPaint.cs:52
            c = c.lerp(Self::DIRT, camp * (0.55 + 0.35 * p).clamp(0.0, 1.0));
        }

        // Slopes turn to rock gradually; the steepest faces show lighter stone
        let steep = Self::steepness(normal_y);
        // godot: World/GroundPaint.cs:57
        let rock = Self::ROCK.lerp(Self::ROCK_LIGHT, smooth_step(0.55, 0.4, normal_y));
        c.lerp(rock, steep)
    }
}
