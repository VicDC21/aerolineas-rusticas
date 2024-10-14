use crate::cassandra::errors::error::Error;

use super::data_types::keyspace_name::KeyspaceName;

/// table_name::= [keyspace_name '.' ] name
pub struct TableName {
    pub keyspace: Option<KeyspaceName>,
    pub name: KeyspaceName,
}

impl TableName {
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if lista.is_empty() {
            return Ok(None);
        }
        let keyspace = KeyspaceName::check_kind_of_name(lista)?;

        let name = match KeyspaceName::check_kind_of_name(lista)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError("No se proporciono un nombre de keyspace valido".to_string()))
        };
        Ok(Some(TableName {
            keyspace,
            name,
        }))
    }
}
