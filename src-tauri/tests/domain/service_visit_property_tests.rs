use moto_workshop_lib::domain::service_visit::{NewServiceVisitInput, ServiceVisit};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_complaint_input_never_panics(complaint in any::<String>()) {
        // # Arrange
        let input = input_with_complaint(complaint);

        // # Act
        let result = std::panic::catch_unwind(|| ServiceVisit::open(input));

        // # Assert
        prop_assert!(result.is_ok());
    }

    #[test]
    fn successful_complaint_is_normalized_and_bounded(complaint in any::<String>()) {
        // # Act
        let result = ServiceVisit::open(input_with_complaint(complaint));

        // # Assert
        if let Ok(visit) = result {
            prop_assert!(!visit.customer_complaint().is_empty());
            prop_assert_eq!(visit.customer_complaint(), visit.customer_complaint().trim());
            prop_assert!(visit.customer_complaint().chars().count() <= 4_000);
        }
    }

    #[test]
    fn successful_odometer_is_always_in_range(odometer in any::<i64>()) {
        // # Arrange
        let mut input = input_with_complaint("Complaint".to_string());
        input.odometer_km = Some(odometer);

        // # Act
        let result = ServiceVisit::open(input);

        // # Assert
        if let Ok(visit) = result {
            prop_assert!((0..=9_999_999).contains(&visit.odometer_km().unwrap()));
        }
    }
}

fn input_with_complaint(customer_complaint: String) -> NewServiceVisitInput {
    NewServiceVisitInput {
        motorcycle_id: 1,
        owner_customer_id: 2,
        opened_at: 1_000,
        odometer_km: None,
        customer_complaint,
        notes: None,
    }
}
