//! Unit tests for the dev test flags.

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI, TAU};

use engine::EngineArgs;
use game::guild::{RecruitDef, RecruitRole, RecruitState};
use game::rescue_place::RescueSpot;
use glam::{Vec2, Vec3};
use rotland_ui::gunsmith_ui::GunsmithTab;
use world::biome::layout::BiomeSample;
use world::weather::WeatherKind;
use world::{BiomeId, TowerStyle};

use super::biome_pose::{find_biome_point, BIOME_TP_PAST_BLEND_M};
use super::pose::tp_rescue_pose;
use super::recruits::build_rescued_recruits;
use super::TestFlags;

    fn make_args(raw: &[&str]) -> EngineArgs {
        EngineArgs::parse(raw.iter().map(|s| (*s).to_string()).collect())
    }

    fn make_spot(id: &str, pos: Vec3) -> RescueSpot {
        RescueSpot {
            id: id.to_string(),
            poi_id: format!("poi_{id}"),
            building_id: "house_small".to_string(),
            name: "Sarah Jenkins".to_string(),
            role: RecruitRole::Scavenger,
            level: 1,
            daily_wage: 12,
            max_health: 80,
            position: pos,
            yaw: 0.0,
        }
    }

    fn fake_world_sample(camp: Vec2, x: f32, z: f32) -> BiomeSample {
        let d = Vec2::new(x - camp.x, z - camp.y);
        let r = d.length();
        let core_radius = 1000.0;
        let blend_band = 200.0;
        if r <= core_radius {
            let primary = if d.x > 120.0 {
                BiomeId::Forest
            } else {
                BiomeId::Plains
            };
            return BiomeSample {
                primary,
                secondary: primary,
                blend: 0.0,
                outer: 0.0,
                tier: 0,
            };
        }
        let outer = ((r - core_radius) / blend_band).clamp(0.0, 1.0);
        let mut ang = d.y.atan2(d.x);
        if ang < 0.0 {
            ang += TAU;
        }
        let primary = if ang < FRAC_PI_2 {
            BiomeId::Desert
        } else if ang < PI {
            BiomeId::Snow
        } else if ang < PI + FRAC_PI_2 {
            BiomeId::Swamp
        } else {
            BiomeId::Mountains
        };
        BiomeSample {
            primary,
            secondary: primary,
            blend: 0.0,
            outer,
            tier: 1,
        }
    }

    #[test]
    fn parses_all_test_flags_and_defaults() {
        let empty = TestFlags::from_args(&make_args(&["rotland"]));
        assert_eq!(empty, TestFlags::default());

        let parsed = TestFlags::from_args(&make_args(&[
            "rotland",
            "--down-at=1.5",
            "--team=3",
            "--tp-rescue",
            "--hold-e",
            "--biome=desert",
            "--weather=sandstorm",
            "--town=Dusthollow",
            "--metro=Top",
            "--tower-style=deco",
            "--map",
            "--trailhead=2",
            "--bridge=1",
            "--river=0",
            "--gunsmith=upgrade",
            "--gunsmith-town=Frostmere",
            "--wear=pistol:0.35",
            "--mods=rifle:scope_4x,suppressor",
        ]));
        assert_eq!(parsed.down_at, Some(1.5));
        assert_eq!(parsed.team, 3);
        assert!(parsed.tp_rescue);
        assert!(parsed.hold_e);
        assert_eq!(parsed.biome, Some(BiomeId::Desert));
        assert_eq!(parsed.weather, Some(WeatherKind::Sandstorm));
        assert_eq!(parsed.town.as_deref(), Some("dusthollow"));
        assert_eq!(parsed.metro.as_deref(), Some("top"));
        assert_eq!(parsed.tower_style, Some(TowerStyle::Deco));
        assert!(parsed.map);
        assert_eq!(parsed.trailhead, Some(2));
        assert_eq!(parsed.bridge, Some(1));
        assert_eq!(parsed.river, Some(0));
        assert_eq!(parsed.gunsmith, Some(GunsmithTab::Upgrade));
        assert_eq!(parsed.gunsmith_town.as_deref(), Some("frostmere"));
        assert_eq!(parsed.wear, Some(("pistol".to_string(), 0.35)));
        assert_eq!(
            parsed.mods,
            Some((
                "rifle".to_string(),
                vec!["scope_4x".to_string(), "suppressor".to_string()]
            ))
        );

        let invalid = TestFlags::from_args(&make_args(&[
            "rotland",
            "--down-at=-2",
            "--team=abc",
            "--biome=mars",
            "--weather=acid",
            "--tower-style=tall",
            "--gunsmith=smithing",
            "--wear=pistol:nope",
            "--mods=rifle:",
        ]));
        assert_eq!(invalid.down_at, None);
        assert_eq!(invalid.team, 0);
        assert_eq!(invalid.biome, None);
        assert_eq!(invalid.weather, None);
        assert_eq!(invalid.tower_style, None);
        assert!(!invalid.map);
        assert_eq!(invalid.gunsmith, None);
        assert_eq!(invalid.wear, None);
        assert_eq!(invalid.mods, None);

        let map_only = TestFlags::from_args(&make_args(&["rotland", "--map"]));
        assert!(map_only.map);
        assert!(map_only.gunsmith.is_none());
        assert!(map_only.gunsmith_town.is_none());
        assert!(map_only.metro.is_none());
    }

    #[test]
    fn find_biome_point_places_outer_biome_at_core_plus_blend_plus_300m_facing_outward() {
        let camp = Vec2::new(50.0, -30.0);
        let (pos, yaw) =
            find_biome_point(camp, BiomeId::Desert, |x, z| fake_world_sample(camp, x, z))
                .expect("desert point");
        let delta = pos - camp;
        let expected_r = 1000.0 + 200.0 + BIOME_TP_PAST_BLEND_M;
        assert!(
            (delta.length() - expected_r).abs() < 2.0,
            "expected radius {expected_r}, got {}",
            delta.length()
        );
        let angle = delta.y.atan2(delta.x);
        assert!(
            (angle - FRAC_PI_4).abs() < 0.08,
            "expected sector centre ~PI/4, got {angle}"
        );

        let forward = game::yaw_forward(yaw);
        let dir = delta.normalize();
        let dot = forward.x * dir.x + forward.z * dir.y;
        assert!(dot > 0.99, "expected player facing outward, dot={dot}");
    }

    #[test]
    fn find_biome_point_finds_nearest_core_point_for_forest_and_plains() {
        let camp = Vec2::ZERO;
        let (plains_pos, _) =
            find_biome_point(camp, BiomeId::Plains, |x, z| fake_world_sample(camp, x, z))
                .expect("plains");
        assert!(plains_pos.length() < 1.0);

        let (forest_pos, yaw) =
            find_biome_point(camp, BiomeId::Forest, |x, z| fake_world_sample(camp, x, z))
                .expect("forest");
        assert!(forest_pos.x > 120.0 && forest_pos.length() < 200.0);
        let sample = fake_world_sample(camp, forest_pos.x, forest_pos.y);
        assert_eq!(sample.primary, BiomeId::Forest);
        assert_eq!(sample.outer, 0.0);

        let forward = game::yaw_forward(yaw);
        let dir = forest_pos.normalize();
        assert!(forward.x * dir.x + forward.z * dir.y > 0.99);
    }

    #[test]
    fn tp_rescue_pose_places_player_6m_away_facing_nearest_spot() {
        let spots = vec![
            make_spot("far", Vec3::new(100.0, 0.0, 0.0)),
            make_spot("near", Vec3::new(20.0, 2.0, 0.0)),
        ];
        let (pos, yaw) = tp_rescue_pose(Vec3::new(0.0, 0.0, 0.0), &spots).expect("spot");
        let dist = Vec3::new(pos.x - 20.0, 0.0, pos.z).length();
        assert!((dist - 6.0).abs() < 1e-4, "expected 6m distance, got {dist}");
        assert!((pos.x - 14.0).abs() < 1e-4);

        // Player is west of spot (14, 0) -> faces east (+X) -> Godot yaw = -PI/2.
        let forward = game::yaw_forward(yaw);
        assert!(forward.x > 0.99 && forward.z.abs() < 1e-4);
    }

    #[test]
    fn build_rescued_recruits_cycles_templates_and_sets_follow_when_added() {
        let templates = vec![
            RecruitDef {
                id: "rec_mark".to_string(),
                name: "Mark Vance".to_string(),
                role: "fighter".to_string(),
                level: 1,
                daily_wage: 15,
                health: 100,
                hire_cost: 30,
            },
            RecruitDef {
                id: "rec_sarah".to_string(),
                name: "Sarah Jenkins".to_string(),
                role: "scavenger".to_string(),
                level: 1,
                daily_wage: 12,
                health: 80,
                hire_cost: 25,
            },
        ];
        let built = build_rescued_recruits(3, &templates);
        assert_eq!(built.len(), 3);
        assert_eq!(built[0].name, "Mark Vance");
        assert_eq!(built[1].name, "Sarah Jenkins");
        assert_eq!(built[2].name, "Mark Vance");

        let mut roster = game::guild::GuildRecruits::default();
        for r in built {
            roster.add_rescued(r, 3).expect("add_rescued");
        }
        assert_eq!(roster.hired().len(), 3);
        assert!(
            roster
                .hired()
                .iter()
                .all(|r| r.hire_cost == 0 && r.state == RecruitState::FollowingPlayer)
        );
    }
