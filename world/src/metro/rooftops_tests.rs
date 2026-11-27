//! Unit tests for `rooftops.rs` and `fire_escapes.rs`.

use std::collections::{HashMap, HashSet};

use glam::Vec2;

use super::climb::{climb_towers, roof_deck_y};
use super::config::MetroConfig;
use super::damage::{DamageSites, Site};
use super::fire_escapes::{fire_escape_ladders, fire_escape_towers};
use super::highway::{Highway, HighwayHeightField};
use super::layout::MetroLayout;
use super::paint::MetroLook;
use super::place::MetroSite;
use super::rooftops::{obb_overlaps_square, roof_links, rooftops_in_chunk};
use super::state::{MetroWorld, index_towers_by_block};
use super::styles::{TowerStyle, default_styles, style_for};
use super::towers::plan_towers;

fn test_world(seed_x: f32) -> MetroWorld {
    let config = MetroConfig::load_from_file(paths::repo_path("data/metro.json")).expect("metro.json");
    let look = MetroLook::load_from_file(paths::repo_path("data/metro.json")).expect("look");
    let site = MetroSite {
        key: config.key.clone(),
        name: config.name.clone(),
        center: Vec2::new(seed_x, -2000.0),
        half_size: config.size_blocks as f32 * config.block_size * 0.5,
        yaw: 0.6,
        ground_y: 10.0,
    };
    let layout = MetroLayout::generate(site, &config);
    let towers = plan_towers(&layout, &config);
    let avenue = layout
        .streets
        .iter()
        .find(|s| s.avenue && layout.contains(layout.site.to_world((s.a + s.b) * 0.5)))
        .expect("the layout has an avenue");
    let entrance = layout.site.to_world(avenue.a);
    let out = (entrance - layout.site.center).normalize();
    let points = (0..=30)
        .map(|i| entrance + out * 3000.0 * (1.0 - i as f32 / 30.0))
        .collect();
    let highway = Highway {
        from_town: "blackwater".into(),
        points,
        width: config.highway.highway_width,
    };
    let highway_field = HighwayHeightField::build(&highway, layout.site.ground_y, &|_, _| 10.0, &config);
    let jam = crate::metro::plan_jam(&highway, &layout, &config);
    let towers_by_block = index_towers_by_block(&towers);
    let damage = DamageSites {
        fallen: Site { block: (0, 0), pos: Vec2::ZERO, yaw: 0.0 },
        leaning: Site { block: (0, 1), pos: Vec2::ZERO, yaw: 0.0 },
        burnt_block: (1, 0),
        replaced_towers: Vec::new(),
    };
    MetroWorld { config, layout, towers, highway, highway_field, jam, look, towers_by_block, damage }
}

#[test]
fn links_respect_gap_overlap_plank_length_and_counts() {
    let metro = test_world(1200.0);
    let cfg = &metro.config.rooftops;
    let links = roof_links(&metro);
    let mut per_source: HashMap<usize, usize> = HashMap::new();
    let mut pairs: HashSet<(usize, usize)> = HashSet::new();

    for link in &links {
        assert!(link.gap >= 1.5 && link.gap <= cfg.max_gap_m, "gap {} out of range", link.gap);
        assert!(link.overlap >= cfg.min_overlap_m, "overlap {} too small", link.overlap);
        assert!(
            link.plank_length as f32 >= link.gap + 2.0,
            "plank {} too short for gap {}",
            link.plank_length,
            link.gap
        );
        let pair = (link.from_tower.min(link.to_tower), link.from_tower.max(link.to_tower));
        assert!(pairs.insert(pair), "duplicate pair {pair:?}");
        *per_source.entry(link.from_tower).or_default() += 1;
    }
    for (src, count) in per_source {
        assert!(count <= cfg.links_per_tower, "source {src} has {count} links");
    }
    assert_eq!(links, roof_links(&metro), "roof_links must be deterministic");
}

#[test]
fn planks_never_overlap_tower_hulls_and_sit_on_lower_parapet() {
    let metro = test_world(1200.0);
    let styles = default_styles();
    let ground_y = metro.layout.site.ground_y;
    let links = roof_links(&metro);

    for link in &links {
        let a = &metro.towers[link.from_tower];
        let b = &metro.towers[link.to_tower];
        let sa = style_for(styles, a.style).unwrap();
        let sb = style_for(styles, b.style).unwrap();
        let da = roof_deck_y(a, sa, ground_y);
        let db = roof_deck_y(b, sb, ground_y);
        let expected_y = if da <= db {
            da + metro.config.rooftops.parapet_top.for_style(a.style)
        } else {
            db + metro.config.rooftops.parapet_top.for_style(b.style)
        };
        assert!((link.plank_y - expected_y).abs() < 1e-4);
        assert!((link.plank_pos.y - expected_y).abs() < 1e-4);

        let plank_xz = Vec2::new(link.plank_pos.x, link.plank_pos.z);
        let half_l = link.plank_length as f32 * 0.5;
        for (idx, t) in metro.towers.iter().enumerate() {
            if metro.damage.tower_is_replaced(idx) {
                continue;
            }
            let st = style_for(styles, t.style).unwrap();
            let hull_lo = ground_y + st.base_height;
            let hull_hi = if t.landmark { ground_y + t.height } else { roof_deck_y(t, st, ground_y) };
            if link.plank_y + 0.12 <= hull_lo || link.plank_y + 0.02 >= hull_hi {
                continue;
            }
            assert!(
                !obb_overlaps_square(plank_xz, Vec2::new(0.6, half_l), link.plank_yaw, t.pos, t.half.x, t.yaw),
                "plank {}_{} overlaps tower {idx}",
                link.from_tower,
                link.to_tower
            );
        }
    }
}

#[test]
fn roof_ladders_only_when_dh_exceeds_flat_step() {
    let metro = test_world(1200.0);
    let styles = default_styles();
    let ground_y = metro.layout.site.ground_y;

    for link in roof_links(&metro) {
        let a = &metro.towers[link.from_tower];
        let b = &metro.towers[link.to_tower];
        let da = roof_deck_y(a, style_for(styles, a.style).unwrap(), ground_y);
        let db = roof_deck_y(b, style_for(styles, b.style).unwrap(), ground_y);
        let higher_deck = da.max(db);
        let dh = higher_deck - link.plank_y;
        if dh > metro.config.rooftops.flat_step_m {
            let ladder = link.ladder.as_ref().expect("expected ladder for dh > flatStepM");
            assert!((ladder.foot.y - link.plank_y).abs() < 1e-4);
            assert!((ladder.deck.y - higher_deck).abs() < 1e-4);
        } else {
            assert!(link.ladder.is_none(), "unexpected ladder for dh = {dh}");
        }
    }
}

#[test]
fn fire_escapes_pick_valid_spaced_brick_towers_with_two_ladders_each() {
    let metro = test_world(1200.0);
    let cfg = &metro.config.fire_escapes;
    let climb_set: HashSet<usize> = climb_towers(&metro).into_iter().map(|c| c.tower).collect();
    let picks = fire_escape_towers(&metro);
    assert_eq!(picks.len(), cfg.count);

    let styles = default_styles();
    let ground_y = metro.layout.site.ground_y;

    for (i, &idx) in picks.iter().enumerate() {
        let t = &metro.towers[idx];
        assert_eq!(t.style, TowerStyle::Brick);
        assert!(!t.landmark);
        assert!(t.floors <= cfg.max_floors);
        assert!(!climb_set.contains(&idx));
        for &other in &picks[i + 1..] {
            let ob = metro.towers[other].block;
            let dist = (t.block.0 - ob.0).abs().max((t.block.1 - ob.1).abs());
            assert!(dist >= cfg.min_spacing_blocks, "spacing {dist} < {}", cfg.min_spacing_blocks);
        }

        let ladders = fire_escape_ladders(&metro, idx);
        assert_eq!(ladders.len(), 2);
        assert!((ladders[0].foot.y - ground_y).abs() < 1e-4, "drop ladder foot must be at street level");
        let deck_y = roof_deck_y(t, style_for(styles, t.style).unwrap(), ground_y);
        assert!((ladders[1].deck.y - deck_y).abs() < 1e-4, "roof ladder deck must be at roof_deck_y");
    }

    // Chunk output smoke check: total emitted planks matches roof_links count.
    let links = roof_links(&metro);
    let mut emitted_planks = 0usize;
    let mut chunks: HashSet<(i32, i32)> = HashSet::new();
    for l in &links {
        chunks.insert(((l.plank_pos.x / 64.0).floor() as i32, (l.plank_pos.z / 64.0).floor() as i32));
    }
    for ch in chunks {
        let c = rooftops_in_chunk(&metro, ch);
        emitted_planks += c.props.iter().filter(|p| p.model.starts_with("roof_plank_")).count();
    }
    assert_eq!(emitted_planks, links.len());
}
