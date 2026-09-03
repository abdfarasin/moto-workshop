use moto_workshop_lib::{
    application::inventory::{
        AdjustInventoryStockInput, CreateInventoryItemInput, InventoryApplicationError,
        InventoryApplicationService, LoadInventoryItemDetailsInput, SearchInventoryItemsInput,
        UpdateInventoryItemInput,
    },
    db::{migrate_database, open_database},
};
use tempfile::tempdir;

#[test]
fn inventory_vertical_slice_uses_real_items_exact_money_and_auditable_stock() {
    // # Arrange
    let directory = tempdir().unwrap();
    let mut connection = open_database(directory.path().join("inventory.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let piece_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let mut service = InventoryApplicationService::new(&mut connection);

    // # Act
    let created = service
        .create(CreateInventoryItemInput {
            name: " Oil Filter ".into(),
            sku: Some(" FILTER-1 ".into()),
            unit_id: piece_id,
            default_purchase_price_fils: Some(3_000),
            default_selling_price_fils: 4_500,
            minimum_stock_quantity: 3,
            notes: None,
            opening_quantity: 5,
            created_at: 1_000,
        })
        .unwrap();
    service
        .adjust_stock(AdjustInventoryStockInput {
            inventory_item_id: created.id,
            quantity_delta: -7,
            notes: Some("Count correction".into()),
            created_at: 2_000,
        })
        .unwrap();
    let updated = service
        .update(UpdateInventoryItemInput {
            inventory_item_id: created.id,
            name: "Premium Oil Filter".into(),
            sku: Some("FILTER-1".into()),
            default_purchase_price_fils: Some(3_200),
            default_selling_price_fils: 5_000,
            minimum_stock_quantity: 4,
            notes: Some("Shelf A".into()),
            updated_at: 3_000,
        })
        .unwrap();
    let details = service
        .load(LoadInventoryItemDetailsInput {
            inventory_item_id: created.id,
        })
        .unwrap()
        .unwrap();
    let found = service
        .search(SearchInventoryItemsInput {
            query: "filter-1".into(),
            limit: None,
        })
        .unwrap();
    let duplicate = service
        .create(CreateInventoryItemInput {
            name: "Duplicate SKU".into(),
            sku: Some("filter-1".into()),
            unit_id: piece_id,
            default_purchase_price_fils: None,
            default_selling_price_fils: 1_000,
            minimum_stock_quantity: 0,
            notes: None,
            opening_quantity: 0,
            created_at: 4_000,
        })
        .unwrap_err();

    // # Assert
    assert_eq!(updated.name, "Premium Oil Filter");
    assert_eq!(details.current_quantity, -2);
    assert_eq!(details.default_selling_price_fils, 5_000);
    assert_eq!(details.movements.len(), 2);
    assert_eq!(details.movements[0].quantity_delta, -7);
    assert_eq!(found.len(), 1);
    assert!(found[0].low_stock);
    assert!(matches!(
        duplicate,
        InventoryApplicationError::InventorySkuAlreadyExists
    ));
}

#[test]
fn inventory_directory_is_bounded_excludes_archived_and_unknown_details_are_none() {
    // # Arrange
    let directory = tempdir().unwrap();
    let mut connection = open_database(directory.path().join("bounded.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    for index in 0..105 {
        connection.execute("INSERT INTO inventory_items(name,sku,unit_id,default_selling_price_fils,created_at,updated_at) VALUES (?1,?2,?3,0,1,1)", rusqlite::params![format!("Item {index}"),format!("SKU-{index}"),unit_id]).unwrap();
    }
    connection
        .execute("UPDATE inventory_items SET archived_at=2 WHERE id=1", [])
        .unwrap();
    let service = InventoryApplicationService::new(&mut connection);

    // # Act
    let rows = service
        .search(SearchInventoryItemsInput {
            query: "Item".into(),
            limit: Some(1_000),
        })
        .unwrap();

    // # Assert
    assert_eq!(rows.len(), 100);
    assert!(rows.iter().all(|row| row.id != 1));
    assert!(service
        .load(LoadInventoryItemDetailsInput {
            inventory_item_id: 999_999
        })
        .unwrap()
        .is_none());
}
