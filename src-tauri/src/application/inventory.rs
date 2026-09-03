use crate::{
    domain::inventory::{
        InventoryItem, InventoryValidationError, NewInventoryItemInput, NewStockMovementInput,
        StockMovement, StockMovementType,
    },
    repositories::inventory::{
        InventoryManagementItemRow, InventoryRepository, InventoryUnitRow, StockMovementRow,
    },
};
use rusqlite::Connection;
use std::{error::Error, fmt};

pub const DEFAULT_INVENTORY_LIMIT: u32 = 50;
pub const MAX_INVENTORY_LIMIT: u32 = 100;
const MOVEMENT_LIMIT: i64 = 100;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchInventoryItemsInput {
    pub query: String,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInventoryItemDetailsInput {
    pub inventory_item_id: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateInventoryItemInput {
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub opening_quantity: i64,
    pub created_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInventoryItemInput {
    pub inventory_item_id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub updated_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustInventoryStockInput {
    pub inventory_item_id: i64,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub created_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItemSummary {
    pub id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub current_quantity: i64,
    pub low_stock: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItemDetails {
    pub id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub current_quantity: i64,
    pub low_stock: bool,
    pub movements: Vec<StockMovementEntry>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryUnitSummary {
    pub id: i64,
    pub name: String,
    pub quantity_scale: i64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StockMovementEntry {
    pub id: i64,
    pub movement_type: StockMovementType,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub service_visit_part_id: Option<i64>,
    pub created_at: i64,
}
#[derive(Debug)]
pub enum InventoryApplicationError {
    Validation(InventoryValidationError),
    InventoryItemNotFound,
    InventoryUnitNotFound,
    InventorySkuAlreadyExists,
    Database(rusqlite::Error),
}
impl fmt::Display for InventoryApplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "inventory operation failed")
    }
}
impl Error for InventoryApplicationError {}
impl From<InventoryValidationError> for InventoryApplicationError {
    fn from(v: InventoryValidationError) -> Self {
        Self::Validation(v)
    }
}
impl From<rusqlite::Error> for InventoryApplicationError {
    fn from(v: rusqlite::Error) -> Self {
        Self::Database(v)
    }
}

pub struct InventoryApplicationService<'a> {
    connection: &'a mut Connection,
}
impl<'a> InventoryApplicationService<'a> {
    pub fn new(connection: &'a mut Connection) -> Self {
        Self { connection }
    }
    pub fn search(
        &self,
        input: SearchInventoryItemsInput,
    ) -> Result<Vec<InventoryItemSummary>, InventoryApplicationError> {
        let limit = input
            .limit
            .unwrap_or(DEFAULT_INVENTORY_LIMIT)
            .min(MAX_INVENTORY_LIMIT);
        Ok(InventoryRepository::new(self.connection)
            .search(input.query.trim(), i64::from(limit))?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub fn list_units(&self) -> Result<Vec<InventoryUnitSummary>, InventoryApplicationError> {
        Ok(InventoryRepository::new(self.connection)
            .list_units()?
            .into_iter()
            .map(Into::into)
            .collect())
    }
    pub fn load(
        &self,
        input: LoadInventoryItemDetailsInput,
    ) -> Result<Option<InventoryItemDetails>, InventoryApplicationError> {
        load_details(self.connection, input.inventory_item_id)
    }
    pub fn create(
        &mut self,
        input: CreateInventoryItemInput,
    ) -> Result<InventoryItemDetails, InventoryApplicationError> {
        if input.created_at < 0 {
            return Err(InventoryValidationError::InvalidTimestamp.into());
        }
        if input.opening_quantity < 0 {
            return Err(InventoryValidationError::InvalidQuantityDelta {
                movement_type: StockMovementType::OpeningStock,
            }
            .into());
        }
        let transaction = self.connection.transaction()?;
        let repository = InventoryRepository::new(&transaction);
        if !repository.active_unit_exists(input.unit_id)? {
            return Err(InventoryApplicationError::InventoryUnitNotFound);
        }
        let item = InventoryItem::new(NewInventoryItemInput {
            name: input.name,
            sku: input.sku,
            unit_id: input.unit_id,
            default_purchase_price_fils: input.default_purchase_price_fils,
            default_selling_price_fils: input.default_selling_price_fils,
            minimum_stock_quantity: input.minimum_stock_quantity,
            notes: input.notes,
        })?;
        let item_id = repository
            .insert_item(&item, input.created_at)
            .map_err(classify_write_error)?;
        if input.opening_quantity > 0 {
            let movement = StockMovement::new(NewStockMovementInput {
                inventory_item_id: item_id,
                service_visit_part_id: None,
                movement_type: StockMovementType::OpeningStock,
                quantity_delta: input.opening_quantity,
                notes: Some("Opening stock".into()),
                created_at: input.created_at,
            })?;
            repository.insert_movement(&movement)?;
        }
        let details = load_details(&transaction, item_id)?
            .ok_or(InventoryApplicationError::InventoryItemNotFound)?;
        transaction.commit()?;
        Ok(details)
    }
    pub fn update(
        &mut self,
        input: UpdateInventoryItemInput,
    ) -> Result<InventoryItemDetails, InventoryApplicationError> {
        if input.updated_at < 0 {
            return Err(InventoryValidationError::InvalidTimestamp.into());
        }
        let transaction = self.connection.transaction()?;
        let repository = InventoryRepository::new(&transaction);
        let existing = repository
            .find_management_item(input.inventory_item_id)?
            .ok_or(InventoryApplicationError::InventoryItemNotFound)?;
        let item = InventoryItem::new(NewInventoryItemInput {
            name: input.name,
            sku: input.sku,
            unit_id: existing.unit_id,
            default_purchase_price_fils: input.default_purchase_price_fils,
            default_selling_price_fils: input.default_selling_price_fils,
            minimum_stock_quantity: input.minimum_stock_quantity,
            notes: input.notes,
        })?;
        repository
            .update_item(input.inventory_item_id, &item, input.updated_at)
            .map_err(classify_write_error)?;
        let details = load_details(&transaction, input.inventory_item_id)?.unwrap();
        transaction.commit()?;
        Ok(details)
    }
    pub fn adjust_stock(
        &mut self,
        input: AdjustInventoryStockInput,
    ) -> Result<InventoryItemDetails, InventoryApplicationError> {
        let transaction = self.connection.transaction()?;
        let repository = InventoryRepository::new(&transaction);
        if repository
            .find_management_item(input.inventory_item_id)?
            .is_none()
        {
            return Err(InventoryApplicationError::InventoryItemNotFound);
        }
        let movement_type = if input.quantity_delta > 0 {
            StockMovementType::AdjustmentIn
        } else {
            StockMovementType::AdjustmentOut
        };
        let movement = StockMovement::new(NewStockMovementInput {
            inventory_item_id: input.inventory_item_id,
            service_visit_part_id: None,
            movement_type,
            quantity_delta: input.quantity_delta,
            notes: input.notes,
            created_at: input.created_at,
        })?;
        repository.insert_movement(&movement)?;
        let details = load_details(&transaction, input.inventory_item_id)?.unwrap();
        transaction.commit()?;
        Ok(details)
    }
}
fn load_details(
    connection: &Connection,
    item_id: i64,
) -> Result<Option<InventoryItemDetails>, InventoryApplicationError> {
    let repository = InventoryRepository::new(connection);
    let Some(row) = repository.find_management_item(item_id)? else {
        return Ok(None);
    };
    let movements = repository
        .list_movements(item_id, MOVEMENT_LIMIT)?
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(Some(InventoryItemDetails {
        movements,
        ..InventoryItemSummary::from(row).into()
    }))
}
impl From<InventoryManagementItemRow> for InventoryItemSummary {
    fn from(r: InventoryManagementItemRow) -> Self {
        let low_stock = r.current_quantity <= r.minimum_stock_quantity;
        Self {
            id: r.id,
            name: r.name,
            sku: r.sku,
            unit_id: r.unit_id,
            unit_name: r.unit_name,
            quantity_scale: r.quantity_scale,
            default_purchase_price_fils: r.default_purchase_price_fils,
            default_selling_price_fils: r.default_selling_price_fils,
            minimum_stock_quantity: r.minimum_stock_quantity,
            notes: r.notes,
            current_quantity: r.current_quantity,
            low_stock,
        }
    }
}
impl From<InventoryItemSummary> for InventoryItemDetails {
    fn from(r: InventoryItemSummary) -> Self {
        Self {
            id: r.id,
            name: r.name,
            sku: r.sku,
            unit_id: r.unit_id,
            unit_name: r.unit_name,
            quantity_scale: r.quantity_scale,
            default_purchase_price_fils: r.default_purchase_price_fils,
            default_selling_price_fils: r.default_selling_price_fils,
            minimum_stock_quantity: r.minimum_stock_quantity,
            notes: r.notes,
            current_quantity: r.current_quantity,
            low_stock: r.low_stock,
            movements: vec![],
        }
    }
}
impl From<InventoryUnitRow> for InventoryUnitSummary {
    fn from(r: InventoryUnitRow) -> Self {
        Self {
            id: r.id,
            name: r.name,
            quantity_scale: r.quantity_scale,
        }
    }
}
impl From<StockMovementRow> for StockMovementEntry {
    fn from(r: StockMovementRow) -> Self {
        Self {
            id: r.id,
            movement_type: r.movement_type,
            quantity_delta: r.quantity_delta,
            notes: r.notes,
            service_visit_part_id: r.service_visit_part_id,
            created_at: r.created_at,
        }
    }
}
fn classify_write_error(error: rusqlite::Error) -> InventoryApplicationError {
    if matches!(&error,rusqlite::Error::SqliteFailure(code,_) if code.extended_code==rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE)
    {
        InventoryApplicationError::InventorySkuAlreadyExists
    } else {
        InventoryApplicationError::Database(error)
    }
}
