use moto_workshop_lib::domain::motorcycle::{
    MotorcycleValidationError, NewMotorcycle, NewMotorcycleInput, PlateNumber,
    PlateNumberValidationError, Vin, VinValidationError,
};

const CURRENT_YEAR: i32 = 2026;

#[test]
fn plate_number_parses_valid_numeric_input() {
    // # Arrange
    let cases = [
        ("1", 1),
        ("12345", 12345),
        ("99999", 99999),
        ("  00042  ", 42),
    ];

    for (input, expected) in cases {
        // # Act
        let plate_number = PlateNumber::parse(input).expect("plate number should be valid");

        // # Assert
        assert_eq!(plate_number.value(), expected, "input: {input:?}");
    }
}

#[test]
fn plate_number_rejects_invalid_input_without_panicking() {
    // # Arrange
    let invalid_inputs = [
        "",
        "   ",
        "abc",
        "12abc",
        "123-45",
        "-1",
        "0",
        "100000",
        "999999999999999999999999999999999999999999999999",
        "١٢٣",
        "12\u{0000}3",
    ];

    for input in invalid_inputs {
        // # Act
        let result = PlateNumber::parse(input);

        // # Assert
        assert!(result.is_err(), "input should be rejected: {input:?}");
    }
}

#[test]
fn vin_normalizes_and_accepts_valid_input() {
    // # Arrange
    let input = "  1hgcm82633a004352  ";

    // # Act
    let vin = Vin::parse(input).expect("VIN should be valid");

    // # Assert
    assert_eq!(vin.as_str(), "1HGCM82633A004352");
}

#[test]
fn vin_rejects_invalid_input() {
    // # Arrange
    let invalid_inputs = [
        "",
        "   ",
        "1HGCM82633A00435",
        "1HGCM82633A004352X",
        "1HGCM82633I004352",
        "1HGCM82633O004352",
        "1HGCM82633Q004352",
        "1HGCM82633A00435-",
        "1HGCM82633A00 352",
        "1HGCM82633A00435é",
    ];

    for input in invalid_inputs {
        // # Act
        let result = Vin::parse(input);

        // # Assert
        assert!(result.is_err(), "input should be rejected: {input:?}");
    }
}

#[test]
fn vin_normalization_is_idempotent() {
    // # Arrange
    let vin = Vin::parse("1hgcm82633a004352").expect("VIN should be valid");

    // # Act
    let renormalized = Vin::parse(vin.as_str()).expect("normalized VIN should remain valid");

    // # Assert
    assert_eq!(renormalized, vin);
}

#[test]
fn new_motorcycle_accepts_and_trims_supported_models() {
    // # Arrange
    let models = ["  MT-07  ", "CBR600RR", "1290 Super Duke R", "Ninja ZX-6R"];

    for model in models {
        let mut input = valid_input();
        input.model = model.to_string();

        // # Act
        let motorcycle =
            NewMotorcycle::new(input, CURRENT_YEAR).expect("supported model should be valid");

        // # Assert
        assert_eq!(motorcycle.model(), model.trim(), "input: {model:?}");
    }
}

#[test]
fn new_motorcycle_enforces_model_length_and_character_rules() {
    // # Arrange
    let eighty_characters = "M".repeat(80);
    let mut accepted = valid_input();
    accepted.model = eighty_characters.clone();

    // # Act
    let motorcycle =
        NewMotorcycle::new(accepted, CURRENT_YEAR).expect("an 80-character model should be valid");

    // # Assert
    assert_eq!(motorcycle.model(), eighty_characters);

    let invalid_cases = [
        ("   ".to_string(), MotorcycleValidationError::BlankModel),
        ("M".repeat(81), MotorcycleValidationError::ModelTooLong),
        (
            "MT-\u{0007}07".to_string(),
            MotorcycleValidationError::ModelContainsControlCharacter,
        ),
    ];

    for (model, expected_error) in invalid_cases {
        let mut input = valid_input();
        input.model = model;

        // # Act
        let result = NewMotorcycle::new(input, CURRENT_YEAR);

        // # Assert
        assert_eq!(result, Err(expected_error));
    }
}

#[test]
fn new_motorcycle_accepts_valid_optional_years() {
    // # Arrange
    let years = [None, Some(1885), Some(2026), Some(2027)];

    for year in years {
        let mut input = valid_input();
        input.year = year;

        // # Act
        let motorcycle = NewMotorcycle::new(input, CURRENT_YEAR).expect("year should be valid");

        // # Assert
        assert_eq!(motorcycle.year(), year, "input year: {year:?}");
    }
}

#[test]
fn new_motorcycle_rejects_year_outside_allowed_range() {
    // # Arrange
    let invalid_years = [1884, 2028, 9999, -1];

    for year in invalid_years {
        let mut input = valid_input();
        input.year = Some(year);

        // # Act
        let result = NewMotorcycle::new(input, CURRENT_YEAR);

        // # Assert
        assert_eq!(
            result,
            Err(MotorcycleValidationError::InvalidYear),
            "input year: {year}"
        );
    }
}

#[test]
fn new_motorcycle_accepts_plate_only_identity() {
    // # Arrange
    let mut input = valid_input();
    input.plate_code_id = Some(7);
    input.plate_number = Some("  00042  ".to_string());
    input.vin = None;

    // # Act
    let motorcycle =
        NewMotorcycle::new(input, CURRENT_YEAR).expect("plate-only identity should be valid");

    // # Assert
    let plate = motorcycle.plate().expect("plate should be present");
    assert_eq!(plate.code_id(), 7);
    assert_eq!(plate.number().value(), 42);
    assert_eq!(motorcycle.vin(), None);
}

#[test]
fn new_motorcycle_accepts_vin_only_and_combined_identity() {
    // # Arrange
    let mut vin_only_input = valid_input();
    vin_only_input.plate_code_id = None;
    vin_only_input.plate_number = None;
    vin_only_input.vin = Some("1hgcm82633a004352".to_string());

    // # Act
    let vin_only = NewMotorcycle::new(vin_only_input, CURRENT_YEAR)
        .expect("VIN-only identity should be valid");

    // # Assert
    assert_eq!(vin_only.plate(), None);
    assert_eq!(vin_only.vin().map(Vin::as_str), Some("1HGCM82633A004352"));

    // # Arrange
    let mut combined_input = valid_input();
    combined_input.vin = Some("1HGCM82633A004352".to_string());

    // # Act
    let combined = NewMotorcycle::new(combined_input, CURRENT_YEAR)
        .expect("combined identity should be valid");

    // # Assert
    assert!(combined.plate().is_some());
    assert!(combined.vin().is_some());
}

#[test]
fn new_motorcycle_rejects_missing_or_incomplete_identity() {
    // # Arrange
    let cases = [
        (None, None, None, MotorcycleValidationError::MissingIdentity),
        (
            Some(1),
            None,
            None,
            MotorcycleValidationError::IncompletePlate,
        ),
        (
            None,
            Some("12345"),
            None,
            MotorcycleValidationError::IncompletePlate,
        ),
    ];

    for (plate_code_id, plate_number, vin, expected_error) in cases {
        let mut input = valid_input();
        input.plate_code_id = plate_code_id;
        input.plate_number = plate_number.map(str::to_owned);
        input.vin = vin.map(str::to_owned);

        // # Act
        let result = NewMotorcycle::new(input, CURRENT_YEAR);

        // # Assert
        assert_eq!(result, Err(expected_error));
    }
}

#[test]
fn new_motorcycle_reports_invalid_plate_and_vin() {
    // # Arrange
    let mut invalid_plate = valid_input();
    invalid_plate.plate_number = Some("abc".to_string());

    // # Act
    let plate_result = NewMotorcycle::new(invalid_plate, CURRENT_YEAR);

    // # Assert
    assert_eq!(
        plate_result,
        Err(MotorcycleValidationError::InvalidPlateNumber(
            PlateNumberValidationError::NonNumeric
        ))
    );

    // # Arrange
    let mut invalid_vin = valid_input();
    invalid_vin.vin = Some("1HGCM82633Q004352".to_string());

    // # Act
    let vin_result = NewMotorcycle::new(invalid_vin, CURRENT_YEAR);

    // # Assert
    assert_eq!(
        vin_result,
        Err(MotorcycleValidationError::InvalidVin(
            VinValidationError::InvalidCharacter
        ))
    );
}

#[test]
fn new_motorcycle_normalizes_and_limits_notes() {
    // # Arrange
    let cases = [
        (None, None),
        (Some("   "), None),
        (
            Some("  customer note\nsecond line  "),
            Some("customer note\nsecond line"),
        ),
    ];

    for (notes, expected) in cases {
        let mut input = valid_input();
        input.notes = notes.map(str::to_owned);

        // # Act
        let motorcycle = NewMotorcycle::new(input, CURRENT_YEAR).expect("notes should be valid");

        // # Assert
        assert_eq!(motorcycle.notes(), expected);
    }

    let mut maximum_notes = valid_input();
    maximum_notes.notes = Some("N".repeat(2000));
    assert!(NewMotorcycle::new(maximum_notes, CURRENT_YEAR).is_ok());

    let mut excessive_notes = valid_input();
    excessive_notes.notes = Some("N".repeat(2001));
    assert_eq!(
        NewMotorcycle::new(excessive_notes, CURRENT_YEAR),
        Err(MotorcycleValidationError::NotesTooLong)
    );
}

#[test]
fn new_motorcycle_preserves_catalog_ids() {
    // # Arrange
    let mut input = valid_input();
    input.make_id = 11;
    input.color_id = 4;

    // # Act
    let motorcycle = NewMotorcycle::new(input, CURRENT_YEAR).expect("motorcycle should be valid");

    // # Assert
    assert_eq!(motorcycle.make_id(), 11);
    assert_eq!(motorcycle.color_id(), 4);
}

fn valid_input() -> NewMotorcycleInput {
    NewMotorcycleInput {
        make_id: 1,
        model: "MT-07".to_string(),
        year: Some(2026),
        plate_code_id: Some(1),
        plate_number: Some("12345".to_string()),
        vin: None,
        color_id: 1,
        notes: None,
    }
}
