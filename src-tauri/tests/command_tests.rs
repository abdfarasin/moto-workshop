use rusqlite::{params, Connection};
use serde_json::json;
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    commands::service_visit_workspace::{
        handle_add_service_visit_part, handle_list_service_visit_inventory_items,
        handle_load_service_visit_workspace, handle_update_service_visit_work,
        handle_void_service_visit_part, AddServiceVisitPartCommandInput, CommandErrorCategory,
        UpdateServiceVisitWorkCommandInput, VoidServiceVisitPartCommandInput,
    },
    db::open_database,
    runtime::database::{RuntimeDatabase, DATABASE_FILE_NAME},
};

#[test]
fn runtime_database_uses_stable_app_data_path_and_migrates_to_schema_seven() {
    // # Arrange
    let temp_dir = tempdir().unwrap();
    let app_data_dir = temp_dir.path().join("nested").join("app-data");

    // # Act
    let database =
        RuntimeDatabase::initialize(&app_data_dir).expect("runtime database should initialize");

    // # Assert
    assert_eq!(
        database.database_path(),
        app_data_dir.join(DATABASE_FILE_NAME)
    );
    assert!(app_data_dir.is_dir());
    let connection = open_database(database.database_path()).unwrap();
    let version: i64 = connection
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .unwrap();
    assert_eq!(version, 7);
    let part_table_exists: bool = connection
        .query_row(
            "SELECT EXISTS(
                SELECT 1 FROM sqlite_master
                WHERE type = 'table' AND name = 'service_visit_parts'
             )",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(part_table_exists);
}

#[test]
fn workspace_handler_maps_complete_camel_case_dto_and_stable_statuses() {
    // # Arrange
    let fixture = fixture();
    let active_part_id = insert_part(
        &fixture.seed_connection,
        fixture.visit_id,
        fixture.filter_item_id,
        PartValues::new("Oil Filter", "Piece", 2, 1, 4_500, 9_000, 2_000),
    );
    let voided_part_id = insert_part(
        &fixture.seed_connection,
        fixture.visit_id,
        fixture.oil_item_id,
        PartValues::new("Engine Oil", "Liter", 2_500, 1_000, 7_000, 17_500, 2_100),
    );
    fixture
        .seed_connection
        .execute(
            "UPDATE service_visit_parts
             SET status = 'VOIDED', voided_at = 2200, void_reason = 'Wrong oil'
             WHERE id = ?1",
            [voided_part_id],
        )
        .unwrap();

    // # Act
    let workspace = handle_load_service_visit_workspace(&fixture.database, fixture.visit_id)
        .expect("workspace command handler should succeed");
    let serialized = serde_json::to_value(&workspace).unwrap();

    // # Assert
    assert_eq!(workspace.visit.id, fixture.visit_id);
    assert_eq!(workspace.owner.name, "Ahmad Ali");
    assert_eq!(workspace.motorcycle.make_name, "Honda");
    assert_eq!(workspace.parts[0].id, active_part_id);
    assert_eq!(workspace.parts[1].id, voided_part_id);
    assert_eq!(serialized["visit"]["status"], "OPEN");
    assert_eq!(serialized["visit"]["motorcycleId"], fixture.motorcycle_id);
    assert_eq!(serialized["visit"]["ownerCustomerId"], fixture.owner_id);
    assert_eq!(serialized["visit"]["customerComplaint"], "Oil leak");
    assert_eq!(serialized["visit"]["laborChargeFils"], 0);
    assert_eq!(serialized["motorcycle"]["makeName"], "Honda");
    assert_eq!(serialized["motorcycle"]["plateCode"], "29");
    assert_eq!(serialized["parts"][0]["status"], "ACTIVE");
    assert_eq!(serialized["parts"][0]["serviceVisitId"], fixture.visit_id);
    assert_eq!(serialized["parts"][0]["lineTotalFils"], 9_000);
    assert_eq!(serialized["parts"][1]["status"], "VOIDED");
    assert_eq!(serialized["parts"][1]["voidReason"], "Wrong oil");
}

#[test]
fn inventory_and_add_part_handlers_expose_only_safe_authoritative_data() {
    // # Arrange
    let fixture = fixture();
    let input = AddServiceVisitPartCommandInput {
        service_visit_id: fixture.visit_id,
        inventory_item_id: fixture.oil_item_id,
        quantity: 333,
        unit_price_fils: 5_500,
        created_at: 2_000,
    };

    // # Act
    let inventory = handle_list_service_visit_inventory_items(&fixture.database)
        .expect("inventory command handler should succeed");
    let part = handle_add_service_visit_part(&fixture.database, input)
        .expect("add-part command handler should succeed");
    let inventory_json = serde_json::to_value(&inventory).unwrap();
    let part_json = serde_json::to_value(&part).unwrap();

    // # Assert
    assert_eq!(inventory.len(), 2);
    assert_eq!(inventory_json[0]["itemName"], "Engine Oil");
    assert_eq!(inventory_json[0]["unitName"], "Liter");
    assert_eq!(inventory_json[0]["quantityScale"], 1_000);
    assert_eq!(inventory_json[0]["defaultSellingPriceFils"], 7_000);
    assert!(inventory_json[0].get("currentStock").is_none());
    assert_eq!(part.item_name, "Engine Oil");
    assert_eq!(part.unit_name, "Liter");
    assert_eq!(part.quantity_scale, 1_000);
    assert_eq!(part.line_total_fils, 1_832);
    assert_eq!(part_json["status"], "ACTIVE");
    assert_eq!(part_json["unitPriceFils"], 5_500);
    let movement: (String, i64) = fixture
        .seed_connection
        .query_row(
            "SELECT movement_type, quantity_delta FROM stock_movements
             WHERE service_visit_part_id = ?1",
            [part.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(movement, ("SERVICE_USAGE".into(), -333));
}

#[test]
fn update_and_void_handlers_return_stable_error_categories_without_sql_details() {
    // # Arrange
    let fixture = fixture();

    // # Act
    let update_error = handle_update_service_visit_work(
        &fixture.database,
        UpdateServiceVisitWorkCommandInput {
            service_visit_id: fixture.visit_id,
            diagnosis: None,
            work_performed: None,
            labor_charge_fils: -1,
            notes: None,
            odometer_km: None,
            updated_at: 2_000,
        },
    )
    .expect_err("invalid work input should map to a command error");
    let void_error = handle_void_service_visit_part(
        &fixture.database,
        VoidServiceVisitPartCommandInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: 999_999,
            voided_at: 2_000,
            reason: None,
        },
    )
    .expect_err("missing part should map to a command error");
    let missing_visit = handle_load_service_visit_workspace(&fixture.database, 999_999)
        .expect_err("missing visit should map to a command error");
    let missing_item = handle_add_service_visit_part(
        &fixture.database,
        AddServiceVisitPartCommandInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: 999_999,
            quantity: 1,
            unit_price_fils: 1,
            created_at: 2_000,
        },
    )
    .expect_err("missing inventory item should map to a command error");
    fixture
        .seed_connection
        .execute(
            "UPDATE service_visits
             SET status = 'CANCELLED', cancelled_at = 2100,
                 cancellation_reason = 'Customer declined', updated_at = 2100
             WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();
    let lifecycle_error = handle_add_service_visit_part(
        &fixture.database,
        AddServiceVisitPartCommandInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 1,
            unit_price_fils: 4_500,
            created_at: 2_200,
        },
    )
    .expect_err("terminal visit should map to a lifecycle error");

    // # Assert
    assert_eq!(update_error.category, CommandErrorCategory::ValidationError);
    assert_eq!(
        void_error.category,
        CommandErrorCategory::ServiceVisitPartNotFound
    );
    assert_eq!(
        missing_visit.category,
        CommandErrorCategory::ServiceVisitNotFound
    );
    assert_eq!(
        missing_item.category,
        CommandErrorCategory::InventoryItemNotFound
    );
    assert_eq!(
        lifecycle_error.category,
        CommandErrorCategory::LifecycleRejected
    );
    assert_eq!(
        serde_json::to_value(&update_error).unwrap()["category"],
        "validationError"
    );
    assert!(!update_error.message.to_ascii_lowercase().contains("sql"));
    assert!(!void_error.message.to_ascii_lowercase().contains("select"));
}

#[test]
fn command_inputs_reject_arbitrary_database_paths_and_snapshot_fields() {
    // # Arrange
    let unsafe_add = json!({
        "serviceVisitId": 1,
        "inventoryItemId": 2,
        "quantity": 1,
        "unitPriceFils": 4500,
        "createdAt": 3,
        "databasePath": "C:/caller-controlled.sqlite3"
    });
    let forged_snapshot = json!({
        "serviceVisitId": 1,
        "inventoryItemId": 2,
        "quantity": 1,
        "unitPriceFils": 4500,
        "createdAt": 3,
        "itemName": "Forged",
        "unitName": "Forged",
        "quantityScale": 1000,
        "lineTotalFils": 1
    });

    // # Act
    let path_result = serde_json::from_value::<AddServiceVisitPartCommandInput>(unsafe_add);
    let snapshot_result =
        serde_json::from_value::<AddServiceVisitPartCommandInput>(forged_snapshot);

    // # Assert
    assert!(path_result.is_err());
    assert!(snapshot_result.is_err());
}

#[test]
fn database_failures_use_a_stable_category_without_exposing_sqlite_details() {
    // # Arrange
    let temp_dir = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(temp_dir.path()).unwrap();
    let sabotage_connection = open_database(database.database_path()).unwrap();
    sabotage_connection
        .pragma_update(None, "foreign_keys", false)
        .unwrap();
    sabotage_connection
        .execute("DROP TABLE inventory_items", [])
        .unwrap();

    // # Act
    let error = handle_list_service_visit_inventory_items(&database)
        .expect_err("broken runtime schema should return a command error");

    // # Assert
    assert_eq!(error.category, CommandErrorCategory::DatabaseError);
    assert_eq!(error.message, "The workshop database operation failed.");
    let serialized = serde_json::to_value(error).unwrap();
    assert_eq!(serialized["category"], "databaseError");
    assert!(!serialized["message"]
        .as_str()
        .unwrap()
        .to_ascii_lowercase()
        .contains("no such table"));
}

struct Fixture {
    _temp_dir: TempDir,
    database: RuntimeDatabase,
    seed_connection: Connection,
    owner_id: i64,
    motorcycle_id: i64,
    visit_id: i64,
    filter_item_id: i64,
    oil_item_id: i64,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let database = RuntimeDatabase::initialize(temp_dir.path()).unwrap();
    let seed_connection = open_database(database.database_path()).unwrap();
    seed_connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad Ali', '+962791234567', 1000, 1000)",
            [],
        )
        .unwrap();
    let owner_id = seed_connection.last_insert_rowid();
    let make_id: i64 = seed_connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = seed_connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    seed_connection
        .execute(
            "INSERT INTO jordan_plate_codes (code, active) VALUES ('29', 1)",
            [],
        )
        .unwrap();
    let plate_code_id = seed_connection.last_insert_rowid();
    seed_connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, year, plate_code_id, plate_number,
                color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'CB150R', 2022, ?3, 12345, ?4, 1000, 1000)",
            params![owner_id, make_id, plate_code_id, color_id],
        )
        .unwrap();
    let motorcycle_id = seed_connection.last_insert_rowid();
    seed_connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at, odometer_km,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 18500, 'Oil leak', 1000, 1000)",
            (motorcycle_id, owner_id),
        )
        .unwrap();
    let visit_id = seed_connection.last_insert_rowid();
    let filter_item_id = insert_item(
        &seed_connection,
        "Oil Filter",
        "FILTER",
        unit_id(&seed_connection, "Piece"),
        4_500,
    );
    let oil_item_id = insert_item(
        &seed_connection,
        "Engine Oil",
        "OIL",
        unit_id(&seed_connection, "Liter"),
        7_000,
    );
    Fixture {
        _temp_dir: temp_dir,
        database,
        seed_connection,
        owner_id,
        motorcycle_id,
        visit_id,
        filter_item_id,
        oil_item_id,
    }
}

fn unit_id(connection: &Connection, name: &str) -> i64 {
    connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name = ?1",
            [name],
            |row| row.get(0),
        )
        .unwrap()
}

fn insert_item(connection: &Connection, name: &str, sku: &str, unit_id: i64, price: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO inventory_items (
                name, sku, unit_id, default_selling_price_fils, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, 1000, 1000)",
            params![name, sku, unit_id, price],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_part(
    connection: &Connection,
    visit_id: i64,
    item_id: i64,
    values: PartValues<'_>,
) -> i64 {
    connection
        .execute(
            "INSERT INTO service_visit_parts (
                service_visit_id, inventory_item_id, item_name, unit_name,
                quantity, quantity_scale, unit_price_fils, line_total_fils, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                visit_id,
                item_id,
                values.item_name,
                values.unit_name,
                values.quantity,
                values.quantity_scale,
                values.unit_price_fils,
                values.line_total_fils,
                values.created_at,
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

struct PartValues<'value> {
    item_name: &'value str,
    unit_name: &'value str,
    quantity: i64,
    quantity_scale: i64,
    unit_price_fils: i64,
    line_total_fils: i64,
    created_at: i64,
}

impl<'value> PartValues<'value> {
    fn new(
        item_name: &'value str,
        unit_name: &'value str,
        quantity: i64,
        quantity_scale: i64,
        unit_price_fils: i64,
        line_total_fils: i64,
        created_at: i64,
    ) -> Self {
        Self {
            item_name,
            unit_name,
            quantity,
            quantity_scale,
            unit_price_fils,
            line_total_fils,
            created_at,
        }
    }
}
