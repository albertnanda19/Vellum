use std::collections::BTreeMap;

use crate::{ConstraintKind, Index, Table};

use super::{column::normalize_column, constraint::normalize_constraint, index::normalize_index};

pub fn normalize_table(table: &Table) -> Table {
    let name = super::normalize_name(&table.name);

    let columns = table
        .columns
        .iter()
        .map(|(_, c)| {
            let c = normalize_column(c);
            (c.name.clone(), c)
        })
        .collect::<BTreeMap<_, _>>();

    let constraints = table
        .constraints
        .iter()
        .map(|(_, c)| {
            let c = normalize_constraint(&name, c);
            (c.name.clone(), c)
        })
        .collect::<BTreeMap<_, _>>();

    let primary_key_columns = constraints.values().find_map(|c| match &c.kind {
        ConstraintKind::PrimaryKey { columns } => Some(columns.clone()),
        _ => None,
    });

    let indexes = table
        .indexes
        .iter()
        .map(|(_, i)| {
            let i = normalize_index(i);
            (i.name.clone(), i)
        })
        .filter(|(index_name, index)| {
            if let Some(pk_columns) = primary_key_columns.as_deref() {
                !is_implicit_primary_key_index(&name, index_name, index, pk_columns)
            } else {
                true
            }
        })
        .collect::<BTreeMap<_, _>>();

    Table {
        name,
        columns,
        indexes,
        constraints,
    }
}

fn is_implicit_primary_key_index(
    table_name: &str,
    index_name: &str,
    index: &Index,
    primary_key_columns: &[String],
) -> bool {
    index_name == format!("{table_name}_pkey") && index.unique && index.columns == primary_key_columns
}
