use rusqlite::{Connection, Result};


pub fn migrate_database(connection: &mut Connection) -> Result<()> {
    let current_version: i64 =
        connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

    if current_version < 1 {
        migrate_to_version_1(connection)?;
    }

    Ok(())
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
