use rusqlite::{Connection, OptionalExtension};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDetailsRow {
    pub id: i64,
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDetailsMotorcycleRow {
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
pub struct CustomerServiceHistoryRow {
    pub id: i64,
    pub motorcycle_id: i64,
    pub opened_at: i64,
    pub odometer_km: Option<i64>,
    pub customer_complaint: String,
    pub status: String,
    pub total_fils: i64,
}

pub struct CustomerDetailsRepository<'connection> {
    connection: &'connection Connection,
}

impl<'connection> CustomerDetailsRepository<'connection> {
    pub fn new(connection: &'connection Connection) -> Self {
        Self { connection }
    }

    pub fn find_customer(&self, customer_id: i64) -> rusqlite::Result<Option<CustomerDetailsRow>> {
        self.connection
            .query_row(
                "SELECT
                    id,
                    name,
                    phone
                 FROM customers
                 WHERE id = ?1
                   AND archived_at IS NULL",
                [customer_id],
                |row| {
                    Ok(CustomerDetailsRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        phone: row.get(2)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_motorcycles(
        &self,
        customer_id: i64,
        limit: i64,
    ) -> rusqlite::Result<Vec<CustomerDetailsMotorcycleRow>> {
        let mut statement = self.connection.prepare(
            "SELECT
                m.id,
                mk.name,
                m.model,
                m.year,
                m.plate_number,
                m.vin,
                m.chassis_number,
                mc.name
             FROM motorcycles m
             JOIN motorcycle_makes mk
               ON mk.id = m.make_id
             JOIN motorcycle_colors mc
               ON mc.id = m.color_id
             WHERE m.customer_id = ?1
               AND m.archived_at IS NULL
             ORDER BY m.created_at DESC, m.id DESC
             LIMIT ?2",
        )?;

        let motorcycles = statement
            .query_map([customer_id, limit], |row| {
                Ok(CustomerDetailsMotorcycleRow {
                    id: row.get(0)?,
                    make_name: row.get(1)?,
                    model: row.get(2)?,
                    year: row.get(3)?,
                    plate_number: row.get(4)?,
                    vin: row.get(5)?,
                    chassis_number: row.get(6)?,
                    color_name: row.get(7)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(motorcycles)
    }

    pub fn list_service_history(
        &self,
        customer_id: i64,
        limit: i64,
    ) -> rusqlite::Result<Vec<CustomerServiceHistoryRow>> {
        let mut statement = self.connection.prepare(
            "SELECT
                v.id,
                v.motorcycle_id,
                v.opened_at,
                v.odometer_km,
                v.customer_complaint,
                v.status,
                v.labor_charge_fils
                    + COALESCE(
                        SUM(
                            CASE
                                WHEN p.status = 'ACTIVE'
                                THEN p.line_total_fils
                                ELSE 0
                            END
                        ),
                        0
                    ) AS total_fils
             FROM service_visits v
             LEFT JOIN service_visit_parts p
               ON p.service_visit_id = v.id
             WHERE v.owner_customer_id = ?1
             GROUP BY
                v.id,
                v.motorcycle_id,
                v.opened_at,
                v.odometer_km,
                v.customer_complaint,
                v.status,
                v.labor_charge_fils
             ORDER BY v.opened_at DESC, v.id DESC
             LIMIT ?2",
        )?;

        let visits = statement
            .query_map([customer_id, limit], |row| {
                Ok(CustomerServiceHistoryRow {
                    id: row.get(0)?,
                    motorcycle_id: row.get(1)?,
                    opened_at: row.get(2)?,
                    odometer_km: row.get(3)?,
                    customer_complaint: row.get(4)?,
                    status: row.get(5)?,
                    total_fils: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(visits)
    }
}
