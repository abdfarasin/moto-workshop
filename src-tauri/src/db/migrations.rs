use std::{error::Error, fmt};

use rusqlite::{Connection, OptionalExtension, Result};

const MAX_SUPPORTED_SCHEMA_VERSION: i64 = 4;

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
