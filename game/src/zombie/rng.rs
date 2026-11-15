//! Deterministic random numbers for one zombie's brain. No global randomness: one seed gives one path.

/// xorshift64. Small, fast, and the same on every run for the same seed.
#[derive(Clone, Debug)]
pub(super) struct Rng(u64);

impl Rng {
    pub(super) fn new(seed: u64) -> Self {
        // xorshift never leaves state 0, so a zero seed gets a fixed non-zero start
        Self(if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// A number in [lo, hi).
    pub(super) fn range(&mut self, lo: f32, hi: f32) -> f32 {
        let unit = (self.next_u64() >> 40) as f32 / 16_777_216.0;
        lo + (hi - lo) * unit
    }
}
