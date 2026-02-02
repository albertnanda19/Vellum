use vellum_sql::parse_sql;

#[test]
fn valid_sql_returns_parsed_sql() {
    let parsed = parse_sql("SELECT 1;").unwrap();
    assert_eq!(parsed.sql(), "SELECT 1;");
}

#[test]
fn invalid_sql_returns_error() {
    let err = parse_sql("SELEC 1;").unwrap_err();
    assert!(!err.message().is_empty());
}

#[test]
fn multi_statement_sql_is_parsed() {
    let parsed = parse_sql("SELECT 1; SELECT 2;").unwrap();
    assert_eq!(parsed.sql(), "SELECT 1; SELECT 2;");
}

#[test]
fn error_message_is_not_empty() {
    let err = parse_sql("SELECT FROM;").unwrap_err();
    assert!(!err.message().is_empty());
}
