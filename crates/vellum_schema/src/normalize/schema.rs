use std::collections::BTreeMap;

use crate::{EnumType, Schema, Table};

use super::table::normalize_table;

pub fn normalize_schema(schema: &Schema) -> Schema {
    let name = super::normalize_name(&schema.name);

    let tables = schema
        .tables
        .iter()
        .map(|(_, t)| {
            let t = normalize_table(t);
            (t.name.clone(), t)
        })
        .collect::<BTreeMap<String, Table>>();

    let enum_types = schema
        .enum_types
        .iter()
        .map(|(_, e)| {
            let e = EnumType {
                name: super::normalize_name(&e.name),
                values: e.values.clone(),
            };
            (e.name.clone(), e)
        })
        .collect::<BTreeMap<String, EnumType>>();

    Schema {
        name,
        tables,
        enum_types,
    }
}
