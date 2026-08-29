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
