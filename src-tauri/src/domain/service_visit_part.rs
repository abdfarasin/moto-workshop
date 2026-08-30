const MAX_QUANTITY: i64 = 1_000_000_000;
const MAX_UNIT_PRICE_FILS: i64 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitPartStatus {
    Active,
    Voided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitPartTextField {
    ItemName,
    UnitName,
    VoidReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitPartValidationError {
    InvalidServiceVisitId,
    InvalidInventoryItemId,
    BlankItemName,
    BlankUnitName,
    InvalidQuantity,
    InvalidQuantityScale,
    InvalidUnitPrice,
    InvalidTimestamp,
    CalculationOverflow,
    PartAlreadyVoided,
    TextTooLong(ServiceVisitPartTextField),
    TextContainsControlCharacter(ServiceVisitPartTextField),
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewServiceVisitPartInput {
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub created_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceVisitPart {
    service_visit_id: i64,
    inventory_item_id: i64,
    item_name: String,
    unit_name: String,
    quantity: i64,
    quantity_scale: i64,
    unit_price_fils: i64,
    line_total_fils: i64,
    status: ServiceVisitPartStatus,
    voided_at: Option<i64>,
    void_reason: Option<String>,
    created_at: i64,
}

impl ServiceVisitPart {
    pub fn new(input: NewServiceVisitPartInput) -> Result<Self, ServiceVisitPartValidationError> {
        if input.service_visit_id <= 0 {
            return Err(ServiceVisitPartValidationError::InvalidServiceVisitId);
        }
        if input.inventory_item_id <= 0 {
            return Err(ServiceVisitPartValidationError::InvalidInventoryItemId);
        }
        if input.created_at < 0 {
            return Err(ServiceVisitPartValidationError::InvalidTimestamp);
        }
        let item_name = normalize_required(
            input.item_name,
            150,
            ServiceVisitPartTextField::ItemName,
            ServiceVisitPartValidationError::BlankItemName,
            false,
        )?;
        let unit_name = normalize_required(
            input.unit_name,
            40,
            ServiceVisitPartTextField::UnitName,
            ServiceVisitPartValidationError::BlankUnitName,
            false,
        )?;
        let line_total_fils =
            calculate_line_total_fils(input.quantity, input.quantity_scale, input.unit_price_fils)?;

        Ok(Self {
            service_visit_id: input.service_visit_id,
            inventory_item_id: input.inventory_item_id,
            item_name,
            unit_name,
            quantity: input.quantity,
            quantity_scale: input.quantity_scale,
            unit_price_fils: input.unit_price_fils,
            line_total_fils,
            status: ServiceVisitPartStatus::Active,
            voided_at: None,
            void_reason: None,
            created_at: input.created_at,
        })
    }

    pub fn void(
        &mut self,
        voided_at: i64,
        reason: Option<String>,
    ) -> Result<(), ServiceVisitPartValidationError> {
        if self.status == ServiceVisitPartStatus::Voided {
            return Err(ServiceVisitPartValidationError::PartAlreadyVoided);
        }
        if voided_at < self.created_at {
            return Err(ServiceVisitPartValidationError::InvalidTimestamp);
        }
        let void_reason =
            normalize_optional(reason, 1_000, ServiceVisitPartTextField::VoidReason, true)?;
        self.status = ServiceVisitPartStatus::Voided;
        self.voided_at = Some(voided_at);
        self.void_reason = void_reason;
        Ok(())
    }

    pub fn service_visit_id(&self) -> i64 {
        self.service_visit_id
    }
    pub fn inventory_item_id(&self) -> i64 {
        self.inventory_item_id
    }
    pub fn item_name(&self) -> &str {
        &self.item_name
    }
    pub fn unit_name(&self) -> &str {
        &self.unit_name
    }
    pub fn quantity(&self) -> i64 {
        self.quantity
    }
    pub fn quantity_scale(&self) -> i64 {
        self.quantity_scale
    }
    pub fn unit_price_fils(&self) -> i64 {
        self.unit_price_fils
    }
    pub fn line_total_fils(&self) -> i64 {
        self.line_total_fils
    }
    pub fn status(&self) -> ServiceVisitPartStatus {
        self.status
    }
    pub fn voided_at(&self) -> Option<i64> {
        self.voided_at
    }
    pub fn void_reason(&self) -> Option<&str> {
        self.void_reason.as_deref()
    }
    pub fn created_at(&self) -> i64 {
        self.created_at
    }
}

pub fn calculate_line_total_fils(
    quantity: i64,
    quantity_scale: i64,
    unit_price_fils: i64,
) -> Result<i64, ServiceVisitPartValidationError> {
    if !(1..=MAX_QUANTITY).contains(&quantity) {
        return Err(ServiceVisitPartValidationError::InvalidQuantity);
    }
    if !matches!(quantity_scale, 1 | 10 | 100 | 1_000) {
        return Err(ServiceVisitPartValidationError::InvalidQuantityScale);
    }
    if !(0..=MAX_UNIT_PRICE_FILS).contains(&unit_price_fils) {
        return Err(ServiceVisitPartValidationError::InvalidUnitPrice);
    }
    let numerator = quantity
        .checked_mul(unit_price_fils)
        .ok_or(ServiceVisitPartValidationError::CalculationOverflow)?;
    numerator
        .checked_add(quantity_scale / 2)
        .ok_or(ServiceVisitPartValidationError::CalculationOverflow)
        .map(|rounded| rounded / quantity_scale)
}

fn normalize_required(
    value: String,
    maximum: usize,
    field: ServiceVisitPartTextField,
    blank: ServiceVisitPartValidationError,
    allow_formatting: bool,
) -> Result<String, ServiceVisitPartValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(blank);
    }
    validate_text(value, maximum, field, allow_formatting)?;
    Ok(value.to_string())
}

fn normalize_optional(
    value: Option<String>,
    maximum: usize,
    field: ServiceVisitPartTextField,
    allow_formatting: bool,
) -> Result<Option<String>, ServiceVisitPartValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    validate_text(value, maximum, field, allow_formatting)?;
    Ok(Some(value.to_string()))
}

fn validate_text(
    value: &str,
    maximum: usize,
    field: ServiceVisitPartTextField,
    allow_formatting: bool,
) -> Result<(), ServiceVisitPartValidationError> {
    if value.chars().count() > maximum {
        return Err(ServiceVisitPartValidationError::TextTooLong(field));
    }
    if value.chars().any(|character| {
        character.is_control() && !(allow_formatting && matches!(character, '\n' | '\r' | '\t'))
    }) {
        return Err(ServiceVisitPartValidationError::TextContainsControlCharacter(field));
    }
    Ok(())
}
