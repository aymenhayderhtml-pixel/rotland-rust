use game::items::Inventory;
use game::vehicles::{
    describe_gauge, car_roll, spawn_rules, CarInput, CarSpawnRules, Cars, VehicleConfigData,
    CAR_SYSTEM_ID,
};
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
fn test_config_loads_completely() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let mut compared = 0;

    assert_eq!(config.mass, 1100.0);
    assert_eq!(config.engine_force, 6500.0);
    assert_eq!(config.reverse_force, 2000.0);
    assert_eq!(config.brake_force, 6000.0);
    assert_eq!(config.top_speed_kmh, 90.0);
    assert_eq!(config.reverse_top_kmh, 25.0);
    assert_eq!(config.max_steer_rad, 0.5);
    assert_eq!(config.steer_speed_rad, 2.5);
    assert_eq!(config.suspension_rest, 0.30);
    assert_eq!(config.suspension_travel, 0.15);
    assert_eq!(config.spring_stiffness, 27000.0);
    assert_eq!(config.spring_damping, 3000.0);
    assert_eq!(config.grip_mu, 1.1);
    assert_eq!(config.handbrake_grip, 0.3);
    assert_eq!(config.tank_litres, 40.0);
    assert_eq!(config.fuel_can_litres, 10.0);
    assert_eq!(config.litres_per_metre_cruise, 0.0005);
    assert_eq!(config.litres_per_metre_full_throttle, 0.001);
    assert_eq!(config.health, 300.0);
    assert_eq!(config.run_over_min_kmh, 15.0);
    assert_eq!(config.run_over_damage_per_kmh, 1.8);
    assert_eq!(config.run_over_car_damage, 2.0);
    assert_eq!(config.run_over_knockback_mps, 5.0);
    assert_eq!(config.hit_half_width, 1.1);
    assert_eq!(config.hit_front, 2.4);
    assert_eq!(config.hit_back, 0.6);
    assert_eq!(config.noise_radius_idle, 40.0);
    assert_eq!(config.noise_radius_full, 80.0);
    assert_eq!(config.horn_radius, 60.0);
    assert_eq!(config.exit_max_kmh, 15.0);
    assert_eq!(config.trunk_slots, 20);
    assert_eq!(config.trunk_max_weight, 150.0);
    assert_eq!(config.spawn_chance, 0.35);
    assert_eq!(config.spawn_buildings.len(), 2);
    assert_eq!(config.spawn_buildings[0], "gas_station");
    assert_eq!(config.spawn_buildings[1], "house_small");
    assert_eq!(config.spawn_fuel_min_frac, 0.0);
    assert_eq!(config.spawn_fuel_max_frac, 0.3);
    assert_eq!(config.camp_fuel_frac, 0.5);
    assert_eq!(config.camp_colour, "red");
    assert_eq!(config.colours.len(), 2);
    assert_eq!(config.colours[0], "red");
    assert_eq!(config.colours[1], "blue");
    compared += 43;

    println!("Verified {compared} fields in data/vehicles.json");
    assert!(compared > 0, "Comparison count must be non-zero");
}

#[test]
fn test_parked_car_settles_at_equilibrium_ride_height() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let world = flat_world();
    let mut cars = Cars::new(config.clone());

    // Spawn 0.6m above flat ground at y=0, with full tank
    cars.spawn_car("settle_test", "red", Vec3::new(0.0, 0.6, 0.0), 0.0, 40.0, 300.0);

    // Let the car fall and settle under gravity and suspension damping for 2.0s (120 frames)
    let idle_input = CarInput::default();
    for _ in 0..120 {
        cars.update(1.0 / 60.0, &idle_input, &world);
    }

    let car = cars.get("settle_test").expect("car exists");
    assert!(car.is_grounded(), "car must be grounded");

    // Theoretical equilibrium from spring numbers:
    // mass = 1100 kg, g = 9.8 m/s^2 -> total weight = 10780 N
    // 4 wheels -> load per wheel = 2695 N
    // spring stiffness = 27000 N/m -> compression = 2695 / 27000 = 0.0998148 m (~0.0998 m)
    // Car body origin at equilibrium sits at y = -compression = -0.0998148 m
    let expected_compression = 2695.0 / 27000.0;
    let expected_height = -expected_compression;

    for i in 0..4 {
        let diff = (car.compression(i) - expected_compression).abs();
        assert!(
            diff < 0.005,
            "wheel {i} compression {:.4} should be within 0.005 of expected {:.4}",
            car.compression(i),
            expected_compression
        );
    }

    let height_diff = (car.body.position.y - expected_height).abs();
    assert!(
        height_diff < 0.005,
        "car origin y {:.4} should settle within 0.005 of expected {:.4}",
        car.body.position.y,
        expected_height
    );

    assert!(
        car.body.linear_velocity.length() < 0.05,
        "car velocity should be near zero after settling"
    );
}

#[test]
fn test_throttle_moves_forward_and_burns_fuel() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let world = flat_world();
    let mut cars = Cars::new(config);

    let car = cars.spawn_car("drive_test", "red", Vec3::new(0.0, 0.6, 0.0), 0.0, 40.0, 300.0);
    car.set_driven(true);
    cars.driven_car_id = Some("drive_test".to_string());

    // Settle 0.5s first
    let idle = CarInput::default();
    for _ in 0..30 {
        cars.update(1.0 / 60.0, &idle, &world);
    }

    let start_pos = cars.get("drive_test").unwrap().body.position;
    let start_fuel = cars.get("drive_test").unwrap().tank.litres();

    // Full throttle for 3.0s (180 frames)
    let full_throttle = CarInput {
        throttle: 1.0,
        ..Default::default()
    };
    for _ in 0..180 {
        cars.update(1.0 / 60.0, &full_throttle, &world);
    }

    let car = cars.get("drive_test").unwrap();
    // Forward is -Z in Godot coordinates
    let along_forward = start_pos.z - car.body.position.z;
    assert!(
        along_forward > 15.0,
        "car should move > 15 m forward in 3s (moved {:.2} m)",
        along_forward
    );

    let fuel_used = start_fuel - car.tank.litres();
    assert!(
        fuel_used > 0.0,
        "driving at full throttle must consume fuel"
    );
    assert!(
        fuel_used < 5.0,
        "driving for 3s should consume realistic fuel (< 5.0 L, used {:.4} L)",
        fuel_used
    );
}

#[test]
fn test_steering_turns_the_car() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let world = flat_world();
    let mut cars = Cars::new(config);

    let car = cars.spawn_car("steer_test", "blue", Vec3::new(0.0, 0.6, 0.0), 0.0, 40.0, 300.0);
    car.set_driven(true);
    cars.driven_car_id = Some("steer_test".to_string());

    // Settle 0.5s
    for _ in 0..30 {
        cars.update(1.0 / 60.0, &CarInput::default(), &world);
    }

    // Drive forward 1.0s at half throttle
    let half_throttle = CarInput {
        throttle: 0.5,
        ..Default::default()
    };
    for _ in 0..60 {
        cars.update(1.0 / 60.0, &half_throttle, &world);
    }

    let before_fwd = cars.get("steer_test").unwrap().body.rotation * -Vec3::Z;

    // Steer right for 2.0s
    let steer_input = CarInput {
        throttle: 0.5,
        steer: 1.0,
        ..Default::default()
    };
    for _ in 0..120 {
        cars.update(1.0 / 60.0, &steer_input, &world);
    }

    let after_fwd = cars.get("steer_test").unwrap().body.rotation * -Vec3::Z;
    let angle_turned = before_fwd.angle_between(after_fwd);

    // CarSteers.cs checks turned > 0.2 rad
    assert!(
        angle_turned > 0.2,
        "car turned {:.3} rad, expected > 0.2 rad",
        angle_turned
    );
}

#[test]
fn test_brake_stops_moving_car() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let world = flat_world();
    let mut cars = Cars::new(config);

    cars.driven_car_id = Some("brake_test".to_string());
    let car = cars.spawn_car("brake_test", "red", Vec3::new(0.0, 0.2, 0.0), 0.0, 40.0, 300.0);
    car.set_driven(true);

    // Give car an initial forward speed of 15 m/s (54 km/h)
    car.body.linear_velocity = car.body.rotation * -Vec3::Z * 15.0;

    let brake_input = CarInput {
        brake: 1.0,
        handbrake: true,
        ..Default::default()
    };

    // Brakes at 6000 N on 1100 kg slow by ~5.45 m/s^2, so 15 m/s needs ~2.75 s: brake for 3.5 s (210 frames)
    for _ in 0..210 {
        cars.update(1.0 / 60.0, &brake_input, &world);
    }

    let car = cars.get("brake_test").unwrap();
    assert!(
        car.speed_kmh() < 1.0,
        "car should come to a near complete stop, speed is {:.2} km/h",
        car.speed_kmh()
    );
}

#[test]
fn test_run_over_zombie_at_speed_reports_damage() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let mut cars = Cars::new(config);

    let car = cars.spawn_car("runover_test", "red", Vec3::ZERO, 0.0, 40.0, 300.0);
    car.set_driven(true);
    // 40 km/h is 11.11 m/s along forward (-Z)
    car.body.linear_velocity = Vec3::new(0.0, 0.0, -40.0 / 3.6);

    // Zombie 1.0m directly ahead of the car origin (-Z)
    let zombie_id = 9999u64;
    let targets = [(zombie_id, Vec3::new(0.0, 0.0, -1.0))];

    let hits = cars.run_over(&targets);
    assert_eq!(hits.len(), 1, "zombie in front must be hit");

    let (id, dmg, knockback) = &hits[0];
    assert_eq!(*id, zombie_id);

    // Damage at 40 km/h with run_over_damage_per_kmh = 1.8: 40 * 1.8 = 72.0
    let expected_damage = 40.0 * 1.8;
    assert!(
        (dmg.amount - expected_damage).abs() < 1.0,
        "damage {:.1} should be ~{:.1}",
        dmg.amount,
        expected_damage
    );
    assert_eq!(dmg.damage_type, "car");
    assert!(knockback.y >= 1.5, "knockback has upward component");

    // Car condition took run_over_car_damage (2.0)
    let car = cars.get("runover_test").unwrap();
    assert_eq!(car.condition.current(), 300.0 - 2.0);
}

#[test]
fn test_enter_and_exit_spots() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let world = flat_world();
    let mut cars = Cars::new(config);

    cars.spawn_car("enter_test", "blue", Vec3::ZERO, 0.0, 40.0, 300.0);

    // Player stands 1.5m to the left of the car facing +X (towards car)
    let player_pos = Vec3::new(-1.5, 0.0, 0.0);
    let player_facing = Vec3::new(1.0, 0.0, 0.0);

    let entered = cars.try_enter(player_pos, player_facing);
    assert_eq!(entered.as_deref(), Some("enter_test"));
    assert_eq!(cars.driven_car_id.as_deref(), Some("enter_test"));

    let car = cars.get("enter_test").unwrap();
    assert!(car.is_driven(), "car must be marked driven");

    // Exit the car
    let exit_spot = cars.exit("enter_test", &world);
    assert!(cars.driven_car_id.is_none());
    assert!(!cars.get("enter_test").unwrap().is_driven());

    // Exit spot should be outside the car (distance > 1.2m)
    let dist = exit_spot.distance(Vec3::ZERO);
    assert!(
        dist > 1.2,
        "exit spot {:.2} m should be outside the car (> 1.2 m)",
        dist
    );
}

#[test]
fn test_save_round_trip() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let mut cars = Cars::new(config);

    let car = cars.spawn_car(
        "car_camp",
        "blue",
        Vec3::new(12.5, 0.5, -25.0),
        0.5,
        22.5,
        280.0,
    );
    // Add items to trunk
    assert_eq!(car.trunk.add("bandage", 2), 0);
    assert_eq!(car.trunk.add("scrap_metal", 3), 0);

    let saved_json = cars.to_save();

    // Verify system id and keys
    assert_eq!(CAR_SYSTEM_ID, "cars");
    let cars_array = saved_json.get("cars").expect("cars array present").as_array().expect("array");
    assert_eq!(cars_array.len(), 1);

    let rec = &cars_array[0];
    assert_eq!(rec.get("id").unwrap().as_str().unwrap(), "car_camp");
    assert_eq!(rec.get("colour").unwrap().as_str().unwrap(), "blue");
    assert_eq!(rec.get("fuel").unwrap().as_f64().unwrap(), 22.5);
    assert_eq!(rec.get("health").unwrap().as_f64().unwrap(), 280.0);
    assert!(rec.get("trunk").is_some());

    // Load into fresh Cars manager
    let config2 = VehicleConfigData::from_repo().unwrap();
    let mut cars2 = Cars::new(config2);
    cars2.from_save(&saved_json);

    // Respawn chunk containing (12.5, -25.0)
    let coord = spawn_rules::chunk_of(Vec3::new(12.5, 0.0, -25.0), 64.0);
    cars2.respawn_chunk(coord);

    let loaded = cars2.get("car_camp").expect("car_camp loaded");
    assert_eq!(loaded.colour, "blue");
    assert_eq!(loaded.tank.litres(), 22.5);
    assert_eq!(loaded.condition.current(), 280.0);
    assert_eq!(loaded.trunk.count("bandage"), 2);
    assert_eq!(loaded.trunk.count("scrap_metal"), 3);
}

#[test]
fn test_refuel_from_can_item() {
    let config = VehicleConfigData::from_repo().expect("data/vehicles.json must load");
    let mut cars = Cars::new(config);

    cars.spawn_car("fuel_test", "red", Vec3::ZERO, 0.0, 10.0, 300.0);

    let mut bag = Inventory::new();
    bag.clear();
    assert_eq!(bag.add("fuel_can", 1), 0);

    // Refuel car
    let success = cars.refuel("fuel_test", &mut bag);
    assert!(success, "refuel should succeed with fuel_can in bag");

    let car = cars.get("fuel_test").unwrap();
    assert_eq!(
        car.tank.litres(),
        20.0,
        "fuel should increase by 10 L (from 10 L to 20 L)"
    );
    assert_eq!(bag.count("fuel_can"), 0, "fuel_can item was consumed");

    // Cannot refuel when full
    car_refuel_full_test(&mut cars, &mut bag);
}

fn car_refuel_full_test(cars: &mut Cars, bag: &mut Inventory) {
    let car = cars.get_mut("fuel_test").unwrap();
    car.tank.set(40.0);
    assert_eq!(bag.add("fuel_can", 1), 0);
    let success2 = cars.refuel("fuel_test", bag);
    assert!(!success2, "refuel must fail when tank is full");
    assert_eq!(bag.count("fuel_can"), 1, "fuel_can item was not consumed");
}

#[test]
fn test_gauge_describe_formatting() {
    let text = describe_gauge(45.0, 20.0, 0.5, 300.0, 300.0, false, false);
    assert_eq!(text, "45 km/h\nFuel [#####-----] 20.0 L\nCar 300/300");

    let wrecked_text = describe_gauge(0.0, 0.0, 0.0, 0.0, 300.0, true, true);
    assert_eq!(wrecked_text, "0 km/h\nFuel [----------] 0.0 L\nCar 0/300  WRECKED");

    let no_fuel_text = describe_gauge(0.0, 0.0, 0.0, 200.0, 300.0, false, true);
    assert_eq!(no_fuel_text, "0 km/h\nFuel [----------] 0.0 L\nCar 200/300  NO FUEL");
}

#[test]
fn test_spawn_rules_deterministic_and_poi_parking() {
    let seed = 12345;
    let hash = CarSpawnRules::stable_hash("car_camp");
    assert_eq!(hash, CarSpawnRules::stable_hash("car_camp"));

    let r1 = car_roll(seed, "poi_gas_station_1", spawn_rules::SPAWN_SALT);
    let r2 = car_roll(seed, "poi_gas_station_1", spawn_rules::SPAWN_SALT);
    assert_eq!(r1, r2);
    assert!((0.0..1.0).contains(&r1));

    let parking = spawn_rules::poi_parking_xz(Vec3::new(10.0, 0.0, 20.0), 0.0, 5.0);
    // local_x = 5.0 + 3.5 = 8.5; with yaw = 0: x = 10.0 + 8.5 = 18.5, z = 20.0 - 0 = 20.0
    assert_eq!(parking.x, 18.5);
    assert_eq!(parking.y, 20.0);
}
