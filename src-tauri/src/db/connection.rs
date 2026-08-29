use std::{path::Path, time::Duration};

use rusqlite::{Connection, Result};

pub fn open_database(path: impl AsRef<Path>) -> Result<Connection> {
    let connection = Connection::open(path)?;

    connection.pragma_update(None, "foreign_keys", true)?;
    connection.pragma_update(None, "journal_mode", "WAL")?;
    connection.pragma_update(None, "synchronous", "FULL")?;
    connection.busy_timeout(Duration::from_secs(5))?;

    Ok(connection)
}
