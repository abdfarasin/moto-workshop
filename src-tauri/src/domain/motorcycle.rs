#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateNumber(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlateNumberValidationError {
    Blank,
    InvalidCharacter,
    InvalidFormat,
}

impl PlateNumber {
    pub fn parse(input: &str) -> Result<Self, PlateNumberValidationError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(PlateNumberValidationError::Blank);
        }

        if !input
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'-')
        {
            return Err(PlateNumberValidationError::InvalidCharacter);
        }

        if input.starts_with('-') || input.ends_with('-') || input.contains("--") {
            return Err(PlateNumberValidationError::InvalidFormat);
        }

        Ok(Self(input.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vin(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VinValidationError {
    Blank,
    InvalidLength,
    InvalidCharacter,
}

impl Vin {
    pub fn parse(input: &str) -> Result<Self, VinValidationError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(VinValidationError::Blank);
        }

        if !input.is_ascii() {
            return Err(VinValidationError::InvalidCharacter);
        }

        if input.len() != 17 {
            return Err(VinValidationError::InvalidLength);
        }

        let normalized = input.to_ascii_uppercase();

        let valid = normalized
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() && !matches!(byte, b'I' | b'O' | b'Q'));

        if !valid {
            return Err(VinValidationError::InvalidCharacter);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChassisNumber(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChassisNumberValidationError {
    Blank,
    InvalidLength,
    InvalidCharacter,
}

impl ChassisNumber {
    pub fn parse(input: &str) -> Result<Self, ChassisNumberValidationError> {
        let input = input.trim();

        if input.is_empty() {
            return Err(ChassisNumberValidationError::Blank);
        }

        if !input.is_ascii() {
            return Err(ChassisNumberValidationError::InvalidCharacter);
        }

        if input.len() > 64 {
            return Err(ChassisNumberValidationError::InvalidLength);
        }

        let normalized = input.to_ascii_uppercase();

        let valid = normalized.bytes().all(|byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'/' | b'.')
        });

        if !valid {
            return Err(ChassisNumberValidationError::InvalidCharacter);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewMotorcycleInput {
    pub make_id: i64,
    pub model: String,
    pub year: Option<i32>,
    pub plate_number: String,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_id: i64,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewMotorcycle {
    make_id: i64,
    model: String,
    year: Option<i32>,
    plate_number: PlateNumber,
    vin: Option<Vin>,
    chassis_number: Option<ChassisNumber>,
    color_id: i64,
    notes: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotorcycleValidationError {
    BlankModel,
    ModelTooLong,
    ModelContainsControlCharacter,
    InvalidYear,
    InvalidPlateNumber(PlateNumberValidationError),
    InvalidVin(VinValidationError),
    InvalidChassisNumber(ChassisNumberValidationError),
    NotesTooLong,
}

impl NewMotorcycle {
    pub fn new(
        input: NewMotorcycleInput,
        current_year: i32,
    ) -> Result<Self, MotorcycleValidationError> {
        let NewMotorcycleInput {
            make_id,
            model,
            year,
            plate_number,
            vin,
            chassis_number,
            color_id,
            notes,
        } = input;

        let model = model.trim().to_string();

        if model.is_empty() {
            return Err(MotorcycleValidationError::BlankModel);
        }

        if model.chars().count() > 80 {
            return Err(MotorcycleValidationError::ModelTooLong);
        }

        if model.chars().any(char::is_control) {
            return Err(MotorcycleValidationError::ModelContainsControlCharacter);
        }

        if year.is_some_and(|year| year < 1885 || year > current_year.saturating_add(1)) {
            return Err(MotorcycleValidationError::InvalidYear);
        }

        let plate_number = PlateNumber::parse(&plate_number)
            .map_err(MotorcycleValidationError::InvalidPlateNumber)?;

        let vin = vin
            .map(|vin| Vin::parse(&vin).map_err(MotorcycleValidationError::InvalidVin))
            .transpose()?;

        let chassis_number = normalize_optional_text(chassis_number)
            .map(|chassis_number| {
                ChassisNumber::parse(&chassis_number)
                    .map_err(MotorcycleValidationError::InvalidChassisNumber)
            })
            .transpose()?;

        let notes = normalize_optional_text(notes);

        if notes
            .as_ref()
            .is_some_and(|notes| notes.chars().count() > 2000)
        {
            return Err(MotorcycleValidationError::NotesTooLong);
        }

        Ok(Self {
            make_id,
            model,
            year,
            plate_number,
            vin,
            chassis_number,
            color_id,
            notes,
        })
    }

    pub fn make_id(&self) -> i64 {
        self.make_id
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn year(&self) -> Option<i32> {
        self.year
    }

    pub fn plate_number(&self) -> &PlateNumber {
        &self.plate_number
    }

    pub fn vin(&self) -> Option<&Vin> {
        self.vin.as_ref()
    }

    pub fn chassis_number(&self) -> Option<&ChassisNumber> {
        self.chassis_number.as_ref()
    }

    pub fn color_id(&self) -> i64 {
        self.color_id
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
