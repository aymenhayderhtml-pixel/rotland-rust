use std::sync::Arc;

use game::economy::*;
use game::items::{Inventory, ItemRegistry};

#[test]
fn economy_config_loads_from_repo_and_validates() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");

    assert_eq!(config.start_dollars, 20);
    assert!((config.sell_rate - 0.5).abs() < 1e-6);
    assert!((config.buy_rate - 1.0).abs() < 1e-6);
    assert_eq!(config.restock_hour, 6);
    assert_eq!(config.bank_slots, 40);
    assert!((config.interact_range - 4.0).abs() < 1e-6);

    config.validate(Some(&items)).expect("validate with real items");

    let mut stock_count = 0;
    for entry in &config.stock {
        assert!(items.contains(&entry.id), "item '{}' must exist in items.json", entry.id);
        assert!(entry.qty > 0, "quantity for '{}' must be positive", entry.id);
        stock_count += 1;
    }

    assert_eq!(stock_count, 23);
    assert!(stock_count > 0);
}

#[test]
fn price_math_on_real_data_counts_and_matches_all_items() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");

    let mut compared = 0;
    for def in items.all() {
        let expected_buy = ((def.value as f32 * config.buy_rate).floor() as i32).max(1);
        let expected_sell = ((def.value as f32 * config.sell_rate).floor() as i32).max(1);

        let actual_buy = buy_price(def, config.buy_rate);
        let actual_sell = sell_price(def, config.sell_rate);

        assert_eq!(actual_buy, expected_buy, "buy price mismatch for '{}'", def.id);
        assert_eq!(actual_sell, expected_sell, "sell price mismatch for '{}'", def.id);
        assert!(actual_buy >= 1);
        assert!(actual_sell >= 1);

        compared += 1;
    }

    // 43 items in data/items.json
    assert_eq!(compared, 43);
    assert!(compared > 0);

    // Verify key hand-calculated prices from items.json
    let beans = items.get("canned_beans").expect("canned_beans");
    assert_eq!(beans.value, 7);
    assert_eq!(buy_price(beans, config.buy_rate), 7);
    assert_eq!(sell_price(beans, config.sell_rate), 3); // floor(7 * 0.5) = 3

    let medkit = items.get("medkit").expect("medkit");
    assert_eq!(medkit.value, 45);
    assert_eq!(buy_price(medkit, config.buy_rate), 45);
    assert_eq!(sell_price(medkit, config.sell_rate), 22); // floor(45 * 0.5) = 22

    let ammo = items.get("pistol_ammo").expect("pistol_ammo");
    assert_eq!(ammo.value, 1);
    assert_eq!(buy_price(ammo, config.buy_rate), 1);
    assert_eq!(sell_price(ammo, config.sell_rate), 1); // floor(1 * 0.5) = 0, clamped to 1
}

#[test]
fn wallet_operations_and_death_penalty_hook() {
    let mut wallet = Wallet::new(20);
    assert_eq!(wallet.dollars(), 20);

    wallet.add(30);
    assert_eq!(wallet.dollars(), 50);

    assert!(wallet.try_spend(15));
    assert_eq!(wallet.dollars(), 35);

    assert!(!wallet.try_spend(100));
    assert_eq!(wallet.dollars(), 35);

    assert!(!wallet.try_spend(-10));
    assert_eq!(wallet.dollars(), 35);

    // Death penalty: empties the wallet completely
    let emptied = wallet.take_all();
    assert_eq!(emptied, 35);
    assert_eq!(wallet.dollars(), 0);

    assert_eq!(wallet.take_all(), 0);
}

#[test]
fn buy_and_sell_moves_money_and_items() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items.clone());
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");

    let mut wallet = Wallet::new(100);
    let mut bag = Inventory::with_capacity_and_registry(24, 40.0, reg_arc);
    let mut stock = TraderStock::new(&config);

    // Initial check: trader has 6 canned beans
    assert_eq!(stock.qty("canned_beans"), 6);
    assert_eq!(bag.count("canned_beans"), 0);

    // Buy 2 canned beans at $7 each = $14
    let bought = buy(
        &mut wallet,
        &mut bag,
        &mut stock,
        "canned_beans",
        2,
        config.buy_rate,
        &items,
    );
    assert_eq!(bought, 2);
    assert_eq!(wallet.dollars(), 100 - 14);
    assert_eq!(bag.count("canned_beans"), 2);
    assert_eq!(stock.qty("canned_beans"), 4);

    // Sell 1 canned bean at $3
    let sold = sell(
        &mut wallet,
        &mut bag,
        "canned_beans",
        1,
        config.sell_rate,
        &items,
    );
    assert_eq!(sold, 1);
    assert_eq!(wallet.dollars(), 86 + 3);
    assert_eq!(bag.count("canned_beans"), 1);
    // Sold items do not increase trader stock
    assert_eq!(stock.qty("canned_beans"), 4);
}

#[test]
fn cannot_buy_without_money_or_bag_space() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items.clone());
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");

    // Case 1: Not enough dollars
    let wallet_broke = Wallet::new(0);
    let bag = Inventory::with_capacity_and_registry(24, 40.0, reg_arc.clone());
    let mut stock = TraderStock::new(&config);

    let blocker = buy_blocker(
        &wallet_broke,
        &bag,
        &stock,
        "canned_beans",
        config.buy_rate,
        &items,
    );
    assert_eq!(blocker, Some(BuyBlocker::NotEnoughDollars));

    // Case 2: Sold out
    let wallet_rich = Wallet::new(100);
    stock.take("medkit", 1); // medkit has initial qty 1, now 0
    assert_eq!(stock.qty("medkit"), 0);

    let blocker_sold_out = buy_blocker(
        &wallet_rich,
        &bag,
        &stock,
        "medkit",
        config.buy_rate,
        &items,
    );
    assert_eq!(blocker_sold_out, Some(BuyBlocker::SoldOut));

    // Case 3: Too heavy
    // Bag with 1.0 kg max weight, holding 1.0 kg of wood
    let mut bag_heavy = Inventory::with_capacity_and_registry(5, 1.0, reg_arc.clone());
    bag_heavy.add("wood", 1); // 1.0 kg
    let blocker_heavy = buy_blocker(
        &wallet_rich,
        &bag_heavy,
        &stock,
        "canned_beans",
        config.buy_rate,
        &items,
    );
    assert_eq!(blocker_heavy, Some(BuyBlocker::TooHeavy));

    // Case 4: Bag is full (no free slots)
    // 1-slot bag with 1 crowbar (max_stack 1, weight 2.0 kg out of 100.0 kg)
    let mut bag_full_slots = Inventory::with_capacity_and_registry(1, 100.0, reg_arc);
    bag_full_slots.add("crowbar", 1);
    let blocker_full = buy_blocker(
        &wallet_rich,
        &bag_full_slots,
        &stock,
        "canned_beans",
        config.buy_rate,
        &items,
    );
    assert_eq!(blocker_full, Some(BuyBlocker::BagIsFull));
}

#[test]
fn trader_stock_deplete_and_restock_after_a_day() {
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");
    let mut stock = TraderStock::new(&config);

    assert_eq!(stock.qty("fuel_can"), 2);
    assert_eq!(stock.qty("water_bottle"), 6);
    assert_eq!(stock.last_restock_day(), 0);

    // Deplete stock
    assert!(stock.take("fuel_can", 2));
    assert_eq!(stock.qty("fuel_can"), 0);
    assert!(stock.take("water_bottle", 4));
    assert_eq!(stock.qty("water_bottle"), 2);

    // Hour 5 on day 1: restock_hour is 6, so no restock yet
    stock.tick(1, 5, config.restock_hour);
    assert_eq!(stock.qty("fuel_can"), 0);
    assert_eq!(stock.qty("water_bottle"), 2);
    assert_eq!(stock.last_restock_day(), 0);

    // Hour 6 on day 1: triggers refill
    stock.tick(1, 6, config.restock_hour);
    assert_eq!(stock.qty("fuel_can"), 2);
    assert_eq!(stock.qty("water_bottle"), 6);
    assert_eq!(stock.last_restock_day(), 1);

    // Take one more on day 1
    assert!(stock.take("fuel_can", 1));
    assert_eq!(stock.qty("fuel_can"), 1);

    // Later on day 1: does not restock again on the same day
    stock.tick(1, 14, config.restock_hour);
    assert_eq!(stock.qty("fuel_can"), 1);
    assert_eq!(stock.last_restock_day(), 1);

    // Day 2 at hour 6: triggers second restock
    stock.tick(2, 6, config.restock_hour);
    assert_eq!(stock.qty("fuel_can"), 2);
    assert_eq!(stock.last_restock_day(), 2);
}

#[test]
fn bank_deposit_withdraw_and_item_transfer() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);

    let mut bank = Bank::with_slots_and_registry(40, reg_arc.clone());
    let mut wallet = Wallet::new(50);

    // Deposit dollars
    let deposited = bank.deposit(&mut wallet, 30);
    assert_eq!(deposited, 30);
    assert_eq!(bank.dollars(), 30);
    assert_eq!(wallet.dollars(), 20);

    // Withdraw dollars
    let withdrawn = bank.withdraw(&mut wallet, 10);
    assert_eq!(withdrawn, 10);
    assert_eq!(bank.dollars(), 20);
    assert_eq!(wallet.dollars(), 30);

    // Transfer items between bag and bank
    let mut bag = Inventory::with_capacity_and_registry(10, 40.0, reg_arc);
    bag.add("canned_beans", 5);
    assert_eq!(bag.slot(0).unwrap().count, 5);

    // Move 3 beans from bag slot 0 to bank
    let moved_to_bank = bank.move_to_bank(&mut bag, 0, 3);
    assert_eq!(moved_to_bank, 3);
    assert_eq!(bag.slot(0).unwrap().count, 2);
    assert_eq!(bank.items().count("canned_beans"), 3);

    // Move 2 beans back from bank slot 0 to bag
    let moved_to_bag = bank.move_to_bag(&mut bag, 0, 2);
    assert_eq!(moved_to_bag, 2);
    assert_eq!(bank.items().count("canned_beans"), 1);
    assert_eq!(bag.count("canned_beans"), 4);
}

#[test]
fn save_round_trip_for_wallet_bank_trader() {
    let items = ItemRegistry::from_repo().expect("ItemRegistry::from_repo");
    let reg_arc = Arc::new(items);
    let config = EconomyConfigData::from_repo().expect("EconomyConfigData::from_repo");

    // 1. Wallet save round trip
    let wallet = Wallet::new(125);
    let wallet_json = wallet.to_save();
    assert_eq!(wallet_json["dollars"], 125);

    let loaded_wallet = Wallet::from_save_value(&wallet_json);
    assert_eq!(loaded_wallet.dollars(), 125);

    // 2. TraderStock save round trip
    let mut stock = TraderStock::new(&config);
    stock.tick(3, 8, config.restock_hour);
    assert!(stock.take("fuel_can", 1));
    assert_eq!(stock.qty("fuel_can"), 1);
    assert_eq!(stock.last_restock_day(), 3);

    let stock_json = stock.to_save();
    assert_eq!(stock_json["restockDay"], 3);
    assert_eq!(stock_json["qty"]["fuel_can"], 1);

    let loaded_stock = TraderStock::from_save_value(&stock_json, &config);
    assert_eq!(loaded_stock.last_restock_day(), 3);
    assert_eq!(loaded_stock.qty("fuel_can"), 1);
    assert_eq!(loaded_stock.qty("water_bottle"), 6);

    // 3. Bank save round trip
    let mut bank = Bank::with_slots_and_registry(40, reg_arc.clone());
    let mut temp_wallet = Wallet::new(75);
    bank.deposit(&mut temp_wallet, 75);
    bank.items_mut().add("bandage", 5);

    let bank_json = bank.to_save();
    assert_eq!(bank_json["dollars"], 75);
    assert!(bank_json.get("items").is_some());

    let loaded_bank = Bank::from_save_value(&bank_json, reg_arc, 40);
    assert_eq!(loaded_bank.dollars(), 75);
    assert_eq!(loaded_bank.items().count("bandage"), 5);
}
