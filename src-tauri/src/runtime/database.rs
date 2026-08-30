use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard},
};

use rusqlite::Connection;

use crate::db::{migrate_database, open_database, MigrationError};

pub const DATABASE_FILE_NAME: &str = "moto-workshop.sqlite3";

pub struct RuntimeDatabase {
    database_path: PathBuf,
    connection: Mutex<Connection>,
}

impl RuntimeDatabase {
    pub fn initialize(
        application_data_directory: impl AsRef<Path>,
    ) -> Result<Self, RuntimeDatabaseInitializationError> {
        let application_data_directory = application_data_directory.as_ref();
        fs::create_dir_all(application_data_directory)
            .map_err(RuntimeDatabaseInitializationError::CreateApplicationDataDirectory)?;
        let database_path = application_data_directory.join(DATABASE_FILE_NAME);
        let mut connection = open_database(&database_path)
            .map_err(RuntimeDatabaseInitializationError::OpenDatabase)?;
        migrate_database(&mut connection)
            .map_err(RuntimeDatabaseInitializationError::MigrateDatabase)?;
        Ok(Self {
            database_path,
            connection: Mutex::new(connection),
        })
    }

    pub fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub(crate) fn lock(&self) -> Result<MutexGuard<'_, Connection>, RuntimeDatabaseAccessError> {
        self.connection
            .lock()
            .map_err(|_| RuntimeDatabaseAccessError)
    }
}

#[derive(Debug)]
pub enum RuntimeDatabaseInitializationError {
    CreateApplicationDataDirectory(std::io::Error),
    OpenDatabase(rusqlite::Error),
    MigrateDatabase(MigrationError),
}

impl fmt::Display for RuntimeDatabaseInitializationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateApplicationDataDirectory(error) => write!(
                formatter,
                "failed to create the application data directory: {error}"
            ),
            Self::OpenDatabase(error) => {
                write!(formatter, "failed to open the workshop database: {error}")
            }
            Self::MigrateDatabase(error) => {
                write!(
                    formatter,
                    "failed to migrate the workshop database: {error}"
                )
            }
        }
    }
}

impl Error for RuntimeDatabaseInitializationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::CreateApplicationDataDirectory(error) => Some(error),
            Self::OpenDatabase(error) => Some(error),
            Self::MigrateDatabase(error) => Some(error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeDatabaseAccessError;
