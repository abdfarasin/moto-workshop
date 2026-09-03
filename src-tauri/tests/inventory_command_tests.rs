use moto_workshop_lib::{
    commands::{
        inventory::{
            handle_adjust_inventory_stock, handle_create_inventory_item,
            handle_list_inventory_units, handle_search_inventory_items,
            AdjustInventoryStockCommandInput, CreateInventoryItemCommandInput,
            SearchInventoryItemsCommandInput,
        },
        service_visit_workspace::CommandErrorCategory,
    },
    db::open_database,
    runtime::database::RuntimeDatabase,
};
use serde_json::json;
use tempfile::tempdir;

#[test]
fn inventory_commands_use_safe_camel_case_and_return_ledger_backed_dtos() {
    // # Arrange
    let directory = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(directory.path()).unwrap();
    let connection = open_database(database.database_path()).unwrap();
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name = 'Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let create: CreateInventoryItemCommandInput = serde_json::from_value(json!({
        "name": "Oil Filter",
        "sku": "FILTER-1",
        "unitId": unit_id,
        "defaultPurchasePriceFils": 3000,
        "defaultSellingPriceFils": 4500,
        "minimumStockQuantity": 3,
        "notes": null,
        "openingQuantity": 5,
        "createdAt": 1000
    }))
    .unwrap();

    // # Act
    let created = handle_create_inventory_item(&database, create.clone()).unwrap();
    let adjusted = handle_adjust_inventory_stock(
        &database,
        AdjustInventoryStockCommandInput {
            inventory_item_id: created.id,
            quantity_delta: -7,
            notes: Some("Count correction".into()),
            created_at: 2_000,
        },
    )
    .unwrap();
    let rows = handle_search_inventory_items(
        &database,
        SearchInventoryItemsCommandInput {
            query: "filter-1".into(),
            limit: Some(50),
        },
    )
    .unwrap();
    let units = handle_list_inventory_units(&database).unwrap();

    // # Assert
    let serialized = serde_json::to_value(&adjusted).unwrap();
    assert_eq!(serialized["currentQuantity"], -2);
    assert_eq!(serialized["defaultSellingPriceFils"], 4_500);
    assert_eq!(serialized["movements"][0]["movementType"], "ADJUSTMENT_OUT");
    assert_eq!(serialized["movements"][1]["movementType"], "OPENING_STOCK");
    assert_eq!(rows.len(), 1);
    assert!(units.iter().any(|unit| unit.id == unit_id));

    let duplicate = handle_create_inventory_item(&database, create).unwrap_err();
    assert_eq!(
        duplicate.category,
        CommandErrorCategory::InventorySkuAlreadyExists
    );
    assert!(!duplicate.message.to_ascii_lowercase().contains("unique"));
}

#[test]
fn inventory_command_inputs_reject_caller_controlled_or_snapshot_fields() {
    // # Arrange
    let unsafe_input = json!({
        "name": "Oil Filter",
        "sku": null,
        "unitId": 1,
        "defaultPurchasePriceFils": null,
        "defaultSellingPriceFils": 4500,
        "minimumStockQuantity": 0,
        "notes": null,
        "openingQuantity": 0,
        "createdAt": 1000,
        "currentQuantity": 999,
        "databasePath": "caller-controlled.sqlite3"
    });

    // # Act
    let result = serde_json::from_value::<CreateInventoryItemCommandInput>(unsafe_input);

    // # Assert
    assert!(result.is_err());
}
