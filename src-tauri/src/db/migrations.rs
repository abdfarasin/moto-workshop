use rusqlite::{Connection, Result};

pub fn migrate_database(connection: &mut Connection) -> Result<()> {
    if schema_version(connection)? < 1 {
        migrate_to_version_1(connection)?;
    }

    if schema_version(connection)? < 2 {
        migrate_to_version_2(connection)?;
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
