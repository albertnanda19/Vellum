mod column;
mod constraint;
mod index;
mod schema;
mod table;

pub use column::normalize_column;
pub use schema::normalize_schema;
pub use table::normalize_table;

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn normalize_whitespace(input: &str) -> String {
    let input = input.trim();

    let mut out = String::with_capacity(input.len());
    let mut prev_was_space = false;

    for ch in input.chars() {
        if ch.is_whitespace() {
            if !prev_was_space {
                out.push(' ');
                prev_was_space = true;
            }
        } else {
            out.push(ch);
            prev_was_space = false;
        }
    }

    out
}

fn normalize_data_type(data_type: &str) -> String {
    let data_type = normalize_whitespace(data_type).to_lowercase();

    match data_type.as_str() {
        "int4" => "integer".to_string(),
        "int8" => "bigint".to_string(),
        "varchar" => "character varying".to_string(),
        "bool" => "boolean".to_string(),
        "timestamptz" => "timestamp with time zone".to_string(),
        _ => data_type,
    }
}

fn normalize_default(default: &Option<String>) -> Option<String> {
    let default = default.as_ref()?;

    let trimmed = default.trim();
    if trimmed.is_empty() {
        return Some(String::new());
    }

    if trimmed.eq_ignore_ascii_case("null") {
        return None;
    }

    let trimmed_lower = trimmed.to_ascii_lowercase();
    if trimmed_lower.ends_with("::text") {
        let prefix = &trimmed[..trimmed.len().saturating_sub("::text".len())];
        let prefix_trimmed = prefix.trim();

        if prefix_trimmed.starts_with('\'') && prefix_trimmed.ends_with('\'') {
            return Some(prefix_trimmed.to_string());
        }
    }

    Some(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::{Column, Constraint, ConstraintKind, Index, Schema, Table};

    #[test]
    fn type_alias_normalization() {
        let c = Column {
            name: "id".to_string(),
            data_type: "INT4".to_string(),
            nullable: false,
            default: None,
        };

        let nc = super::normalize_column(&c);
        assert_eq!(nc.data_type, "integer");

        let c2 = Column {
            name: "name".to_string(),
            data_type: "varchar".to_string(),
            nullable: true,
            default: None,
        };

        let nc2 = super::normalize_column(&c2);
        assert_eq!(nc2.data_type, "character varying");
    }

    #[test]
    fn default_value_normalization() {
        let c = Column {
            name: "x".to_string(),
            data_type: "text".to_string(),
            nullable: false,
            default: Some("  'value'::TEXT  ".to_string()),
        };

        let nc = super::normalize_column(&c);
        assert_eq!(nc.default, Some("'value'".to_string()));

        let c2 = Column {
            name: "y".to_string(),
            data_type: "text".to_string(),
            nullable: true,
            default: Some("NULL".to_string()),
        };

        let nc2 = super::normalize_column(&c2);
        assert_eq!(nc2.default, None);

        let c3 = Column {
            name: "z".to_string(),
            data_type: "timestamp".to_string(),
            nullable: false,
            default: Some("  now()  ".to_string()),
        };

        let nc3 = super::normalize_column(&c3);
        assert_eq!(nc3.default, Some("now()".to_string()));
    }

    #[test]
    fn ordering_is_deterministic() {
        let mut columns = BTreeMap::new();
        columns.insert(
            "B".to_string(),
            Column {
                name: "B".to_string(),
                data_type: "int4".to_string(),
                nullable: false,
                default: None,
            },
        );
        columns.insert(
            "a".to_string(),
            Column {
                name: "a".to_string(),
                data_type: "int4".to_string(),
                nullable: false,
                default: None,
            },
        );

        let table = Table {
            name: "T".to_string(),
            columns,
            indexes: BTreeMap::new(),
            constraints: BTreeMap::new(),
        };

        let nt = super::normalize_table(&table);
        let keys: Vec<String> = nt.columns.keys().cloned().collect();
        assert_eq!(keys, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn idempotency() {
        let mut columns = BTreeMap::new();
        columns.insert(
            "ID".to_string(),
            Column {
                name: "ID".to_string(),
                data_type: "int4".to_string(),
                nullable: false,
                default: Some("  'x'::text ".to_string()),
            },
        );

        let mut constraints = BTreeMap::new();
        constraints.insert(
            "T_PKEY".to_string(),
            Constraint {
                name: "T_PKEY".to_string(),
                kind: ConstraintKind::PrimaryKey {
                    columns: vec!["ID".to_string()],
                },
            },
        );

        let mut indexes = BTreeMap::new();
        indexes.insert(
            "T_PKEY".to_string(),
            Index {
                name: "T_PKEY".to_string(),
                columns: vec!["ID".to_string()],
                unique: true,
                method: "BTREE".to_string(),
            },
        );

        let table = Table {
            name: "T".to_string(),
            columns,
            indexes,
            constraints,
        };

        let mut tables = BTreeMap::new();
        tables.insert("T".to_string(), table);

        let schema = Schema {
            name: "Public".to_string(),
            tables,
            enum_types: BTreeMap::new(),
        };

        let n1 = super::normalize_schema(&schema);
        let n2 = super::normalize_schema(&n1);
        assert_eq!(n1, n2);

        let nt = n1.tables.get("t").expect("normalized table exists");
        assert!(!nt.indexes.contains_key("t_pkey"));
    }
}
