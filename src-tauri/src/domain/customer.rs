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
        let phone = phone.trim().to_string();

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
}
