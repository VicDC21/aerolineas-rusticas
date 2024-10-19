use crate::protocol::errors::error::Error;

use super::data_types::keyspace_name::KeyspaceName;

/// Representa un nombre de tabla en Cassandra, que puede incluir un keyspace opcional.
/// table_name::= [keyspace_name '.' ] name
/// # Campos
///
/// * `keyspace` - Un `Option<KeyspaceName>` que representa el keyspace opcional.
/// * `name` - Un `KeyspaceName` que representa el nombre de la tabla.
#[derive(Debug)]
pub struct TableName {
    /// Un `Option<KeyspaceName>` que representa el keyspace opcional.
    /// keyspace_name::= identifier
    pub keyspace: Option<KeyspaceName>,

    /// Un `KeyspaceName` que representa el nombre de la tabla.
    /// name::= identifier
    pub name: KeyspaceName,
}

impl TableName {
    /// Verifica el tipo de nombre de la lista proporcionada y devuelve un `TableName` si es válido.
    ///
    /// # Argumentos
    ///
    /// * `lista` - Una referencia mutable a un vector de cadenas que representan los nombres.
    ///
    /// # Retornos
    ///
    /// * `Ok(Some(TableName))` si se encuentra un keyspace y nombre válidos.
    /// * `Ok(None)` si la lista está vacía.
    /// * `Err(Error::SyntaxError)` si no se proporciona un nombre de keyspace válido.
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if lista.is_empty() {
            return Ok(None);
        }
        let keyspace = KeyspaceName::check_kind_of_name(lista)?;

        let name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "No se proporciono un nombre de keyspace valido".to_string(),
                ))
            }
        };
        Ok(Some(TableName { keyspace, name }))
    }
}
