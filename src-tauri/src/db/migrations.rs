use std::{error::Error, fmt};

use rusqlite::{Connection, OptionalExtension, Result};

const MAX_SUPPORTED_SCHEMA_VERSION: i64 = 6;

#[derive(Debug)]
pub enum MigrationError {
    Database(rusqlite::Error),
    ForeignKeyIntegrityViolation { table: String },
    InvalidExistingCustomer { customer_id: i64 },
    UnsupportedSchemaVersion { found: i64, max_supported: i64 },
}

impl fmt::Display for MigrationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => write!(formatter, "database migration failed: {error}"),
            Self::ForeignKeyIntegrityViolation { table } => {
                write!(
                    formatter,
                    "foreign-key integrity violation in table {table}"
                )
            }
            Self::InvalidExistingCustomer { customer_id } => write!(
                formatter,
                "customer {customer_id} violates schema version 3 data requirements"
            ),
            Self::UnsupportedSchemaVersion {
                found,
                max_supported,
            } => write!(
                formatter,
                "database schema version {found} is newer than supported version {max_supported}"
            ),
        }
    }
}

impl Error for MigrationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::ForeignKeyIntegrityViolation { .. }
            | Self::InvalidExistingCustomer { .. }
            | Self::UnsupportedSchemaVersion { .. } => None,
        }
    }
}

impl From<rusqlite::Error> for MigrationError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Database(error)
    }
}

pub fn migrate_database(connection: &mut Connection) -> std::result::Result<(), MigrationError> {
    let found = schema_version(connection)?;

    if found > MAX_SUPPORTED_SCHEMA_VERSION {
        return Err(MigrationError::UnsupportedSchemaVersion {
            found,
            max_supported: MAX_SUPPORTED_SCHEMA_VERSION,
        });
    }

    if found < 1 {
        migrate_to_version_1(connection)?;
    }

    if schema_version(connection)? < 2 {
        migrate_to_version_2(connection)?;
    }

    if schema_version(connection)? < 3 {
        migrate_to_version_3(connection)?;
    }

    if schema_version(connection)? < 4 {
        migrate_to_version_4(connection)?;
    }

    if schema_version(connection)? < 5 {
        migrate_to_version_5(connection)?;
    }

    if schema_version(connection)? < 6 {
        migrate_to_version_6(connection)?;
    }

    Ok(())
}

fn schema_version(connection: &Connection) -> Result<i64> {
    connection.pragma_query_value(None, "user_version", |row| row.get(0))
}

fn migrate_to_version_1(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TABLE customers (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            phone       TEXT NOT NULL UNIQUE,
            notes       TEXT,
            created_at  INTEGER NOT NULL,
            updated_at  INTEGER NOT NULL,
            archived_at INTEGER
        );
        ",
    )?;

    transaction.pragma_update(None, "user_version", 1)?;

    transaction.commit()
}

fn migrate_to_version_2(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TABLE motorcycle_makes (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            name   TEXT NOT NULL COLLATE NOCASE UNIQUE
                   CHECK (name = trim(name) AND length(name) BETWEEN 1 AND 80),
            active INTEGER NOT NULL DEFAULT 1
                   CHECK (active IN (0, 1))
        );

        CREATE TABLE motorcycle_colors (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            name   TEXT NOT NULL COLLATE NOCASE UNIQUE
                   CHECK (name = trim(name) AND length(name) BETWEEN 1 AND 40),
            active INTEGER NOT NULL DEFAULT 1
                   CHECK (active IN (0, 1))
        );

        CREATE TABLE jordan_plate_codes (
            id     INTEGER PRIMARY KEY AUTOINCREMENT,
            code   TEXT NOT NULL COLLATE NOCASE UNIQUE
                   CHECK (code = trim(code) AND length(code) BETWEEN 1 AND 20),
            active INTEGER NOT NULL DEFAULT 1
                   CHECK (active IN (0, 1))
        );

        CREATE TABLE motorcycles (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_id  INTEGER NOT NULL,
            make_id      INTEGER NOT NULL,
            model        TEXT NOT NULL
                         CHECK (
                             typeof(model) = 'text'
                             AND model = trim(model)
                             AND length(model) BETWEEN 1 AND 80
                         ),
            year         INTEGER
                         CHECK (
                             year IS NULL
                             OR (typeof(year) = 'integer' AND year >= 1885)
                         ),
            plate_code_id INTEGER,
            plate_number INTEGER
                         CHECK (
                             plate_number IS NULL
                             OR (
                                 typeof(plate_number) = 'integer'
                                 AND plate_number BETWEEN 1 AND 99999
                             )
                         ),
            vin          TEXT UNIQUE
                         CHECK (
                             vin IS NULL
                             OR (
                                 typeof(vin) = 'text'
                                 AND length(vin) = 17
                                 AND vin = upper(vin)
                                 AND vin NOT GLOB '*[^A-Z0-9]*'
                                 AND instr(vin, 'I') = 0
                                 AND instr(vin, 'O') = 0
                                 AND instr(vin, 'Q') = 0
                             )
                         ),
            color_id     INTEGER NOT NULL,
            notes        TEXT
                         CHECK (
                             notes IS NULL
                             OR (typeof(notes) = 'text' AND length(notes) <= 2000)
                         ),
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL,
            archived_at  INTEGER,
            FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT,
            FOREIGN KEY (make_id) REFERENCES motorcycle_makes(id) ON DELETE RESTRICT,
            FOREIGN KEY (plate_code_id) REFERENCES jordan_plate_codes(id) ON DELETE RESTRICT,
            FOREIGN KEY (color_id) REFERENCES motorcycle_colors(id) ON DELETE RESTRICT,
            CHECK (
                (plate_code_id IS NULL AND plate_number IS NULL)
                OR (plate_code_id IS NOT NULL AND plate_number IS NOT NULL)
            ),
            CHECK (vin IS NOT NULL OR plate_code_id IS NOT NULL),
            UNIQUE (plate_code_id, plate_number)
        );

        CREATE INDEX idx_motorcycles_customer_id
            ON motorcycles(customer_id);

        CREATE INDEX idx_motorcycles_make_id
            ON motorcycles(make_id);

        INSERT INTO motorcycle_makes (name) VALUES
            ('Aprilia'),
            ('Bajaj'),
            ('Benelli'),
            ('Beta'),
            ('BMW'),
            ('BSA'),
            ('CFMOTO'),
            ('Ducati'),
            ('GasGas'),
            ('Harley-Davidson'),
            ('Hero'),
            ('Honda'),
            ('Husqvarna'),
            ('Indian'),
            ('Kawasaki'),
            ('Keeway'),
            ('KTM'),
            ('Kymco'),
            ('Lifan'),
            ('Loncin'),
            ('Moto Guzzi'),
            ('MV Agusta'),
            ('Piaggio'),
            ('QJMotor'),
            ('Royal Enfield'),
            ('Sherco'),
            ('Suzuki'),
            ('SYM'),
            ('Triumph'),
            ('TVS'),
            ('Vespa'),
            ('Voge'),
            ('Yamaha'),
            ('Zontes');

        INSERT INTO motorcycle_colors (name) VALUES
            ('Black'),
            ('White'),
            ('Gray'),
            ('Silver'),
            ('Red'),
            ('Blue'),
            ('Green'),
            ('Yellow'),
            ('Orange'),
            ('Brown'),
            ('Beige'),
            ('Gold'),
            ('Purple'),
            ('Pink'),
            ('Bronze'),
            ('Maroon'),
            ('Multicolor');
        ",
    )?;

    transaction.pragma_update(None, "user_version", 2)?;

    transaction.commit()
}

fn migrate_to_version_3(connection: &mut Connection) -> std::result::Result<(), MigrationError> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TRIGGER validate_customers_before_insert_v3
        BEFORE INSERT ON customers
        WHEN
            typeof(NEW.name) != 'text'
            OR NEW.name != trim(NEW.name)
            OR length(NEW.name) NOT BETWEEN 1 AND 100
            OR typeof(NEW.phone) != 'text'
            OR length(NEW.phone) != 13
            OR substr(NEW.phone, 1, 4) != '+962'
            OR substr(NEW.phone, 5) GLOB '*[^0-9]*'
            OR NEW.notes IS NOT NULL AND (
                typeof(NEW.notes) != 'text'
                OR length(NEW.notes) > 2000
            )
        BEGIN
            SELECT RAISE(ABORT, 'invalid customer');
        END;

        CREATE TRIGGER validate_customers_before_update_v3
        BEFORE UPDATE ON customers
        WHEN
            typeof(NEW.name) != 'text'
            OR NEW.name != trim(NEW.name)
            OR length(NEW.name) NOT BETWEEN 1 AND 100
            OR typeof(NEW.phone) != 'text'
            OR length(NEW.phone) != 13
            OR substr(NEW.phone, 1, 4) != '+962'
            OR substr(NEW.phone, 5) GLOB '*[^0-9]*'
            OR NEW.notes IS NOT NULL AND (
                typeof(NEW.notes) != 'text'
                OR length(NEW.notes) > 2000
            )
        BEGIN
            SELECT RAISE(ABORT, 'invalid customer');
        END;
        ",
    )?;

    let invalid_customer_id = transaction
        .query_row(
            "
            SELECT id
            FROM customers
            WHERE
                typeof(name) != 'text'
                OR name != trim(name)
                OR length(name) NOT BETWEEN 1 AND 100
                OR typeof(phone) != 'text'
                OR length(phone) != 13
                OR substr(phone, 1, 4) != '+962'
                OR substr(phone, 5) GLOB '*[^0-9]*'
                OR notes IS NOT NULL AND (
                    typeof(notes) != 'text'
                    OR length(notes) > 2000
                )
            LIMIT 1
            ",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(customer_id) = invalid_customer_id {
        return Err(MigrationError::InvalidExistingCustomer { customer_id });
    }

    transaction.pragma_update(None, "user_version", 3)?;

    transaction.commit()?;

    Ok(())
}

fn migrate_to_version_4(connection: &mut Connection) -> std::result::Result<(), MigrationError> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TABLE motorcycles_v4 (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_id    INTEGER NOT NULL,
            make_id        INTEGER NOT NULL,
            model          TEXT NOT NULL
                           CHECK (
                               typeof(model) = 'text'
                               AND model = trim(model)
                               AND length(model) BETWEEN 1 AND 80
                           ),
            year           INTEGER
                           CHECK (
                               year IS NULL
                               OR (typeof(year) = 'integer' AND year >= 1885)
                           ),
            plate_code_id  INTEGER,
            plate_number   INTEGER
                           CHECK (
                               plate_number IS NULL
                               OR (
                                   typeof(plate_number) = 'integer'
                                   AND plate_number BETWEEN 1 AND 99999
                               )
                           ),
            vin            TEXT UNIQUE
                           CHECK (
                               vin IS NULL
                               OR (
                                   typeof(vin) = 'text'
                                   AND length(vin) = 17
                                   AND vin = upper(vin)
                                   AND vin NOT GLOB '*[^A-Z0-9]*'
                                   AND instr(vin, 'I') = 0
                                   AND instr(vin, 'O') = 0
                                   AND instr(vin, 'Q') = 0
                               )
                           ),
            chassis_number TEXT UNIQUE
                           CHECK (
                               chassis_number IS NULL
                               OR (
                                   typeof(chassis_number) = 'text'
                                   AND length(chassis_number) BETWEEN 1 AND 64
                                   AND chassis_number = upper(chassis_number)
                                   AND chassis_number NOT GLOB '*[^A-Z0-9./-]*'
                               )
                           ),
            color_id       INTEGER NOT NULL,
            notes          TEXT
                           CHECK (
                               notes IS NULL
                               OR (typeof(notes) = 'text' AND length(notes) <= 2000)
                           ),
            created_at     INTEGER NOT NULL,
            updated_at     INTEGER NOT NULL,
            archived_at    INTEGER,
            FOREIGN KEY (customer_id) REFERENCES customers(id) ON DELETE RESTRICT,
            FOREIGN KEY (make_id) REFERENCES motorcycle_makes(id) ON DELETE RESTRICT,
            FOREIGN KEY (plate_code_id) REFERENCES jordan_plate_codes(id) ON DELETE RESTRICT,
            FOREIGN KEY (color_id) REFERENCES motorcycle_colors(id) ON DELETE RESTRICT,
            CHECK (
                (plate_code_id IS NULL AND plate_number IS NULL)
                OR (plate_code_id IS NOT NULL AND plate_number IS NOT NULL)
            ),
            CHECK (
                vin IS NOT NULL
                OR chassis_number IS NOT NULL
                OR plate_code_id IS NOT NULL
            ),
            UNIQUE (plate_code_id, plate_number)
        );

        INSERT INTO motorcycles_v4 (
            id,
            customer_id,
            make_id,
            model,
            year,
            plate_code_id,
            plate_number,
            vin,
            chassis_number,
            color_id,
            notes,
            created_at,
            updated_at,
            archived_at
        )
        SELECT
            id,
            customer_id,
            make_id,
            model,
            year,
            plate_code_id,
            plate_number,
            vin,
            NULL,
            color_id,
            notes,
            created_at,
            updated_at,
            archived_at
        FROM motorcycles;

        DROP TABLE motorcycles;
        ALTER TABLE motorcycles_v4 RENAME TO motorcycles;

        CREATE INDEX idx_motorcycles_customer_id
            ON motorcycles(customer_id);

        CREATE INDEX idx_motorcycles_make_id
            ON motorcycles(make_id);
        ",
    )?;

    let foreign_key_violation = transaction
        .query_row(
            "SELECT \"table\" FROM pragma_foreign_key_check LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    if let Some(table) = foreign_key_violation {
        return Err(MigrationError::ForeignKeyIntegrityViolation { table });
    }

    transaction.pragma_update(None, "user_version", 4)?;

    transaction.commit()?;

    Ok(())
}

fn migrate_to_version_5(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TABLE service_visits (
            id                  INTEGER PRIMARY KEY AUTOINCREMENT,
            motorcycle_id       INTEGER NOT NULL,
            owner_customer_id   INTEGER NOT NULL,
            status              TEXT NOT NULL,
            opened_at           INTEGER NOT NULL,
            completed_at        INTEGER,
            closed_at           INTEGER,
            cancelled_at        INTEGER,
            odometer_km         INTEGER,
            customer_complaint  TEXT NOT NULL,
            diagnosis           TEXT,
            work_performed      TEXT,
            labor_charge_fils   INTEGER NOT NULL DEFAULT 0,
            cancellation_reason TEXT,
            notes                TEXT,
            created_at           INTEGER NOT NULL,
            updated_at           INTEGER NOT NULL,
            FOREIGN KEY (motorcycle_id) REFERENCES motorcycles(id) ON DELETE RESTRICT,
            FOREIGN KEY (owner_customer_id) REFERENCES customers(id) ON DELETE RESTRICT,
            CHECK (typeof(motorcycle_id) = 'integer'),
            CHECK (typeof(owner_customer_id) = 'integer'),
            CHECK (status IN ('OPEN', 'READY_FOR_PICKUP', 'CLOSED', 'CANCELLED')),
            CHECK (typeof(opened_at) = 'integer' AND opened_at >= 0),
            CHECK (typeof(created_at) = 'integer'),
            CHECK (typeof(updated_at) = 'integer'),
            CHECK (
                odometer_km IS NULL
                OR (
                    typeof(odometer_km) = 'integer'
                    AND odometer_km BETWEEN 0 AND 9999999
                )
            ),
            CHECK (
                typeof(customer_complaint) = 'text'
                AND customer_complaint = trim(customer_complaint)
                AND length(customer_complaint) BETWEEN 1 AND 4000
            ),
            CHECK (
                diagnosis IS NULL
                OR (
                    typeof(diagnosis) = 'text'
                    AND diagnosis = trim(diagnosis)
                    AND length(diagnosis) BETWEEN 1 AND 4000
                )
            ),
            CHECK (
                work_performed IS NULL
                OR (
                    typeof(work_performed) = 'text'
                    AND work_performed = trim(work_performed)
                    AND length(work_performed) BETWEEN 1 AND 4000
                )
            ),
            CHECK (
                typeof(labor_charge_fils) = 'integer'
                AND labor_charge_fils >= 0
            ),
            CHECK (
                cancellation_reason IS NULL
                OR (
                    typeof(cancellation_reason) = 'text'
                    AND cancellation_reason = trim(cancellation_reason)
                    AND length(cancellation_reason) BETWEEN 1 AND 1000
                )
            ),
            CHECK (
                notes IS NULL
                OR (
                    typeof(notes) = 'text'
                    AND notes = trim(notes)
                    AND length(notes) BETWEEN 1 AND 4000
                )
            ),
            CHECK (
                (
                    status = 'OPEN'
                    AND completed_at IS NULL
                    AND closed_at IS NULL
                    AND cancelled_at IS NULL
                    AND cancellation_reason IS NULL
                )
                OR (
                    status = 'READY_FOR_PICKUP'
                    AND typeof(completed_at) = 'integer'
                    AND completed_at >= opened_at
                    AND closed_at IS NULL
                    AND cancelled_at IS NULL
                    AND work_performed IS NOT NULL
                    AND cancellation_reason IS NULL
                )
                OR (
                    status = 'CLOSED'
                    AND typeof(completed_at) = 'integer'
                    AND completed_at >= opened_at
                    AND typeof(closed_at) = 'integer'
                    AND closed_at >= completed_at
                    AND cancelled_at IS NULL
                    AND work_performed IS NOT NULL
                    AND cancellation_reason IS NULL
                )
                OR (
                    status = 'CANCELLED'
                    AND completed_at IS NULL
                    AND closed_at IS NULL
                    AND typeof(cancelled_at) = 'integer'
                    AND cancelled_at >= opened_at
                    AND cancellation_reason IS NOT NULL
                )
            )
        );

        CREATE TABLE invoices (
            id               INTEGER PRIMARY KEY AUTOINCREMENT,
            service_visit_id INTEGER NOT NULL UNIQUE,
            status           TEXT NOT NULL DEFAULT 'DRAFT',
            invoice_number   TEXT UNIQUE,
            issued_at        INTEGER,
            cancelled_at     INTEGER,
            notes            TEXT,
            created_at       INTEGER NOT NULL,
            updated_at       INTEGER NOT NULL,
            FOREIGN KEY (service_visit_id) REFERENCES service_visits(id) ON DELETE RESTRICT,
            CHECK (typeof(service_visit_id) = 'integer'),
            CHECK (status IN ('DRAFT', 'ISSUED', 'CANCELLED')),
            CHECK (issued_at IS NULL OR typeof(issued_at) = 'integer'),
            CHECK (cancelled_at IS NULL OR typeof(cancelled_at) = 'integer'),
            CHECK (typeof(created_at) = 'integer'),
            CHECK (typeof(updated_at) = 'integer')
        );

        CREATE TRIGGER validate_service_visit_owner_v5
        BEFORE INSERT ON service_visits
        WHEN NOT EXISTS (
            SELECT 1
            FROM motorcycles
            WHERE id = NEW.motorcycle_id
              AND customer_id = NEW.owner_customer_id
        )
        BEGIN
            SELECT RAISE(ABORT, 'service visit owner must match motorcycle owner');
        END;

        CREATE TRIGGER protect_service_visit_identity_v5
        BEFORE UPDATE OF motorcycle_id, owner_customer_id, opened_at ON service_visits
        WHEN OLD.motorcycle_id IS NOT NEW.motorcycle_id
          OR OLD.owner_customer_id IS NOT NEW.owner_customer_id
          OR OLD.opened_at IS NOT NEW.opened_at
        BEGIN
            SELECT RAISE(ABORT, 'service visit historical identity is immutable');
        END;

        CREATE TRIGGER validate_service_visit_transition_v5
        BEFORE UPDATE OF status ON service_visits
        WHEN OLD.status != NEW.status
         AND NOT (
            (OLD.status = 'OPEN' AND NEW.status IN ('READY_FOR_PICKUP', 'CANCELLED'))
            OR (OLD.status = 'READY_FOR_PICKUP' AND NEW.status IN ('OPEN', 'CLOSED'))
         )
        BEGIN
            SELECT RAISE(ABORT, 'invalid service visit status transition');
        END;

        CREATE TRIGGER prevent_terminal_service_visit_update_v5
        BEFORE UPDATE ON service_visits
        WHEN OLD.status IN ('CLOSED', 'CANCELLED')
        BEGIN
            SELECT RAISE(ABORT, 'terminal service visit cannot be updated');
        END;

        CREATE TRIGGER prevent_service_visit_delete_v5
        BEFORE DELETE ON service_visits
        BEGIN
            SELECT RAISE(ABORT, 'service visit cannot be deleted');
        END;

        CREATE TRIGGER create_draft_invoice_for_service_visit_v5
        AFTER INSERT ON service_visits
        BEGIN
            INSERT INTO invoices (
                service_visit_id,
                status,
                invoice_number,
                issued_at,
                cancelled_at,
                notes,
                created_at,
                updated_at
            ) VALUES (
                NEW.id,
                'DRAFT',
                NULL,
                NULL,
                NULL,
                NULL,
                NEW.created_at,
                NEW.created_at
            );
        END;

        CREATE TRIGGER protect_invoice_visit_identity_v5
        BEFORE UPDATE OF service_visit_id ON invoices
        WHEN OLD.service_visit_id IS NOT NEW.service_visit_id
        BEGIN
            SELECT RAISE(ABORT, 'invoice service visit is immutable');
        END;

        CREATE TRIGGER prevent_invoice_delete_v5
        BEFORE DELETE ON invoices
        BEGIN
            SELECT RAISE(ABORT, 'invoice cannot be deleted');
        END;

        CREATE UNIQUE INDEX one_active_service_visit_per_motorcycle
            ON service_visits(motorcycle_id)
            WHERE status IN ('OPEN', 'READY_FOR_PICKUP');

        CREATE INDEX idx_service_visits_motorcycle_id
            ON service_visits(motorcycle_id);
        ",
    )?;

    transaction.pragma_update(None, "user_version", 5)?;

    transaction.commit()
}

fn migrate_to_version_6(connection: &mut Connection) -> Result<()> {
    let transaction = connection.transaction()?;

    transaction.execute_batch(
        "
        CREATE TABLE inventory_units (
            id             INTEGER PRIMARY KEY AUTOINCREMENT,
            name           TEXT NOT NULL COLLATE NOCASE UNIQUE,
            quantity_scale INTEGER NOT NULL,
            active         INTEGER NOT NULL DEFAULT 1,
            CHECK (
                typeof(name) = 'text'
                AND name = trim(name)
                AND length(name) BETWEEN 1 AND 40
            ),
            CHECK (
                typeof(quantity_scale) = 'integer'
                AND quantity_scale IN (1, 10, 100, 1000)
            ),
            CHECK (typeof(active) = 'integer' AND active IN (0, 1))
        );

        CREATE TABLE inventory_items (
            id                          INTEGER PRIMARY KEY AUTOINCREMENT,
            name                        TEXT NOT NULL,
            sku                         TEXT COLLATE NOCASE UNIQUE,
            unit_id                     INTEGER NOT NULL,
            default_purchase_price_fils INTEGER,
            default_selling_price_fils  INTEGER NOT NULL,
            minimum_stock_quantity      INTEGER NOT NULL DEFAULT 0,
            notes                       TEXT,
            created_at                  INTEGER NOT NULL,
            updated_at                  INTEGER NOT NULL,
            archived_at                 INTEGER,
            FOREIGN KEY (unit_id) REFERENCES inventory_units(id) ON DELETE RESTRICT,
            CHECK (
                typeof(name) = 'text'
                AND name = trim(name)
                AND length(name) BETWEEN 1 AND 150
            ),
            CHECK (
                sku IS NULL
                OR (
                    typeof(sku) = 'text'
                    AND sku = trim(sku)
                    AND length(sku) BETWEEN 1 AND 64
                )
            ),
            CHECK (typeof(unit_id) = 'integer'),
            CHECK (
                default_purchase_price_fils IS NULL
                OR (
                    typeof(default_purchase_price_fils) = 'integer'
                    AND default_purchase_price_fils BETWEEN 0 AND 1000000000
                )
            ),
            CHECK (
                typeof(default_selling_price_fils) = 'integer'
                AND default_selling_price_fils BETWEEN 0 AND 1000000000
            ),
            CHECK (
                typeof(minimum_stock_quantity) = 'integer'
                AND minimum_stock_quantity BETWEEN 0 AND 1000000000
            ),
            CHECK (
                notes IS NULL
                OR (
                    typeof(notes) = 'text'
                    AND notes = trim(notes)
                    AND length(notes) BETWEEN 1 AND 2000
                )
            ),
            CHECK (typeof(created_at) = 'integer'),
            CHECK (typeof(updated_at) = 'integer'),
            CHECK (archived_at IS NULL OR typeof(archived_at) = 'integer')
        );

        CREATE TABLE stock_movements (
            id                INTEGER PRIMARY KEY AUTOINCREMENT,
            inventory_item_id INTEGER NOT NULL,
            movement_type     TEXT NOT NULL,
            quantity_delta    INTEGER NOT NULL,
            notes             TEXT,
            created_at        INTEGER NOT NULL,
            FOREIGN KEY (inventory_item_id) REFERENCES inventory_items(id) ON DELETE RESTRICT,
            CHECK (typeof(inventory_item_id) = 'integer'),
            CHECK (
                movement_type IN (
                    'OPENING_STOCK',
                    'PURCHASE',
                    'ADJUSTMENT_IN',
                    'ADJUSTMENT_OUT'
                )
            ),
            CHECK (
                typeof(quantity_delta) = 'integer'
                AND (
                    (
                        movement_type IN ('OPENING_STOCK', 'PURCHASE', 'ADJUSTMENT_IN')
                        AND quantity_delta BETWEEN 1 AND 1000000000
                    )
                    OR (
                        movement_type = 'ADJUSTMENT_OUT'
                        AND quantity_delta BETWEEN -1000000000 AND -1
                    )
                )
            ),
            CHECK (
                notes IS NULL
                OR (
                    typeof(notes) = 'text'
                    AND notes = trim(notes)
                    AND length(notes) BETWEEN 1 AND 2000
                )
            ),
            CHECK (typeof(created_at) = 'integer' AND created_at >= 0)
        );

        CREATE TRIGGER protect_referenced_inventory_unit_scale_v6
        BEFORE UPDATE OF quantity_scale ON inventory_units
        WHEN OLD.quantity_scale IS NOT NEW.quantity_scale
         AND EXISTS (
            SELECT 1
            FROM inventory_items
            WHERE unit_id = OLD.id
         )
        BEGIN
            SELECT RAISE(ABORT, 'referenced inventory unit scale cannot change');
        END;

        CREATE TRIGGER prevent_inventory_unit_delete_v6
        BEFORE DELETE ON inventory_units
        BEGIN
            SELECT RAISE(ABORT, 'inventory unit cannot be deleted');
        END;

        CREATE TRIGGER protect_inventory_item_unit_with_history_v6
        BEFORE UPDATE OF unit_id ON inventory_items
        WHEN OLD.unit_id IS NOT NEW.unit_id
         AND EXISTS (
            SELECT 1
            FROM stock_movements
            WHERE inventory_item_id = OLD.id
         )
        BEGIN
            SELECT RAISE(ABORT, 'inventory item unit cannot change after stock movement');
        END;

        CREATE TRIGGER prevent_inventory_item_delete_v6
        BEFORE DELETE ON inventory_items
        BEGIN
            SELECT RAISE(ABORT, 'inventory item cannot be deleted');
        END;

        CREATE TRIGGER prevent_stock_movement_update_v6
        BEFORE UPDATE ON stock_movements
        BEGIN
            SELECT RAISE(ABORT, 'stock movement cannot be updated');
        END;

        CREATE TRIGGER prevent_stock_movement_delete_v6
        BEFORE DELETE ON stock_movements
        BEGIN
            SELECT RAISE(ABORT, 'stock movement cannot be deleted');
        END;

        INSERT INTO inventory_units (name, quantity_scale, active)
        VALUES
            ('Piece', 1, 1),
            ('Liter', 1000, 1);

        CREATE INDEX idx_stock_movements_inventory_item_id
            ON stock_movements(inventory_item_id);
        ",
    )?;

    transaction.pragma_update(None, "user_version", 6)?;

    transaction.commit()
}
