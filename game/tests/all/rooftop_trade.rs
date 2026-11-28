//! Rooftop trader stock config (R44): data/rooftop_trader.json loads, validates and fills a shelf.

use game::economy::RooftopTraderConfig;
use game::items::ItemRegistry;

#[test]
fn rooftop_trader_config_loads_from_repo() {
    let cfg = RooftopTraderConfig::from_repo().expect("data/rooftop_trader.json");
    assert_eq!(cfg.map_label, "Rooftop Camp");
    assert!(cfg.stock.len() >= 7, "expected at least 7 stock lines, got {}", cfg.stock.len());
}

#[test]
fn rooftop_trader_stock_ids_exist_in_item_registry() {
    let items = ItemRegistry::from_repo().expect("data/items.json");
    let cfg = RooftopTraderConfig::from_repo().expect("data/rooftop_trader.json");
    cfg.validate(&items).expect("every rooftop stock id must be in items.json");
}

#[test]
fn rooftop_trader_validate_rejects_unknown_item() {
    let items = ItemRegistry::from_repo().expect("data/items.json");
    let json = r#"{ "mapLabel": "x", "stock": [ { "id": "no_such_item_xyz", "qty": 1 } ] }"#;
    let cfg = RooftopTraderConfig::from_json(json).expect("parses");
    assert!(cfg.validate(&items).is_err());
}

#[test]
fn rooftop_stock_has_each_quantity_and_take_works() {
    let cfg = RooftopTraderConfig::from_repo().expect("data/rooftop_trader.json");
    let mut stock = cfg.rooftop_stock();
    for entry in &cfg.stock {
        assert_eq!(stock.qty(&entry.id), entry.qty, "qty for '{}'", entry.id);
    }

    let first = cfg.stock[0].clone();
    assert!(stock.take(&first.id, 1));
    assert_eq!(stock.qty(&first.id), first.qty - 1);

    assert!(!stock.take(&first.id, first.qty), "taking more than in stock must fail");
    assert_eq!(stock.qty(&first.id), first.qty - 1, "failed take must not change stock");
}

#[test]
fn broken_rooftop_trader_json_is_an_error() {
    assert!(RooftopTraderConfig::from_json("{ not json").is_err());
}
