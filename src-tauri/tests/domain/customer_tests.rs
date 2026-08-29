use moto_workshop_lib::domain::customer::{CustomerValidationError, NewCustomer};
use proptest::prelude::*;

#[test]
fn new_customer_rejects_blank_phone() {
    // # Arrange
    let name = "Ahmad".to_string();
    let phone = "   ".to_string();

    // # Act
    let result = NewCustomer::new(name, phone, None);

    // # Assert
    assert_eq!(result, Err(CustomerValidationError::BlankPhone),);
}

#[test]
fn new_customer_rejects_blank_name() {
    // # Arrange
    let name = "   ".to_string();
    let phone = "+962791234567".to_string();

    // # Act
    let result = NewCustomer::new(name, phone, None);

    // # Assert
    assert_eq!(result, Err(CustomerValidationError::BlankName),);
}

#[test]
fn new_customer_trims_name() {
    // # Arrange
    let name = "  Ahmad  ".to_string();
    let phone = "+962791234567".to_string();

    // # Act
    let customer = NewCustomer::new(name, phone, None).expect("customer should be valid");

    // # Assert
    assert_eq!(customer.name(), "Ahmad");
}

#[test]
fn new_customer_trims_phone() {
    // # Arrange
    let name = "Ahmad".to_string();
    let phone = "  +962791234567  ".to_string();

    // # Act
    let customer = NewCustomer::new(name, phone, None).expect("customer should be valid");

    // # Assert
    assert_eq!(customer.phone(), "+962791234567");
}

#[test]
fn new_customer_preserves_provided_notes() {
    // # Arrange
    let notes = Some("Call before starting work".to_string());

    // # Act
    let customer = NewCustomer::new("Ahmad".to_string(), "+962791234567".to_string(), notes)
        .expect("customer should be valid");

    // # Assert
    assert_eq!(customer.notes(), Some("Call before starting work"));
}

#[test]
fn new_customer_allows_notes_to_be_omitted() {
    // # Arrange
    let notes = None;

    // # Act
    let customer = NewCustomer::new("Ahmad".to_string(), "+962791234567".to_string(), notes)
        .expect("customer should be valid");

    // # Assert
    assert_eq!(customer.notes(), None);
}

#[test]
fn new_customer_normalizes_established_jordanian_phone_representations() {
    // # Arrange
    let cases = [
        ("0791234567", "+962791234567"),
        ("00962791234567", "+962791234567"),
        ("+962791234567", "+962791234567"),
        ("0096212345", "+962096212345"),
    ];

    for (phone, expected_phone) in cases {
        // # Act
        let customer = NewCustomer::new("Ahmad".to_string(), phone.to_string(), None)
            .expect("customer should be valid");

        // # Assert
        assert_eq!(customer.phone(), expected_phone, "input phone: {phone}");
    }
}

#[test]
fn new_customer_phone_normalization_is_idempotent() {
    // # Arrange
    let customer = NewCustomer::new("Ahmad".to_string(), "0791234567".to_string(), None)
        .expect("customer should be valid");

    // # Act
    let renormalized_customer =
        NewCustomer::new("Ahmad".to_string(), customer.phone().to_string(), None)
            .expect("customer should remain valid after normalization");

    // # Assert
    assert_eq!(renormalized_customer.phone(), customer.phone());
}

#[test]
fn new_customer_accepts_arabic_english_and_internal_spaces_in_name() {
    // # Arrange
    let names = ["Ahmad Saleh", "أحمد صالح", "Ahmad أحمد"];

    for name in names {
        // # Act
        let customer = NewCustomer::new(name.to_string(), "+962791234567".to_string(), None)
            .expect("supported customer name should be valid");

        // # Assert
        assert_eq!(customer.name(), name);
    }
}

#[test]
fn new_customer_enforces_name_character_limit_using_unicode_characters() {
    // # Arrange
    let maximum_name = "أ".repeat(100);
    let oversized_name = "أ".repeat(101);

    // # Act
    let maximum_result = NewCustomer::new(maximum_name.clone(), "+962791234567".to_string(), None);
    let oversized_result = NewCustomer::new(oversized_name, "+962791234567".to_string(), None);

    // # Assert
    assert_eq!(
        maximum_result
            .expect("one hundred characters should be valid")
            .name(),
        maximum_name
    );
    assert_eq!(oversized_result, Err(CustomerValidationError::NameTooLong));
}

#[test]
fn new_customer_rejects_control_characters_in_name() {
    // # Arrange
    let names = ["Ahmad\nSaleh", "Ahmad\tSaleh", "Ahmad\u{0000}Saleh"];

    for name in names {
        // # Act
        let result = NewCustomer::new(name.to_string(), "+962791234567".to_string(), None);

        // # Assert
        assert_eq!(
            result,
            Err(CustomerValidationError::NameContainsControlCharacter),
            "control characters should be rejected in {name:?}"
        );
    }
}

#[test]
fn new_customer_rejects_invalid_phone_representations() {
    // # Arrange
    let invalid_phones = [
        "banana".to_string(),
        "+962banana".to_string(),
        "079 123 4567".to_string(),
        "079-123-4567".to_string(),
        "٠٧٩١٢٣٤٥٦٧".to_string(),
        "791234567".to_string(),
        "+96279123456".to_string(),
        "+9627912345678".to_string(),
        "0096279123456".to_string(),
        "009627912345678".to_string(),
        "0".repeat(10_000),
    ];

    for phone in invalid_phones {
        // # Act
        let result = NewCustomer::new("Ahmad".to_string(), phone.clone(), None);

        // # Assert
        assert_eq!(
            result,
            Err(CustomerValidationError::InvalidPhone),
            "phone should be rejected: {phone:?}"
        );
    }
}

#[test]
fn new_customer_normalizes_optional_notes() {
    // # Arrange
    let cases = [
        (None, None),
        (Some("   ".to_string()), None),
        (
            Some("  First line\nSecond line  ".to_string()),
            Some("First line\nSecond line"),
        ),
    ];

    for (notes, expected_notes) in cases {
        // # Act
        let customer = NewCustomer::new("Ahmad".to_string(), "+962791234567".to_string(), notes)
            .expect("notes should be valid");

        // # Assert
        assert_eq!(customer.notes(), expected_notes);
    }
}

#[test]
fn new_customer_enforces_notes_character_limit_using_unicode_characters() {
    // # Arrange
    let maximum_notes = "أ".repeat(2_000);
    let oversized_notes = "أ".repeat(2_001);

    // # Act
    let maximum_result = NewCustomer::new(
        "Ahmad".to_string(),
        "+962791234567".to_string(),
        Some(maximum_notes.clone()),
    );
    let oversized_result = NewCustomer::new(
        "Ahmad".to_string(),
        "+962791234568".to_string(),
        Some(oversized_notes),
    );

    // # Assert
    assert_eq!(
        maximum_result
            .expect("two thousand characters should be valid")
            .notes(),
        Some(maximum_notes.as_str())
    );
    assert_eq!(oversized_result, Err(CustomerValidationError::NotesTooLong));
}

proptest! {
    #[test]
    fn arbitrary_phone_input_never_panics(phone in any::<String>()) {
        // # Arrange
        let name = "Ahmad".to_string();

        // # Act
        let result = std::panic::catch_unwind(|| NewCustomer::new(name, phone, None));

        // # Assert
        prop_assert!(result.is_ok());
    }

    #[test]
    fn successful_phone_is_always_canonical(phone in any::<String>()) {
        // # Arrange / # Act
        let result = NewCustomer::new("Ahmad".to_string(), phone, None);

        // # Assert
        if let Ok(customer) = result {
            prop_assert_eq!(customer.phone().len(), 13);
            prop_assert!(customer.phone().starts_with("+962"));
            prop_assert!(customer.phone()[4..].bytes().all(|byte| byte.is_ascii_digit()));
        }
    }

    #[test]
    fn phone_normalization_is_idempotent_for_arbitrary_valid_input(
        digits in prop::collection::vec(0_u8..=9, 9),
        representation in 0_u8..3,
    ) {
        // # Arrange
        let subscriber = digits
            .into_iter()
            .map(|digit| char::from(b'0' + digit))
            .collect::<String>();
        let phone = match representation {
            0 => format!("0{subscriber}"),
            1 => format!("00962{subscriber}"),
            _ => format!("+962{subscriber}"),
        };

        // # Act
        let customer = NewCustomer::new("Ahmad".to_string(), phone, None)
            .expect("generated representation should be valid");
        let renormalized = NewCustomer::new(
            "Ahmad".to_string(),
            customer.phone().to_string(),
            None,
        )
        .expect("canonical representation should remain valid");

        // # Assert
        prop_assert_eq!(renormalized.phone(), customer.phone());
    }
}
