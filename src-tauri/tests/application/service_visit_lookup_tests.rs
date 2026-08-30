use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::service_visit_lookup::{
        ActiveServiceVisitStatus, SearchCustomersInput, ServiceVisitLookupError,
        ServiceVisitLookupService, DEFAULT_CUSTOMER_SEARCH_LIMIT, MAX_CUSTOMER_SEARCH_LIMIT,
    },
    db::{migrate_database, open_database},
};

#[test]
fn customer_search_is_trimmed_literal_parameterized_and_matches_name_or_phone() {
    // # Arrange
    let fixture = fixture();
    let ahmad_id = insert_customer(
        &fixture.connection,
        "Ahmad 100% Workshop",
        "+962791234567",
        1_100,
    );
    insert_customer(&fixture.connection, "Maya_Saleh", "+962791234568", 1_200);
    insert_customer(
        &fixture.connection,
        "Archived Ahmad",
        "+962791234569",
        1_300,
    );
    fixture
        .connection
        .execute(
            "UPDATE customers SET archived_at = 1400 WHERE phone = '+962791234569'",
            [],
        )
        .unwrap();

    // # Act
    let by_name = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "  aHmAd  ".into(),
            limit: Some(25),
        })
        .unwrap();
    let by_phone = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "791234567".into(),
            limit: Some(25),
        })
        .unwrap();
    let literal_percent = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "%".into(),
            limit: Some(25),
        })
        .unwrap();
    let literal_underscore = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "_".into(),
            limit: Some(25),
        })
        .unwrap();
    let injection = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "%' OR 1=1 --".into(),
            limit: Some(25),
        })
        .unwrap();

    // # Assert
    assert_eq!(by_name.len(), 1);
    assert_eq!(by_name[0].id, ahmad_id);
    assert_eq!(by_name[0].name, "Ahmad 100% Workshop");
    assert_eq!(by_name[0].phone, "+962791234567");
    assert_eq!(by_phone, by_name);
    assert_eq!(literal_percent, by_name);
    assert_eq!(literal_underscore.len(), 1);
    assert_eq!(literal_underscore[0].name, "Maya_Saleh");
    assert!(injection.is_empty());
}

#[test]
fn customer_search_uses_recent_deterministic_order_default_and_hard_limits() {
    // # Arrange
    let fixture = fixture();
    for index in 0..110 {
        insert_customer(
            &fixture.connection,
            &format!("Customer {index:03}"),
            &format!("+9627{index:08}"),
            2_000 + index,
        );
    }

    // # Act
    let default_results = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: "   ".into(),
            limit: None,
        })
        .unwrap();
    let requested_three = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: String::new(),
            limit: Some(3),
        })
        .unwrap();
    let requested_huge = ServiceVisitLookupService::new(&fixture.connection)
        .search_customers(SearchCustomersInput {
            query: String::new(),
            limit: Some(10_000),
        })
        .unwrap();

    // # Assert
    assert_eq!(
        default_results.len(),
        DEFAULT_CUSTOMER_SEARCH_LIMIT as usize
    );
    assert_eq!(requested_three.len(), 3);
    assert_eq!(requested_huge.len(), MAX_CUSTOMER_SEARCH_LIMIT as usize);
    assert_eq!(requested_three[0].name, "Customer 109");
    assert_eq!(requested_three[1].name, "Customer 108");
    assert_eq!(requested_three[2].name, "Customer 107");
}

#[test]
fn motorcycle_lookup_distinguishes_missing_customer_from_existing_empty_customer() {
    // # Arrange
    let fixture = fixture();
    let empty_customer_id = insert_customer(
        &fixture.connection,
        "No Motorcycles",
        "+962791111111",
        1_000,
    );

    // # Act
    let empty = ServiceVisitLookupService::new(&fixture.connection)
        .list_customer_motorcycles(empty_customer_id)
        .unwrap();
    let missing = ServiceVisitLookupService::new(&fixture.connection)
        .list_customer_motorcycles(999_999)
        .expect_err("a missing Customer must be distinguished from an empty result");

    // # Assert
    assert!(empty.is_empty());
    assert!(matches!(
        missing,
        ServiceVisitLookupError::CustomerNotFound(999_999)
    ));
}

#[test]
fn motorcycle_lookup_joins_presentation_and_only_open_or_ready_active_visits_in_one_result() {
    // # Arrange
    let fixture = fixture();
    let owner_id = insert_customer(
        &fixture.connection,
        "Motorcycle Owner",
        "+962792222222",
        1_000,
    );
    let other_owner_id =
        insert_customer(&fixture.connection, "Other Owner", "+962793333333", 1_000);
    let open_motorcycle = insert_motorcycle(
        &fixture.connection,
        owner_id,
        MotorcycleValues::new(
            "Yamaha",
            "YBR125",
            Some(2020),
            Some("29"),
            Some(12345),
            Some("LOOKUP-OPEN"),
            "Red",
        ),
    );
    let ready_motorcycle = insert_motorcycle(
        &fixture.connection,
        owner_id,
        MotorcycleValues::new(
            "Honda",
            "CB150R",
            Some(2022),
            None,
            None,
            Some("LOOKUP-READY"),
            "Black",
        ),
    );
    let closed_motorcycle = insert_motorcycle(
        &fixture.connection,
        owner_id,
        MotorcycleValues::new(
            "Honda",
            "CB500",
            None,
            None,
            None,
            Some("LOOKUP-CLOSED"),
            "Black",
        ),
    );
    let cancelled_motorcycle = insert_motorcycle(
        &fixture.connection,
        owner_id,
        MotorcycleValues::new(
            "Honda",
            "Wave",
            Some(2019),
            None,
            None,
            Some("LOOKUP-CANCELLED"),
            "Red",
        ),
    );
    let other_motorcycle = insert_motorcycle(
        &fixture.connection,
        other_owner_id,
        MotorcycleValues::new(
            "Honda",
            "Other",
            None,
            None,
            None,
            Some("LOOKUP-OTHER"),
            "Black",
        ),
    );
    let archived_motorcycle = insert_motorcycle(
        &fixture.connection,
        owner_id,
        MotorcycleValues::new(
            "Honda",
            "Archived",
            None,
            None,
            None,
            Some("LOOKUP-ARCHIVED"),
            "Black",
        ),
    );
    fixture
        .connection
        .execute(
            "UPDATE motorcycles SET archived_at = 1300 WHERE id = ?1",
            [archived_motorcycle],
        )
        .unwrap();
    let open_visit = insert_visit(&fixture.connection, open_motorcycle, owner_id, "OPEN");
    let ready_visit = insert_visit(
        &fixture.connection,
        ready_motorcycle,
        owner_id,
        "READY_FOR_PICKUP",
    );
    insert_visit(&fixture.connection, closed_motorcycle, owner_id, "CLOSED");
    insert_visit(
        &fixture.connection,
        cancelled_motorcycle,
        owner_id,
        "CANCELLED",
    );

    // # Act
    let motorcycles = ServiceVisitLookupService::new(&fixture.connection)
        .list_customer_motorcycles(owner_id)
        .unwrap();

    // # Assert
    assert_eq!(motorcycles.len(), 4);
    assert_eq!(
        motorcycles.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![
            ready_motorcycle,
            closed_motorcycle,
            cancelled_motorcycle,
            open_motorcycle
        ]
    );
    let open = motorcycles
        .iter()
        .find(|item| item.id == open_motorcycle)
        .unwrap();
    assert_eq!(open.make_name, "Yamaha");
    assert_eq!(open.model, "YBR125");
    assert_eq!(open.year, Some(2020));
    assert_eq!(open.color_name, "Red");
    assert_eq!(open.plate_code.as_deref(), Some("29"));
    assert_eq!(open.plate_number, Some(12345));
    assert_eq!(open.vin, None);
    assert_eq!(open.chassis_number.as_deref(), Some("LOOKUP-OPEN"));
    assert_eq!(open.active_service_visit_id, Some(open_visit));
    assert_eq!(
        open.active_service_visit_status,
        Some(ActiveServiceVisitStatus::Open)
    );
    let ready = motorcycles
        .iter()
        .find(|item| item.id == ready_motorcycle)
        .unwrap();
    assert_eq!(ready.active_service_visit_id, Some(ready_visit));
    assert_eq!(
        ready.active_service_visit_status,
        Some(ActiveServiceVisitStatus::ReadyForPickup)
    );
    for inactive_id in [closed_motorcycle, cancelled_motorcycle] {
        let inactive = motorcycles
            .iter()
            .find(|item| item.id == inactive_id)
            .unwrap();
        assert_eq!(inactive.active_service_visit_id, None);
        assert_eq!(inactive.active_service_visit_status, None);
    }
    assert!(motorcycles.iter().all(|item| item.id != other_motorcycle));
    assert!(motorcycles
        .iter()
        .all(|item| item.id != archived_motorcycle));
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("lookup-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

fn insert_customer(connection: &Connection, name: &str, phone: &str, updated_at: i64) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?3)",
            params![name, phone, updated_at],
        )
        .unwrap();
    connection.last_insert_rowid()
}

struct MotorcycleValues<'value> {
    make_name: &'value str,
    model: &'value str,
    year: Option<i64>,
    plate_code: Option<&'value str>,
    plate_number: Option<i64>,
    chassis_number: Option<&'value str>,
    color_name: &'value str,
}

impl<'value> MotorcycleValues<'value> {
    fn new(
        make_name: &'value str,
        model: &'value str,
        year: Option<i64>,
        plate_code: Option<&'value str>,
        plate_number: Option<i64>,
        chassis_number: Option<&'value str>,
        color_name: &'value str,
    ) -> Self {
        Self {
            make_name,
            model,
            year,
            plate_code,
            plate_number,
            chassis_number,
            color_name,
        }
    }
}

fn insert_motorcycle(connection: &Connection, owner_id: i64, values: MotorcycleValues<'_>) -> i64 {
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = ?1",
            [values.make_name],
            |row| row.get(0),
        )
        .unwrap();
    let color_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_colors WHERE name = ?1",
            [values.color_name],
            |row| row.get(0),
        )
        .unwrap();
    let plate_code_id = values.plate_code.map(|code| {
        connection
            .execute(
                "INSERT OR IGNORE INTO jordan_plate_codes (code, active) VALUES (?1, 1)",
                [code],
            )
            .unwrap();
        connection
            .query_row(
                "SELECT id FROM jordan_plate_codes WHERE code = ?1",
                [code],
                |row| row.get::<_, i64>(0),
            )
            .unwrap()
    });
    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, year, plate_code_id, plate_number,
                chassis_number, color_id, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1000, 1000)",
            params![
                owner_id,
                make_id,
                values.model,
                values.year,
                plate_code_id,
                values.plate_number,
                values.chassis_number,
                color_id,
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_visit(connection: &Connection, motorcycle_id: i64, owner_id: i64, status: &str) -> i64 {
    connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 'Lookup visit', 1000, 1000)",
            (motorcycle_id, owner_id),
        )
        .unwrap();
    let visit_id = connection.last_insert_rowid();
    match status {
        "OPEN" => {}
        "READY_FOR_PICKUP" => {
            connection
                .execute(
                    "UPDATE service_visits
                 SET work_performed = 'Done', status = 'READY_FOR_PICKUP',
                     completed_at = 1100, updated_at = 1100
                 WHERE id = ?1",
                    [visit_id],
                )
                .unwrap();
        }
        "CLOSED" => {
            connection
                .execute(
                    "UPDATE service_visits
                 SET work_performed = 'Done', status = 'READY_FOR_PICKUP',
                     completed_at = 1100, updated_at = 1100
                 WHERE id = ?1",
                    [visit_id],
                )
                .unwrap();
            connection
                .execute(
                    "UPDATE service_visits
                 SET status = 'CLOSED', closed_at = 1200, updated_at = 1200
                 WHERE id = ?1",
                    [visit_id],
                )
                .unwrap();
        }
        "CANCELLED" => {
            connection
                .execute(
                    "UPDATE service_visits
                 SET status = 'CANCELLED', cancelled_at = 1100,
                     cancellation_reason = 'Cancelled', updated_at = 1100
                 WHERE id = ?1",
                    [visit_id],
                )
                .unwrap();
        }
        _ => panic!("unsupported test status"),
    }
    visit_id
}
