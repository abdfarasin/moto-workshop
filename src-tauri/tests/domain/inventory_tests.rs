use moto_workshop_lib::domain::inventory::{
    InventoryItem, InventoryTextField, InventoryUnit, InventoryValidationError,
    NewInventoryItemInput, NewInventoryUnitInput, NewStockMovementInput, QuantityScale,
    StockMovement, StockMovementType,
};

const MAX: i64 = 1_000_000_000;

#[test]
fn inventory_unit_accepts_supported_scales_and_normalizes_names() {
    for (name, scale) in [
        (" Piece ", 1),
        ("Tenth", 10),
        ("Hundredth", 100),
        (" لتر ", 1_000),
    ] {
        // # Arrange / # Act
        let unit = InventoryUnit::new(NewInventoryUnitInput {
            name: name.to_string(),
            quantity_scale: scale,
        })
        .expect("supported unit should be valid");

        // # Assert
        assert_eq!(unit.name(), name.trim());
        assert_eq!(unit.quantity_scale().as_i64(), scale);
    }
}

#[test]
fn inventory_unit_rejects_invalid_scales() {
    for scale in [0, 2, 50, 10_000] {
        // # Arrange / # Act
        let result = InventoryUnit::new(NewInventoryUnitInput {
            name: "Unit".to_string(),
            quantity_scale: scale,
        });

        // # Assert
        assert_eq!(result, Err(InventoryValidationError::InvalidQuantityScale));
    }
}

#[test]
fn inventory_unit_name_enforces_blank_length_and_control_rules() {
    let forty = "ع".repeat(40);
    let forty_one = "a".repeat(41);

    for (name, expected) in [
        ("   ".to_string(), InventoryValidationError::BlankUnitName),
        (
            forty_one,
            InventoryValidationError::TextTooLong(InventoryTextField::UnitName),
        ),
        (
            "Bad\0Unit".to_string(),
            InventoryValidationError::TextContainsControlCharacter(InventoryTextField::UnitName),
        ),
    ] {
        // # Arrange / # Act
        let result = InventoryUnit::new(NewInventoryUnitInput {
            name,
            quantity_scale: 1,
        });

        // # Assert
        assert_eq!(result, Err(expected));
    }

    assert!(InventoryUnit::new(NewInventoryUnitInput {
        name: forty,
        quantity_scale: 1,
    })
    .is_ok());
}

#[test]
fn inventory_item_normalizes_supported_english_and_arabic_names() {
    for name in [" Oil Filter ", " زيت المحرك "] {
        // # Arrange / # Act
        let item = InventoryItem::new(valid_item(name)).expect("item should be valid");

        // # Assert
        assert_eq!(item.name(), name.trim());
        assert_eq!(item.unit_id(), 1);
    }
}

#[test]
fn inventory_item_name_enforces_blank_length_and_control_rules() {
    let maximum = "ز".repeat(150);
    let over_maximum = "a".repeat(151);
    assert!(InventoryItem::new(valid_item(&maximum)).is_ok());

    for (name, expected) in [
        ("   ".to_string(), InventoryValidationError::BlankItemName),
        (
            over_maximum,
            InventoryValidationError::TextTooLong(InventoryTextField::ItemName),
        ),
        (
            "Bad\0Item".to_string(),
            InventoryValidationError::TextContainsControlCharacter(InventoryTextField::ItemName),
        ),
    ] {
        // # Arrange / # Act
        let result = InventoryItem::new(valid_item(&name));

        // # Assert
        assert_eq!(result, Err(expected));
    }
}

#[test]
fn inventory_item_sku_is_optional_normalized_bounded_and_preserves_punctuation() {
    for (sku, expected) in [
        (None, None),
        (Some("   ".to_string()), None),
        (Some("  NGK-CR8E/01  ".to_string()), Some("NGK-CR8E/01")),
        (Some("S".repeat(64)), Some("S".repeat(64).as_str())),
    ] {
        // # Arrange
        let mut input = valid_item("Spark Plug");
        input.sku = sku;

        // # Act
        let item = InventoryItem::new(input).expect("SKU should be valid");

        // # Assert
        assert_eq!(item.sku(), expected);
    }

    for (sku, expected) in [
        (
            "S".repeat(65),
            InventoryValidationError::TextTooLong(InventoryTextField::Sku),
        ),
        (
            "BAD\u{0007}SKU".to_string(),
            InventoryValidationError::TextContainsControlCharacter(InventoryTextField::Sku),
        ),
    ] {
        let mut input = valid_item("Spark Plug");
        input.sku = Some(sku);
        assert_eq!(InventoryItem::new(input), Err(expected));
    }
}

#[test]
fn inventory_item_requires_positive_unit_id() {
    for unit_id in [0, -1] {
        // # Arrange
        let mut input = valid_item("Item");
        input.unit_id = unit_id;

        // # Act / # Assert
        assert_eq!(
            InventoryItem::new(input),
            Err(InventoryValidationError::InvalidUnitId)
        );
    }
}

#[test]
fn inventory_item_enforces_purchase_and_selling_price_bounds() {
    for purchase in [None, Some(0), Some(MAX)] {
        let mut input = valid_item("Item");
        input.default_purchase_price_fils = purchase;
        assert!(InventoryItem::new(input).is_ok());
    }
    for purchase in [Some(-1), Some(MAX + 1)] {
        let mut input = valid_item("Item");
        input.default_purchase_price_fils = purchase;
        assert_eq!(
            InventoryItem::new(input),
            Err(InventoryValidationError::InvalidPurchasePrice)
        );
    }
    for selling in [0, MAX] {
        let mut input = valid_item("Item");
        input.default_selling_price_fils = selling;
        assert!(InventoryItem::new(input).is_ok());
    }
    for selling in [-1, MAX + 1] {
        let mut input = valid_item("Item");
        input.default_selling_price_fils = selling;
        assert_eq!(
            InventoryItem::new(input),
            Err(InventoryValidationError::InvalidSellingPrice)
        );
    }
}

#[test]
fn inventory_item_enforces_minimum_stock_bounds() {
    for minimum in [0, MAX] {
        let mut input = valid_item("Item");
        input.minimum_stock_quantity = minimum;
        assert!(InventoryItem::new(input).is_ok());
    }
    for minimum in [-1, MAX + 1] {
        let mut input = valid_item("Item");
        input.minimum_stock_quantity = minimum;
        assert_eq!(
            InventoryItem::new(input),
            Err(InventoryValidationError::InvalidMinimumStock)
        );
    }
}

#[test]
fn inventory_item_notes_are_optional_normalized_multiline_and_bounded() {
    for (notes, expected) in [
        (None, None),
        (Some("   ".to_string()), None),
        (
            Some("  Shelf A\nFragile\tpackaging  ".to_string()),
            Some("Shelf A\nFragile\tpackaging"),
        ),
        (Some("ن".repeat(2_000)), Some("ن".repeat(2_000).as_str())),
    ] {
        let mut input = valid_item("Item");
        input.notes = notes;
        let item = InventoryItem::new(input).expect("notes should be valid");
        assert_eq!(item.notes(), expected);
    }

    for (notes, expected) in [
        (
            "n".repeat(2_001),
            InventoryValidationError::TextTooLong(InventoryTextField::Notes),
        ),
        (
            "bad\0note".to_string(),
            InventoryValidationError::TextContainsControlCharacter(InventoryTextField::Notes),
        ),
    ] {
        let mut input = valid_item("Item");
        input.notes = Some(notes);
        assert_eq!(InventoryItem::new(input), Err(expected));
    }
}

#[test]
fn stock_movement_accepts_exact_type_sign_and_boundaries() {
    for (movement_type, quantity_delta) in [
        (StockMovementType::OpeningStock, 1),
        (StockMovementType::OpeningStock, MAX),
        (StockMovementType::Purchase, 1),
        (StockMovementType::Purchase, MAX),
        (StockMovementType::AdjustmentIn, 1),
        (StockMovementType::AdjustmentIn, MAX),
        (StockMovementType::AdjustmentOut, -1),
        (StockMovementType::AdjustmentOut, -MAX),
    ] {
        // # Arrange / # Act
        let movement = StockMovement::new(valid_movement(movement_type, quantity_delta))
            .expect("movement should be valid");

        // # Assert
        assert_eq!(movement.movement_type(), movement_type);
        assert_eq!(movement.quantity_delta(), quantity_delta);
    }
}

#[test]
fn stock_movement_rejects_zero_wrong_sign_and_out_of_bounds_safely() {
    for (movement_type, quantity_delta) in [
        (StockMovementType::OpeningStock, 0),
        (StockMovementType::OpeningStock, -1),
        (StockMovementType::OpeningStock, MAX + 1),
        (StockMovementType::Purchase, -1),
        (StockMovementType::AdjustmentIn, -1),
        (StockMovementType::AdjustmentOut, 1),
        (StockMovementType::AdjustmentOut, -MAX - 1),
        (StockMovementType::AdjustmentOut, i64::MIN),
    ] {
        // # Arrange / # Act
        let result = StockMovement::new(valid_movement(movement_type, quantity_delta));

        // # Assert
        assert_eq!(
            result,
            Err(InventoryValidationError::InvalidQuantityDelta { movement_type })
        );
    }
}

#[test]
fn stock_movement_validates_identity_timestamp_and_notes() {
    let mut invalid_id = valid_movement(StockMovementType::Purchase, 1);
    invalid_id.inventory_item_id = 0;
    assert_eq!(
        StockMovement::new(invalid_id),
        Err(InventoryValidationError::InvalidInventoryItemId)
    );

    let mut invalid_time = valid_movement(StockMovementType::Purchase, 1);
    invalid_time.created_at = -1;
    assert_eq!(
        StockMovement::new(invalid_time),
        Err(InventoryValidationError::InvalidTimestamp)
    );

    for (notes, expected) in [
        (None, None),
        (Some("   ".to_string()), None),
        (
            Some("  Received\nby Ahmad  ".to_string()),
            Some("Received\nby Ahmad"),
        ),
        (Some("م".repeat(2_000)), Some("م".repeat(2_000).as_str())),
    ] {
        let mut input = valid_movement(StockMovementType::Purchase, 1);
        input.notes = notes;
        let movement = StockMovement::new(input).expect("notes should be valid");
        assert_eq!(movement.notes(), expected);
    }

    for (notes, expected) in [
        (
            "n".repeat(2_001),
            InventoryValidationError::TextTooLong(InventoryTextField::MovementNotes),
        ),
        (
            "bad\0note".to_string(),
            InventoryValidationError::TextContainsControlCharacter(
                InventoryTextField::MovementNotes,
            ),
        ),
    ] {
        let mut input = valid_movement(StockMovementType::Purchase, 1);
        input.notes = Some(notes);
        assert_eq!(StockMovement::new(input), Err(expected));
    }
}

#[test]
fn stock_usage_types_require_positive_part_reference_and_exact_sign() {
    for (movement_type, delta) in [
        (StockMovementType::ServiceUsage, -1),
        (StockMovementType::ServiceUsage, -MAX),
        (StockMovementType::ServiceUsageReversal, 1),
        (StockMovementType::ServiceUsageReversal, MAX),
    ] {
        let mut input = valid_movement(movement_type, delta);
        input.service_visit_part_id = Some(7);
        let movement = StockMovement::new(input).expect("linked usage movement should be valid");
        assert_eq!(movement.service_visit_part_id(), Some(7));
    }
    for (movement_type, delta) in [
        (StockMovementType::ServiceUsage, 1),
        (StockMovementType::ServiceUsageReversal, -1),
    ] {
        let mut input = valid_movement(movement_type, delta);
        input.service_visit_part_id = Some(7);
        assert_eq!(
            StockMovement::new(input),
            Err(InventoryValidationError::InvalidQuantityDelta { movement_type })
        );
    }
    for reference in [None, Some(0), Some(-1)] {
        let mut input = valid_movement(StockMovementType::ServiceUsage, -1);
        input.service_visit_part_id = reference;
        assert_eq!(
            StockMovement::new(input),
            Err(InventoryValidationError::InvalidServiceVisitPartReference)
        );
    }
    let mut manual = valid_movement(StockMovementType::Purchase, 1);
    manual.service_visit_part_id = Some(7);
    assert_eq!(
        StockMovement::new(manual),
        Err(InventoryValidationError::InvalidServiceVisitPartReference)
    );
}

#[test]
fn successful_item_preserves_all_integer_financial_and_quantity_inputs() {
    // # Arrange
    let input = NewInventoryItemInput {
        name: "Engine Oil".to_string(),
        sku: Some("OIL-10W40".to_string()),
        unit_id: 2,
        default_purchase_price_fils: Some(5_500),
        default_selling_price_fils: 7_000,
        minimum_stock_quantity: 2_500,
        notes: Some("Synthetic".to_string()),
    };

    // # Act
    let item = InventoryItem::new(input).expect("item should be valid");

    // # Assert
    assert_eq!(item.sku(), Some("OIL-10W40"));
    assert_eq!(item.default_purchase_price_fils(), Some(5_500));
    assert_eq!(item.default_selling_price_fils(), 7_000);
    assert_eq!(item.minimum_stock_quantity(), 2_500);
}

fn valid_item(name: &str) -> NewInventoryItemInput {
    NewInventoryItemInput {
        name: name.to_string(),
        sku: None,
        unit_id: 1,
        default_purchase_price_fils: None,
        default_selling_price_fils: 0,
        minimum_stock_quantity: 0,
        notes: None,
    }
}

fn valid_movement(movement_type: StockMovementType, quantity_delta: i64) -> NewStockMovementInput {
    NewStockMovementInput {
        inventory_item_id: 1,
        service_visit_part_id: None,
        movement_type,
        quantity_delta,
        notes: None,
        created_at: 1_000,
    }
}

#[test]
fn quantity_scale_constructor_exposes_only_supported_values() {
    for scale in [1, 10, 100, 1_000] {
        let value = QuantityScale::new(scale).expect("scale should be valid");
        assert_eq!(value.as_i64(), scale);
    }
}
