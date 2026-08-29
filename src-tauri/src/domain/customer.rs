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
}

impl NewCustomer {
    pub fn new(
        name: String,
        phone: String,
        notes: Option<String>,
    ) -> Result<Self, CustomerValidationError> {
        let name = name.trim().to_string();
        let phone = normalize_phone(&phone);

        if name.is_empty() {
            return Err(CustomerValidationError::BlankName);
        }

        if phone.is_empty() {
            return Err(CustomerValidationError::BlankPhone);
        }

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

fn normalize_phone(phone: &str) -> String {
    let phone = phone.trim();

    if let Some(number) = phone.strip_prefix("00962") {
        return format!("+962{number}");
    }

    if let Some(local_number) = phone.strip_prefix('0') {
        return format!("+962{local_number}");
    }

    phone.to_string()
}
