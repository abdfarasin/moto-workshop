use moto_workshop_lib::domain::motorcycle::{PlateNumber, Vin};
use proptest::prelude::*;

proptest! {
    #[test]
    fn arbitrary_plate_number_input_never_panics(input in any::<String>()) {
        // # Act
        let _ = PlateNumber::parse(&input);
    }

    #[test]
    fn successful_plate_number_is_always_in_range(input in any::<String>()) {
        // # Act
        let result = PlateNumber::parse(&input);

        // # Assert
        if let Ok(plate_number) = result {
            prop_assert!((1..=99_999).contains(&plate_number.value()));
        }
    }

    #[test]
    fn arbitrary_vin_input_never_panics(input in any::<String>()) {
        // # Act
        let _ = Vin::parse(&input);
    }

    #[test]
    fn successful_vin_always_has_canonical_vin_characters(input in any::<String>()) {
        // # Act
        let result = Vin::parse(&input);

        // # Assert
        if let Ok(vin) = result {
            let value = vin.as_str();
            prop_assert_eq!(value.len(), 17);
            prop_assert!(value.is_ascii());
            let has_valid_characters = value
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
            prop_assert!(has_valid_characters);
            prop_assert!(!value.bytes().any(|byte| matches!(byte, b'I' | b'O' | b'Q')));
        }
    }

    #[test]
    fn vin_normalization_is_idempotent_for_arbitrary_valid_input(input in any::<String>()) {
        // # Act
        let first = Vin::parse(&input);

        // # Assert
        if let Ok(vin) = first {
            let second = Vin::parse(vin.as_str()).expect("canonical VIN should remain valid");
            prop_assert_eq!(second, vin);
        }
    }
}
