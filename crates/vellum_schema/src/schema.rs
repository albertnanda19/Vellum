use std::collections::BTreeMap;

use crate::{EnumType, Table};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: String,
    pub tables: BTreeMap<String, Table>,
    pub enum_types: BTreeMap<String, EnumType>,
}
