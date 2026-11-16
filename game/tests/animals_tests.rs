use game::animals::{
    AnimalConfigData, AnimalState, Animals, AnimalUpdateContext,
};
use game::damage::DamageInfo;
use game::items::Inventory;
use glam::{IVec2, Vec3};
use physics::{PhysicsWorld, CHUNK_SAMPLES};

fn flat_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new();
    let heights = vec![0.0f32; CHUNK_SAMPLES * CHUNK_SAMPLES];
    for cz in -3..=3 {
        for cx in -3..=3 {
            world.add_ground_chunk(IVec2::new(cx, cz), &heights);
        }
    }
    world
}

#[test]
fn test_data_loads_completely() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut compared = 0;

    assert_eq!(config.species.len(), 3, "animals.json has 3 species");
    compared += 1;

    let deer = config.species("deer").expect("deer species");
    assert_eq!(deer.health, 60.0);
    assert_eq!(deer.meat, 4);
    assert_eq!(deer.hide, 1);
    assert_eq!(deer.antler, 1);
    assert_eq!(deer.buck_model.as_deref(), Some("deer_buck"));
    compared += 5;

    let rabbit = config.species("rabbit").expect("rabbit species");
    assert_eq!(rabbit.health, 15.0);
    assert_eq!(rabbit.meat, 1);
    assert!(rabbit.zigzag);
    compared += 3;

    let boar = config.species("boar").expect("boar species");
    assert_eq!(boar.health, 120.0);
    assert!(boar.danger);
    assert_eq!(boar.attack_damage, 20.0);
    assert_eq!(boar.attack_knockback, 6.0);
    compared += 4;

    assert_eq!(config.spawn.start_delay, 10.0);
    assert_eq!(config.spawn.despawn_dist, 150.0);
    assert_eq!(config.butcher.seconds, 3.0);
    compared += 3;

    println!("Verified {compared} fields in data/animals.json");
    assert!(compared > 0, "Comparison count must be non-zero");
}

#[test]
fn test_spawn_picks_on_real_data_for_fixed_seed() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 42);
    let physics = flat_world();

    let mut ctx = AnimalUpdateContext::new(Vec3::ZERO);
    ctx.player_forward = Vec3::NEG_Z;
    ctx.biome_name = Some("forest");

    // Advance time past start_delay (10s)
    let _ = animals.update(10.1, &ctx, &physics);

    let spawned = animals.active_count();
    println!("Spawned {spawned} animals in forest biome with fixed seed");
    assert!(spawned > 0, "Animals should have spawned");
    assert!(spawned <= 6, "Must not exceed day cap (6)");

    let mut compared = 0;
    for animal in &animals.list {
        assert!(
            animal.species_id == "deer" || animal.species_id == "boar",
            "Forest species must be deer or boar, got {}",
            animal.species_id
        );
        let dist = animal.position.length();
        assert!(
            (35.0..=130.0).contains(&dist),
            "Spawn distance out of expected range: {dist}"
        );
        compared += 1;
    }
    assert!(compared > 0, "Comparison count must be non-zero");
}

#[test]
fn test_flee_moves_away_from_player() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 100);
    let physics = flat_world();

    // Spawn a deer at (0, 0, 20), player is at (0, 0, 0)
    let deer_pos = Vec3::new(0.0, 0.0, 20.0);
    let id = animals.spawn_at("deer", deer_pos, None, Some("deer")).expect("spawn deer");

    // Threat comes from player at (0, 0, 0)
    let b = animals.config.behaviour.clone();
    let deer = animals.list.iter_mut().find(|a| a.id == id).unwrap();
    deer.ai.on_hurt(Vec3::ZERO, &deer.species_data, &b);
    assert_eq!(deer.ai.state, AnimalState::Flee);

    let z_initial = deer.position.z;
    let ctx = AnimalUpdateContext::new(Vec3::ZERO);

    let mut compared = 0;
    let mut prev_z = z_initial;
    for _ in 0..5 {
        animals.update(0.5, &ctx, &physics);
        let curr_z = animals.list.iter().find(|a| a.id == id).unwrap().position.z;
        assert!(
            curr_z > prev_z,
            "Deer should move away from player (+Z): prev {prev_z}, curr {curr_z}"
        );
        prev_z = curr_z;
        compared += 1;
    }

    println!("Verified flee moved deer away from player over {compared} steps");
    assert!(compared > 0, "Comparison count must be non-zero");
}

#[test]
fn test_boar_charges_and_hits() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 200);
    let physics = flat_world();

    // Spawn a boar at (0, 0, 6), player is at (0, 0, 0)
    let id = animals.spawn_at("boar", Vec3::new(0.0, 0.0, 6.0), None, None).expect("spawn boar");
    let b = animals.config.behaviour.clone();

    // Boar alerted by hurt from player at (0, 0, 0)
    let boar = animals.list.iter_mut().find(|a| a.id == id).unwrap();
    boar.ai.on_hurt(Vec3::ZERO, &boar.species_data, &b);
    assert_eq!(boar.ai.state, AnimalState::Charge);

    let ctx = AnimalUpdateContext::new(Vec3::ZERO);

    // Run physics until boar charges close and strikes
    let mut total_attacks = 0;
    for _ in 0..10 {
        let frame = animals.update(0.2, &ctx, &physics);
        for atk in frame.attacks {
            assert_eq!(atk.animal_id, id);
            assert_eq!(atk.damage, 20.0, "Boar attack damage matches data");
            let knock_len = atk.knockback.length();
            assert!(
                (knock_len - 6.0).abs() < 0.1,
                "Boar knockback should be ~6.0, got {knock_len}"
            );
            total_attacks += 1;
        }
        if total_attacks > 0 {
            break;
        }
    }

    println!("Verified boar attack strikes: {total_attacks}");
    assert!(total_attacks > 0, "Boar must charge and hit player");
}

#[test]
fn test_damage_kills() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 300);

    // Rabbit has 15 HP
    let id = animals.spawn_at("rabbit", Vec3::new(0.0, 0.0, 10.0), None, None).expect("spawn rabbit");
    assert_eq!(animals.targets().len(), 1);

    // 10 damage: does not kill
    let killed1 = animals.hit(id, DamageInfo::new(10.0), Vec3::ZERO);
    assert!(!killed1, "Rabbit should not be dead after 10 dmg (15 max)");
    assert_eq!(animals.list[0].health.current(), 5.0);

    // Another 10 damage: kills
    let killed2 = animals.hit(id, DamageInfo::new(10.0), Vec3::ZERO);
    assert!(killed2, "Rabbit should die after taking lethal damage");
    assert!(animals.list[0].is_dead());
    assert_eq!(animals.list[0].current_clip(), "die");
    assert_eq!(animals.targets().len(), 0, "Dead animal is not a melee target");

    let kills = animals.take_kills();
    assert_eq!(kills, vec!["rabbit".to_string()]);
    println!("Verified rabbit kill reported for bounties: {:?}", kills);
}

#[test]
fn test_butcher_gives_data_items_into_inventory() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 400);

    // 1. Regular deer (meat: 4, hide: 1)
    let deer_id = animals.spawn_at("deer", Vec3::ZERO, None, Some("deer")).expect("spawn deer");
    animals.hit(deer_id, DamageInfo::new(100.0), Vec3::ZERO);
    assert!(animals.list[0].is_dead());

    let mut bag = Inventory::new();
    let added = animals.butcher(deer_id, &mut bag);

    assert_eq!(bag.count("raw_meat"), 4);
    assert_eq!(bag.count("hide"), 1);
    assert_eq!(bag.count("antler"), 0);
    assert_eq!(added.len(), 2);
    let kills = animals.take_kills();
    assert!(kills.contains(&"deer".to_string()));

    // 2. Buck with antlers (antler: 1)
    let buck_id = animals.spawn_at("deer", Vec3::ZERO, None, Some("deer_buck")).expect("spawn buck");
    animals.hit(buck_id, DamageInfo::new(100.0), Vec3::ZERO);
    let mut buck_bag = Inventory::new();
    let buck_added = animals.butcher(buck_id, &mut buck_bag);

    assert_eq!(buck_bag.count("raw_meat"), 4);
    assert_eq!(buck_bag.count("hide"), 1);
    assert_eq!(buck_bag.count("antler"), 1);
    assert_eq!(buck_added.len(), 3);

    // 3. Extra meat with Hunting skill level 3 (+1 meat)
    let deer3_id = animals.spawn_at("deer", Vec3::ZERO, None, Some("deer")).expect("spawn deer");
    animals.hit(deer3_id, DamageInfo::new(100.0), Vec3::ZERO);
    let mut bag3 = Inventory::new();
    let added3 = animals.butcher_with_hunting_level(deer3_id, &mut bag3, 3);

    assert_eq!(bag3.count("raw_meat"), 5, "Level 3 hunting gives 1 extra meat");
    assert_eq!(bag3.count("hide"), 1);
    assert_eq!(added3.len(), 2);

    println!("Verified butchering yields for regular deer, buck, and skill level 3");
}

#[test]
fn test_despawn_far_away() {
    let config = AnimalConfigData::from_repo().expect("data/animals.json must load");
    let mut animals = Animals::new(config, 500);

    let _id = animals.spawn_at("rabbit", Vec3::ZERO, None, None).expect("spawn rabbit");
    assert_eq!(animals.active_count(), 1);
    assert_eq!(animals.pool.parked_count(), 0);

    // Player moves 200m away (> 150m despawn distance)
    animals.despawn_far(Vec3::new(200.0, 0.0, 0.0));

    assert_eq!(animals.active_count(), 0, "Distant animal should be despawned");
    assert_eq!(animals.pool.parked_count(), 1, "Despawned animal returned to pool");
    println!("Verified despawn at distance > 150m returned animal to pool");
}

#[test]
fn a_kill_counts_once_and_butchering_gives_meat() {
    let mut animals = Animals::from_repo().unwrap();
    animals.set_first_id(1 << 32);
    let id = animals.spawn_at("rabbit", Vec3::ZERO, None, None).unwrap();
    assert!(id >= 1 << 32, "animal ids stay apart from zombie ids");
    assert!(animals.hit(id, DamageInfo::new(1000.0), Vec3::ZERO), "the killing blow says so");
    assert_eq!(animals.take_kills(), vec!["rabbit".to_string()]);
    let mut bag = Inventory::new();
    let got = animals.butcher_with_hunting_level(id, &mut bag, 1);
    assert!(bag.has("raw_meat", 1), "rabbit gives meat: {got:?}");
    assert!(animals.take_kills().is_empty(), "butchering is not a second kill");
    assert!(animals.butcher_with_hunting_level(id, &mut bag, 1).is_empty(), "a corpse is butchered once");
}
