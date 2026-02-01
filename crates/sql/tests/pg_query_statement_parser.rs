use vellum_sql::{PgQueryStatementParser, StatementParser};

#[test]
fn splits_plpgsql_do_block_as_single_statement() {
    let sql = "DO $$ BEGIN PERFORM 1; PERFORM 2; END $$;\nSELECT 3;\n";
    let parser = PgQueryStatementParser::new();
    let out = parser.parse_statements(sql, Some("test.sql")).unwrap();
    assert_eq!(out.len(), 2);
    assert!(out[0].sql.contains("PERFORM 1;"));
    assert!(out[0].sql.contains("PERFORM 2;"));
}

#[test]
fn preserves_original_slices() {
    let sql = "\n  SELECT 1;\n\n  SELECT 2;\n";
    let parser = PgQueryStatementParser::new();
    let out = parser.parse_statements(sql, Some("test.sql")).unwrap();
    assert_eq!(out.len(), 2);
    assert!(out[0].sql.contains("SELECT 1"));
}
