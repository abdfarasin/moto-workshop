use moto_workshop_lib::domain::service_visit_part::{
    calculate_line_total_fils, NewServiceVisitPartInput, ServiceVisitPart,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_calculation_inputs_never_panic(quantity in any::<i64>(), scale in any::<i64>(), price in any::<i64>()) {
        let _ = calculate_line_total_fils(quantity, scale, price);
    }

    #[test]
    fn successful_calculation_matches_integer_half_up(
        quantity in 1_i64..=1_000_000_000,
        scale in prop::sample::select(vec![1_i64, 10, 100, 1_000]),
        price in 0_i64..=1_000_000_000,
    ) {
        let result = calculate_line_total_fils(quantity, scale, price).unwrap();
        let expected = (quantity * price + scale / 2) / scale;
        prop_assert_eq!(result, expected);
        prop_assert!(result >= 0);
    }

    #[test]
    fn successful_part_quantity_is_bounded(quantity in any::<i64>()) {
        let result = ServiceVisitPart::new(NewServiceVisitPartInput {
            service_visit_id: 1, inventory_item_id: 1, item_name: "Item".into(),
            unit_name: "Piece".into(), quantity, quantity_scale: 1,
            unit_price_fils: 1, created_at: 0,
        });
        if let Ok(part) = result {
            prop_assert!((1..=1_000_000_000).contains(&part.quantity()));
        }
    }

    #[test]
    fn arbitrary_void_reason_never_panics(reason in proptest::option::of(any::<String>())) {
        let mut part = ServiceVisitPart::new(NewServiceVisitPartInput {
            service_visit_id: 1, inventory_item_id: 1, item_name: "Item".into(),
            unit_name: "Piece".into(), quantity: 1, quantity_scale: 1,
            unit_price_fils: 1, created_at: 0,
        }).unwrap();
        let _ = part.void(0, reason);
    }
}
