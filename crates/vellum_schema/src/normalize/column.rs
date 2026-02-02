use crate::Column;

pub fn normalize_column(column: &Column) -> Column {
    Column {
        name: super::normalize_name(&column.name),
        data_type: super::normalize_data_type(&column.data_type),
        nullable: column.nullable,
        default: super::normalize_default(&column.default),
    }
}
