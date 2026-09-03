use moto_workshop_lib::application::motorcycle_directory::{
    LoadMotorcycleDetailsInput, MotorcycleDirectoryApplicationService,
    SearchMotorcycleDirectoryInput,
};
use moto_workshop_lib::db::{migrate_database, open_database};
use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

#[test]
fn directory_search_and_details_use_real_bounded_persisted_data() {
    // # Arrange
    let mut fixture = fixture();
    let motorcycle_id = seed_motorcycle(&fixture.connection);
    let visit_id = seed_visit(&fixture.connection, motorcycle_id);
    let owner_id: i64 = fixture
        .connection
        .query_row(
            "SELECT customer_id FROM motorcycles WHERE id=?1",
            [motorcycle_id],
            |row| row.get(0),
        )
        .unwrap();
    fixture.connection.execute("INSERT INTO service_visits (motorcycle_id,owner_customer_id,status,opened_at,completed_at,closed_at,customer_complaint,work_performed,created_at,updated_at) VALUES (?1,?2,'CLOSED',1000,1010,1020,'Old repair','Completed',1000,1020)",params![motorcycle_id,owner_id]).unwrap();
    let older_visit_id = fixture.connection.last_insert_rowid();
    seed_parts(&fixture.connection, visit_id);
    let service = MotorcycleDirectoryApplicationService::new(&mut fixture.connection);

    // # Act
    let directory = service
        .search(SearchMotorcycleDirectoryInput {
            query: "Honda CB".into(),
            limit: None,
        })
        .unwrap();
    let by_owner = service
        .search(SearchMotorcycleDirectoryInput {
            query: "Ahmad".into(),
            limit: None,
        })
        .unwrap();
    let by_phone = service
        .search(SearchMotorcycleDirectoryInput {
            query: "1234567".into(),
            limit: None,
        })
        .unwrap();
    let by_plate = service
        .search(SearchMotorcycleDirectoryInput {
            query: "29-12345".into(),
            limit: None,
        })
        .unwrap();
    let by_vin = service
        .search(SearchMotorcycleDirectoryInput {
            query: "JH2RC4468".into(),
            limit: None,
        })
        .unwrap();
    let by_chassis = service
        .search(SearchMotorcycleDirectoryInput {
            query: "FRAME-11".into(),
            limit: None,
        })
        .unwrap();
    let details = service
        .load(LoadMotorcycleDetailsInput { motorcycle_id })
        .unwrap()
        .unwrap();

    // # Assert
    assert_eq!(directory.len(), 1);
    assert_eq!(by_owner.len(), 1);
    assert_eq!(by_phone.len(), 1);
    assert_eq!(by_plate.len(), 1);
    assert_eq!(by_vin.len(), 1);
    assert_eq!(by_chassis.len(), 1);
    assert_eq!(directory[0].active_service_visit_id, Some(visit_id));
    assert_eq!(details.owner_name, "Ahmad Ali");
    assert_eq!(details.service_history[0].id, visit_id);
    assert_eq!(details.service_history[1].id, older_visit_id);
    assert_eq!(details.service_history[0].total_fils, 9_500);
    assert!(service
        .load(LoadMotorcycleDetailsInput {
            motorcycle_id: 999_999
        })
        .unwrap()
        .is_none());
}

#[test]
fn directory_excludes_archived_rows_and_caps_large_requested_limits() {
    // # Arrange
    let mut fixture = fixture();
    let first = seed_motorcycle(&fixture.connection);
    let customer_id: i64 = fixture
        .connection
        .query_row(
            "SELECT customer_id FROM motorcycles WHERE id=?1",
            [first],
            |row| row.get(0),
        )
        .unwrap();
    let make_id: i64 = fixture
        .connection
        .query_row(
            "SELECT make_id FROM motorcycles WHERE id=?1",
            [first],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = fixture
        .connection
        .query_row(
            "SELECT color_id FROM motorcycles WHERE id=?1",
            [first],
            |row| row.get(0),
        )
        .unwrap();
    fixture
        .connection
        .execute("UPDATE motorcycles SET archived_at=10 WHERE id=?1", [first])
        .unwrap();
    for index in 0..105 {
        fixture.connection.execute("INSERT INTO motorcycles (customer_id,make_id,model,plate_number,color_id,created_at,updated_at) VALUES (?1,?2,'CB150R',?3,?4,?5,?5)",params![customer_id,make_id,format!("40-{index}"),color_id,index+20]).unwrap();
    }
    let service = MotorcycleDirectoryApplicationService::new(&mut fixture.connection);

    // # Act
    let rows = service
        .search(SearchMotorcycleDirectoryInput {
            query: String::new(),
            limit: Some(1_000),
        })
        .unwrap();

    // # Assert
    assert_eq!(rows.len(), 100);
    assert!(rows.iter().all(|row| row.id != first));
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("motorcycles.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

fn seed_motorcycle(connection: &Connection) -> i64 {
    connection.execute(
        "INSERT INTO customers (name, phone, created_at, updated_at) VALUES ('Ahmad Ali', '+962791234567', 1, 1)",
        [],
    ).unwrap();
    let customer_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = 'Honda'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = 'Black'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection.execute(
        "INSERT INTO motorcycles (customer_id, make_id, model, year, plate_number, vin, chassis_number, color_id, created_at, updated_at) VALUES (?1, ?2, 'CB150R', 2022, '29-12345', 'JH2RC4468MK123456', 'FRAME-11', ?3, 1, 1)",
        params![customer_id, make_id, color_id],
    ).unwrap();
    connection.last_insert_rowid()
}

fn seed_visit(connection: &Connection, motorcycle_id: i64) -> i64 {
    let customer_id: i64 = connection
        .query_row(
            "SELECT customer_id FROM motorcycles WHERE id = ?1",
            [motorcycle_id],
            |row| row.get(0),
        )
        .unwrap();
    connection.execute(
        "INSERT INTO service_visits (motorcycle_id, owner_customer_id, status, opened_at, customer_complaint, labor_charge_fils, created_at, updated_at) VALUES (?1, ?2, 'OPEN', 2000, 'Oil leak', 5000, 2000, 2000)",
        params![motorcycle_id, customer_id],
    ).unwrap();
    connection.last_insert_rowid()
}

fn seed_parts(connection: &Connection, visit_id: i64) {
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name='Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection.execute("INSERT INTO inventory_items (name,unit_id,default_selling_price_fils,minimum_stock_quantity,created_at,updated_at) VALUES ('Oil Filter',?1,4500,0,1,1)",[unit_id]).unwrap();
    let item_id = connection.last_insert_rowid();
    for price in [4_500, 8_000] {
        connection.execute("INSERT INTO service_visit_parts (service_visit_id,inventory_item_id,item_name,unit_name,quantity,quantity_scale,unit_price_fils,line_total_fils,status,created_at) VALUES (?1,?2,'Oil Filter','Piece',1,1,?3,?3,'ACTIVE',3000)",params![visit_id,item_id,price]).unwrap();
        if price == 8_000 {
            let part_id = connection.last_insert_rowid();
            connection.execute("UPDATE service_visit_parts SET status='VOIDED',voided_at=3001,void_reason='Mistake' WHERE id=?1",[part_id]).unwrap();
        }
    }
}
