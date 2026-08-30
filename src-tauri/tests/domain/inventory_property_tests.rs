use moto_workshop_lib::domain::inventory::{
    InventoryItem, InventoryUnit, NewInventoryItemInput, NewInventoryUnitInput,
    NewStockMovementInput, QuantityScale, StockMovement, StockMovementType,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_quantity_delta_never_panics(
        quantity_delta in any::<i64>(),
        movement_index in 0_u8..4,
    ) {
        // # Arrange
        let movement_type = movement_type(movement_index);

        // # Act
        let result = StockMovement::new(NewStockMovementInput {
            inventory_item_id: 1,
            movement_type,
            quantity_delta,
            notes: None,
            created_at: 0,
        });

        // # Assert
        if let Ok(movement) = result {
            let delta = movement.quantity_delta();
            match movement_type {
                StockMovementType::OpeningStock
                | StockMovementType::Purchase
                | StockMovementType::AdjustmentIn => {
                    prop_assert!((1..=1_000_000_000).contains(&delta));
                }
                StockMovementType::AdjustmentOut => {
                    prop_assert!((-1_000_000_000..=-1).contains(&delta));
                }
            }
        }
    }

    #[test]
    fn arbitrary_unit_text_validation_never_panics(name in any::<String>()) {
        // # Arrange / # Act
        let result = InventoryUnit::new(NewInventoryUnitInput {
            name,
            quantity_scale: 1,
        });

        // # Assert
        if let Ok(unit) = result {
            prop_assert!(!unit.name().is_empty());
            prop_assert!(unit.name().chars().count() <= 40);
        }
    }

    #[test]
    fn arbitrary_item_text_validation_never_panics(
        name in any::<String>(),
        sku in proptest::option::of(any::<String>()),
        notes in proptest::option::of(any::<String>()),
    ) {
        // # Arrange / # Act
        let result = InventoryItem::new(NewInventoryItemInput {
            name,
            sku,
            unit_id: 1,
            default_purchase_price_fils: None,
            default_selling_price_fils: 0,
            minimum_stock_quantity: 0,
            notes,
        });

        // # Assert
        if let Ok(item) = result {
            prop_assert!(!item.name().is_empty());
            prop_assert!(item.name().chars().count() <= 150);
            prop_assert!(item.sku().is_none_or(|value| value.chars().count() <= 64));
            prop_assert!(item.notes().is_none_or(|value| value.chars().count() <= 2_000));
        }
    }

    #[test]
    fn successful_quantity_scale_is_always_supported(scale in any::<i64>()) {
        // # Arrange / # Act
        let result = QuantityScale::new(scale);

        // # Assert
        if let Ok(value) = result {
            prop_assert!([1, 10, 100, 1_000].contains(&value.as_i64()));
        }
    }
}

fn movement_type(index: u8) -> StockMovementType {
    match index {
        0 => StockMovementType::OpeningStock,
        1 => StockMovementType::Purchase,
        2 => StockMovementType::AdjustmentIn,
        _ => StockMovementType::AdjustmentOut,
    }
}
