use fastnoise_lite::{FastNoiseLite, FractalType, NoiseType};

/// A wrapper around `FastNoiseLite` configured with Godot 4 defaults:
/// - NoiseType: Simplex (FastNoiseLite OpenSimplex2)
/// - FractalType: FBm
/// - FractalOctaves: 5
/// - FractalLacunarity: 2.0
/// - FractalGain: 0.5
/// - FractalWeightedStrength: 0.0
/// - Frequency: 0.01
pub struct GodotNoise {
    noise: FastNoiseLite,
}

impl Clone for GodotNoise {
    fn clone(&self) -> Self {
        let mut n = FastNoiseLite::new();
        n.set_seed(Some(self.noise.seed));
        n.set_frequency(Some(self.noise.frequency));
        n.set_noise_type(Some(self.noise.noise_type));
        n.set_rotation_type_3d(Some(self.noise.rotation_type_3d));
        n.set_fractal_type(Some(self.noise.fractal_type));
        n.set_fractal_octaves(Some(self.noise.octaves));
        n.set_fractal_lacunarity(Some(self.noise.lacunarity));
        n.set_fractal_gain(Some(self.noise.gain));
        n.set_fractal_weighted_strength(Some(self.noise.weighted_strength));
        n.set_fractal_ping_pong_strength(Some(self.noise.ping_pong_strength));
        n.set_cellular_distance_function(Some(self.noise.cellular_distance_function));
        n.set_cellular_return_type(Some(self.noise.cellular_return_type));
        n.set_cellular_jitter(Some(self.noise.cellular_jitter_modifier));
        n.set_domain_warp_type(Some(self.noise.domain_warp_type));
        n.set_domain_warp_amp(Some(self.noise.domain_warp_amp));
        Self { noise: n }
    }
}

impl std::fmt::Debug for GodotNoise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GodotNoise")
            .field("seed", &self.noise.seed)
            .field("frequency", &self.noise.frequency)
            .field("noise_type", &self.noise.noise_type)
            .field("fractal_type", &self.noise.fractal_type)
            .field("octaves", &self.noise.octaves)
            .finish()
    }
}

impl GodotNoise {
    pub fn new() -> Self {
        let mut noise = FastNoiseLite::new();
        noise.set_noise_type(Some(NoiseType::OpenSimplex2));
        noise.set_fractal_type(Some(FractalType::FBm));
        noise.set_fractal_octaves(Some(5));
        noise.set_fractal_lacunarity(Some(2.0));
        noise.set_fractal_gain(Some(0.5));
        noise.set_fractal_weighted_strength(Some(0.0));
        noise.set_frequency(Some(0.01));
        Self { noise }
    }

    pub fn with_seed(seed: i32) -> Self {
        let mut g = Self::new();
        g.set_seed(seed);
        g
    }

    pub fn simplex(seed: i32, frequency: f32) -> Self {
        let mut g = Self::with_seed(seed);
        g.set_noise_type(NoiseType::OpenSimplex2);
        g.set_fractal_type(FractalType::None);
        g.set_frequency(frequency);
        g
    }

    pub fn fbm(seed: i32, frequency: f32, octaves: i32) -> Self {
        let mut g = Self::with_seed(seed);
        g.set_noise_type(NoiseType::OpenSimplex2);
        g.set_fractal_type(FractalType::FBm);
        g.set_fractal_octaves(octaves);
        g.set_frequency(frequency);
        g
    }

    #[inline]
    pub fn set_seed(&mut self, seed: i32) {
        self.noise.set_seed(Some(seed));
    }

    #[inline]
    pub fn set_frequency(&mut self, frequency: f32) {
        self.noise.set_frequency(Some(frequency));
    }

    #[inline]
    pub fn set_noise_type(&mut self, noise_type: NoiseType) {
        self.noise.set_noise_type(Some(noise_type));
    }

    #[inline]
    pub fn set_fractal_type(&mut self, fractal_type: FractalType) {
        self.noise.set_fractal_type(Some(fractal_type));
    }

    #[inline]
    pub fn set_fractal_octaves(&mut self, octaves: i32) {
        self.noise.set_fractal_octaves(Some(octaves));
    }

    #[inline]
    pub fn set_fractal_lacunarity(&mut self, lacunarity: f32) {
        self.noise.set_fractal_lacunarity(Some(lacunarity));
    }

    #[inline]
    pub fn set_fractal_gain(&mut self, gain: f32) {
        self.noise.set_fractal_gain(Some(gain));
    }

    #[inline]
    pub fn set_fractal_weighted_strength(&mut self, weighted_strength: f32) {
        self.noise.set_fractal_weighted_strength(Some(weighted_strength));
    }

    #[inline]
    pub fn get_noise_2d(&self, x: f32, y: f32) -> f32 {
        self.noise.get_noise_2d(x, y)
    }
}

impl Default for GodotNoise {
    fn default() -> Self {
        Self::new()
    }
}
