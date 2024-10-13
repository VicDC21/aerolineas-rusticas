use crate::cassandra::errors::error::Error;

pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
}

impl ColumnDefinition {
    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let name = lista.remove(0);
        let data_type = lista.remove(0);
        Ok(ColumnDefinition { name, data_type })
    }
}
