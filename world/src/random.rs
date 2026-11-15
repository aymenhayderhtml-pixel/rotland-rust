//! Bit-exact port of .NET's System.Random (CompatPrng / Knuth subtractive generator).

#[derive(Debug, Clone)]
pub struct DotnetRandom {
    inext: usize,
    inextp: usize,
    seed_array: [i32; 56],
}

impl DotnetRandom {
    const MBIG: i32 = i32::MAX;
    const MSEED: i32 = 161803398;

    pub fn new(seed: i32) -> Self {
        let mut seed_array = [0i32; 56];
        let subtraction = if seed == i32::MIN {
            i32::MAX
        } else {
            seed.abs()
        };
        let mut mj = Self::MSEED - subtraction;
        seed_array[55] = mj;
        let mut mk = 1i32;
        for i in 1..55 {
            let ii = (21 * i) % 55;
            seed_array[ii] = mk;
            mk = mj.wrapping_sub(mk);
            if mk < 0 {
                mk = mk.wrapping_add(Self::MBIG);
            }
            mj = seed_array[ii];
        }
        for _ in 1..5 {
            for i in 1..56 {
                seed_array[i] = seed_array[i].wrapping_sub(seed_array[1 + (i + 30) % 55]);
                if seed_array[i] < 0 {
                    seed_array[i] = seed_array[i].wrapping_add(Self::MBIG);
                }
            }
        }
        Self {
            inext: 0,
            inextp: 21,
            seed_array,
        }
    }

    fn internal_sample(&mut self) -> i32 {
        let mut loc_inext = self.inext + 1;
        let mut loc_inextp = self.inextp + 1;

        if loc_inext >= 56 {
            loc_inext = 1;
        }
        if loc_inextp >= 56 {
            loc_inextp = 1;
        }

        let mut ret_val = self.seed_array[loc_inext].wrapping_sub(self.seed_array[loc_inextp]);
        if ret_val == Self::MBIG {
            ret_val -= 1;
        }
        if ret_val < 0 {
            ret_val = ret_val.wrapping_add(Self::MBIG);
        }

        self.seed_array[loc_inext] = ret_val;
        self.inext = loc_inext;
        self.inextp = loc_inextp;
        ret_val
    }

    pub fn next_double(&mut self) -> f64 {
        (self.internal_sample() as f64) * (1.0 / (Self::MBIG as f64))
    }

    pub fn next(&mut self, max_value: i32) -> i32 {
        assert!(max_value >= 0);
        (self.next_double() * (max_value as f64)) as i32
    }

    pub fn next_range(&mut self, min: i32, max: i32) -> i32 {
        assert!(min <= max);
        min + self.next(max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_dotnet_seed_42() {
        let mut rng = DotnetRandom::new(42);
        assert!((rng.next_double() - 0.6681064659115423).abs() < 1e-12);
        assert_eq!(rng.next(4), 0);
        assert!((rng.next_double() - 0.12551828945312568).abs() < 1e-12);
    }
}
