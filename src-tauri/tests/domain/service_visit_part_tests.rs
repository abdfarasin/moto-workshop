use moto_workshop_lib::domain::service_visit_part::{
    calculate_line_total_fils, NewServiceVisitPartInput, ServiceVisitPart, ServiceVisitPartStatus,
    ServiceVisitPartTextField, ServiceVisitPartValidationError,
};

const MAX: i64 = 1_000_000_000;

#[test]
fn creates_piece_liter_and_arabic_snapshot_lines() {
    for (name, unit, quantity, scale, price, total) in [
        ("Oil Filter", "Piece", 2, 1, 3_500, 7_000),
        ("Engine Oil", "Liter", 2_500, 1_000, 7_000, 17_500),
        ("فلتر زيت", "قطعة", 1, 1, 4_500, 4_500),
    ] {
        // # Arrange / # Act
        let part = ServiceVisitPart::new(input(name, unit, quantity, scale, price))
            .expect("part should be valid");
        // # Assert
        assert_eq!(part.status(), ServiceVisitPartStatus::Active);
        assert_eq!(part.item_name(), name);
        assert_eq!(part.unit_name(), unit);
        assert_eq!(part.line_total_fils(), total);
        assert_eq!(part.voided_at(), None);
    }
}

#[test]
fn validates_ids_timestamp_quantity_scale_and_price_boundaries() {
    let cases = [
        (
            0,
            1,
            1,
            1,
            0,
            0,
            ServiceVisitPartValidationError::InvalidServiceVisitId,
        ),
        (
            1,
            0,
            1,
            1,
            0,
            0,
            ServiceVisitPartValidationError::InvalidInventoryItemId,
        ),
        (
            1,
            1,
            0,
            1,
            0,
            0,
            ServiceVisitPartValidationError::InvalidQuantity,
        ),
        (
            1,
            1,
            MAX + 1,
            1,
            0,
            0,
            ServiceVisitPartValidationError::InvalidQuantity,
        ),
        (
            1,
            1,
            1,
            2,
            0,
            0,
            ServiceVisitPartValidationError::InvalidQuantityScale,
        ),
        (
            1,
            1,
            1,
            1,
            -1,
            0,
            ServiceVisitPartValidationError::InvalidUnitPrice,
        ),
        (
            1,
            1,
            1,
            1,
            MAX + 1,
            0,
            ServiceVisitPartValidationError::InvalidUnitPrice,
        ),
        (
            1,
            1,
            1,
            1,
            0,
            -1,
            ServiceVisitPartValidationError::InvalidTimestamp,
        ),
    ];
    for (visit_id, item_id, quantity, scale, price, created_at, expected) in cases {
        let mut value = input("Item", "Piece", quantity, scale, price);
        value.service_visit_id = visit_id;
        value.inventory_item_id = item_id;
        value.created_at = created_at;
        assert_eq!(ServiceVisitPart::new(value), Err(expected));
    }
    for quantity in [1, MAX] {
        assert!(ServiceVisitPart::new(input("Item", "Piece", quantity, 1, MAX)).is_ok());
    }
    for scale in [1, 10, 100, 1_000] {
        assert!(ServiceVisitPart::new(input("Item", "Unit", 1, scale, 0)).is_ok());
    }
}

#[test]
fn snapshot_names_are_trimmed_bounded_and_control_safe() {
    let accepted_item = "ع".repeat(150);
    let accepted_unit = "و".repeat(40);
    assert!(ServiceVisitPart::new(input(&accepted_item, &accepted_unit, 1, 1, 0)).is_ok());
    for (item, unit, expected) in [
        (
            " ".to_string(),
            "Piece".to_string(),
            ServiceVisitPartValidationError::BlankItemName,
        ),
        (
            "x".repeat(151),
            "Piece".to_string(),
            ServiceVisitPartValidationError::TextTooLong(ServiceVisitPartTextField::ItemName),
        ),
        (
            "Item".to_string(),
            " ".to_string(),
            ServiceVisitPartValidationError::BlankUnitName,
        ),
        (
            "Item".to_string(),
            "x".repeat(41),
            ServiceVisitPartValidationError::TextTooLong(ServiceVisitPartTextField::UnitName),
        ),
        (
            "Bad\0item".to_string(),
            "Piece".to_string(),
            ServiceVisitPartValidationError::TextContainsControlCharacter(
                ServiceVisitPartTextField::ItemName,
            ),
        ),
    ] {
        assert_eq!(
            ServiceVisitPart::new(input(&item, &unit, 1, 1, 0)),
            Err(expected)
        );
    }
}

#[test]
fn half_up_calculation_covers_exact_below_half_half_above_and_maximum() {
    for (quantity, scale, price, expected) in [
        (2, 1, 3_500, 7_000),
        (2_500, 1_000, 7_000, 17_500),
        (332, 1_000, 5_500, 1_826),
        (333, 1_000, 5_500, 1_832),
        (334, 1_000, 5_500, 1_837),
        (1, 1, 0, 0),
        (MAX, 1, MAX, 1_000_000_000_000_000_000),
    ] {
        assert_eq!(
            calculate_line_total_fils(quantity, scale, price),
            Ok(expected)
        );
    }
}

#[test]
fn calculation_rejects_extremes_without_panicking() {
    for (quantity, scale, price) in [
        (i64::MIN, 1, 1),
        (i64::MAX, 1, 1),
        (1, i64::MIN, 1),
        (1, 0, 1),
        (1, 1, i64::MAX),
    ] {
        assert!(calculate_line_total_fils(quantity, scale, price).is_err());
    }
}

#[test]
fn active_part_voids_once_with_optional_normalized_reason() {
    for (reason, expected) in [
        (None, None),
        (Some("   ".to_string()), None),
        (
            Some("  Wrong quantity\nchecked  ".to_string()),
            Some("Wrong quantity\nchecked"),
        ),
        (Some("س".repeat(1_000)), Some("س".repeat(1_000).as_str())),
    ] {
        let mut part = ServiceVisitPart::new(input("Item", "Piece", 1, 1, 1)).unwrap();
        part.void(2_000, reason).expect("void should succeed");
        assert_eq!(part.status(), ServiceVisitPartStatus::Voided);
        assert_eq!(part.void_reason(), expected);
        assert_eq!(part.voided_at(), Some(2_000));
        assert_eq!(
            part.void(3_000, None),
            Err(ServiceVisitPartValidationError::PartAlreadyVoided)
        );
    }
}

#[test]
fn void_validates_chronology_reason_length_and_controls() {
    for (at, reason, expected) in [
        (999, None, ServiceVisitPartValidationError::InvalidTimestamp),
        (
            2_000,
            Some("x".repeat(1_001)),
            ServiceVisitPartValidationError::TextTooLong(ServiceVisitPartTextField::VoidReason),
        ),
        (
            2_000,
            Some("bad\0reason".to_string()),
            ServiceVisitPartValidationError::TextContainsControlCharacter(
                ServiceVisitPartTextField::VoidReason,
            ),
        ),
    ] {
        let mut part = ServiceVisitPart::new(input("Item", "Piece", 1, 1, 1)).unwrap();
        assert_eq!(part.void(at, reason), Err(expected));
        assert_eq!(part.status(), ServiceVisitPartStatus::Active);
    }
}

fn input(
    name: &str,
    unit: &str,
    quantity: i64,
    scale: i64,
    price: i64,
) -> NewServiceVisitPartInput {
    NewServiceVisitPartInput {
        service_visit_id: 1,
        inventory_item_id: 1,
        item_name: name.to_string(),
        unit_name: unit.to_string(),
        quantity,
        quantity_scale: scale,
        unit_price_fils: price,
        created_at: 1_000,
    }
}
