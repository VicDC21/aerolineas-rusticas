use crate::cassandra::errors::error::Error;

pub struct TableName {
    pub keyspace: Option<String>,
    pub name: String,
}

impl TableName {
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if lista.is_empty() {
            return Ok(None);
        }

        let name = lista.remove(0);
        Ok(Some(TableName {
            keyspace: None,
            name,
        }))
    }
}
