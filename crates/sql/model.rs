#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatement {
    pub ordinal: i32,
    pub sql: String,
}
