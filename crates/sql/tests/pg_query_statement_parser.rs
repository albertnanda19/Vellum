use vellum_sql_engine::{PgQueryStatementParser, StatementParser};

#[test]
fn splits_plpgsql_do_block_as_single_statement() {
    let sql = r#"
DO $$
BEGIN
    RAISE NOTICE 'hello world';
END $$;
"#;
    let parser = PgQueryStatementParser::new();
    let stmts = parser.parse_statements(sql, Some("test.sql")).unwrap();
    assert_eq!(stmts.len(), 1);
    assert!(stmts[0].sql.contains("RAISE NOTICE 'hello world';"));
}

#[test]
fn preserves_original_slices() {
    let sql = "\n  SELECT 1;\n\n  SELECT 2;\n";
    let parser = PgQueryStatementParser::new();
    let out = parser.parse_statements(sql, Some("test.sql")).unwrap();
    assert_eq!(out.len(), 2);
    assert!(out[0].sql.contains("SELECT 1"));
}
