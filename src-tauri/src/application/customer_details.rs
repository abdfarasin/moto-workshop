use std::{error::Error, fmt};

use rusqlite::Connection;

use crate::repositories::customer_details::{
    CustomerDetailsMotorcycleRow, CustomerDetailsRepository, CustomerDetailsRow,
    CustomerServiceHistoryRow,
};

const CUSTOMER_DETAILS_MOTORCYCLE_LIMIT: i64 = 100;
const CUSTOMER_DETAILS_SERVICE_HISTORY_LIMIT: i64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadCustomerDetailsInput {
    pub customer_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDetails {
    pub id: i64,
    pub name: String,
    pub phone: String,
    pub motorcycles: Vec<CustomerDetailsMotorcycle>,
    pub service_history: Vec<CustomerServiceHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDetailsMotorcycle {
    pub id: i64,
    pub make_name: String,
    pub model: String,
    pub year: Option<i64>,
    pub plate_number: Option<String>,
    pub vin: Option<String>,
    pub chassis_number: Option<String>,
    pub color_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerServiceHistoryEntry {
    pub id: i64,
    pub motorcycle_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: String,
    pub total_fils: i64,
}

#[derive(Debug)]
pub enum CustomerDetailsApplicationError {
    Database(rusqlite::Error),
}

impl fmt::Display for CustomerDetailsApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => {
                write!(formatter, "database operation failed: {error}",)
            }
        }
    }
}

impl Error for CustomerDetailsApplicationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
        }
    }
}

impl From<rusqlite::Error> for CustomerDetailsApplicationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub struct CustomerDetailsApplicationService<'connection> {
    connection: &'connection mut Connection,
}

impl<'connection> CustomerDetailsApplicationService<'connection> {
    pub fn new(connection: &'connection mut Connection) -> Self {
        Self { connection }
    }

    pub fn load(
        &self,
        input: LoadCustomerDetailsInput,
    ) -> Result<Option<CustomerDetails>, CustomerDetailsApplicationError> {
        let repository = CustomerDetailsRepository::new(self.connection);

        let Some(customer) = repository.find_customer(input.customer_id)? else {
            return Ok(None);
        };

        let motorcycles = repository
            .list_motorcycles(input.customer_id, CUSTOMER_DETAILS_MOTORCYCLE_LIMIT)?
            .into_iter()
            .map(Into::into)
            .collect();

        let service_history = repository
            .list_service_history(input.customer_id, CUSTOMER_DETAILS_SERVICE_HISTORY_LIMIT)?
            .into_iter()
            .map(Into::into)
            .collect();

        Ok(Some(CustomerDetails {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            motorcycles,
            service_history,
        }))
    }
}

impl From<CustomerDetailsRow> for CustomerDetails {
    fn from(customer: CustomerDetailsRow) -> Self {
        Self {
            id: customer.id,
            name: customer.name,
            phone: customer.phone,
            motorcycles: Vec::new(),
            service_history: Vec::new(),
        }
    }
}

impl From<CustomerDetailsMotorcycleRow> for CustomerDetailsMotorcycle {
    fn from(motorcycle: CustomerDetailsMotorcycleRow) -> Self {
        Self {
            id: motorcycle.id,
            make_name: motorcycle.make_name,
            model: motorcycle.model,
            year: motorcycle.year,
            plate_number: motorcycle.plate_number,
            vin: motorcycle.vin,
            chassis_number: motorcycle.chassis_number,
            color_name: motorcycle.color_name,
        }
    }
}

impl From<CustomerServiceHistoryRow> for CustomerServiceHistoryEntry {
    fn from(visit: CustomerServiceHistoryRow) -> Self {
        Self {
            id: visit.id,
            motorcycle_id: visit.motorcycle_id,
            opened_at: visit.opened_at,
            odometer_km: visit.odometer_km,
            customer_complaint: visit.customer_complaint,
            status: visit.status,
            total_fils: visit.total_fils,
        }
    }
}
