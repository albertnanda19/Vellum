use crate::error::MigrationDriftError;
use crate::model::Migration;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbMigration {
    pub version: i64,
    pub checksum: String,
}

pub fn detect_drift(
    filesystem: &[Migration],
    db: &[DbMigration],
) -> Result<(), MigrationDriftError> {
    let mut fs_by_version: HashMap<i64, &Migration> = HashMap::with_capacity(filesystem.len());
    for m in filesystem {
        fs_by_version.insert(m.version, m);
    }

    for dbm in db {
        let Some(fsm) = fs_by_version.get(&dbm.version) else {
            return Err(MigrationDriftError::MissingMigrationFile {
                version: dbm.version,
            });
        };

        if fsm.checksum != dbm.checksum {
            return Err(MigrationDriftError::ChecksumMismatch {
                version: dbm.version,
                expected: dbm.checksum.clone(),
                actual: fsm.checksum.clone(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{detect_drift, DbMigration};
    use crate::model::Migration;

    fn m(version: i64, checksum: &str) -> Migration {
        Migration::new(
            version,
            format!("m{version}"),
            format!("{version}_m{version}.sql"),
            checksum.to_string(),
            "select 1;".to_string(),
        )
    }

    #[test]
    fn ok_when_db_is_prefix_and_new_files_exist() {
        let fs = vec![m(1, "a"), m(2, "b"), m(3, "c")];
        let db = vec![
            DbMigration {
                version: 1,
                checksum: "a".to_string(),
            },
            DbMigration {
                version: 2,
                checksum: "b".to_string(),
            },
        ];
        assert!(detect_drift(&fs, &db).is_ok());
    }

    #[test]
    fn error_when_db_has_missing_file() {
        let fs = vec![m(2, "b")];
        let db = vec![DbMigration {
            version: 1,
            checksum: "a".to_string(),
        }];
        let err = detect_drift(&fs, &db).unwrap_err();
        assert!(matches!(err, crate::error::MigrationDriftError::MissingMigrationFile { version: 1 }));
    }

    #[test]
    fn error_when_checksum_mismatch() {
        let fs = vec![m(1, "fs")];
        let db = vec![DbMigration {
            version: 1,
            checksum: "db".to_string(),
        }];
        let err = detect_drift(&fs, &db).unwrap_err();
        assert!(matches!(err, crate::error::MigrationDriftError::ChecksumMismatch { version: 1, .. }));
    }
}
