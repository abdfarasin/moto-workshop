#[derive(Debug, PartialEq)]
pub struct NewCustomer {
    name: String,
    phone: String,
    notes: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum CustomerValidationError {
    BlankName,
    BlankPhone,
    NameTooLong,
    NameContainsControlCharacter,
    InvalidPhone,
    NotesTooLong,
}

impl NewCustomer {
    pub fn new(
        name: String,
        phone: String,
        notes: Option<String>,
    ) -> Result<Self, CustomerValidationError> {
        let name = name.trim().to_string();

        if name.is_empty() {
            return Err(CustomerValidationError::BlankName);
        }

        if name.chars().count() > 100 {
            return Err(CustomerValidationError::NameTooLong);
        }

        if name.chars().any(char::is_control) {
            return Err(CustomerValidationError::NameContainsControlCharacter);
        }

        let phone = normalize_phone(&phone)?;
        let notes = normalize_notes(notes)?;

        Ok(Self { name, phone, notes })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

fn normalize_phone(phone: &str) -> Result<String, CustomerValidationError> {
    let phone = phone.trim();

    if phone.is_empty() {
        return Err(CustomerValidationError::BlankPhone);
    }

    let subscriber = [
        phone.strip_prefix("+962"),
        phone.strip_prefix("00962"),
        phone.strip_prefix('0'),
    ]
    .into_iter()
    .flatten()
    .find(|subscriber| {
        subscriber.len() == 9 && subscriber.bytes().all(|byte| byte.is_ascii_digit())
    })
    .ok_or(CustomerValidationError::InvalidPhone)?;

    Ok(format!("+962{subscriber}"))
}

fn normalize_notes(notes: Option<String>) -> Result<Option<String>, CustomerValidationError> {
    let Some(notes) = notes else {
        return Ok(None);
    };

    let notes = notes.trim();

    if notes.is_empty() {
        return Ok(None);
    }

    if notes.chars().count() > 2_000 {
        return Err(CustomerValidationError::NotesTooLong);
    }

    Ok(Some(notes.to_string()))
}
