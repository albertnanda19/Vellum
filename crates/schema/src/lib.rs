#[derive(Clone, Debug)]
pub struct SchemaSnapshot;

pub struct DefaultSchemaIntrospector;

impl DefaultSchemaIntrospector {
    pub fn new() -> Self {
        Self
    }
}

impl vellum_contracts::schema::SchemaIntrospector for DefaultSchemaIntrospector {
    type Error = vellum_contracts::Error;

    fn snapshot(&self) -> Result<vellum_contracts::schema::SchemaSnapshot, Self::Error> {
        Ok(vellum_contracts::schema::SchemaSnapshot)
    }
}
