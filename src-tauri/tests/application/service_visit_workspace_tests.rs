use rusqlite::{params, Connection};
use tempfile::{tempdir, TempDir};

use moto_workshop_lib::{
    application::service_visit_workspace::{
        AddServiceVisitPartInput, CancelServiceVisitInput, CloseServiceVisitInput,
        MarkServiceVisitReadyForPickupInput, ReopenServiceVisitInput, ServiceVisitWorkspaceError,
        ServiceVisitWorkspaceService, UpdateServiceVisitWorkInput, VoidServiceVisitPartInput,
    },
    db::{migrate_database, open_database},
    domain::{
        service_visit::{ServiceVisitStatus, ServiceVisitValidationError},
        service_visit_part::{ServiceVisitPartStatus, ServiceVisitPartValidationError},
    },
};

#[test]
fn loads_complete_workspace_with_active_and_voided_part_history() {
    // # Arrange
    let mut fixture = fixture();
    let active_part_id = insert_part(
        &fixture.connection,
        fixture.visit_id,
        fixture.filter_item_id,
        "Oil Filter",
        "Piece",
        PartValues::new(2, 1, 4_500, 9_000, 2_000),
    );
    let voided_part_id = insert_part(
        &fixture.connection,
        fixture.visit_id,
        fixture.oil_item_id,
        "Engine Oil",
        "Liter",
        PartValues::new(2_500, 1_000, 7_000, 17_500, 2_100),
    );
    fixture
        .connection
        .execute(
            "UPDATE service_visit_parts
             SET status = 'VOIDED', voided_at = 2200, void_reason = 'Wrong oil'
             WHERE id = ?1",
            [voided_part_id],
        )
        .unwrap();

    // # Act
    let workspace = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .load_workspace(fixture.visit_id)
        .expect("workspace should load");

    // # Assert
    assert_eq!(workspace.visit.id, fixture.visit_id);
    assert_eq!(workspace.visit.status, ServiceVisitStatus::Open);
    assert_eq!(workspace.visit.customer_complaint, "Oil leak");
    assert_eq!(workspace.visit.odometer_km, Some(18_500));
    assert_eq!(workspace.owner.id, fixture.owner_id);
    assert_eq!(workspace.owner.name, "Ahmad Ali");
    assert_eq!(workspace.owner.phone, "+962791234567");
    assert_eq!(workspace.motorcycle.id, fixture.motorcycle_id);
    assert_eq!(workspace.motorcycle.make_name, "Honda");
    assert_eq!(workspace.motorcycle.model, "CB150R");
    assert_eq!(workspace.motorcycle.year, Some(2022));
    assert_eq!(
        workspace.motorcycle.plate_number.as_deref(),
        Some("29-12345")
    );
    assert_eq!(workspace.motorcycle.color_name, "Black");
    assert_eq!(workspace.parts.len(), 2);
    assert_eq!(workspace.parts[0].id, active_part_id);
    assert_eq!(workspace.parts[0].status, ServiceVisitPartStatus::Active);
    assert_eq!(workspace.parts[0].line_total_fils, 9_000);
    assert_eq!(workspace.parts[1].id, voided_part_id);
    assert_eq!(workspace.parts[1].status, ServiceVisitPartStatus::Voided);
    assert_eq!(workspace.parts[1].voided_at, Some(2_200));
    assert_eq!(workspace.parts[1].void_reason.as_deref(), Some("Wrong oil"));
}

#[test]
fn lifecycle_ready_reopen_and_close_persist_refreshed_workspace_and_updated_at() {
    // # Arrange
    let mut fixture = fixture();
    let visit_id = fixture.visit_id;
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: visit_id,
            diagnosis: Some("Worn seal".into()),
            work_performed: Some("Replaced seal".into()),
            labor_charge_fils: 12_500,
            notes: None,
            odometer_km: Some(18_510),
            updated_at: 1_500,
        })
        .unwrap();

    // # Act
    let ready = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .mark_ready_for_pickup(MarkServiceVisitReadyForPickupInput {
            service_visit_id: visit_id,
            completed_at: 2_000,
            updated_at: 2_010,
        })
        .expect("open visit with work should become ready");
    let reopened = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .reopen(ReopenServiceVisitInput {
            service_visit_id: visit_id,
            updated_at: 2_020,
        })
        .expect("ready visit should reopen");
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .mark_ready_for_pickup(MarkServiceVisitReadyForPickupInput {
            service_visit_id: visit_id,
            completed_at: 2_030,
            updated_at: 2_040,
        })
        .unwrap();
    let closed = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .close(CloseServiceVisitInput {
            service_visit_id: visit_id,
            closed_at: 2_050,
            updated_at: 2_060,
        })
        .expect("ready visit should close");

    // # Assert
    assert_eq!(ready.visit.status, ServiceVisitStatus::ReadyForPickup);
    assert_eq!(ready.visit.completed_at, Some(2_000));
    assert_eq!(ready.visit.updated_at, 2_010);
    assert_eq!(reopened.visit.status, ServiceVisitStatus::Open);
    assert_eq!(reopened.visit.completed_at, None);
    assert_eq!(reopened.visit.updated_at, 2_020);
    assert_eq!(closed.visit.status, ServiceVisitStatus::Closed);
    assert_eq!(closed.visit.completed_at, Some(2_030));
    assert_eq!(closed.visit.closed_at, Some(2_050));
    assert_eq!(closed.visit.updated_at, 2_060);
    let persisted = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .load_workspace(visit_id)
        .unwrap();
    assert_eq!(persisted, closed);
}

#[test]
fn lifecycle_validation_and_invalid_transitions_roll_back_authoritative_row() {
    // # Arrange
    let mut fixture = fixture();
    let visit_id = fixture.visit_id;

    // # Act
    let missing_work = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .mark_ready_for_pickup(MarkServiceVisitReadyForPickupInput {
            service_visit_id: visit_id,
            completed_at: 2_000,
            updated_at: 2_010,
        })
        .expect_err("ready requires work performed");
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: visit_id,
            diagnosis: None,
            work_performed: Some("Replaced seal".into()),
            labor_charge_fils: 12_500,
            notes: None,
            odometer_km: Some(18_510),
            updated_at: 2_020,
        })
        .unwrap();
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .mark_ready_for_pickup(MarkServiceVisitReadyForPickupInput {
            service_visit_id: visit_id,
            completed_at: 2_100,
            updated_at: 2_110,
        })
        .unwrap();
    let early_close = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .close(CloseServiceVisitInput {
            service_visit_id: visit_id,
            closed_at: 2_099,
            updated_at: 2_120,
        })
        .expect_err("close cannot precede completion");
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .reopen(ReopenServiceVisitInput {
            service_visit_id: visit_id,
            updated_at: 2_130,
        })
        .unwrap();
    let invalid_close = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .close(CloseServiceVisitInput {
            service_visit_id: visit_id,
            closed_at: 2_140,
            updated_at: 2_150,
        })
        .expect_err("open visit cannot close directly");
    let blank_cancel = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .cancel(CancelServiceVisitInput {
            service_visit_id: visit_id,
            cancelled_at: 2_140,
            reason: "   ".into(),
            updated_at: 2_150,
        })
        .expect_err("blank cancellation reason must be rejected");

    // # Assert
    assert!(matches!(
        missing_work,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::MissingWorkPerformed
        )
    ));
    assert!(matches!(
        early_close,
        ServiceVisitWorkspaceError::VisitValidation(ServiceVisitValidationError::InvalidTimestamp)
    ));
    assert!(matches!(
        invalid_close,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::InvalidTransition {
                from: ServiceVisitStatus::Open,
                to: ServiceVisitStatus::Closed,
            }
        )
    ));
    assert!(matches!(
        blank_cancel,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::BlankCancellationReason
        )
    ));
    let persisted: (String, Option<i64>, Option<i64>, Option<i64>, i64) = fixture
        .connection
        .query_row(
            "SELECT status, completed_at, closed_at, cancelled_at, updated_at
             FROM service_visits WHERE id = ?1",
            [visit_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .unwrap();
    assert_eq!(persisted, ("OPEN".into(), None, None, None, 2_130));
}

#[test]
fn cancel_normalizes_reason_persists_timestamp_and_remains_terminal() {
    // # Arrange
    let mut fixture = fixture();
    let visit_id = fixture.visit_id;

    // # Act
    let cancelled = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .cancel(CancelServiceVisitInput {
            service_visit_id: visit_id,
            cancelled_at: 2_000,
            reason: "  Customer declined repair  ".into(),
            updated_at: 2_010,
        })
        .expect("open visit should cancel");
    let terminal_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .reopen(ReopenServiceVisitInput {
            service_visit_id: visit_id,
            updated_at: 2_020,
        })
        .expect_err("cancelled visit must remain terminal");

    // # Assert
    assert_eq!(cancelled.visit.status, ServiceVisitStatus::Cancelled);
    assert_eq!(cancelled.visit.cancelled_at, Some(2_000));
    assert_eq!(
        cancelled.visit.cancellation_reason.as_deref(),
        Some("Customer declined repair")
    );
    assert_eq!(cancelled.visit.updated_at, 2_010);
    assert!(matches!(
        terminal_error,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::InvalidTransition {
                from: ServiceVisitStatus::Cancelled,
                to: ServiceVisitStatus::Open,
            }
        )
    ));
    let persisted = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .load_workspace(visit_id)
        .unwrap();
    assert_eq!(persisted, cancelled);
}

#[test]
fn updates_open_and_ready_work_fields_but_rejects_terminal_visits() {
    // # Arrange
    let mut fixture = fixture();
    let visit_id = fixture.visit_id;

    // # Act
    let updated = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: visit_id,
            diagnosis: Some("  Worn seal  ".into()),
            work_performed: Some("  Replaced seal  ".into()),
            labor_charge_fils: 12_500,
            notes: Some("  Test ride complete  ".into()),
            odometer_km: Some(18_510),
            updated_at: 2_000,
        })
        .expect("open visit should update");

    // # Assert
    assert_eq!(updated.visit.diagnosis.as_deref(), Some("Worn seal"));
    assert_eq!(
        updated.visit.work_performed.as_deref(),
        Some("Replaced seal")
    );
    assert_eq!(updated.visit.labor_charge_fils, 12_500);
    assert_eq!(updated.visit.notes.as_deref(), Some("Test ride complete"));
    assert_eq!(updated.visit.updated_at, 2_000);

    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'READY_FOR_PICKUP', completed_at = 2100, updated_at = 2100
             WHERE id = ?1",
            [visit_id],
        )
        .unwrap();
    let ready = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: visit_id,
            diagnosis: Some("Seal failure".into()),
            work_performed: Some("Replaced seal and tested".into()),
            labor_charge_fils: 13_000,
            notes: None,
            odometer_km: Some(18_511),
            updated_at: 2_200,
        })
        .expect("ready visit should remain editable");
    assert_eq!(ready.visit.status, ServiceVisitStatus::ReadyForPickup);
    assert_eq!(ready.visit.labor_charge_fils, 13_000);

    fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CLOSED', closed_at = 2300 WHERE id = ?1",
            [visit_id],
        )
        .unwrap();
    let error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: visit_id,
            diagnosis: None,
            work_performed: None,
            labor_charge_fils: 0,
            notes: None,
            odometer_km: None,
            updated_at: 2_400,
        })
        .expect_err("closed visit must reject work edits");
    assert!(matches!(
        error,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::TerminalVisitCannotBeEdited
        )
    ));
    let persisted_updated_at: i64 = fixture
        .connection
        .query_row(
            "SELECT updated_at FROM service_visits WHERE id = ?1",
            [visit_id],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(persisted_updated_at, 2_200);
}

#[test]
fn lists_only_usable_inventory_with_current_unit_information() {
    // # Arrange
    let mut fixture = fixture();
    let piece_unit_id = unit_id(&fixture.connection, "Piece");
    let inactive_unit_id = insert_unit(&fixture.connection, "Can", 1, false);
    let archived_item_id = insert_item(
        &fixture.connection,
        "Archived Plug",
        Some("OLD-PLUG"),
        piece_unit_id,
        3_000,
        Some(3_000),
    );
    let inactive_unit_item_id = insert_item(
        &fixture.connection,
        "Paint Can",
        Some("PAINT"),
        inactive_unit_id,
        8_000,
        None,
    );

    // # Act
    let choices = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .list_usable_inventory_items()
        .expect("inventory choices should load");

    // # Assert
    assert_eq!(choices.len(), 2);
    assert_eq!(choices[0].item_name, "Engine Oil");
    assert_eq!(choices[0].unit_name, "Liter");
    assert_eq!(choices[0].quantity_scale, 1_000);
    assert_eq!(choices[0].default_selling_price_fils, 7_000);
    assert_eq!(choices[0].current_quantity, 0);
    assert_eq!(choices[1].item_name, "Oil Filter");
    assert_eq!(choices[1].sku.as_deref(), Some("FILTER"));
    assert_eq!(choices[1].unit_name, "Piece");
    assert_eq!(choices[1].current_quantity, 0);
    assert!(!choices.iter().any(|item| item.id == archived_item_id));
    assert!(!choices.iter().any(|item| item.id == inactive_unit_item_id));
}

#[test]
fn inventory_selection_derives_isolated_scaled_stock_with_usage_and_reversal() {
    // # Arrange
    let mut fixture = fixture();
    let liter_unit_id = unit_id(&fixture.connection, "Liter");
    let piece_unit_id = unit_id(&fixture.connection, "Piece");
    let negative_item_id = insert_item(
        &fixture.connection,
        "Negative Oil",
        Some("NEG-OIL"),
        liter_unit_id,
        7_000,
        None,
    );
    let zero_history_item_id = insert_item(
        &fixture.connection,
        "Zero History",
        Some("ZERO"),
        piece_unit_id,
        1_000,
        None,
    );
    for (item_id, movement_type, delta, created_at) in [
        (fixture.filter_item_id, "OPENING_STOCK", 20, 1_100),
        (fixture.filter_item_id, "ADJUSTMENT_OUT", -3, 1_200),
        (fixture.oil_item_id, "OPENING_STOCK", 10_000, 1_300),
        (negative_item_id, "OPENING_STOCK", 1_000, 1_400),
    ] {
        fixture
            .connection
            .execute(
                "INSERT INTO stock_movements (
                    inventory_item_id, movement_type, quantity_delta, created_at
                 ) VALUES (?1, ?2, ?3, ?4)",
                params![item_id, movement_type, delta, created_at],
            )
            .unwrap();
    }
    let oil_part = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.oil_item_id,
            quantity: 2_500,
            unit_price_fils: 7_000,
            created_at: 2_000,
        })
        .unwrap();
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: negative_item_id,
            quantity: 2_500,
            unit_price_fils: 7_000,
            created_at: 2_100,
        })
        .unwrap();

    // # Act
    let before_reversal = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .list_usable_inventory_items()
        .unwrap();
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: oil_part.id,
            voided_at: 2_200,
            reason: None,
        })
        .unwrap();
    let after_reversal = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .list_usable_inventory_items()
        .unwrap();

    // # Assert
    assert_eq!(
        current_quantity(&before_reversal, fixture.filter_item_id),
        17
    );
    assert_eq!(
        current_quantity(&before_reversal, fixture.oil_item_id),
        7_500
    );
    assert_eq!(current_quantity(&before_reversal, negative_item_id), -1_500);
    assert_eq!(current_quantity(&before_reversal, zero_history_item_id), 0);
    assert_eq!(
        current_quantity(&after_reversal, fixture.oil_item_id),
        10_000
    );
    assert_eq!(
        current_quantity(&after_reversal, fixture.filter_item_id),
        17
    );
    assert_eq!(current_quantity(&after_reversal, negative_item_id), -1_500);
}

#[test]
fn adds_part_from_authoritative_catalog_data_and_calculates_total_in_rust() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let part = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.oil_item_id,
            quantity: 333,
            unit_price_fils: 5_500,
            created_at: 2_000,
        })
        .expect("part should be added");

    // # Assert
    assert_eq!(part.item_name, "Engine Oil");
    assert_eq!(part.unit_name, "Liter");
    assert_eq!(part.quantity_scale, 1_000);
    assert_eq!(part.unit_price_fils, 5_500);
    assert_eq!(part.line_total_fils, 1_832);
    assert_eq!(part.status, ServiceVisitPartStatus::Active);
    let movement: (String, i64, i64, Option<String>) = fixture
        .connection
        .query_row(
            "SELECT movement_type, quantity_delta, created_at, notes
             FROM stock_movements WHERE service_visit_part_id = ?1",
            [part.id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(movement, ("SERVICE_USAGE".into(), -333, 2_000, None));

    fixture
        .connection
        .execute(
            "UPDATE inventory_items SET archived_at = 3000 WHERE id = ?1",
            [fixture.filter_item_id],
        )
        .unwrap();
    let error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 1,
            unit_price_fils: 4_500,
            created_at: 3_100,
        })
        .expect_err("archived item should not be selectable");
    assert!(matches!(
        error,
        ServiceVisitWorkspaceError::InventoryItemNotFound(id)
            if id == fixture.filter_item_id
    ));
}

#[test]
fn voids_active_part_and_returns_truthful_history_with_one_reversal() {
    // # Arrange
    let mut fixture = fixture();
    let part = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 2,
            unit_price_fils: 4_500,
            created_at: 2_000,
        })
        .unwrap();

    // # Act
    let voided = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: part.id,
            voided_at: 2_500,
            reason: Some("  Wrong quantity  ".into()),
        })
        .expect("active part should void");

    // # Assert
    assert_eq!(voided.status, ServiceVisitPartStatus::Voided);
    assert_eq!(voided.voided_at, Some(2_500));
    assert_eq!(voided.void_reason.as_deref(), Some("Wrong quantity"));
    let movements: Vec<(String, i64)> = {
        let mut statement = fixture
            .connection
            .prepare(
                "SELECT movement_type, quantity_delta FROM stock_movements
                 WHERE service_visit_part_id = ?1 ORDER BY id",
            )
            .unwrap();
        statement
            .query_map([part.id], |row| Ok((row.get(0)?, row.get(1)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap()
    };
    assert_eq!(
        movements,
        vec![
            ("SERVICE_USAGE".into(), -2),
            ("SERVICE_USAGE_REVERSAL".into(), 2)
        ]
    );
    let workspace = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .load_workspace(fixture.visit_id)
        .unwrap();
    assert_eq!(workspace.parts, vec![voided.clone()]);

    let second_void = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: part.id,
            voided_at: 2_600,
            reason: None,
        })
        .expect_err("voided part is terminal");
    assert!(matches!(
        second_void,
        ServiceVisitWorkspaceError::PartValidation(
            ServiceVisitPartValidationError::PartAlreadyVoided
        )
    ));
}

#[test]
fn ready_visit_allows_part_changes_while_closed_visit_rejects_them_atomically() {
    // # Arrange
    let mut fixture = fixture();
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'READY_FOR_PICKUP', completed_at = 2000,
                 work_performed = 'Inspection complete', updated_at = 2000
             WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();

    // # Act
    let ready_part = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 1,
            unit_price_fils: 4_500,
            created_at: 2_100,
        })
        .expect("ready visit should accept a part");
    ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: ready_part.id,
            voided_at: 2_200,
            reason: None,
        })
        .expect("ready visit should permit voiding");
    let active_part = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 2,
            unit_price_fils: 4_500,
            created_at: 2_300,
        })
        .unwrap();
    fixture
        .connection
        .execute(
            "UPDATE service_visits SET status = 'CLOSED', closed_at = 2400 WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();

    // # Assert
    let add_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.oil_item_id,
            quantity: 1_000,
            unit_price_fils: 7_000,
            created_at: 2_500,
        })
        .expect_err("closed visit must reject new parts");
    assert!(matches!(
        add_error,
        ServiceVisitWorkspaceError::VisitDoesNotAllowPartChanges(ServiceVisitStatus::Closed)
    ));
    let void_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: active_part.id,
            voided_at: 2_500,
            reason: None,
        })
        .expect_err("closed visit must reject voiding");
    assert!(matches!(
        void_error,
        ServiceVisitWorkspaceError::VisitDoesNotAllowPartChanges(ServiceVisitStatus::Closed)
    ));
    let active_snapshot: (String, i64) = fixture
        .connection
        .query_row(
            "SELECT p.status,
                    (SELECT COUNT(*) FROM stock_movements m
                     WHERE m.service_visit_part_id = p.id)
             FROM service_visit_parts p WHERE p.id = ?1",
            [active_part.id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(active_snapshot, ("ACTIVE".into(), 1));
}

#[test]
fn cancelled_visit_rejects_part_add_without_persisting_history() {
    // # Arrange
    let mut fixture = fixture();
    fixture
        .connection
        .execute(
            "UPDATE service_visits
             SET status = 'CANCELLED', cancelled_at = 2000,
                 cancellation_reason = 'Customer declined', updated_at = 2000
             WHERE id = ?1",
            [fixture.visit_id],
        )
        .unwrap();

    // # Act
    let error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 1,
            unit_price_fils: 4_500,
            created_at: 2_100,
        })
        .expect_err("cancelled visit must reject parts");

    // # Assert
    assert!(matches!(
        error,
        ServiceVisitWorkspaceError::VisitDoesNotAllowPartChanges(ServiceVisitStatus::Cancelled)
    ));
    let part_count: i64 = fixture
        .connection
        .query_row("SELECT COUNT(*) FROM service_visit_parts", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(part_count, 0);
}

#[test]
fn validation_and_not_found_failures_leave_workspace_history_unchanged() {
    // # Arrange
    let mut fixture = fixture();

    // # Act
    let update_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .update_work(UpdateServiceVisitWorkInput {
            service_visit_id: fixture.visit_id,
            diagnosis: Some("Invalid update".into()),
            work_performed: None,
            labor_charge_fils: -1,
            notes: None,
            odometer_km: None,
            updated_at: 2_000,
        })
        .expect_err("negative labor must fail in the domain");
    let add_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .add_part(AddServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            inventory_item_id: fixture.filter_item_id,
            quantity: 0,
            unit_price_fils: 4_500,
            created_at: 2_000,
        })
        .expect_err("zero quantity must fail in the part domain");
    let void_error = ServiceVisitWorkspaceService::new(&mut fixture.connection)
        .void_part(VoidServiceVisitPartInput {
            service_visit_id: fixture.visit_id,
            service_visit_part_id: 999_999,
            voided_at: 2_000,
            reason: None,
        })
        .expect_err("missing part must return a typed error");

    // # Assert
    assert!(matches!(
        update_error,
        ServiceVisitWorkspaceError::VisitValidation(
            ServiceVisitValidationError::NegativeLaborCharge
        )
    ));
    assert!(matches!(
        add_error,
        ServiceVisitWorkspaceError::PartValidation(
            ServiceVisitPartValidationError::InvalidQuantity
        )
    ));
    assert!(matches!(
        void_error,
        ServiceVisitWorkspaceError::ServiceVisitPartNotFound {
            service_visit_id,
            service_visit_part_id: 999_999,
        } if service_visit_id == fixture.visit_id
    ));
    let persisted: (Option<String>, i64, i64, i64) = fixture
        .connection
        .query_row(
            "SELECT diagnosis, updated_at,
                    (SELECT COUNT(*) FROM service_visit_parts),
                    (SELECT COUNT(*) FROM stock_movements)
             FROM service_visits WHERE id = ?1",
            [fixture.visit_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(persisted, (None, 1_000, 0, 0));
}

struct Fixture {
    _temp_dir: TempDir,
    connection: Connection,
    owner_id: i64,
    motorcycle_id: i64,
    visit_id: i64,
    filter_item_id: i64,
    oil_item_id: i64,
}

fn current_quantity(
    items: &[moto_workshop_lib::application::service_visit_workspace::InventoryItemSelection],
    item_id: i64,
) -> i64 {
    items
        .iter()
        .find(|item| item.id == item_id)
        .expect("inventory item should be selectable")
        .current_quantity
}

fn fixture() -> Fixture {
    let temp_dir = tempdir().unwrap();
    let mut connection = open_database(temp_dir.path().join("application-test.db")).unwrap();
    migrate_database(&mut connection).unwrap();

    connection
        .execute(
            "INSERT INTO customers (name, phone, created_at, updated_at)
             VALUES ('Ahmad Ali', '+962791234567', 1000, 1000)",
            [],
        )
        .unwrap();
    let owner_id = connection.last_insert_rowid();
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
    connection
        .execute(
            "INSERT INTO motorcycles (
                customer_id, make_id, model, year, plate_number,
                color_id, created_at, updated_at
             ) VALUES (?1, ?2, 'CB150R', 2022, '29-12345', ?3, 1000, 1000)",
            params![owner_id, make_id, color_id],
        )
        .unwrap();
    let motorcycle_id = connection.last_insert_rowid();
    connection
        .execute(
            "INSERT INTO service_visits (
                motorcycle_id, owner_customer_id, status, opened_at, odometer_km,
                customer_complaint, created_at, updated_at
             ) VALUES (?1, ?2, 'OPEN', 1000, 18500, 'Oil leak', 1000, 1000)",
            (motorcycle_id, owner_id),
        )
        .unwrap();
    let visit_id = connection.last_insert_rowid();

    let filter_item_id = insert_item(
        &connection,
        "Oil Filter",
        Some("FILTER"),
        unit_id(&connection, "Piece"),
        4_500,
        None,
    );
    let oil_item_id = insert_item(
        &connection,
        "Engine Oil",
        Some("OIL"),
        unit_id(&connection, "Liter"),
        7_000,
        None,
    );

    Fixture {
        _temp_dir: temp_dir,
        connection,
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

fn insert_unit(connection: &Connection, name: &str, scale: i64, active: bool) -> i64 {
    connection
        .execute(
            "INSERT INTO inventory_units (name, quantity_scale, active) VALUES (?1, ?2, ?3)",
            params![name, scale, i64::from(active)],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_item(
    connection: &Connection,
    name: &str,
    sku: Option<&str>,
    unit_id: i64,
    selling_price: i64,
    archived_at: Option<i64>,
) -> i64 {
    connection
        .execute(
            "INSERT INTO inventory_items (
                name, sku, unit_id, default_selling_price_fils,
                created_at, updated_at, archived_at
             ) VALUES (?1, ?2, ?3, ?4, 1000, 1000, ?5)",
            params![name, sku, unit_id, selling_price, archived_at],
        )
        .unwrap();
    connection.last_insert_rowid()
}

fn insert_part(
    connection: &Connection,
    visit_id: i64,
    item_id: i64,
    item_name: &str,
    unit_name: &str,
    values: PartValues,
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
                item_name,
                unit_name,
                values.quantity,
                values.scale,
                values.price,
                values.total,
                values.created_at
            ],
        )
        .unwrap();
    connection.last_insert_rowid()
}

struct PartValues {
    quantity: i64,
    scale: i64,
    price: i64,
    total: i64,
    created_at: i64,
}

impl PartValues {
    fn new(quantity: i64, scale: i64, price: i64, total: i64, created_at: i64) -> Self {
        Self {
            quantity,
            scale,
            price,
            total,
            created_at,
        }
    }
}
