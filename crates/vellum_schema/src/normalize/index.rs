use crate::Index;

pub(crate) fn normalize_index(index: &Index) -> Index {
    Index {
        name: super::normalize_name(&index.name),
        columns: index
            .columns
            .iter()
            .map(|c| super::normalize_name(c))
            .collect(),
        unique: index.unique,
        method: super::normalize_whitespace(&index.method).to_lowercase(),
    }
}
