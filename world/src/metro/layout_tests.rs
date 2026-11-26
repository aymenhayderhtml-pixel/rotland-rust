//! Tests for `layout.rs` (kept in a sibling file to stay under the size rule).

use glam::Vec2;

use crate::metro::config::MetroConfig;
use crate::metro::layout::{MetroDistrict, MetroLayout, district_for};
use crate::metro::place::MetroSite;


    fn cfg() -> MetroConfig {
        MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json loads")
    }

    fn site(cfg: &MetroConfig) -> MetroSite {
        MetroSite {
            key: cfg.key.clone(),
            name: cfg.name.clone(),
            center: Vec2::new(1000.0, -2000.0),
            half_size: cfg.size_blocks as f32 * cfg.block_size * 0.5,
            yaw: 0.6,
            ground_y: 10.0,
        }
    }

    fn layout() -> (MetroConfig, MetroLayout) {
        let cfg = cfg();
        let layout = MetroLayout::generate(site(&cfg), &cfg);
        (cfg, layout)
    }

    #[test]
    fn generation_is_deterministic_for_a_site() {
        let (cfg, a) = layout();
        assert_eq!(a, MetroLayout::generate(site(&cfg), &cfg));
        let mut moved = site(&cfg);
        moved.center.x += 64.0;
        assert_ne!(a.blocks, MetroLayout::generate(moved, &cfg).blocks);
    }

    #[test]
    fn districts_follow_the_rings() {
        assert_eq!(district_for(0, 16, 32, &cfg()), MetroDistrict::Edge);
        assert_eq!(district_for(5, 16, 32, &cfg()), MetroDistrict::Midtown);
        assert_eq!(district_for(10, 16, 32, &cfg()), MetroDistrict::Midtown);
        assert_eq!(district_for(11, 16, 32, &cfg()), MetroDistrict::Core);
        assert_eq!(district_for(16, 16, 32, &cfg()), MetroDistrict::Core);
    }

    #[test]
    fn corners_are_ragged_but_every_side_keeps_blocks() {
        let (cfg, l) = layout();
        let cells = (cfg.size_blocks * cfg.size_blocks) as usize;
        assert!(l.blocks.len() < cells, "some corner block must be dropped");
        assert!(l.blocks.len() > cells - 12, "at most 3 drops per corner");
        let n = cfg.size_blocks;
        assert!((0..n).any(|i| l.block_at(i, 0).is_some()), "north side empty");
        assert!((0..n).any(|i| l.block_at(i, n - 1).is_some()), "south side empty");
        assert!((0..n).any(|i| l.block_at(0, i).is_some()), "west side empty");
        assert!((0..n).any(|i| l.block_at(n - 1, i).is_some()), "east side empty");
    }

    #[test]
    fn reserved_blocks_are_present_and_labelled() {
        let (cfg, l) = layout();
        for area in &cfg.reserved {
            for z in area.iz..area.iz + area.h {
                for x in area.ix..area.ix + area.w {
                    let block = l.block_at(x, z).unwrap_or_else(|| panic!("{:?} missing", area.kind));
                    assert_eq!(block.reserved, Some(area.kind));
                }
            }
        }
    }

    #[test]
    fn streets_stay_inside_the_square_and_avenues_are_wide() {
        let (cfg, l) = layout();
        let half = l.site.half_size;
        assert!(!l.streets.is_empty());
        for s in &l.streets {
            assert!(s.a.abs().max_element() <= half + 1e-3 && s.b.abs().max_element() <= half + 1e-3);
            let want = if s.avenue { cfg.avenue_width } else { cfg.street_width };
            assert_eq!(s.width, want);
        }
        assert!(l.streets.iter().any(|s| s.avenue));
    }

    #[test]
    fn block_and_street_queries_find_the_city() {
        let (_, l) = layout();
        let first = &l.blocks[0];
        let centre = l.site.to_world((first.rect.0 + first.rect.1) * 0.5);
        assert!(l.contains(centre));
        assert_eq!(l.district_at(centre), Some(first.district));
        assert!(l.block_at_point(centre).is_some());
        let street = &l.streets[0];
        let mid = l.site.to_world((street.a + street.b) * 0.5);
        assert!(l.street_at(mid).is_some());
        assert!(!l.contains(l.site.to_world(Vec2::splat(l.site.half_size * 10.0))));
    }

    #[test]
    fn flat_inside_and_blends_to_raw_over_the_skirt() {
        let (cfg, l) = layout();
        let g = l.site.ground_y;
        assert_eq!(l.height_at(l.site.center), Some(g));
        assert_eq!(l.height_at(l.site.to_world(Vec2::splat(l.site.half_size * 10.0))), None);
        let raw = 50.0;
        assert_eq!(l.flatten(raw, l.site.center), g);
        let edge = l.site.half_size;
        let mut last = g;
        for step in 0..=10 {
            let d = edge + cfg.skirt_width * step as f32 / 10.0;
            let h = l.flatten(raw, l.site.to_world(Vec2::new(d, 0.0)));
            assert!(h >= last - 1e-4, "not monotone at step {step}");
            last = h;
        }
        assert!((last - raw).abs() < 1e-3, "skirt must end at raw height");
        assert!(l.near(l.site.center));
        assert!(!l.near(l.site.to_world(Vec2::new(edge + cfg.skirt_width + 5.0, 0.0))));
    }
