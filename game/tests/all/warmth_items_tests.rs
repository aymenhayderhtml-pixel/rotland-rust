use game::items::{ItemCategory, ItemRegistry};

#[test]
fn warmth_items_load_with_expected_fields() {
    let items_path = paths::repo_path("data/items.json");
    let registry = ItemRegistry::load(&items_path).expect("ItemRegistry::load succeeds");

    let text = std::fs::read_to_string(&items_path).expect("read data/items.json");
    let raw: serde_json::Value = serde_json::from_str(&text).expect("valid items.json");

    for id in ["warm_jacket", "wool_hat", "desert_scarf", "hand_warmer"] {
        let def = registry
            .get(id)
            .unwrap_or_else(|| panic!("missing item `{id}` in ItemRegistry"));
        assert_eq!(def.category, ItemCategory::Tool, "item `{id}` category");
        assert!(def.weight > 0.0, "item `{id}` weight must be positive");
        assert!(def.max_stack >= 1, "item `{id}` max_stack must be >= 1");
    }

    let jacket = &raw["warm_jacket"];
    assert_eq!(jacket["slot"].as_str(), Some("body"));
    assert!((jacket["insulation"].as_f64().unwrap() - 0.5).abs() < 1e-6);

    let hat = &raw["wool_hat"];
    assert_eq!(hat["slot"].as_str(), Some("head"));
    assert!((hat["insulation"].as_f64().unwrap() - 0.2).abs() < 1e-6);

    let scarf = &raw["desert_scarf"];
    assert_eq!(scarf["slot"].as_str(), Some("head"));
    assert!((scarf["sun_protection"].as_f64().unwrap() - 0.5).abs() < 1e-6);

    let warmer = &raw["hand_warmer"];
    assert_eq!(warmer["consumable"].as_bool(), Some(true));
    assert!((warmer["warmth_seconds"].as_f64().unwrap() - 60.0).abs() < 1e-6);
}

#[test]
fn recipes_use_only_existing_items_and_include_warmth_gear() {
    let registry = ItemRegistry::from_repo().expect("items registry");
    let text =
        std::fs::read_to_string(paths::repo_path("data/recipes.json")).expect("read recipes.json");
    let recipes: Vec<serde_json::Value> = serde_json::from_str(&text).expect("valid recipes.json");

    let mut found_ids = Vec::new();
    for recipe in &recipes {
        let recipe_id = recipe["id"].as_str().expect("recipe id string");
        found_ids.push(recipe_id.to_owned());

        let inputs = recipe["inputs"].as_array().expect("recipe inputs array");
        assert!(!inputs.is_empty(), "recipe `{recipe_id}` has no inputs");
        for input in inputs {
            let item_id = input["id"].as_str().expect("input item id");
            let count = input["count"].as_u64().expect("input count");
            assert!(
                registry.get(item_id).is_some(),
                "recipe `{recipe_id}` references unknown input `{item_id}`"
            );
            assert!(count > 0, "recipe `{recipe_id}` input `{item_id}` count must be > 0");
        }

        let outputs = recipe["outputs"].as_array().expect("recipe outputs array");
        assert!(!outputs.is_empty(), "recipe `{recipe_id}` has no outputs");
        for output in outputs {
            let item_id = output["id"].as_str().expect("output item id");
            let count = output["count"].as_u64().expect("output count");
            assert!(
                registry.get(item_id).is_some(),
                "recipe `{recipe_id}` references unknown output `{item_id}`"
            );
            assert!(count > 0, "recipe `{recipe_id}` output `{item_id}` count must be > 0");
        }
    }

    for expected in ["warm_jacket", "wool_hat", "desert_scarf", "hand_warmer"] {
        assert!(
            found_ids.iter().any(|id| id == expected),
            "missing warmth recipe `{expected}` in data/recipes.json"
        );
    }
}

#[test]
fn loot_tables_reference_only_existing_items_and_seed_biome_warmth_gear() {
    let registry = ItemRegistry::from_repo().expect("items registry");
    let text = std::fs::read_to_string(paths::repo_path("data/loot_tables.json"))
        .expect("read loot_tables.json");
    let tables: serde_json::Value = serde_json::from_str(&text).expect("valid loot_tables.json");
    let table_map = tables.as_object().expect("loot_tables root object");

    for (table_name, table) in table_map {
        let entries = table["entries"]
            .as_array()
            .unwrap_or_else(|| panic!("table `{table_name}` missing entries"));
        for entry in entries {
            let item_id = entry["id"].as_str().expect("loot entry id");
            let weight = entry["weight"].as_u64().expect("loot entry weight");
            let count = entry["count"].as_array().expect("loot entry count");
            assert!(
                registry.get(item_id).is_some(),
                "loot table `{table_name}` references unknown item `{item_id}`"
            );
            assert!(
                weight > 0,
                "loot table `{table_name}` entry `{item_id}` weight must be > 0"
            );
            assert_eq!(count.len(), 2, "loot table `{table_name}` count must be [min, max]");
            assert!(count[0].as_u64().unwrap() <= count[1].as_u64().unwrap());
        }
    }

    let has_entry = |table_name: &str, item_id: &str| {
        table_map[table_name]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["id"].as_str() == Some(item_id))
    };

    for cold_table in ["snow", "mountains"] {
        for item in ["warm_jacket", "wool_hat", "hand_warmer"] {
            assert!(
                has_entry(cold_table, item),
                "expected `{item}` in `{cold_table}` loot table"
            );
        }
    }
    assert!(
        has_entry("desert", "desert_scarf"),
        "expected `desert_scarf` in `desert` loot table"
    );
}

#[test]
fn warmth_hints_exist_with_expected_text() {
    let text =
        std::fs::read_to_string(paths::repo_path("data/hints.json")).expect("read hints.json");
    let raw: serde_json::Value = serde_json::from_str(&text).expect("valid hints.json");
    let hints = raw["hints"].as_array().expect("hints array");

    let find = |id: &str| {
        hints
            .iter()
            .find(|h| h["id"].as_str() == Some(id))
            .and_then(|h| h["text"].as_str())
    };

    assert_eq!(
        find("cold"),
        Some("You're getting cold. A campfire, warm clothes or shelter will help.")
    );
    assert_eq!(
        find("hot"),
        Some("The heat makes you thirsty. Drink water and cover your head.")
    );
}
