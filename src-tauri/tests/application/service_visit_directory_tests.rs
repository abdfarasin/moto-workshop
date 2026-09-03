use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::service_visit_directory::{
        ListServiceVisitsInput, ServiceVisitDirectoryApplicationService,
        ServiceVisitDirectoryStatusFilter,
    },
    db::{migrate_database, open_database},
    domain::service_visit::ServiceVisitStatus,
};

#[test]
fn default_directory_returns_only_active_work_in_workflow_order_with_one_query_total() {
    // # Arrange
    let mut fixture = fixture();
    let open = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Ahmad Ali",
            "+962791234561",
            "Honda",
            "CB150R",
            "29-100",
            "OPEN",
            1_000,
        )
        .with_labor(5_000),
    );
    let ready = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Maya Saleh",
            "+962791234562",
            "Yamaha",
            "YBR125",
            "30-200",
            "READY_FOR_PICKUP",
            2_000,
        ),
    );
    seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Omar Khaled",
            "+962791234563",
            "Suzuki",
            "GSX150",
            "31-300",
            "CLOSED",
            3_000,
        ),
    );
    let item_id = insert_inventory_item(&fixture.connection);
    insert_part(&fixture.connection, open, item_id, "ACTIVE", 4_500);
    insert_part(&fixture.connection, open, item_id, "VOIDED", 8_000);

    // # Act
    let visits = ServiceVisitDirectoryApplicationService::new(&mut fixture.connection)
        .list(ListServiceVisitsInput {
            query: String::new(),
            status_filter: None,
            limit: None,
        })
        .unwrap();

    // # Assert
    assert_eq!(visits.len(), 2);
    assert_eq!(visits[0].id, open);
    assert_eq!(visits[0].status, ServiceVisitStatus::Open);
    assert_eq!(visits[0].total_fils, 9_500);
    assert_eq!(visits[1].id, ready);
    assert_eq!(visits[1].status, ServiceVisitStatus::ReadyForPickup);
}

#[test]
fn directory_searches_persisted_identity_fields_and_applies_exact_status_filters() {
    // # Arrange
    let mut fixture = fixture();
    let open = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Ahmad Ali",
            "+962791234561",
            "Honda",
            "CB150R",
            "29-100",
            "OPEN",
            1_000,
        ),
    );
    let ready = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Maya Saleh",
            "+962791234562",
            "Yamaha",
            "YBR125",
            "30-200",
            "READY_FOR_PICKUP",
            2_000,
        ),
    );
    let closed = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Omar Khaled",
            "+962791234563",
            "Suzuki",
            "GSX150",
            "31-300",
            "CLOSED",
            3_000,
        ),
    );
    let cancelled = seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Lina Naser",
            "+962791234564",
            "Kawasaki",
            "Ninja",
            "32-400",
            "CANCELLED",
            4_000,
        ),
    );
    let service = ServiceVisitDirectoryApplicationService::new(&mut fixture.connection);

    // # Act
    let by_name = list(
        &service,
        "  Ahmad  ",
        ServiceVisitDirectoryStatusFilter::All,
    );
    let by_phone = list(&service, "1234562", ServiceVisitDirectoryStatusFilter::All);
    let by_plate = list(&service, "31-300", ServiceVisitDirectoryStatusFilter::All);
    let by_motorcycle = list(
        &service,
        "suzuki gsx",
        ServiceVisitDirectoryStatusFilter::All,
    );
    let only_closed = list(&service, "", ServiceVisitDirectoryStatusFilter::Closed);
    let only_cancelled = list(&service, "", ServiceVisitDirectoryStatusFilter::Cancelled);

    // # Assert
    assert_eq!(by_name[0].id, open);
    assert_eq!(by_phone[0].id, ready);
    assert_eq!(by_plate[0].id, closed);
    assert_eq!(by_motorcycle[0].id, closed);
    assert_eq!(
        only_closed.iter().map(|visit| visit.id).collect::<Vec<_>>(),
        vec![closed]
    );
    assert_eq!(
        only_cancelled
            .iter()
            .map(|visit| visit.id)
            .collect::<Vec<_>>(),
        vec![cancelled]
    );
}

#[test]
fn directory_escapes_like_metacharacters_and_caps_requested_limits() {
    // # Arrange
    let mut fixture = fixture();
    seed_visit(
        &fixture.connection,
        VisitSeed::new(
            "Percent % Customer",
            "+962791234561",
            "Honda",
            "CB150R",
            "29-100",
            "CLOSED",
            1_000,
        ),
    );
    for index in 0..105 {
        seed_visit(
            &fixture.connection,
            VisitSeed::new(
                &format!("Customer {index}"),
                &format!("+96278{:07}", index),
                "Honda",
                "CB150R",
                &format!("40-{index}"),
                "CLOSED",
                2_000 + index,
            ),
        );
    }
    let service = ServiceVisitDirectoryApplicationService::new(&mut fixture.connection);

    // # Act
    let literal_percent = list(&service, "%", ServiceVisitDirectoryStatusFilter::All);
    let capped = service
        .list(ListServiceVisitsInput {
            query: String::new(),
            status_filter: Some(ServiceVisitDirectoryStatusFilter::All),
            limit: Some(1_000),
        })
        .unwrap();

    // # Assert
    assert_eq!(literal_percent.len(), 1);
    assert_eq!(literal_percent[0].customer_name, "Percent % Customer");
    assert_eq!(capped.len(), 100);
}

fn list(
    service: &ServiceVisitDirectoryApplicationService<'_>,
    query: &str,
    filter: ServiceVisitDirectoryStatusFilter,
) -> Vec<moto_workshop_lib::application::service_visit_directory::ServiceVisitDirectoryEntry> {
    service
        .list(ListServiceVisitsInput {
            query: query.into(),
            status_filter: Some(filter),
            limit: None,
        })
        .unwrap()
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("service-directory.db")).unwrap();
    migrate_database(&mut connection).unwrap();
    Fixture {
        _temp_dir: temp_dir,
        connection,
    }
}

struct VisitSeed<'value> {
    customer_name: &'value str,
    phone: &'value str,
    make: &'value str,
    model: &'value str,
    plate: &'value str,
    status: &'value str,
    opened_at: i64,
    labor: i64,
}

impl<'value> VisitSeed<'value> {
    fn new(
        customer_name: &'value str,
        phone: &'value str,
        make: &'value str,
        model: &'value str,
        plate: &'value str,
        status: &'value str,
        opened_at: i64,
    ) -> Self {
        Self {
            customer_name,
            phone,
            make,
            model,
            plate,
            status,
            opened_at,
            labor: 0,
        }
    }

    fn with_labor(mut self, labor: i64) -> Self {
        self.labor = labor;
        self
    }
}

fn seed_visit(connection: &Connection, seed: VisitSeed<'_>) -> i64 {
    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
            params![seed.customer_name, seed.phone, seed.opened_at],
        )
        .unwrap();
    let customer_id = connection.last_insert_rowid();
    let make_id: i64 = connection
        .query_row(
            "SELECT id FROM motorcycle_makes WHERE name = ?1",
            [seed.make],
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
    connection
        .execute(
            "INSERT INTO motorcycles (
            customer_id, make_id, model, plate_number, color_id, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                customer_id,
                make_id,
                seed.model,
                seed.plate,
                color_id,
                seed.opened_at
            ],
        )
        .unwrap();
    let motorcycle_id = connection.last_insert_rowid();
    let completed_at =
        matches!(seed.status, "READY_FOR_PICKUP" | "CLOSED").then_some(seed.opened_at + 10);
    let closed_at = (seed.status == "CLOSED").then_some(seed.opened_at + 20);
    let cancelled_at = (seed.status == "CANCELLED").then_some(seed.opened_at + 10);
    let work_performed =
        matches!(seed.status, "READY_FOR_PICKUP" | "CLOSED").then_some("Completed work");
    let cancellation_reason = (seed.status == "CANCELLED").then_some("Customer request");
    connection
        .execute(
            "INSERT INTO service_visits (
            motorcycle_id, owner_customer_id, status, opened_at, completed_at,
            closed_at, cancelled_at, customer_complaint, work_performed,
            labor_charge_fils, cancellation_reason, created_at, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'Engine noise', ?8, ?9, ?10, ?4, ?4)",
            params![
                motorcycle_id,
                customer_id,
                seed.status,
                seed.opened_at,
                completed_at,
                closed_at,
                cancelled_at,
                work_performed,
                seed.labor,
                cancellation_reason,
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_inventory_item(connection: &Connection) -> i64 {
    let unit_id: i64 = connection
        .query_row(
            "SELECT id FROM inventory_units WHERE name = 'Piece'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    connection
        .execute(
            "INSERT INTO inventory_items (
            name, unit_id, default_selling_price_fils, minimum_stock_quantity,
            created_at, updated_at
         ) VALUES ('Oil Filter', ?1, 4500, 0, 1, 1)",
            [unit_id],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_part(connection: &Connection, visit_id: i64, item_id: i64, status: &str, total: i64) {
    connection
        .execute(
            "INSERT INTO service_visit_parts (
            service_visit_id, inventory_item_id, item_name, unit_name, quantity,
            quantity_scale, unit_price_fils, line_total_fils, status, created_at
         ) VALUES (?1, ?2, 'Oil Filter', 'Piece', 1, 1, ?3, ?3, 'ACTIVE', 5000)",
            params![visit_id, item_id, total],
        )
        .unwrap();
    if status == "VOIDED" {
        let part_id = connection.last_insert_rowid();
        connection
            .execute(
                "UPDATE service_visit_parts
             SET status = 'VOIDED', voided_at = 5001, void_reason = 'Mistake'
             WHERE id = ?1",
                [part_id],
            )
            .unwrap();
    }
}
