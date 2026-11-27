//! Level-of-detail choice by distance, the dither cross-fade between LOD levels, and which LOD casts
//! the sun shadow.

/// Cross-fade band width (10 m) around each LOD switch distance.
pub const FADE: f32 = 10.0; // new: PERF R30.5

const TREE_SHADOW_LOD1_DIST: f32 = 15.0; // new: PERF R34

/// Return type of `lod_fade`: up to 2 active LOD levels with their dither fade factors.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodFades {
    items: [(usize, f32, bool); 2],
    len: usize,
}

impl LodFades {
    fn empty() -> Self {
        Self { items: [(0, 0.0, false); 2], len: 0 }
    }

    fn one(lod: usize, fade: f32, complementary: bool) -> Self {
        Self { items: [(lod, fade, complementary), (0, 0.0, false)], len: 1 }
    }

    fn two(first: (usize, f32, bool), second: (usize, f32, bool)) -> Self {
        Self { items: [first, second], len: 2 }
    }

    pub fn as_slice(&self) -> &[(usize, f32, bool)] {
        &self.items[..self.len]
    }
}

impl std::ops::Deref for LodFades {
    type Target = [(usize, f32, bool)];
    fn deref(&self) -> &Self::Target {
        &self.items[..self.len]
    }
}

impl<'a> IntoIterator for &'a LodFades {
    type Item = &'a (usize, f32, bool);
    type IntoIter = std::slice::Iter<'a, (usize, f32, bool)>;
    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl IntoIterator for LodFades {
    type Item = (usize, f32, bool);
    type IntoIter = std::iter::Take<std::array::IntoIter<(usize, f32, bool), 2>>;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter().take(self.len)
    }
}

/// Pure helper: returns the active LOD(s) and dither fade values at `dist`.
/// Outside fade bands: exactly one LOD at fade 1.0.
/// In a fade band around an LOD boundary: two LODs whose fades sum to 1.0.
/// Ends of the band match the old hard switch.
pub fn lod_fade(dist: f32, ends: &[f32], band: f32) -> LodFades {
    if ends.is_empty() {
        return LodFades::empty();
    }
    let last_end = ends[ends.len() - 1];
    if dist >= last_end {
        return LodFades::empty();
    }
    let half = band * 0.5;
    for (i, &switch_dist) in ends.iter().enumerate().take(ends.len().saturating_sub(1)) {
        let start = switch_dist - half;
        let end = switch_dist + half;
        if dist < start {
            return LodFades::one(i, 1.0, false);
        }
        if dist <= end {
            let t = if band > 0.0 { ((dist - start) / band).clamp(0.0, 1.0) } else { 1.0 };
            if t <= 0.0 {
                return LodFades::one(i, 1.0, false);
            }
            if t >= 1.0 {
                return LodFades::one(i + 1, 1.0, false);
            }
            return LodFades::two((i, 1.0 - t, false), (i + 1, t, true));
        }
    }
    LodFades::one(ends.len() - 1, 1.0, false)
}

fn shadow_lod_for(nearer_lod: usize, lod_count: usize, dist: f32) -> Option<usize> {
    let shadow_lod = if lod_count >= 3 && dist > TREE_SHADOW_LOD1_DIST {
        nearer_lod.max(1).min(lod_count - 1)
    } else {
        nearer_lod
    };
    (shadow_lod < 2).then_some(shadow_lod)
}

/// Shadow LOD selection: tower models (`lod0_only == true`) only cast shadows from LOD0.
pub(crate) fn shadow_lod_for_rule(
    nearer_lod: usize,
    lod_count: usize,
    dist: f32,
    lod0_only: bool,
) -> Option<usize> {
    if lod0_only {
        return (nearer_lod == 0).then_some(0);
    }
    shadow_lod_for(nearer_lod, lod_count, dist)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::props::rules::TREE;

    #[test]
    fn test_lod_fade_outside_bands() {
        let ends = [70.0, 220.0];
        // Closer than the first fade band (65..75 m)
        let near = lod_fade(30.0, &ends, FADE);
        assert_eq!(near.len(), 1);
        assert_eq!(near[0], (0, 1.0, false));
        assert_eq!(near.as_slice(), &[(0, 1.0, false)]);

        // Farther than the fade band (65..75 m), before max range
        let far = lod_fade(120.0, &ends, FADE);
        assert_eq!(far.len(), 1);
        assert_eq!(far[0], (1, 1.0, false));

        // Beyond max range
        let beyond = lod_fade(220.0, &ends, FADE);
        assert!(beyond.is_empty());
        let way_beyond = lod_fade(300.0, &ends, FADE);
        assert!(way_beyond.is_empty());
    }

    #[test]
    fn test_lod_fade_in_band_sums_to_one() {
        let ends = [70.0, 220.0];
        // Sample points across the 65..75 m band
        for step in 1..10 {
            let dist = 65.0 + step as f32 * 1.0;
            let res = lod_fade(dist, &ends, FADE);
            assert_eq!(res.len(), 2, "dist {dist} should have 2 LODs in fade band");
            let (lod0, fade0, comp0) = res[0];
            let (lod1, fade1, comp1) = res[1];
            assert_eq!(lod0, 0);
            assert_eq!(lod1, 1);
            assert!(!comp0, "outgoing LOD should not be complementary");
            assert!(comp1, "incoming LOD should be complementary");
            assert!((fade0 + fade1 - 1.0).abs() < 1e-6, "fades must sum to 1.0 at {dist}");
        }

        // At midpoint 70.0 m, fades must be 0.5 each
        let mid = lod_fade(70.0, &ends, FADE);
        assert_eq!(mid.len(), 2);
        assert_eq!(mid[0], (0, 0.5, false));
        assert_eq!(mid[1], (1, 0.5, true));
    }

    #[test]
    fn test_lod_fade_ends_match_old_hard_switch() {
        let ends = [70.0, 220.0];
        // Lower end of band: 65.0 m matches old switch (LOD 0 at 1.0)
        let lower = lod_fade(65.0, &ends, FADE);
        assert_eq!(lower.len(), 1);
        assert_eq!(lower[0], (0, 1.0, false));

        // Upper end of band: 75.0 m matches old switch (LOD 1 at 1.0)
        let upper = lod_fade(75.0, &ends, FADE);
        assert_eq!(upper.len(), 1);
        assert_eq!(upper[0], (1, 1.0, false));
    }

    #[test]
    fn test_lod_fade_multi_lod_tree() {
        let ends = [45.0, 95.0, TREE];
        // Below band 0 (40..50)
        assert_eq!(&lod_fade(20.0, &ends, FADE)[..], &[(0, 1.0, false)]);
        // In band 0 (45.0)
        let b0 = lod_fade(45.0, &ends, FADE);
        assert_eq!(b0.len(), 2);
        assert_eq!(b0[0], (0, 0.5, false));
        assert_eq!(b0[1], (1, 0.5, true));
        // Between band 0 and band 1 (50..90)
        assert_eq!(&lod_fade(70.0, &ends, FADE)[..], &[(1, 1.0, false)]);
        // In band 1 (95.0)
        let b1 = lod_fade(95.0, &ends, FADE);
        assert_eq!(b1.len(), 2);
        assert_eq!(b1[0], (1, 0.5, false));
        assert_eq!(b1[1], (2, 0.5, true));
        // Past band 1 (100..170)
        assert_eq!(&lod_fade(120.0, &ends, FADE)[..], &[(2, 1.0, false)]);
        // Beyond visibility
        assert!(lod_fade(175.0, &ends, FADE).is_empty());
    }

    #[test]
    fn test_lod_fade_single_lod() {
        let ends = [80.0];
        assert_eq!(&lod_fade(40.0, &ends, FADE)[..], &[(0, 1.0, false)]);
        assert!(lod_fade(85.0, &ends, FADE).is_empty());
    }

    #[test]
    fn test_lod_fade_empty_ends() {
        assert!(lod_fade(10.0, &[], FADE).is_empty());
    }

    #[test]
    fn test_lod_fade_zero_band_never_fades() {
        let ends = [28.0, 65.0];
        for d in [0.0, 23.0, 28.0, 33.0, 64.9] {
            let res = lod_fade(d, &ends, 0.0);
            assert_eq!(res.len(), 1);
            let (_, fade, comp) = res[0];
            assert_eq!(fade, 1.0);
            assert!(!comp);
        }
        assert!(lod_fade(65.0, &ends, 0.0).is_empty());
    }

    #[test]
    fn test_shadow_lod_for_trees_and_buildings() {
        // Close tree uses LOD0, mid-distance tree uses LOD1, far LOD2 casts no shadow.
        assert_eq!(shadow_lod_for(0, 3, 10.0), Some(0));
        assert_eq!(shadow_lod_for(0, 3, 25.0), Some(1));
        assert_eq!(shadow_lod_for(1, 3, 45.0), Some(1));
        assert_eq!(shadow_lod_for(2, 3, 80.0), None);
        // 2-LOD building keeps nearer_lod unchanged.
        assert_eq!(shadow_lod_for(0, 2, 30.0), Some(0));
        assert_eq!(shadow_lod_for(1, 2, 45.0), Some(1));
    }
}
