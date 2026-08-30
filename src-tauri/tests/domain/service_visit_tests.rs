use moto_workshop_lib::domain::service_visit::{
    NewServiceVisitInput, ServiceVisit, ServiceVisitDetailsInput, ServiceVisitStatus,
    ServiceVisitTextField, ServiceVisitValidationError,
};

#[test]
fn new_visit_starts_open_with_valid_english_or_arabic_complaint() {
    // # Arrange
    let complaints = ["Engine cuts out when hot", "المحرك يطفئ بعد أن يسخن"];

    for complaint in complaints {
        let mut input = valid_input();
        input.customer_complaint = format!("  {complaint}  ");

        // # Act
        let visit = ServiceVisit::open(input).expect("visit should open");

        // # Assert
        assert_eq!(visit.status(), ServiceVisitStatus::Open);
        assert_eq!(visit.customer_complaint(), complaint);
        assert_eq!(visit.completed_at(), None);
        assert_eq!(visit.closed_at(), None);
        assert_eq!(visit.cancelled_at(), None);
        assert_eq!(visit.labor_charge_fils(), 0);
    }
}

#[test]
fn creation_validates_ids_opened_at_and_complaint() {
    // # Arrange
    let mut invalid_motorcycle = valid_input();
    invalid_motorcycle.motorcycle_id = 0;
    let mut invalid_owner = valid_input();
    invalid_owner.owner_customer_id = -1;
    let mut invalid_opened_at = valid_input();
    invalid_opened_at.opened_at = -1;
    let invalid_complaints = [
        (
            "   ".to_string(),
            ServiceVisitValidationError::BlankComplaint,
        ),
        (
            "C".repeat(4_001),
            ServiceVisitValidationError::TextTooLong(ServiceVisitTextField::CustomerComplaint),
        ),
        (
            "Bad\u{0000}text".to_string(),
            ServiceVisitValidationError::TextContainsControlCharacter(
                ServiceVisitTextField::CustomerComplaint,
            ),
        ),
    ];

    // # Act / # Assert
    assert_eq!(
        ServiceVisit::open(invalid_motorcycle),
        Err(ServiceVisitValidationError::InvalidMotorcycleId)
    );
    assert_eq!(
        ServiceVisit::open(invalid_owner),
        Err(ServiceVisitValidationError::InvalidOwnerCustomerId)
    );
    assert_eq!(
        ServiceVisit::open(invalid_opened_at),
        Err(ServiceVisitValidationError::InvalidTimestamp)
    );
    for (complaint, expected) in invalid_complaints {
        let mut input = valid_input();
        input.customer_complaint = complaint;
        assert_eq!(ServiceVisit::open(input), Err(expected));
    }

    let mut maximum = valid_input();
    maximum.customer_complaint = "C".repeat(4_000);
    assert!(ServiceVisit::open(maximum).is_ok());
}

#[test]
fn creation_validates_odometer_boundaries() {
    // # Arrange / # Act / # Assert
    for odometer in [None, Some(0), Some(9_999_999)] {
        let mut input = valid_input();
        input.odometer_km = odometer;
        assert!(ServiceVisit::open(input).is_ok(), "odometer: {odometer:?}");
    }
    for odometer in [-1, 10_000_000] {
        let mut input = valid_input();
        input.odometer_km = Some(odometer);
        assert_eq!(
            ServiceVisit::open(input),
            Err(ServiceVisitValidationError::InvalidOdometer)
        );
    }
}

#[test]
fn active_visit_details_are_normalized_and_bounded() {
    // # Arrange
    let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");
    let details = ServiceVisitDetailsInput {
        diagnosis: Some("  Diagnosis\nline two  ".to_string()),
        work_performed: Some("  Repair completed  ".to_string()),
        labor_charge_fils: 12_500,
        notes: Some("   ".to_string()),
        odometer_km: Some(500),
    };

    // # Act
    visit
        .update_details(details)
        .expect("details should update");

    // # Assert
    assert_eq!(visit.diagnosis(), Some("Diagnosis\nline two"));
    assert_eq!(visit.work_performed(), Some("Repair completed"));
    assert_eq!(visit.notes(), None);
    assert_eq!(visit.labor_charge_fils(), 12_500);
    assert_eq!(visit.odometer_km(), Some(500));

    let excessive = ServiceVisitDetailsInput {
        diagnosis: Some("D".repeat(4_001)),
        work_performed: None,
        labor_charge_fils: 0,
        notes: None,
        odometer_km: None,
    };
    assert_eq!(
        visit.update_details(excessive),
        Err(ServiceVisitValidationError::TextTooLong(
            ServiceVisitTextField::Diagnosis
        ))
    );

    let negative_labor = ServiceVisitDetailsInput {
        diagnosis: None,
        work_performed: Some("Work".to_string()),
        labor_charge_fils: -1,
        notes: None,
        odometer_km: None,
    };
    assert_eq!(
        visit.update_details(negative_labor),
        Err(ServiceVisitValidationError::NegativeLaborCharge)
    );
}

#[test]
fn optional_visit_text_fields_enforce_limits_and_control_rules() {
    // # Arrange
    let cases = [
        (
            ServiceVisitDetailsInput {
                diagnosis: Some("D".repeat(4_001)),
                work_performed: None,
                labor_charge_fils: 0,
                notes: None,
                odometer_km: None,
            },
            ServiceVisitTextField::Diagnosis,
        ),
        (
            ServiceVisitDetailsInput {
                diagnosis: None,
                work_performed: Some("W".repeat(4_001)),
                labor_charge_fils: 0,
                notes: None,
                odometer_km: None,
            },
            ServiceVisitTextField::WorkPerformed,
        ),
        (
            ServiceVisitDetailsInput {
                diagnosis: None,
                work_performed: None,
                labor_charge_fils: 0,
                notes: Some("Bad\u{0000}note".to_string()),
                odometer_km: None,
            },
            ServiceVisitTextField::Notes,
        ),
    ];

    for (details, field) in cases {
        let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");

        // # Act
        let result = visit.update_details(details);

        // # Assert
        assert!(matches!(
            result,
            Err(ServiceVisitValidationError::TextTooLong(found))
                | Err(ServiceVisitValidationError::TextContainsControlCharacter(found))
                if found == field
        ));
    }
}

#[test]
fn open_to_ready_requires_work_and_valid_chronology() {
    // # Arrange
    let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");

    // # Act / # Assert
    assert_eq!(
        visit.mark_ready_for_pickup(1_100),
        Err(ServiceVisitValidationError::MissingWorkPerformed)
    );

    visit
        .update_details(details_with_work())
        .expect("work should update");
    assert_eq!(
        visit.mark_ready_for_pickup(999),
        Err(ServiceVisitValidationError::InvalidTimestamp)
    );
    visit
        .mark_ready_for_pickup(1_100)
        .expect("visit should become ready");
    assert_eq!(visit.status(), ServiceVisitStatus::ReadyForPickup);
    assert_eq!(visit.completed_at(), Some(1_100));
}

#[test]
fn ready_to_open_clears_completed_timestamp() {
    // # Arrange
    let mut visit = ready_visit();

    // # Act
    visit.reopen().expect("ready visit should reopen");

    // # Assert
    assert_eq!(visit.status(), ServiceVisitStatus::Open);
    assert_eq!(visit.completed_at(), None);
    assert_eq!(visit.closed_at(), None);
}

#[test]
fn ready_to_closed_requires_valid_chronology() {
    // # Arrange
    let mut visit = ready_visit();

    // # Act / # Assert
    assert_eq!(
        visit.close(1_099),
        Err(ServiceVisitValidationError::InvalidTimestamp)
    );
    visit.close(1_200).expect("ready visit should close");
    assert_eq!(visit.status(), ServiceVisitStatus::Closed);
    assert_eq!(visit.closed_at(), Some(1_200));
}

#[test]
fn open_to_cancelled_requires_reason_and_valid_chronology() {
    // # Arrange
    let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");

    // # Act / # Assert
    assert_eq!(
        visit.cancel(1_100, "   ".to_string()),
        Err(ServiceVisitValidationError::BlankCancellationReason)
    );
    assert_eq!(
        visit.cancel(999, "Customer declined".to_string()),
        Err(ServiceVisitValidationError::InvalidTimestamp)
    );
    visit
        .cancel(1_100, "  Customer declined repair  ".to_string())
        .expect("visit should cancel");
    assert_eq!(visit.status(), ServiceVisitStatus::Cancelled);
    assert_eq!(visit.cancelled_at(), Some(1_100));
    assert_eq!(
        visit.cancellation_reason(),
        Some("Customer declined repair")
    );
}

#[test]
fn cancellation_reason_enforces_length_and_control_rules() {
    // # Arrange
    let cases = [
        (
            "R".repeat(1_001),
            ServiceVisitValidationError::TextTooLong(ServiceVisitTextField::CancellationReason),
        ),
        (
            "Bad\u{0000}reason".to_string(),
            ServiceVisitValidationError::TextContainsControlCharacter(
                ServiceVisitTextField::CancellationReason,
            ),
        ),
    ];

    for (reason, expected) in cases {
        let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");

        // # Act
        let result = visit.cancel(1_100, reason);

        // # Assert
        assert_eq!(result, Err(expected));
    }
}

#[test]
fn forbidden_transitions_return_typed_errors() {
    // # Arrange
    let mut open = ServiceVisit::open(valid_input()).expect("visit should open");
    let mut ready = ready_visit();
    let mut closed = ready_visit();
    closed.close(1_200).expect("visit should close");
    let mut cancelled = ServiceVisit::open(valid_input()).expect("visit should open");
    cancelled
        .cancel(1_100, "Declined".to_string())
        .expect("visit should cancel");

    // # Act / # Assert
    assert!(matches!(
        open.close(1_200),
        Err(ServiceVisitValidationError::InvalidTransition { .. })
    ));
    assert!(matches!(
        ready.cancel(1_200, "Declined".to_string()),
        Err(ServiceVisitValidationError::InvalidTransition { .. })
    ));
    assert!(matches!(
        closed.reopen(),
        Err(ServiceVisitValidationError::InvalidTransition { .. })
    ));
    assert!(matches!(
        cancelled.reopen(),
        Err(ServiceVisitValidationError::InvalidTransition { .. })
    ));
}

#[test]
fn terminal_visits_reject_normal_edits() {
    // # Arrange
    let mut closed = ready_visit();
    closed.close(1_200).expect("visit should close");
    let mut cancelled = ServiceVisit::open(valid_input()).expect("visit should open");
    cancelled
        .cancel(1_100, "Declined".to_string())
        .expect("visit should cancel");

    // # Act / # Assert
    for visit in [&mut closed, &mut cancelled] {
        assert_eq!(
            visit.update_details(details_with_work()),
            Err(ServiceVisitValidationError::TerminalVisitCannotBeEdited)
        );
    }
}

fn valid_input() -> NewServiceVisitInput {
    NewServiceVisitInput {
        motorcycle_id: 1,
        owner_customer_id: 2,
        opened_at: 1_000,
        odometer_km: None,
        customer_complaint: "Engine cuts out when hot".to_string(),
        notes: None,
    }
}

fn details_with_work() -> ServiceVisitDetailsInput {
    ServiceVisitDetailsInput {
        diagnosis: Some("Fuel issue".to_string()),
        work_performed: Some("Cleaned fuel system".to_string()),
        labor_charge_fils: 5_000,
        notes: None,
        odometer_km: None,
    }
}

fn ready_visit() -> ServiceVisit {
    let mut visit = ServiceVisit::open(valid_input()).expect("visit should open");
    visit
        .update_details(details_with_work())
        .expect("work should update");
    visit
        .mark_ready_for_pickup(1_100)
        .expect("visit should become ready");
    visit
}
