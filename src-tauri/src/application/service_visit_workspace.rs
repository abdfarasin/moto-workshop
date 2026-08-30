use std::{error::Error, fmt};

use rusqlite::{Connection, TransactionBehavior};

use crate::{
    domain::{
        service_visit::{
            NewServiceVisitInput, ServiceVisit, ServiceVisitDetailsInput, ServiceVisitStatus,
            ServiceVisitValidationError,
        },
        service_visit_part::{
            NewServiceVisitPartInput, ServiceVisitPart, ServiceVisitPartStatus,
            ServiceVisitPartValidationError,
        },
    },
    repositories::{
        inventory::{InventoryItemRow, InventoryRepository},
        service_visit::{
            MotorcycleRow, OwnerRow, ServiceVisitLifecycleFields, ServiceVisitPartRow,
            ServiceVisitRepository, ServiceVisitRow, ServiceVisitWorkFields,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitWorkspace {
    pub visit: ServiceVisitDetails,
    pub owner: ServiceVisitOwner,
    pub motorcycle: ServiceVisitMotorcycle,
    pub parts: Vec<ServiceVisitPartHistoryLine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitDetails {
    pub id: i64,
    pub motorcycle_id: i64,
    pub owner_customer_id: i64,
    pub status: ServiceVisitStatus,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitOwner {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitMotorcycle {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceVisitPartHistoryLine {
    pub id: i64,
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub item_name: String,
    pub unit_name: String,
    pub quantity: i64,
    pub quantity_scale: i64,
    pub unit_price_fils: i64,
    pub line_total_fils: i64,
    pub status: ServiceVisitPartStatus,
    pub voided_at: Option<i64>,
    pub void_reason: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InventoryItemSelection {
    pub id: i64,
    pub item_name: String,
    pub sku: Option<String>,
    pub unit_id: i64,
    pub unit_name: String,
    pub quantity_scale: i64,
    pub default_selling_price_fils: i64,
    pub current_quantity: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CreateServiceVisitInput {
    pub motorcycle_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub notes: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UpdateServiceVisitWorkInput {
    pub service_visit_id: i64,
    pub diagnosis: Option<String>,
    pub work_performed: Option<String>,
    pub labor_charge_fils: i64,
    pub notes: Option<String>,
    pub odometer_km: Option<i64>,
    pub updated_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct AddServiceVisitPartInput {
    pub service_visit_id: i64,
    pub inventory_item_id: i64,
    pub quantity: i64,
    pub unit_price_fils: i64,
    pub created_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct VoidServiceVisitPartInput {
    pub service_visit_id: i64,
    pub service_visit_part_id: i64,
    pub voided_at: i64,
    pub reason: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MarkServiceVisitReadyForPickupInput {
    pub service_visit_id: i64,
    pub completed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ReopenServiceVisitInput {
    pub service_visit_id: i64,
    pub updated_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CloseServiceVisitInput {
    pub service_visit_id: i64,
    pub closed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CancelServiceVisitInput {
    pub service_visit_id: i64,
    pub cancelled_at: i64,
    pub reason: String,
    pub updated_at: i64,
}

#[derive(Debug)]
pub enum ServiceVisitWorkspaceError {
    MotorcycleNotFound(i64),
    ActiveServiceVisitExists(i64),
    ServiceVisitNotFound(i64),
    InventoryItemNotFound(i64),
    ServiceVisitPartNotFound {
        service_visit_id: i64,
        service_visit_part_id: i64,
    },
    VisitDoesNotAllowPartChanges(ServiceVisitStatus),
    VisitValidation(ServiceVisitValidationError),
    PartValidation(ServiceVisitPartValidationError),
    Database(rusqlite::Error),
}

impl fmt::Display for ServiceVisitWorkspaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MotorcycleNotFound(id) => write!(formatter, "motorcycle {id} was not found"),
            Self::ActiveServiceVisitExists(id) => {
                write!(formatter, "motorcycle {id} already has an active service visit")
            }
            Self::ServiceVisitNotFound(id) => write!(formatter, "service visit {id} was not found"),
            Self::InventoryItemNotFound(id) => {
                write!(formatter, "usable inventory item {id} was not found")
            }
            Self::ServiceVisitPartNotFound {
                service_visit_id,
                service_visit_part_id,
            } => write!(
                formatter,
                "service visit part {service_visit_part_id} was not found on visit {service_visit_id}"
            ),
            Self::VisitDoesNotAllowPartChanges(status) => {
                write!(formatter, "service visit status {status:?} does not allow part changes")
            }
            Self::VisitValidation(error) => write!(formatter, "invalid service visit: {error:?}"),
            Self::PartValidation(error) => write!(formatter, "invalid service visit part: {error:?}"),
            Self::Database(error) => write!(formatter, "database operation failed: {error}"),
        }
    }
}

impl Error for ServiceVisitWorkspaceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for ServiceVisitWorkspaceError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

impl From<ServiceVisitValidationError> for ServiceVisitWorkspaceError {
    fn from(error: ServiceVisitValidationError) -> Self {
        Self::VisitValidation(error)
    }
}

impl From<ServiceVisitPartValidationError> for ServiceVisitWorkspaceError {
    fn from(error: ServiceVisitPartValidationError) -> Self {
        Self::PartValidation(error)
    }
}

pub struct ServiceVisitWorkspaceService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> ServiceVisitWorkspaceService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn load_workspace(
        &self,
        service_visit_id: i64,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        load_workspace(self.connection, service_visit_id)
    }

    pub fn list_usable_inventory_items(
        &self,
    ) -> Result<Vec<InventoryItemSelection>, ServiceVisitWorkspaceError> {
        Ok(InventoryRepository::new(self.connection)
            .list_usable()?
            .into_iter()
            .map(InventoryItemSelection::from)
            .collect())
    }

    pub fn create_service_visit(
        &mut self,
        input: CreateServiceVisitInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        if input.created_at < 0 {
            return Err(ServiceVisitValidationError::InvalidTimestamp.into());
        }
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let repository = ServiceVisitRepository::new(&transaction);
        let owner_customer_id = repository
            .find_motorcycle_owner(input.motorcycle_id)?
            .ok_or(ServiceVisitWorkspaceError::MotorcycleNotFound(
                input.motorcycle_id,
            ))?;
        if repository
            .find_active_visit_id(input.motorcycle_id)?
            .is_some()
        {
            return Err(ServiceVisitWorkspaceError::ActiveServiceVisitExists(
                input.motorcycle_id,
            ));
        }
        let visit = ServiceVisit::open(NewServiceVisitInput {
            motorcycle_id: input.motorcycle_id,
            owner_customer_id,
            opened_at: input.opened_at,
            odometer_km: input.odometer_km,
            customer_complaint: input.customer_complaint,
            notes: input.notes,
        })?;
        let service_visit_id = repository.insert_service_visit(&visit, input.created_at)?;
        let workspace = load_workspace(&transaction, service_visit_id)?;
        transaction.commit()?;
        Ok(workspace)
    }

    pub fn update_work(
        &mut self,
        input: UpdateServiceVisitWorkInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        if input.updated_at < 0 {
            return Err(ServiceVisitValidationError::InvalidTimestamp.into());
        }
        let transaction = self.connection.transaction()?;
        let repository = ServiceVisitRepository::new(&transaction);
        let header = repository
            .find_workspace_header(input.service_visit_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitNotFound(
                input.service_visit_id,
            ))?;
        let mut visit = restore_visit(&header.visit)?;
        visit.update_details(ServiceVisitDetailsInput {
            diagnosis: input.diagnosis,
            work_performed: input.work_performed,
            labor_charge_fils: input.labor_charge_fils,
            notes: input.notes,
            odometer_km: input.odometer_km,
        })?;
        repository.update_work(
            input.service_visit_id,
            ServiceVisitWorkFields {
                diagnosis: visit.diagnosis(),
                work_performed: visit.work_performed(),
                labor_charge_fils: visit.labor_charge_fils(),
                notes: visit.notes(),
                odometer_km: visit.odometer_km(),
                updated_at: input.updated_at,
            },
        )?;
        let workspace = load_workspace(&transaction, input.service_visit_id)?;
        transaction.commit()?;
        Ok(workspace)
    }

    pub fn mark_ready_for_pickup(
        &mut self,
        input: MarkServiceVisitReadyForPickupInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        self.transition_lifecycle(input.service_visit_id, input.updated_at, |visit| {
            visit.mark_ready_for_pickup(input.completed_at)
        })
    }

    pub fn reopen(
        &mut self,
        input: ReopenServiceVisitInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        self.transition_lifecycle(
            input.service_visit_id,
            input.updated_at,
            ServiceVisit::reopen,
        )
    }

    pub fn close(
        &mut self,
        input: CloseServiceVisitInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        self.transition_lifecycle(input.service_visit_id, input.updated_at, |visit| {
            visit.close(input.closed_at)
        })
    }

    pub fn cancel(
        &mut self,
        input: CancelServiceVisitInput,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        self.transition_lifecycle(input.service_visit_id, input.updated_at, |visit| {
            visit.cancel(input.cancelled_at, input.reason)
        })
    }

    fn transition_lifecycle(
        &mut self,
        service_visit_id: i64,
        updated_at: i64,
        transition: impl FnOnce(&mut ServiceVisit) -> Result<(), ServiceVisitValidationError>,
    ) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
        if updated_at < 0 {
            return Err(ServiceVisitValidationError::InvalidTimestamp.into());
        }
        let transaction = self.connection.transaction()?;
        let repository = ServiceVisitRepository::new(&transaction);
        let header = repository.find_workspace_header(service_visit_id)?.ok_or(
            ServiceVisitWorkspaceError::ServiceVisitNotFound(service_visit_id),
        )?;
        let mut visit = restore_visit(&header.visit)?;
        transition(&mut visit)?;
        repository.update_lifecycle(
            service_visit_id,
            ServiceVisitLifecycleFields {
                status: visit.status(),
                completed_at: visit.completed_at(),
                closed_at: visit.closed_at(),
                cancelled_at: visit.cancelled_at(),
                cancellation_reason: visit.cancellation_reason(),
                updated_at,
            },
        )?;
        let workspace = load_workspace(&transaction, service_visit_id)?;
        transaction.commit()?;
        Ok(workspace)
    }

    pub fn add_part(
        &mut self,
        input: AddServiceVisitPartInput,
    ) -> Result<ServiceVisitPartHistoryLine, ServiceVisitWorkspaceError> {
        let transaction = self.connection.transaction()?;
        let visit_repository = ServiceVisitRepository::new(&transaction);
        let header = visit_repository
            .find_workspace_header(input.service_visit_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitNotFound(
                input.service_visit_id,
            ))?;
        require_part_changes_allowed(header.visit.status)?;
        let item = InventoryRepository::new(&transaction)
            .find_usable(input.inventory_item_id)?
            .ok_or(ServiceVisitWorkspaceError::InventoryItemNotFound(
                input.inventory_item_id,
            ))?;
        let part = ServiceVisitPart::new(NewServiceVisitPartInput {
            service_visit_id: input.service_visit_id,
            inventory_item_id: item.id,
            item_name: item.name,
            unit_name: item.unit_name,
            quantity: input.quantity,
            quantity_scale: item.quantity_scale,
            unit_price_fils: input.unit_price_fils,
            created_at: input.created_at,
        })?;
        let part_id = visit_repository.insert_part(&part)?;
        let inserted = visit_repository
            .find_part(input.service_visit_id, part_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitPartNotFound {
                service_visit_id: input.service_visit_id,
                service_visit_part_id: part_id,
            })?;
        transaction.commit()?;
        Ok(inserted.into())
    }

    pub fn void_part(
        &mut self,
        input: VoidServiceVisitPartInput,
    ) -> Result<ServiceVisitPartHistoryLine, ServiceVisitWorkspaceError> {
        let transaction = self.connection.transaction()?;
        let repository = ServiceVisitRepository::new(&transaction);
        let header = repository
            .find_workspace_header(input.service_visit_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitNotFound(
                input.service_visit_id,
            ))?;
        require_part_changes_allowed(header.visit.status)?;
        let persisted = repository
            .find_part(input.service_visit_id, input.service_visit_part_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitPartNotFound {
                service_visit_id: input.service_visit_id,
                service_visit_part_id: input.service_visit_part_id,
            })?;
        if persisted.status == ServiceVisitPartStatus::Voided {
            return Err(ServiceVisitPartValidationError::PartAlreadyVoided.into());
        }
        let mut part = ServiceVisitPart::new(NewServiceVisitPartInput {
            service_visit_id: persisted.service_visit_id,
            inventory_item_id: persisted.inventory_item_id,
            item_name: persisted.item_name,
            unit_name: persisted.unit_name,
            quantity: persisted.quantity,
            quantity_scale: persisted.quantity_scale,
            unit_price_fils: persisted.unit_price_fils,
            created_at: persisted.created_at,
        })?;
        part.void(input.voided_at, input.reason)?;
        repository.void_part(
            input.service_visit_id,
            input.service_visit_part_id,
            input.voided_at,
            part.void_reason(),
        )?;
        let updated = repository
            .find_part(input.service_visit_id, input.service_visit_part_id)?
            .ok_or(ServiceVisitWorkspaceError::ServiceVisitPartNotFound {
                service_visit_id: input.service_visit_id,
                service_visit_part_id: input.service_visit_part_id,
            })?;
        transaction.commit()?;
        Ok(updated.into())
    }
}

fn load_workspace(
    connection: &Connection,
    service_visit_id: i64,
) -> Result<ServiceVisitWorkspace, ServiceVisitWorkspaceError> {
    let repository = ServiceVisitRepository::new(connection);
    let header = repository.find_workspace_header(service_visit_id)?.ok_or(
        ServiceVisitWorkspaceError::ServiceVisitNotFound(service_visit_id),
    )?;
    let parts = repository
        .list_parts(service_visit_id)?
        .into_iter()
        .map(ServiceVisitPartHistoryLine::from)
        .collect();
    Ok(ServiceVisitWorkspace {
        visit: header.visit.into(),
        owner: header.owner.into(),
        motorcycle: header.motorcycle.into(),
        parts,
    })
}

fn restore_visit(row: &ServiceVisitRow) -> Result<ServiceVisit, ServiceVisitValidationError> {
    let mut visit = ServiceVisit::open(NewServiceVisitInput {
        motorcycle_id: row.motorcycle_id,
        owner_customer_id: row.owner_customer_id,
        opened_at: row.opened_at,
        odometer_km: row.odometer_km,
        customer_complaint: row.customer_complaint.clone(),
        notes: row.notes.clone(),
    })?;
    visit.update_details(ServiceVisitDetailsInput {
        diagnosis: row.diagnosis.clone(),
        work_performed: row.work_performed.clone(),
        labor_charge_fils: row.labor_charge_fils,
        notes: row.notes.clone(),
        odometer_km: row.odometer_km,
    })?;
    match row.status {
        ServiceVisitStatus::Open => {}
        ServiceVisitStatus::ReadyForPickup => visit.mark_ready_for_pickup(
            row.completed_at
                .ok_or(ServiceVisitValidationError::InvalidTimestamp)?,
        )?,
        ServiceVisitStatus::Closed => {
            visit.mark_ready_for_pickup(
                row.completed_at
                    .ok_or(ServiceVisitValidationError::InvalidTimestamp)?,
            )?;
            visit.close(
                row.closed_at
                    .ok_or(ServiceVisitValidationError::InvalidTimestamp)?,
            )?;
        }
        ServiceVisitStatus::Cancelled => visit.cancel(
            row.cancelled_at
                .ok_or(ServiceVisitValidationError::InvalidTimestamp)?,
            row.cancellation_reason
                .clone()
                .ok_or(ServiceVisitValidationError::BlankCancellationReason)?,
        )?,
    }
    Ok(visit)
}

fn require_part_changes_allowed(
    status: ServiceVisitStatus,
) -> Result<(), ServiceVisitWorkspaceError> {
    if matches!(
        status,
        ServiceVisitStatus::Open | ServiceVisitStatus::ReadyForPickup
    ) {
        Ok(())
    } else {
        Err(ServiceVisitWorkspaceError::VisitDoesNotAllowPartChanges(
            status,
        ))
    }
}

impl From<ServiceVisitRow> for ServiceVisitDetails {
    fn from(row: ServiceVisitRow) -> Self {
        Self {
            id: row.id,
            motorcycle_id: row.motorcycle_id,
            owner_customer_id: row.owner_customer_id,
            status: row.status,
            opened_at: row.opened_at,
            completed_at: row.completed_at,
            closed_at: row.closed_at,
            cancelled_at: row.cancelled_at,
            odometer_km: row.odometer_km,
            customer_complaint: row.customer_complaint,
            diagnosis: row.diagnosis,
            work_performed: row.work_performed,
            labor_charge_fils: row.labor_charge_fils,
            cancellation_reason: row.cancellation_reason,
            notes: row.notes,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<OwnerRow> for ServiceVisitOwner {
    fn from(row: OwnerRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            phone: row.phone,
        }
    }
}

impl From<MotorcycleRow> for ServiceVisitMotorcycle {
    fn from(row: MotorcycleRow) -> Self {
        Self {
            id: row.id,
            make_name: row.make_name,
            model: row.model,
            year: row.year,
            plate_code: row.plate_code,
            plate_number: row.plate_number,
            vin: row.vin,
            chassis_number: row.chassis_number,
            color_name: row.color_name,
        }
    }
}

impl From<ServiceVisitPartRow> for ServiceVisitPartHistoryLine {
    fn from(row: ServiceVisitPartRow) -> Self {
        Self {
            id: row.id,
            service_visit_id: row.service_visit_id,
            inventory_item_id: row.inventory_item_id,
            item_name: row.item_name,
            unit_name: row.unit_name,
            quantity: row.quantity,
            quantity_scale: row.quantity_scale,
            unit_price_fils: row.unit_price_fils,
            line_total_fils: row.line_total_fils,
            status: row.status,
            voided_at: row.voided_at,
            void_reason: row.void_reason,
            created_at: row.created_at,
        }
    }
}

impl From<InventoryItemRow> for InventoryItemSelection {
    fn from(row: InventoryItemRow) -> Self {
        Self {
            id: row.id,
            item_name: row.name,
            sku: row.sku,
            unit_id: row.unit_id,
            unit_name: row.unit_name,
            quantity_scale: row.quantity_scale,
            default_selling_price_fils: row.default_selling_price_fils,
            current_quantity: row.current_quantity,
        }
    }
}
