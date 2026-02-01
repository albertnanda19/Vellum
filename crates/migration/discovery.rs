use crate::checksum::sha256_hex;
use crate::error::MigrationDiscoveryError;
use crate::model::Migration;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_migrations(dir: impl AsRef<Path>) -> Result<Vec<Migration>, MigrationDiscoveryError> {
    let dir = dir.as_ref();
    let dir_display = dir.display().to_string();

    let entries = fs::read_dir(dir).map_err(|e| MigrationDiscoveryError::Io {
        path: dir_display.clone(),
        message: e.to_string(),
    })?;

    let mut files: Vec<PathBuf> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| MigrationDiscoveryError::Io {
            path: dir_display.clone(),
            message: e.to_string(),
        })?;

        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    if files.is_empty() {
        return Err(MigrationDiscoveryError::EmptyMigrationsDir { dir: dir_display });
    }

    files.sort_by(|a, b| {
        let a_name = a.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
        let b_name = b.file_name().map(|s| s.to_string_lossy()).unwrap_or_default();
        a_name.cmp(&b_name)
    });

    let mut candidates: Vec<(i64, String, String, PathBuf)> = Vec::new();
    for path in files {
        let filename_os = path.file_name().ok_or_else(|| MigrationDiscoveryError::Io {
            path: path.display().to_string(),
            message: "missing filename".to_string(),
        })?;
        let filename = filename_os.to_string_lossy().to_string();

        if !filename.ends_with(".sql") {
            return Err(MigrationDiscoveryError::InvalidFilename {
                filename,
                reason: "file extension must be .sql".to_string(),
            });
        }

        let (version, name) = parse_filename(&filename)?;
        candidates.push((version, name, filename, path));
    }

    candidates.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.2.cmp(&b.2)));

    let mut seen: HashMap<i64, String> = HashMap::new();
    let mut out = Vec::with_capacity(candidates.len());

    for (version, name, filename, path) in candidates {

        if let Some(first) = seen.insert(version, filename.clone()) {
            return Err(MigrationDiscoveryError::DuplicateVersion {
                version,
                first,
                second: filename,
            });
        }

        let bytes = fs::read(&path).map_err(|e| MigrationDiscoveryError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;

        let checksum = sha256_hex(&bytes);
        let sql = String::from_utf8(bytes).map_err(|e| MigrationDiscoveryError::Io {
            path: path.display().to_string(),
            message: format!("file is not valid UTF-8: {e}"),
        })?;

        out.push(Migration::new(version, name, filename, checksum, sql));
    }

    Ok(out)
}

fn parse_filename(filename: &str) -> Result<(i64, String), MigrationDiscoveryError> {
    if filename.contains(std::path::MAIN_SEPARATOR) {
        return Err(MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "filename must not contain path separators".to_string(),
        });
    }

    if !filename.ends_with(".sql") {
        return Err(MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "file extension must be .sql".to_string(),
        });
    }

    let base = &filename[..filename.len() - 4];
    let (version_str, name) = base.split_once('_').ok_or_else(|| {
        MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "expected format <version>_<name>.sql".to_string(),
        }
    })?;

    if name.is_empty() {
        return Err(MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "name segment must not be empty".to_string(),
        });
    }

    if !version_str.chars().all(|c| c.is_ascii_digit()) {
        return Err(MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "version must be a positive integer".to_string(),
        });
    }

    let version: i64 = version_str.parse().map_err(|_| MigrationDiscoveryError::InvalidFilename {
        filename: filename.to_string(),
        reason: "version is not a valid i64".to_string(),
    })?;

    if version <= 0 {
        return Err(MigrationDiscoveryError::InvalidFilename {
            filename: filename.to_string(),
            reason: "version must be a positive integer".to_string(),
        });
    }

    Ok((version, name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::discover_migrations;
    use crate::error::MigrationDiscoveryError;
    use std::fs;

    #[test]
    fn empty_dir_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = discover_migrations(tmp.path()).unwrap_err();
        assert!(matches!(err, MigrationDiscoveryError::EmptyMigrationsDir { .. }));
    }

    #[test]
    fn invalid_filename_is_error() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("init.sql"), b"select 1;").unwrap();
        let err = discover_migrations(tmp.path()).unwrap_err();
        assert!(matches!(err, MigrationDiscoveryError::InvalidFilename { .. }));
    }

    #[test]
    fn sorts_numerically_and_rejects_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("2_b.sql"), b"select 2;").unwrap();
        fs::write(tmp.path().join("10_c.sql"), b"select 10;").unwrap();
        fs::write(tmp.path().join("1_a.sql"), b"select 1;").unwrap();

        let migrations = discover_migrations(tmp.path()).unwrap();
        let versions: Vec<i64> = migrations.into_iter().map(|m| m.version).collect();
        assert_eq!(versions, vec![1, 2, 10]);

        fs::write(tmp.path().join("2_dup.sql"), b"select 2b;").unwrap();
        let err = discover_migrations(tmp.path()).unwrap_err();
        assert!(matches!(err, MigrationDiscoveryError::DuplicateVersion { version: 2, .. }));
    }
}
