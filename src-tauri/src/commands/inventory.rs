use crate::{
    application::inventory::{
        AdjustInventoryStockInput, CreateInventoryItemInput, InventoryApplicationError,
        InventoryApplicationService, InventoryItemDetails, InventoryItemSummary,
        InventoryUnitSummary, LoadInventoryItemDetailsInput, SearchInventoryItemsInput,
        StockMovementEntry, UpdateInventoryItemInput,
    },
    commands::service_visit_workspace::{CommandError, CommandErrorCategory, CommandResult},
    domain::inventory::StockMovementType,
    runtime::database::RuntimeDatabase,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SearchInventoryItemsCommandInput {
    pub query: String,
    pub limit: Option<u32>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LoadInventoryItemDetailsCommandInput {
    pub inventory_item_id: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateInventoryItemCommandInput {
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
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateInventoryItemCommandInput {
    pub inventory_item_id: i64,
    pub name: String,
    pub sku: Option<String>,
    pub default_purchase_price_fils: Option<i64>,
    pub default_selling_price_fils: i64,
    pub minimum_stock_quantity: i64,
    pub notes: Option<String>,
    pub updated_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AdjustInventoryStockCommandInput {
    pub inventory_item_id: i64,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub created_at: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemDto {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemDetailsDto {
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
    pub movements: Vec<StockMovementDto>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryUnitDto {
    pub id: i64,
    pub name: String,
    pub quantity_scale: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovementDto {
    pub id: i64,
    pub movement_type: StockMovementTypeDto,
    pub quantity_delta: i64,
    pub notes: Option<String>,
    pub service_visit_part_id: Option<i64>,
    pub created_at: i64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StockMovementTypeDto {
    OpeningStock,
    Purchase,
    AdjustmentIn,
    AdjustmentOut,
    ServiceUsage,
    ServiceUsageReversal,
}

#[tauri::command]
pub fn search_inventory_items(
    database: tauri::State<'_, RuntimeDatabase>,
    input: SearchInventoryItemsCommandInput,
) -> CommandResult<Vec<InventoryItemDto>> {
    handle_search_inventory_items(&database, input)
}
pub fn handle_search_inventory_items(
    database: &RuntimeDatabase,
    input: SearchInventoryItemsCommandInput,
) -> CommandResult<Vec<InventoryItemDto>> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InventoryApplicationService::new(&mut connection)
        .search(input.into())
        .map(|r| r.into_iter().map(Into::into).collect())
        .map_err(map_error)
}
#[tauri::command]
pub fn load_inventory_item_details(
    database: tauri::State<'_, RuntimeDatabase>,
    input: LoadInventoryItemDetailsCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    handle_load_inventory_item_details(&database, input)
}
pub fn handle_load_inventory_item_details(
    database: &RuntimeDatabase,
    input: LoadInventoryItemDetailsCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    match InventoryApplicationService::new(&mut connection).load(input.into()) {
        Ok(Some(v)) => Ok(v.into()),
        Ok(None) => Err(not_found()),
        Err(e) => Err(map_error(e)),
    }
}
#[tauri::command]
pub fn list_inventory_units(
    database: tauri::State<'_, RuntimeDatabase>,
) -> CommandResult<Vec<InventoryUnitDto>> {
    handle_list_inventory_units(&database)
}
pub fn handle_list_inventory_units(
    database: &RuntimeDatabase,
) -> CommandResult<Vec<InventoryUnitDto>> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InventoryApplicationService::new(&mut connection)
        .list_units()
        .map(|r| r.into_iter().map(Into::into).collect())
        .map_err(map_error)
}
#[tauri::command]
pub fn create_inventory_item(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CreateInventoryItemCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    handle_create_inventory_item(&database, input)
}
pub fn handle_create_inventory_item(
    database: &RuntimeDatabase,
    input: CreateInventoryItemCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InventoryApplicationService::new(&mut connection)
        .create(input.into())
        .map(Into::into)
        .map_err(map_error)
}
#[tauri::command]
pub fn update_inventory_item(
    database: tauri::State<'_, RuntimeDatabase>,
    input: UpdateInventoryItemCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    handle_update_inventory_item(&database, input)
}
pub fn handle_update_inventory_item(
    database: &RuntimeDatabase,
    input: UpdateInventoryItemCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InventoryApplicationService::new(&mut connection)
        .update(input.into())
        .map(Into::into)
        .map_err(map_error)
}
#[tauri::command]
pub fn adjust_inventory_stock(
    database: tauri::State<'_, RuntimeDatabase>,
    input: AdjustInventoryStockCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    handle_adjust_inventory_stock(&database, input)
}
pub fn handle_adjust_inventory_stock(
    database: &RuntimeDatabase,
    input: AdjustInventoryStockCommandInput,
) -> CommandResult<InventoryItemDetailsDto> {
    let mut connection = database.lock().map_err(|_| CommandError::database())?;
    InventoryApplicationService::new(&mut connection)
        .adjust_stock(input.into())
        .map(Into::into)
        .map_err(map_error)
}
fn not_found() -> CommandError {
    CommandError {
        category: CommandErrorCategory::InventoryItemNotFound,
        message: "The Inventory Item could not be found.".into(),
    }
}
fn map_error(error: InventoryApplicationError) -> CommandError {
    match error {
        InventoryApplicationError::InventoryItemNotFound => not_found(),
        InventoryApplicationError::InventoryUnitNotFound => CommandError {
            category: CommandErrorCategory::InventoryUnitNotFound,
            message: "The Inventory Unit could not be found.".into(),
        },
        InventoryApplicationError::InventorySkuAlreadyExists => CommandError {
            category: CommandErrorCategory::InventorySkuAlreadyExists,
            message: "An Inventory Item with this SKU already exists.".into(),
        },
        InventoryApplicationError::Validation(_) => CommandError {
            category: CommandErrorCategory::ValidationError,
            message: "The Inventory input is invalid.".into(),
        },
        InventoryApplicationError::Database(_) => CommandError::database(),
    }
}
impl From<SearchInventoryItemsCommandInput> for SearchInventoryItemsInput {
    fn from(i: SearchInventoryItemsCommandInput) -> Self {
        Self {
            query: i.query,
            limit: i.limit,
        }
    }
}
impl From<LoadInventoryItemDetailsCommandInput> for LoadInventoryItemDetailsInput {
    fn from(i: LoadInventoryItemDetailsCommandInput) -> Self {
        Self {
            inventory_item_id: i.inventory_item_id,
        }
    }
}
impl From<CreateInventoryItemCommandInput> for CreateInventoryItemInput {
    fn from(i: CreateInventoryItemCommandInput) -> Self {
        Self {
            name: i.name,
            sku: i.sku,
            unit_id: i.unit_id,
            default_purchase_price_fils: i.default_purchase_price_fils,
            default_selling_price_fils: i.default_selling_price_fils,
            minimum_stock_quantity: i.minimum_stock_quantity,
            notes: i.notes,
            opening_quantity: i.opening_quantity,
            created_at: i.created_at,
        }
    }
}
impl From<UpdateInventoryItemCommandInput> for UpdateInventoryItemInput {
    fn from(i: UpdateInventoryItemCommandInput) -> Self {
        Self {
            inventory_item_id: i.inventory_item_id,
            name: i.name,
            sku: i.sku,
            default_purchase_price_fils: i.default_purchase_price_fils,
            default_selling_price_fils: i.default_selling_price_fils,
            minimum_stock_quantity: i.minimum_stock_quantity,
            notes: i.notes,
            updated_at: i.updated_at,
        }
    }
}
impl From<AdjustInventoryStockCommandInput> for AdjustInventoryStockInput {
    fn from(i: AdjustInventoryStockCommandInput) -> Self {
        Self {
            inventory_item_id: i.inventory_item_id,
            quantity_delta: i.quantity_delta,
            notes: i.notes,
            created_at: i.created_at,
        }
    }
}
impl From<InventoryItemSummary> for InventoryItemDto {
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
        }
    }
}
impl From<InventoryItemDetails> for InventoryItemDetailsDto {
    fn from(r: InventoryItemDetails) -> Self {
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
            movements: r.movements.into_iter().map(Into::into).collect(),
        }
    }
}
impl From<InventoryUnitSummary> for InventoryUnitDto {
    fn from(r: InventoryUnitSummary) -> Self {
        Self {
            id: r.id,
            name: r.name,
            quantity_scale: r.quantity_scale,
        }
    }
}
impl From<StockMovementEntry> for StockMovementDto {
    fn from(r: StockMovementEntry) -> Self {
        Self {
            id: r.id,
            movement_type: r.movement_type.into(),
            quantity_delta: r.quantity_delta,
            notes: r.notes,
            service_visit_part_id: r.service_visit_part_id,
            created_at: r.created_at,
        }
    }
}
impl From<StockMovementType> for StockMovementTypeDto {
    fn from(v: StockMovementType) -> Self {
        match v {
            StockMovementType::OpeningStock => Self::OpeningStock,
            StockMovementType::Purchase => Self::Purchase,
            StockMovementType::AdjustmentIn => Self::AdjustmentIn,
            StockMovementType::AdjustmentOut => Self::AdjustmentOut,
            StockMovementType::ServiceUsage => Self::ServiceUsage,
            StockMovementType::ServiceUsageReversal => Self::ServiceUsageReversal,
        }
    }
}
