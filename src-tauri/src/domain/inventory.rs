const MAX_STORED_QUANTITY: i64 = 1_000_000_000;
const MAX_PRICE_FILS: i64 = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryTextField {
    UnitName,
    ItemName,
    Sku,
    Notes,
    MovementNotes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StockMovementType {
    OpeningStock,
    Purchase,
    AdjustmentIn,
    AdjustmentOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryValidationError {
    BlankUnitName,
    BlankItemName,
    InvalidQuantityScale,
    InvalidUnitId,
    InvalidPurchasePrice,
    InvalidSellingPrice,
    InvalidMinimumStock,
    InvalidInventoryItemId,
    InvalidTimestamp,
    InvalidQuantityDelta { movement_type: StockMovementType },
    TextTooLong(InventoryTextField),
    TextContainsControlCharacter(InventoryTextField),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuantityScale(i64);

impl QuantityScale {
    pub fn new(value: i64) -> Result<Self, InventoryValidationError> {
        if !matches!(value, 1 | 10 | 100 | 1_000) {
            return Err(InventoryValidationError::InvalidQuantityScale);
        }
        Ok(Self(value))
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewInventoryUnitInput {
    pub name: String,
    pub quantity_scale: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryUnit {
    name: String,
    quantity_scale: QuantityScale,
}

impl InventoryUnit {
    pub fn new(input: NewInventoryUnitInput) -> Result<Self, InventoryValidationError> {
        let name = normalize_required_text(
            input.name,
            40,
            InventoryTextField::UnitName,
            InventoryValidationError::BlankUnitName,
            false,
        )?;
        let quantity_scale = QuantityScale::new(input.quantity_scale)?;

        Ok(Self {
            name,
            quantity_scale,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn quantity_scale(&self) -> QuantityScale {
        self.quantity_scale
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewInventoryItemInput {
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InventoryItem {
    name: String,
    sku: Option<String>,
    unit_id: i64,
    default_purchase_price_fils: Option<i64>,
    default_selling_price_fils: i64,
    minimum_stock_quantity: i64,
    notes: Option<String>,
}

impl InventoryItem {
    pub fn new(input: NewInventoryItemInput) -> Result<Self, InventoryValidationError> {
        if input.unit_id <= 0 {
            return Err(InventoryValidationError::InvalidUnitId);
        }
        if input
            .default_purchase_price_fils
            .is_some_and(|price| !(0..=MAX_PRICE_FILS).contains(&price))
        {
            return Err(InventoryValidationError::InvalidPurchasePrice);
        }
        if !(0..=MAX_PRICE_FILS).contains(&input.default_selling_price_fils) {
            return Err(InventoryValidationError::InvalidSellingPrice);
        }
        if !(0..=MAX_STORED_QUANTITY).contains(&input.minimum_stock_quantity) {
            return Err(InventoryValidationError::InvalidMinimumStock);
        }

        let name = normalize_required_text(
            input.name,
            150,
            InventoryTextField::ItemName,
            InventoryValidationError::BlankItemName,
            false,
        )?;
        let sku = normalize_optional_text(input.sku, 64, InventoryTextField::Sku, false)?;
        let notes = normalize_optional_text(input.notes, 2_000, InventoryTextField::Notes, true)?;

        Ok(Self {
            name,
            sku,
            unit_id: input.unit_id,
            default_purchase_price_fils: input.default_purchase_price_fils,
            default_selling_price_fils: input.default_selling_price_fils,
            minimum_stock_quantity: input.minimum_stock_quantity,
            notes,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn sku(&self) -> Option<&str> {
        self.sku.as_deref()
    }

    pub fn unit_id(&self) -> i64 {
        self.unit_id
    }

    pub fn default_purchase_price_fils(&self) -> Option<i64> {
        self.default_purchase_price_fils
    }

    pub fn default_selling_price_fils(&self) -> i64 {
        self.default_selling_price_fils
    }

    pub fn minimum_stock_quantity(&self) -> i64 {
        self.minimum_stock_quantity
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewStockMovementInput {
    pub inventory_item_id: i64,
    pub movement_type: StockMovementType,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct StockMovement {
    inventory_item_id: i64,
    movement_type: StockMovementType,
    quantity_delta: i64,
    notes: Option<String>,
    created_at: i64,
}

impl StockMovement {
    pub fn new(input: NewStockMovementInput) -> Result<Self, InventoryValidationError> {
        if input.inventory_item_id <= 0 {
            return Err(InventoryValidationError::InvalidInventoryItemId);
        }
        if input.created_at < 0 {
            return Err(InventoryValidationError::InvalidTimestamp);
        }

        let valid_quantity = match input.movement_type {
            StockMovementType::OpeningStock
            | StockMovementType::Purchase
            | StockMovementType::AdjustmentIn => {
                (1..=MAX_STORED_QUANTITY).contains(&input.quantity_delta)
            }
            StockMovementType::AdjustmentOut => {
                (-MAX_STORED_QUANTITY..=-1).contains(&input.quantity_delta)
            }
        };
        if !valid_quantity {
            return Err(InventoryValidationError::InvalidQuantityDelta {
                movement_type: input.movement_type,
            });
        }

        let notes =
            normalize_optional_text(input.notes, 2_000, InventoryTextField::MovementNotes, true)?;

        Ok(Self {
            inventory_item_id: input.inventory_item_id,
            movement_type: input.movement_type,
            quantity_delta: input.quantity_delta,
            notes,
            created_at: input.created_at,
        })
    }

    pub fn inventory_item_id(&self) -> i64 {
        self.inventory_item_id
    }

    pub fn movement_type(&self) -> StockMovementType {
        self.movement_type
    }

    pub fn quantity_delta(&self) -> i64 {
        self.quantity_delta
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }

    pub fn created_at(&self) -> i64 {
        self.created_at
    }
}

fn normalize_required_text(
    value: String,
    maximum_characters: usize,
    field: InventoryTextField,
    blank_error: InventoryValidationError,
    allow_formatting_controls: bool,
) -> Result<String, InventoryValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(blank_error);
    }
    validate_text(value, maximum_characters, field, allow_formatting_controls)?;
    Ok(value.to_string())
}

fn normalize_optional_text(
    value: Option<String>,
    maximum_characters: usize,
    field: InventoryTextField,
    allow_formatting_controls: bool,
) -> Result<Option<String>, InventoryValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    validate_text(value, maximum_characters, field, allow_formatting_controls)?;
    Ok(Some(value.to_string()))
}

fn validate_text(
    value: &str,
    maximum_characters: usize,
    field: InventoryTextField,
    allow_formatting_controls: bool,
) -> Result<(), InventoryValidationError> {
    if value.chars().count() > maximum_characters {
        return Err(InventoryValidationError::TextTooLong(field));
    }
    if value.chars().any(|character| {
        character.is_control()
            && !(allow_formatting_controls && matches!(character, '\n' | '\r' | '\t'))
    }) {
        return Err(InventoryValidationError::TextContainsControlCharacter(
            field,
        ));
    }
    Ok(())
}
