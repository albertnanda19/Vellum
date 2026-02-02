use crate::{Constraint, ConstraintKind};

pub(crate) fn normalize_constraint(table_name: &str, constraint: &Constraint) -> Constraint {
    let kind = normalize_constraint_kind(&constraint.kind);

    let raw_name = super::normalize_name(&constraint.name);
    let name = if is_system_generated_name(table_name, &raw_name) {
        canonical_name_for_kind(&kind)
    } else {
        raw_name
    };

    Constraint { name, kind }
}

fn normalize_constraint_kind(kind: &ConstraintKind) -> ConstraintKind {
    match kind {
        ConstraintKind::PrimaryKey { columns } => ConstraintKind::PrimaryKey {
            columns: columns.iter().map(|c| super::normalize_name(c)).collect(),
        },
        ConstraintKind::ForeignKey {
            columns,
            referenced_table,
            referenced_columns,
        } => ConstraintKind::ForeignKey {
            columns: columns.iter().map(|c| super::normalize_name(c)).collect(),
            referenced_table: super::normalize_name(referenced_table),
            referenced_columns: referenced_columns
                .iter()
                .map(|c| super::normalize_name(c))
                .collect(),
        },
        ConstraintKind::Unique { columns } => ConstraintKind::Unique {
            columns: columns.iter().map(|c| super::normalize_name(c)).collect(),
        },
        ConstraintKind::Check { expression } => ConstraintKind::Check {
            expression: super::normalize_whitespace(expression),
        },
    }
}

fn is_system_generated_name(table_name: &str, name: &str) -> bool {
    let prefix = format!("{table_name}_");

    name == format!("{table_name}_pkey")
        || (name.starts_with(&prefix) && name.ends_with("_key"))
        || (name.starts_with(&prefix) && name.ends_with("_fkey"))
        || (name.starts_with(&prefix) && name.ends_with("_check"))
}

fn canonical_name_for_kind(kind: &ConstraintKind) -> String {
    match kind {
        ConstraintKind::PrimaryKey { .. } => "primary_key".to_string(),
        ConstraintKind::Unique { columns } => {
            format!("unique({})", columns.join(","))
        }
        ConstraintKind::ForeignKey {
            columns,
            referenced_table,
            referenced_columns,
        } => format!(
            "foreign_key({}->{ }({}))",
            columns.join(","),
            referenced_table,
            referenced_columns.join(",")
        )
        .replace("-> ", "->"),
        ConstraintKind::Check { expression } => format!("check({})", expression),
    }
}
