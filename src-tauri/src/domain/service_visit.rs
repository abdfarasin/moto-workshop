#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitStatus {
    Open,
    ReadyForPickup,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitTextField {
    CustomerComplaint,
    Diagnosis,
    WorkPerformed,
    CancellationReason,
    Notes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceVisitValidationError {
    InvalidMotorcycleId,
    InvalidOwnerCustomerId,
    InvalidTimestamp,
    InvalidOdometer,
    BlankComplaint,
    BlankCancellationReason,
    TextTooLong(ServiceVisitTextField),
    TextContainsControlCharacter(ServiceVisitTextField),
    NegativeLaborCharge,
    MissingWorkPerformed,
    InvalidTransition {
        from: ServiceVisitStatus,
        to: ServiceVisitStatus,
    },
    TerminalVisitCannotBeEdited,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewServiceVisitInput {
    pub motorcycle_id: i64,
    pub owner_customer_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub notes: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceVisitDetailsInput {
    pub diagnosis: Option<String>,
    pub work_performed: Option<String>,
    pub labor_charge_fils: i64,
    pub notes: Option<String>,
    pub odometer_km: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ServiceVisit {
    motorcycle_id: i64,
    owner_customer_id: i64,
    status: ServiceVisitStatus,
    opened_at: i64,
    completed_at: Option<i64>,
    closed_at: Option<i64>,
    cancelled_at: Option<i64>,
    odometer_km: Option<i64>,
    customer_complaint: String,
    diagnosis: Option<String>,
    work_performed: Option<String>,
    labor_charge_fils: i64,
    cancellation_reason: Option<String>,
    notes: Option<String>,
}

impl ServiceVisit {
    pub fn open(input: NewServiceVisitInput) -> Result<Self, ServiceVisitValidationError> {
        if input.motorcycle_id <= 0 {
            return Err(ServiceVisitValidationError::InvalidMotorcycleId);
        }
        if input.owner_customer_id <= 0 {
            return Err(ServiceVisitValidationError::InvalidOwnerCustomerId);
        }
        if input.opened_at < 0 {
            return Err(ServiceVisitValidationError::InvalidTimestamp);
        }

        validate_odometer(input.odometer_km)?;
        let customer_complaint = normalize_required_text(
            input.customer_complaint,
            4_000,
            ServiceVisitTextField::CustomerComplaint,
            ServiceVisitValidationError::BlankComplaint,
        )?;
        let notes = normalize_optional_text(input.notes, 4_000, ServiceVisitTextField::Notes)?;

        Ok(Self {
            motorcycle_id: input.motorcycle_id,
            owner_customer_id: input.owner_customer_id,
            status: ServiceVisitStatus::Open,
            opened_at: input.opened_at,
            completed_at: None,
            closed_at: None,
            cancelled_at: None,
            odometer_km: input.odometer_km,
            customer_complaint,
            diagnosis: None,
            work_performed: None,
            labor_charge_fils: 0,
            cancellation_reason: None,
            notes,
        })
    }

    pub fn update_details(
        &mut self,
        input: ServiceVisitDetailsInput,
    ) -> Result<(), ServiceVisitValidationError> {
        if matches!(
            self.status,
            ServiceVisitStatus::Closed | ServiceVisitStatus::Cancelled
        ) {
            return Err(ServiceVisitValidationError::TerminalVisitCannotBeEdited);
        }
        if input.labor_charge_fils < 0 {
            return Err(ServiceVisitValidationError::NegativeLaborCharge);
        }
        validate_odometer(input.odometer_km)?;

        let diagnosis =
            normalize_optional_text(input.diagnosis, 4_000, ServiceVisitTextField::Diagnosis)?;
        let work_performed = normalize_optional_text(
            input.work_performed,
            4_000,
            ServiceVisitTextField::WorkPerformed,
        )?;
        let notes = normalize_optional_text(input.notes, 4_000, ServiceVisitTextField::Notes)?;

        if self.status == ServiceVisitStatus::ReadyForPickup && work_performed.is_none() {
            return Err(ServiceVisitValidationError::MissingWorkPerformed);
        }

        self.diagnosis = diagnosis;
        self.work_performed = work_performed;
        self.labor_charge_fils = input.labor_charge_fils;
        self.notes = notes;
        self.odometer_km = input.odometer_km;
        Ok(())
    }

    pub fn mark_ready_for_pickup(
        &mut self,
        completed_at: i64,
    ) -> Result<(), ServiceVisitValidationError> {
        self.require_transition(ServiceVisitStatus::Open, ServiceVisitStatus::ReadyForPickup)?;
        if self.work_performed.is_none() {
            return Err(ServiceVisitValidationError::MissingWorkPerformed);
        }
        if completed_at < self.opened_at {
            return Err(ServiceVisitValidationError::InvalidTimestamp);
        }

        self.status = ServiceVisitStatus::ReadyForPickup;
        self.completed_at = Some(completed_at);
        Ok(())
    }

    pub fn reopen(&mut self) -> Result<(), ServiceVisitValidationError> {
        self.require_transition(ServiceVisitStatus::ReadyForPickup, ServiceVisitStatus::Open)?;
        self.status = ServiceVisitStatus::Open;
        self.completed_at = None;
        self.closed_at = None;
        Ok(())
    }

    pub fn close(&mut self, closed_at: i64) -> Result<(), ServiceVisitValidationError> {
        self.require_transition(
            ServiceVisitStatus::ReadyForPickup,
            ServiceVisitStatus::Closed,
        )?;
        let completed_at = self
            .completed_at
            .ok_or(ServiceVisitValidationError::InvalidTimestamp)?;
        if closed_at < completed_at {
            return Err(ServiceVisitValidationError::InvalidTimestamp);
        }

        self.status = ServiceVisitStatus::Closed;
        self.closed_at = Some(closed_at);
        Ok(())
    }

    pub fn cancel(
        &mut self,
        cancelled_at: i64,
        reason: String,
    ) -> Result<(), ServiceVisitValidationError> {
        self.require_transition(ServiceVisitStatus::Open, ServiceVisitStatus::Cancelled)?;
        let cancellation_reason = normalize_required_text(
            reason,
            1_000,
            ServiceVisitTextField::CancellationReason,
            ServiceVisitValidationError::BlankCancellationReason,
        )?;
        if cancelled_at < self.opened_at {
            return Err(ServiceVisitValidationError::InvalidTimestamp);
        }

        self.status = ServiceVisitStatus::Cancelled;
        self.cancelled_at = Some(cancelled_at);
        self.cancellation_reason = Some(cancellation_reason);
        Ok(())
    }

    fn require_transition(
        &self,
        required_status: ServiceVisitStatus,
        target_status: ServiceVisitStatus,
    ) -> Result<(), ServiceVisitValidationError> {
        if self.status != required_status {
            return Err(ServiceVisitValidationError::InvalidTransition {
                from: self.status,
                to: target_status,
            });
        }
        Ok(())
    }

    pub fn motorcycle_id(&self) -> i64 {
        self.motorcycle_id
    }

    pub fn owner_customer_id(&self) -> i64 {
        self.owner_customer_id
    }

    pub fn status(&self) -> ServiceVisitStatus {
        self.status
    }

    pub fn opened_at(&self) -> i64 {
        self.opened_at
    }

    pub fn completed_at(&self) -> Option<i64> {
        self.completed_at
    }

    pub fn closed_at(&self) -> Option<i64> {
        self.closed_at
    }

    pub fn cancelled_at(&self) -> Option<i64> {
        self.cancelled_at
    }

    pub fn odometer_km(&self) -> Option<i64> {
        self.odometer_km
    }

    pub fn customer_complaint(&self) -> &str {
        &self.customer_complaint
    }

    pub fn diagnosis(&self) -> Option<&str> {
        self.diagnosis.as_deref()
    }

    pub fn work_performed(&self) -> Option<&str> {
        self.work_performed.as_deref()
    }

    pub fn labor_charge_fils(&self) -> i64 {
        self.labor_charge_fils
    }

    pub fn cancellation_reason(&self) -> Option<&str> {
        self.cancellation_reason.as_deref()
    }

    pub fn notes(&self) -> Option<&str> {
        self.notes.as_deref()
    }
}

fn validate_odometer(odometer_km: Option<i64>) -> Result<(), ServiceVisitValidationError> {
    if odometer_km.is_some_and(|value| !(0..=9_999_999).contains(&value)) {
        return Err(ServiceVisitValidationError::InvalidOdometer);
    }
    Ok(())
}

fn normalize_required_text(
    value: String,
    maximum_characters: usize,
    field: ServiceVisitTextField,
    blank_error: ServiceVisitValidationError,
) -> Result<String, ServiceVisitValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(blank_error);
    }
    validate_text(value, maximum_characters, field)?;
    Ok(value.to_string())
}

fn normalize_optional_text(
    value: Option<String>,
    maximum_characters: usize,
    field: ServiceVisitTextField,
) -> Result<Option<String>, ServiceVisitValidationError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    validate_text(value, maximum_characters, field)?;
    Ok(Some(value.to_string()))
}

fn validate_text(
    value: &str,
    maximum_characters: usize,
    field: ServiceVisitTextField,
) -> Result<(), ServiceVisitValidationError> {
    if value.chars().count() > maximum_characters {
        return Err(ServiceVisitValidationError::TextTooLong(field));
    }
    if value
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(ServiceVisitValidationError::TextContainsControlCharacter(
            field,
        ));
    }
    Ok(())
}
