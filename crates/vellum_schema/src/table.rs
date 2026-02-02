use std::collections::BTreeMap;

use crate::{Column, Constraint, Index};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: BTreeMap<String, Column>,
    pub indexes: BTreeMap<String, Index>,
    pub constraints: BTreeMap<String, Constraint>,
}
