#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Migration {
    pub version: i64,
    pub name: String,
    pub filename: String,
    pub checksum: String,
    pub sql: String,
}

impl Migration {
    pub fn new(version: i64, name: String, filename: String, checksum: String, sql: String) -> Self {
        Self {
            version,
            name,
            filename,
            checksum,
            sql,
        }
    }
}
