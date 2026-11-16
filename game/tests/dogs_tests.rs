use game::damage::DamageInfo;
use game::dogs::{DogBreeder, DogConfigData, DogGrowth, DogMode, DogPoiSite, DogUpdateContext, Dogs};
use game::economy::Wallet;
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
    let cfg = DogConfigData::from_repo().expect("data/dog.json must load");
    let mut compared = 0;

    assert_eq!(cfg.price, 150);
    compared += 1;
    assert_eq!(cfg.default_name, "Rex");
    compared += 1;
    assert_eq!(cfg.max_name_length, 12);
    compared += 1;
    assert_eq!(cfg.max_health, 80.0);
    compared += 1;
    assert_eq!(cfg.regen_per_second, 1.0);
    compared += 1;
    assert_eq!(cfg.regen_delay_seconds, 10.0);
    compared += 1;
    assert_eq!(cfg.feed_heal, 30.0);
    compared += 1;
    assert_eq!(cfg.feed_items, vec!["raw_meat".to_string(), "cooked_meat".to_string()]);
    compared += 1;
    assert_eq!(cfg.follow_distance, 3.0);
    compared += 1;
    assert_eq!(cfg.stop_distance, 0.5);
    compared += 1;
    assert_eq!(cfg.walk_speed, 2.5);
    compared += 1;
    assert_eq!(cfg.run_speed, 6.0);
    compared += 1;
    assert_eq!(cfg.catch_up_distance, 8.0);
    compared += 1;
    assert_eq!(cfg.sit_after_seconds, 5.0);
    compared += 1;
    assert_eq!(cfg.lie_after_seconds, 20.0);
    compared += 1;
    assert_eq!(cfg.teleport_distance, 60.0);
    compared += 1;
    assert_eq!(cfg.home_arrive_distance, 4.0);
    compared += 1;
    assert_eq!(cfg.bark_range, 20.0);
    compared += 1;
    assert_eq!(cfg.bark_cooldown, 4.0);
    compared += 1;
    assert_eq!(cfg.bark_noise_radius, 15.0);
    compared += 1;
    assert_eq!(cfg.attack_range, 10.0);
    compared += 1;
    assert_eq!(cfg.bite_range, 1.2);
    compared += 1;
    assert_eq!(cfg.bite_damage, 15.0);
    compared += 1;
    assert_eq!(cfg.bite_cooldown, 1.2);
    compared += 1;
    assert_eq!(cfg.body_seconds, 60.0);
    compared += 1;
    assert_eq!(cfg.pack_slots, 4);
    compared += 1;
    assert_eq!(cfg.pack_weight, 10.0);
    compared += 1;
    assert_eq!(cfg.scan_interval, 0.25);
    compared += 1;
    assert_eq!(cfg.puppy_scale, 0.8);
    compared += 1;
    assert_eq!(cfg.grow_days, 1.0);
    compared += 1;
    assert_eq!(cfg.stray_chance, 0.05);
    compared += 1;
    assert_eq!(cfg.stray_buildings, vec!["house_small".to_string(), "cabin".to_string()]);
    compared += 1;
    assert_eq!(cfg.stray_flee_range, 8.0);
    compared += 1;
    assert_eq!(cfg.stray_flee_seconds, 4.0);
    compared += 1;
    assert_eq!(cfg.stray_flee_distance, 8.0);
    compared += 1;
    assert_eq!(cfg.stray_spawn_range, 70.0);
    compared += 1;
    assert_eq!(cfg.stray_despawn_range, 90.0);
    compared += 1;

    println!("Verified {compared} fields in data/dog.json");
    assert_eq!(compared, 37, "All 37 config fields must be verified");
}

#[test]
fn test_following_dog_walks_toward_player() {
    let world = flat_world();
    let mut dogs = Dogs::new(DogConfigData::default());

    let dog_id = dogs.spawn_dog(
        Vec3::ZERO,
        80.0,
        DogMode::Follow,
        "Rex",
        1,
        DogGrowth::NO_START,
    );

    // Player at (0, 0, 10) facing forward -Z (0, 0, -1). Behind player is (0, 0, 13).
    let player_pos = Vec3::new(0.0, 0.0, 10.0);
    let player_forward = Vec3::NEG_Z;
    let ctx = DogUpdateContext::new(player_pos)
        .with_forward(player_forward);

    let frame = dogs.update(0.5, &ctx, &world);

    let dog = dogs.dog().expect("dog must be alive");
    assert_eq!(dog.id(), dog_id);
    assert!(dog.position().z > 0.0, "Dog should walk forward towards target behind player");
    assert_eq!(frame.draws.len(), 1);
    assert_eq!(frame.draws[0].clip, "run");
}

#[test]
fn test_dog_bites_zombie_in_range() {
    let world = flat_world();
    let mut dogs = Dogs::new(DogConfigData::default());

    let dog_id = dogs.spawn_dog(
        Vec3::ZERO,
        80.0,
        DogMode::Follow,
        "Rex",
        1,
        DogGrowth::NO_START,
    );

    // Zombie at (0, 0, 1.0) which is within bite_range (1.2m)
    let zombies = [(42u64, Vec3::new(0.0, 0.0, 1.0), false)];
    let ctx = DogUpdateContext::new(Vec3::new(0.0, 0.0, 2.0))
        .with_zombies(&zombies);

    let frame = dogs.update(0.1, &ctx, &world);

    assert_eq!(frame.bites.len(), 1, "Dog should bite zombie in range");
    let bite = &frame.bites[0];
    assert_eq!(bite.target_id, 42);
    assert_eq!(bite.damage.amount, 15.0);
    assert_eq!(bite.damage.damage_type, "dog_bite");
    assert_eq!(bite.damage.source, Some(dog_id));

    let dog = dogs.dog().expect("dog must exist");
    assert_eq!(dog.bite_count(), 1);
}

#[test]
fn test_feeding_removes_food_and_restores_health() {
    let mut dogs = Dogs::new(DogConfigData::default());
    let dog_id = dogs.spawn_dog(
        Vec3::ZERO,
        80.0,
        DogMode::Follow,
        "Rex",
        1,
        DogGrowth::NO_START,
    );

    // Damage dog health
    let dog = dogs.dog_mut().unwrap();
    dog.health_mut().take_damage(DamageInfo::new(50.0));
    assert_eq!(dog.health().current(), 30.0);

    // Prepare player inventory with 1 raw_meat
    let mut bag = Inventory::with_capacity(10, 20.0);
    let left = bag.add("raw_meat", 1);
    assert_eq!(left, 0);
    assert!(bag.has("raw_meat", 1));

    // Feed dog
    let res = dogs.feed(dog_id, &mut bag);
    assert!(res.is_ok(), "Feeding should succeed");
    assert_eq!(res.unwrap(), "Fed Rex.");

    // Verify raw_meat removed from bag
    assert!(!bag.has("raw_meat", 1), "Meat should be consumed from inventory");

    // Verify health increased by feedHeal (30.0) -> 30 + 30 = 60
    let dog = dogs.dog().unwrap();
    assert_eq!(dog.health().current(), 60.0);
}

#[test]
fn test_growth_puppy_to_adult() {
    let cfg = DogConfigData::default();
    assert_eq!(cfg.puppy_scale, 0.8);
    assert_eq!(cfg.grow_days, 1.0);

    // Born at hour 0.0
    let start_hours = 0.0;

    // At birth: scale is 0.8
    let s0 = DogGrowth::scale(0.0, start_hours, &cfg);
    assert!((s0 - 0.8).abs() < 1e-5);

    // Halfway (12 hours): scale is 0.8 + 0.2 * 0.5 = 0.9
    let s12 = DogGrowth::scale(12.0, start_hours, &cfg);
    assert!((s12 - 0.9).abs() < 1e-5);

    // After 1 full day (24 hours): scale is 1.0
    let s24 = DogGrowth::scale(24.0, start_hours, &cfg);
    assert!((s24 - 1.0).abs() < 1e-5);

    // Further in time: clamped to 1.0
    let s48 = DogGrowth::scale(48.0, start_hours, &cfg);
    assert!((s48 - 1.0).abs() < 1e-5);

    // Adult dog (start = -1.0)
    let s_adult = DogGrowth::scale(100.0, DogGrowth::NO_START, &cfg);
    assert!((s_adult - 1.0).abs() < 1e-5);
}

#[test]
fn test_save_round_trip() {
    let mut dogs = Dogs::new(DogConfigData::default());
    let _id = dogs.spawn_dog(
        Vec3::new(10.0, 0.0, -5.0),
        55.0,
        DogMode::Stay,
        "Buddy",
        3,
        12.5,
    );

    dogs.dead_count = 2;
    dogs.tamed.insert("cabin_1".to_string());
    dogs.tamed.insert("house_2".to_string());
    dogs.dog_mut().unwrap().pack_mut().add("raw_meat", 2);

    let saved = dogs.to_save();

    // Verify saved structure
    assert_eq!(saved.get("dead_count").and_then(|v| v.as_i64()), Some(2));
    assert_eq!(saved["dog"]["name"], "Buddy");
    assert_eq!(saved["dog"]["mode"], "Stay");
    assert_eq!(saved["dog"]["health"], 55.0);
    assert_eq!(saved["dog"]["bought_day"], 3);
    assert_eq!(saved["dog"]["growth_start"], 12.5);

    // Restore into fresh manager
    let mut loaded = Dogs::new(DogConfigData::default());
    loaded.from_save(&saved);

    assert_eq!(loaded.dead_count(), 2);
    assert!(loaded.is_tamed("cabin_1"));
    assert!(loaded.is_tamed("house_2"));

    let dog = loaded.dog().expect("dog restored from save");
    assert_eq!(dog.name(), "Buddy");
    assert_eq!(dog.mode(), DogMode::Stay);
    assert_eq!(dog.health().current(), 55.0);
    assert_eq!(dog.position(), Vec3::new(10.0, 0.0, -5.0));
    assert_eq!(dog.bought_day, 3);
    assert_eq!(dog.growth_start, 12.5);
    assert!(dog.pack().has("raw_meat", 2));
}

#[test]
fn test_breeder_takes_right_price() {
    let mut dogs = Dogs::new(DogConfigData::default());
    let mut wallet = Wallet::new(200);
    let breeder = DogBreeder::new(Vec3::new(5.0, 0.0, 5.0));

    assert_eq!(breeder.prompt(false, &dogs.config), "Buy a puppy ($150)");

    // Buy puppy successfully
    let spawn_at = Vec3::new(5.0, 0.0, 4.0);
    let res = breeder.interact(&mut dogs, &mut wallet, spawn_at, 1, 0.0);
    assert!(res.is_ok());
    assert_eq!(wallet.dollars(), 50, "150 dollars deducted from wallet (200 - 150 = 50)");
    assert!(dogs.has_dog());
    assert_eq!(dogs.dog().unwrap().name(), "Rex");
    assert_eq!(dogs.dog().unwrap().growth_start, 0.0);

    assert_eq!(breeder.prompt(true, &dogs.config), "Dog breeder (you already have a dog)");

    // Second buy attempt fails because player already has a dog
    let res2 = breeder.interact(&mut dogs, &mut wallet, spawn_at, 1, 0.0);
    assert!(res2.is_err());
    assert_eq!(wallet.dollars(), 50, "Wallet balance unchanged on failure");

    // Attempt buy with insufficient funds
    let mut dogs2 = Dogs::new(DogConfigData::default());
    let mut poor_wallet = Wallet::new(100);
    let res3 = breeder.interact(&mut dogs2, &mut poor_wallet, spawn_at, 1, 0.0);
    assert!(res3.is_err());
    assert_eq!(poor_wallet.dollars(), 100, "Wallet balance unchanged on failure");
}

#[test]
fn test_stray_spawner_and_adopt() {
    let mut dogs = Dogs::new(DogConfigData::default());
    let lot_id = "cabin_42";
    let at = Vec3::new(20.0, 0.0, 30.0);
    let site = DogPoiSite::new(lot_id, "cabin", at, 0.0, 4.0);
    assert_eq!(site.building_id, "cabin");
    dogs.spawn_stray_at(&site.poi_id, site.position);

    assert_eq!(dogs.strays().len(), 1);
    assert_eq!(dogs.strays()[0].lot_id(), lot_id);

    // Prepare food in inventory
    let mut bag = Inventory::with_capacity(10, 20.0);
    bag.add("cooked_meat", 1);

    // Adopt stray
    let adopt_res = dogs.adopt_stray(lot_id, &mut bag, 2);
    assert!(adopt_res.is_ok());
    assert!(!bag.has("cooked_meat", 1));
    assert!(dogs.is_tamed(lot_id));
    assert_eq!(dogs.strays().len(), 0);
    assert!(dogs.has_dog());

    let dog = dogs.dog().unwrap();
    assert_eq!(dog.position(), at);
    assert_eq!(dog.growth_start, DogGrowth::NO_START, "Adopted stray starts as full size adult");
}
