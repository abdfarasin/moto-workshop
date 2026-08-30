use serde::{Deserialize, Serialize};

use crate::{
    application::service_visit_workspace::{
        AddServiceVisitPartInput, CancelServiceVisitInput, CloseServiceVisitInput,
        CreateServiceVisitInput, InventoryItemSelection, MarkServiceVisitReadyForPickupInput,
        ReopenServiceVisitInput, ServiceVisitDetails, ServiceVisitMotorcycle, ServiceVisitOwner,
        ServiceVisitPartHistoryLine, ServiceVisitWorkspace, ServiceVisitWorkspaceError,
        ServiceVisitWorkspaceService, UpdateServiceVisitWorkInput, VoidServiceVisitPartInput,
    },
    domain::{
        service_visit::{ServiceVisitStatus, ServiceVisitValidationError},
        service_visit_part::ServiceVisitPartStatus,
    },
    runtime::database::RuntimeDatabase,
};

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitWorkspaceDto {
    pub visit: ServiceVisitDetailsDto,
    pub owner: ServiceVisitOwnerDto,
    pub motorcycle: ServiceVisitMotorcycleDto,
    pub parts: Vec<ServiceVisitPartDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitDetailsDto {
    pub id: i64,
    pub motorcycle_id: i64,
    pub owner_customer_id: i64,
    pub status: ServiceVisitStatusDto,
    pub opened_at: i64,
    pub completed_at: Option<i64>,
    pub closed_at: Option<i64>,
    pub cancelled_at: Option<i64>,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub diagnosis: Option<String>,
    pub work_performed: Option<String>,
    pub labor_charge_fils: i64,
    pub cancellation_reason: Option<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitOwnerDto {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitMotorcycleDto {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub plate_code: Option<String>,
    pub plate_number: Option<i64>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceVisitPartDto {
    pub id: i64,
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
    pub status: ServiceVisitPartStatusDto,
    pub voided_at: Option<i64>,
    pub void_reason: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItemSelectionDto {
    pub id: i64,
    pub item_name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_selling_price_fils: i64,
    pub current_quantity: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceVisitStatusDto {
    Open,
    ReadyForPickup,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServiceVisitPartStatusDto {
    Active,
    Voided,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateServiceVisitCommandInput {
    pub motorcycle_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateServiceVisitWorkCommandInput {
    pub service_visit_id: i64,
    pub diagnosis: Option<String>,
    pub work_performed: Option<String>,
    pub labor_charge_fils: i64,
    pub notes: Option<String>,
    pub odometer_km: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddServiceVisitPartCommandInput {
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub quantity: i64,
    pub unit_price_fils: i64,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoidServiceVisitPartCommandInput {
    pub service_visit_id: i64,
    pub service_visit_part_id: i64,
    pub voided_at: i64,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MarkServiceVisitReadyForPickupCommandInput {
    pub service_visit_id: i64,
    pub completed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReopenServiceVisitCommandInput {
    pub service_visit_id: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloseServiceVisitCommandInput {
    pub service_visit_id: i64,
    pub closed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CancelServiceVisitCommandInput {
    pub service_visit_id: i64,
    pub cancelled_at: i64,
    pub reason: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    pub category: CommandErrorCategory,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CommandErrorCategory {
    CustomerNotFound,
    CustomerPhoneAlreadyExists,
    MotorcycleNotFound,
    ActiveServiceVisitExists,
    ServiceVisitNotFound,
    InventoryItemNotFound,
    ServiceVisitPartNotFound,
    LifecycleRejected,
    ValidationError,
    DatabaseError,
}

#[tauri::command]
pub fn create_service_visit(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CreateServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_create_service_visit(&database, input)
}

#[tauri::command]
pub fn load_service_visit_workspace(
    database: tauri::State<'_, RuntimeDatabase>,
    service_visit_id: i64,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_load_service_visit_workspace(&database, service_visit_id)
}

#[tauri::command]
pub fn list_service_visit_inventory_items(
    database: tauri::State<'_, RuntimeDatabase>,
) -> CommandResult<Vec<InventoryItemSelectionDto>> {
    handle_list_service_visit_inventory_items(&database)
}

#[tauri::command]
pub fn update_service_visit_work(
    database: tauri::State<'_, RuntimeDatabase>,
    input: UpdateServiceVisitWorkCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_update_service_visit_work(&database, input)
}

#[tauri::command]
pub fn add_service_visit_part(
    database: tauri::State<'_, RuntimeDatabase>,
    input: AddServiceVisitPartCommandInput,
) -> CommandResult<ServiceVisitPartDto> {
    handle_add_service_visit_part(&database, input)
}

#[tauri::command]
pub fn void_service_visit_part(
    database: tauri::State<'_, RuntimeDatabase>,
    input: VoidServiceVisitPartCommandInput,
) -> CommandResult<ServiceVisitPartDto> {
    handle_void_service_visit_part(&database, input)
}

#[tauri::command]
pub fn mark_service_visit_ready_for_pickup(
    database: tauri::State<'_, RuntimeDatabase>,
    input: MarkServiceVisitReadyForPickupCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_mark_service_visit_ready_for_pickup(&database, input)
}

#[tauri::command]
pub fn reopen_service_visit(
    database: tauri::State<'_, RuntimeDatabase>,
    input: ReopenServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_reopen_service_visit(&database, input)
}

#[tauri::command]
pub fn close_service_visit(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CloseServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_close_service_visit(&database, input)
}

#[tauri::command]
pub fn cancel_service_visit(
    database: tauri::State<'_, RuntimeDatabase>,
    input: CancelServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    handle_cancel_service_visit(&database, input)
}

pub fn handle_load_service_visit_workspace(
    database: &RuntimeDatabase,
    service_visit_id: i64,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).load_workspace(service_visit_id)
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_create_service_visit(
    database: &RuntimeDatabase,
    input: CreateServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).create_service_visit(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_list_service_visit_inventory_items(
    database: &RuntimeDatabase,
) -> CommandResult<Vec<InventoryItemSelectionDto>> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).list_usable_inventory_items()
    };
    result
        .map(|items| items.into_iter().map(Into::into).collect())
        .map_err(Into::into)
}

pub fn handle_update_service_visit_work(
    database: &RuntimeDatabase,
    input: UpdateServiceVisitWorkCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).update_work(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_add_service_visit_part(
    database: &RuntimeDatabase,
    input: AddServiceVisitPartCommandInput,
) -> CommandResult<ServiceVisitPartDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).add_part(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_void_service_visit_part(
    database: &RuntimeDatabase,
    input: VoidServiceVisitPartCommandInput,
) -> CommandResult<ServiceVisitPartDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).void_part(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_mark_service_visit_ready_for_pickup(
    database: &RuntimeDatabase,
    input: MarkServiceVisitReadyForPickupCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).mark_ready_for_pickup(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_reopen_service_visit(
    database: &RuntimeDatabase,
    input: ReopenServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).reopen(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_close_service_visit(
    database: &RuntimeDatabase,
    input: CloseServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).close(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

pub fn handle_cancel_service_visit(
    database: &RuntimeDatabase,
    input: CancelServiceVisitCommandInput,
) -> CommandResult<ServiceVisitWorkspaceDto> {
    let result = {
        let mut connection = database.lock().map_err(|_| CommandError::database())?;
        ServiceVisitWorkspaceService::new(&mut connection).cancel(input.into())
    };
    result.map(Into::into).map_err(Into::into)
}

impl CommandError {
    pub(crate) fn database() -> Self {
        Self {
            category: CommandErrorCategory::DatabaseError,
            message: "The workshop database operation failed.".into(),
        }
    }
}

impl From<ServiceVisitWorkspaceError> for CommandError {
    fn from(error: ServiceVisitWorkspaceError) -> Self {
        match error {
            ServiceVisitWorkspaceError::MotorcycleNotFound(id) => Self {
                category: CommandErrorCategory::MotorcycleNotFound,
                message: format!("Motorcycle {id} was not found."),
            },
            ServiceVisitWorkspaceError::ActiveServiceVisitExists(id) => Self {
                category: CommandErrorCategory::ActiveServiceVisitExists,
                message: format!("Motorcycle {id} already has an active Service Visit."),
            },
            ServiceVisitWorkspaceError::ServiceVisitNotFound(id) => Self {
                category: CommandErrorCategory::ServiceVisitNotFound,
                message: format!("Service Visit {id} was not found."),
            },
            ServiceVisitWorkspaceError::InventoryItemNotFound(id) => Self {
                category: CommandErrorCategory::InventoryItemNotFound,
                message: format!("Usable Inventory Item {id} was not found."),
            },
            ServiceVisitWorkspaceError::ServiceVisitPartNotFound {
                service_visit_id,
                service_visit_part_id,
            } => Self {
                category: CommandErrorCategory::ServiceVisitPartNotFound,
                message: format!(
                    "Service Visit Part {service_visit_part_id} was not found on Service Visit {service_visit_id}."
                ),
            },
            ServiceVisitWorkspaceError::VisitDoesNotAllowPartChanges(_) => Self {
                category: CommandErrorCategory::LifecycleRejected,
                message: "The Service Visit status does not allow Part changes.".into(),
            },
            ServiceVisitWorkspaceError::VisitValidation(
                ServiceVisitValidationError::InvalidTransition { .. }
                | ServiceVisitValidationError::TerminalVisitCannotBeEdited,
            ) => Self {
                category: CommandErrorCategory::LifecycleRejected,
                message: "The Service Visit lifecycle transition is not allowed.".into(),
            },
            ServiceVisitWorkspaceError::VisitValidation(_)
            | ServiceVisitWorkspaceError::PartValidation(_) => Self {
                category: CommandErrorCategory::ValidationError,
                message: "The supplied Service Visit data is invalid.".into(),
            },
            ServiceVisitWorkspaceError::Database(_) => Self::database(),
        }
    }
}

impl From<ServiceVisitWorkspace> for ServiceVisitWorkspaceDto {
    fn from(workspace: ServiceVisitWorkspace) -> Self {
        Self {
            visit: workspace.visit.into(),
            owner: workspace.owner.into(),
            motorcycle: workspace.motorcycle.into(),
            parts: workspace.parts.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<ServiceVisitDetails> for ServiceVisitDetailsDto {
    fn from(visit: ServiceVisitDetails) -> Self {
        Self {
            id: visit.id,
            motorcycle_id: visit.motorcycle_id,
            owner_customer_id: visit.owner_customer_id,
            status: visit.status.into(),
            opened_at: visit.opened_at,
            completed_at: visit.completed_at,
            closed_at: visit.closed_at,
            cancelled_at: visit.cancelled_at,
            odometer_km: visit.odometer_km,
            customer_complaint: visit.customer_complaint,
            diagnosis: visit.diagnosis,
            work_performed: visit.work_performed,
            labor_charge_fils: visit.labor_charge_fils,
            cancellation_reason: visit.cancellation_reason,
            notes: visit.notes,
            created_at: visit.created_at,
            updated_at: visit.updated_at,
        }
    }
}

impl From<ServiceVisitOwner> for ServiceVisitOwnerDto {
    fn from(owner: ServiceVisitOwner) -> Self {
        Self {
            id: owner.id,
            name: owner.name,
            phone: owner.phone,
        }
    }
}

impl From<ServiceVisitMotorcycle> for ServiceVisitMotorcycleDto {
    fn from(motorcycle: ServiceVisitMotorcycle) -> Self {
        Self {
            id: motorcycle.id,
            make_name: motorcycle.make_name,
            model: motorcycle.model,
            year: motorcycle.year,
            plate_code: motorcycle.plate_code,
            plate_number: motorcycle.plate_number,
            vin: motorcycle.vin,
            chassis_number: motorcycle.chassis_number,
            color_name: motorcycle.color_name,
        }
    }
}

impl From<ServiceVisitPartHistoryLine> for ServiceVisitPartDto {
    fn from(part: ServiceVisitPartHistoryLine) -> Self {
        Self {
            id: part.id,
            service_visit_id: part.service_visit_id,
            inventory_item_id: part.inventory_item_id,
            item_name: part.item_name,
            unit_name: part.unit_name,
            quantity: part.quantity,
            quantity_scale: part.quantity_scale,
            unit_price_fils: part.unit_price_fils,
            line_total_fils: part.line_total_fils,
            status: part.status.into(),
            voided_at: part.voided_at,
            void_reason: part.void_reason,
            created_at: part.created_at,
        }
    }
}

impl From<InventoryItemSelection> for InventoryItemSelectionDto {
    fn from(item: InventoryItemSelection) -> Self {
        Self {
            id: item.id,
            item_name: item.item_name,
            sku: item.sku,
            unit_id: item.unit_id,
            unit_name: item.unit_name,
            quantity_scale: item.quantity_scale,
            default_selling_price_fils: item.default_selling_price_fils,
            current_quantity: item.current_quantity,
        }
    }
}

impl From<ServiceVisitStatus> for ServiceVisitStatusDto {
    fn from(status: ServiceVisitStatus) -> Self {
        match status {
            ServiceVisitStatus::Open => Self::Open,
            ServiceVisitStatus::ReadyForPickup => Self::ReadyForPickup,
            ServiceVisitStatus::Closed => Self::Closed,
            ServiceVisitStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<ServiceVisitPartStatus> for ServiceVisitPartStatusDto {
    fn from(status: ServiceVisitPartStatus) -> Self {
        match status {
            ServiceVisitPartStatus::Active => Self::Active,
            ServiceVisitPartStatus::Voided => Self::Voided,
        }
    }
}

impl From<UpdateServiceVisitWorkCommandInput> for UpdateServiceVisitWorkInput {
    fn from(input: UpdateServiceVisitWorkCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            diagnosis: input.diagnosis,
            work_performed: input.work_performed,
            labor_charge_fils: input.labor_charge_fils,
            notes: input.notes,
            odometer_km: input.odometer_km,
            updated_at: input.updated_at,
        }
    }
}

impl From<CreateServiceVisitCommandInput> for CreateServiceVisitInput {
    fn from(input: CreateServiceVisitCommandInput) -> Self {
        Self {
            motorcycle_id: input.motorcycle_id,
            opened_at: input.opened_at,
            odometer_km: input.odometer_km,
            customer_complaint: input.customer_complaint,
            notes: input.notes,
            created_at: input.created_at,
        }
    }
}

impl From<AddServiceVisitPartCommandInput> for AddServiceVisitPartInput {
    fn from(input: AddServiceVisitPartCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            inventory_item_id: input.inventory_item_id,
            quantity: input.quantity,
            unit_price_fils: input.unit_price_fils,
            created_at: input.created_at,
        }
    }
}

impl From<VoidServiceVisitPartCommandInput> for VoidServiceVisitPartInput {
    fn from(input: VoidServiceVisitPartCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            service_visit_part_id: input.service_visit_part_id,
            voided_at: input.voided_at,
            reason: input.reason,
        }
    }
}

impl From<MarkServiceVisitReadyForPickupCommandInput> for MarkServiceVisitReadyForPickupInput {
    fn from(input: MarkServiceVisitReadyForPickupCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            completed_at: input.completed_at,
            updated_at: input.updated_at,
        }
    }
}

impl From<ReopenServiceVisitCommandInput> for ReopenServiceVisitInput {
    fn from(input: ReopenServiceVisitCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            updated_at: input.updated_at,
        }
    }
}

impl From<CloseServiceVisitCommandInput> for CloseServiceVisitInput {
    fn from(input: CloseServiceVisitCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            closed_at: input.closed_at,
            updated_at: input.updated_at,
        }
    }
}

impl From<CancelServiceVisitCommandInput> for CancelServiceVisitInput {
    fn from(input: CancelServiceVisitCommandInput) -> Self {
        Self {
            service_visit_id: input.service_visit_id,
            cancelled_at: input.cancelled_at,
            reason: input.reason,
            updated_at: input.updated_at,
        }
    }
}
