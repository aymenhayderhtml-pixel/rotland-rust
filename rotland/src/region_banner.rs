//! Region / danger-tier entry banner state machine.
//!
//! Checks the player's biome & tier twice a second with a 3 s hysteresis cooldown
//! so borders do not flicker, and shows a 3 s centre-top banner on transitions.

use rotland_ui::hud::region_banner::RegionBannerInfo;
use world::biome::layout::BiomeSample;
use world::BiomeId;

const CHECK_INTERVAL_S: f32 = 0.5; // new: DECISIONS 2026-11-25
const HYSTERESIS_S: f32 = 3.0; // new: DECISIONS 2026-11-25
const BANNER_DURATION_S: f32 = 3.0; // new: DECISIONS 2026-11-25
const FADE_IN_S: f32 = 0.25; // new: DECISIONS 2026-11-25
const FADE_OUT_S: f32 = 0.5; // new: DECISIONS 2026-11-25
const DANGER_SUBTITLE_MIN_TIER: u8 = 2; // new: DECISIONS 2026-11-25

/// Normalized region identity used for change detection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RegionState {
    Core,
    Outer { biome: BiomeId, tier: u8 },
}

impl RegionState {
    fn from_sample(sample: BiomeSample) -> Self {
        if sample.outer <= 0.0 || sample.tier == 0 {
            Self::Core
        } else {
            Self::Outer {
                biome: sample.primary,
                tier: sample.tier,
            }
        }
    }
}

#[derive(Clone, Debug)]
struct ActiveBanner {
    title: String,
    subtitle: Option<String>,
    elapsed: f32,
}

/// Tracks biome/tier transitions and produces HUD banner display info.
#[derive(Clone, Debug, Default)]
pub struct RegionBannerTracker {
    current: Option<RegionState>,
    check_accum: f32,
    hysteresis_remaining: f32,
    banner: Option<ActiveBanner>,
}

impl RegionBannerTracker {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Advances timers and samples the world at most twice per second.
    pub fn update(
        &mut self,
        dt: f32,
        mut sample_fn: impl FnMut() -> BiomeSample,
        mut biome_name: impl FnMut(BiomeId) -> String,
    ) {
        let dt = dt.max(0.0);
        self.hysteresis_remaining = (self.hysteresis_remaining - dt).max(0.0);

        if let Some(active) = self.banner.as_mut() {
            active.elapsed += dt;
            if active.elapsed >= BANNER_DURATION_S {
                self.banner = None;
            }
        }

        // Initialize immediately on the first tick without showing a banner.
        if self.current.is_none() {
            self.current = Some(RegionState::from_sample(sample_fn()));
            return;
        }

        self.check_accum += dt;
        if self.check_accum < CHECK_INTERVAL_S {
            return;
        }
        self.check_accum %= CHECK_INTERVAL_S;

        if self.hysteresis_remaining > 0.0 {
            return;
        }

        let next = RegionState::from_sample(sample_fn());
        let Some(prev) = self.current else {
            self.current = Some(next);
            return;
        };
        if next == prev {
            return;
        }

        self.current = Some(next);
        self.hysteresis_remaining = HYSTERESIS_S;
        self.banner = Some(match next {
            RegionState::Core => ActiveBanner {
                title: "Back in home lands".to_string(),
                subtitle: None,
                elapsed: 0.0,
            },
            RegionState::Outer { biome, tier } => ActiveBanner {
                title: format!("Entering: {}", biome_name(biome)),
                subtitle: (tier >= DANGER_SUBTITLE_MIN_TIER).then(|| format!("Danger {tier}")),
                elapsed: 0.0,
            },
        });
    }

    /// Returns the currently visible banner (with fade alpha), if any.
    pub fn active(&self) -> Option<RegionBannerInfo> {
        let active = self.banner.as_ref()?;
        let remaining = (BANNER_DURATION_S - active.elapsed).max(0.0);
        let fade_in = (active.elapsed / FADE_IN_S).clamp(0.0, 1.0);
        let fade_out = (remaining / FADE_OUT_S).clamp(0.0, 1.0);
        Some(RegionBannerInfo {
            title: active.title.clone(),
            subtitle: active.subtitle.clone(),
            alpha: fade_in.min(fade_out),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core_sample(biome: BiomeId) -> BiomeSample {
        BiomeSample {
            primary: biome,
            secondary: biome,
            blend: 0.0,
            outer: 0.0,
            tier: 0,
        }
    }

    fn outer_sample(biome: BiomeId, tier: u8) -> BiomeSample {
        BiomeSample {
            primary: biome,
            secondary: biome,
            blend: 0.0,
            outer: 1.0,
            tier,
        }
    }

    fn display_name(id: BiomeId) -> String {
        match id {
            BiomeId::Plains => "Plains",
            BiomeId::Forest => "Forest",
            BiomeId::Desert => "Desert",
            BiomeId::Snow => "Snow",
            BiomeId::Swamp => "Swamp",
            BiomeId::Mountains => "Mountains",
        }
        .to_string()
    }

    #[test]
    fn no_banner_on_start_or_between_core_biomes() {
        let mut tracker = RegionBannerTracker::default();
        tracker.update(0.016, || core_sample(BiomeId::Plains), display_name);
        assert!(tracker.active().is_none());

        for _ in 0..10 {
            tracker.update(0.5, || core_sample(BiomeId::Forest), display_name);
            assert!(tracker.active().is_none());
        }
    }

    #[test]
    fn entering_outer_biome_shows_banner_and_danger_for_tier_2_plus() {
        let mut tracker = RegionBannerTracker::default();
        tracker.update(0.016, || core_sample(BiomeId::Plains), display_name);

        // Before 0.5 s check interval, no sample is taken.
        tracker.update(0.25, || outer_sample(BiomeId::Desert, 1), display_name);
        assert!(tracker.active().is_none());

        // Crossing 0.5 s triggers tier 1 Desert banner (no Danger line).
        tracker.update(0.3, || outer_sample(BiomeId::Desert, 1), display_name);
        let b1 = tracker.active().expect("desert tier 1 banner");
        assert_eq!(b1.title, "Entering: Desert");
        assert_eq!(b1.subtitle, None);

        // Wait past 3 s hysteresis and enter tier 3.
        tracker.update(3.5, || outer_sample(BiomeId::Desert, 3), display_name);
        let b2 = tracker.active().expect("desert tier 3 banner");
        assert_eq!(b2.title, "Entering: Desert");
        assert_eq!(b2.subtitle.as_deref(), Some("Danger 3"));
    }

    #[test]
    fn hysteresis_blocks_border_flicker_and_return_to_core_shows_home_lands() {
        let mut tracker = RegionBannerTracker::default();
        tracker.update(0.016, || core_sample(BiomeId::Plains), display_name);
        tracker.update(0.5, || outer_sample(BiomeId::Snow, 2), display_name);
        assert_eq!(tracker.active().unwrap().title, "Entering: Snow");

        // Stepping back into core within 3 s hysteresis does not replace the banner.
        tracker.update(1.0, || core_sample(BiomeId::Plains), display_name);
        assert_eq!(tracker.active().unwrap().title, "Entering: Snow");

        // After 3 s total, banner expires and hysteresis clears -> returning to core announces it.
        tracker.update(2.1, || core_sample(BiomeId::Plains), display_name);
        let back = tracker.active().expect("back in home lands");
        assert_eq!(back.title, "Back in home lands");
        assert_eq!(back.subtitle, None);

        // After 3 s display time, banner disappears.
        tracker.update(3.0, || core_sample(BiomeId::Plains), display_name);
        assert!(tracker.active().is_none());
    }
}
