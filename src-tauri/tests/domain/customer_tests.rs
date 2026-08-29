use moto_workshop_lib::domain::customer::{CustomerValidationError, NewCustomer};

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
