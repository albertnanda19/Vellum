use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationDiscoveryError {
    EmptyMigrationsDir { dir: String },
    InvalidFilename { filename: String, reason: String },
    DuplicateVersion { version: i64, first: String, second: String },
    Io { path: String, message: String },
}

impl fmt::Display for MigrationDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationDiscoveryError::EmptyMigrationsDir { dir } => {
                write!(f, "migrations directory is empty: {dir}")
            }
            MigrationDiscoveryError::InvalidFilename { filename, reason } => {
                write!(f, "invalid migration filename '{filename}': {reason}")
            }
            MigrationDiscoveryError::DuplicateVersion {
                version,
                first,
                second,
            } => write!(
                f,
                "duplicate migration version {version}: '{first}' and '{second}'"
            ),
            MigrationDiscoveryError::Io { path, message } => {
                write!(f, "I/O error while reading '{path}': {message}")
            }
        }
    }
}

impl Error for MigrationDiscoveryError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationDriftError {
    MissingMigrationFile { version: i64 },
    ChecksumMismatch {
        version: i64,
        expected: String,
        actual: String,
    },
}

impl fmt::Display for MigrationDriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MigrationDriftError::MissingMigrationFile { version } => write!(
                f,
                "migration drift detected: version {version} exists in database but no corresponding migration file was found"
            ),
            MigrationDriftError::ChecksumMismatch {
                version,
                expected,
                actual,
            } => write!(
                f,
                "migration drift detected: checksum mismatch for version {version} (db={expected}, fs={actual})"
            ),
        }
    }
}

impl Error for MigrationDriftError {}
