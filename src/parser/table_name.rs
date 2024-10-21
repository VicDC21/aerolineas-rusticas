use crate::protocol::errors::error::Error;

use super::{
    data_types::keyspace_name::KeyspaceName,
    statements::ddl_statement::ddl_statement_parser::check_words,
};

/// Representa un nombre de tabla en Cassandra, que puede incluir un keyspace opcional.
/// table_name::= [keyspace_name '.' ] name
/// # Campos
///
/// * `keyspace` - Un `Option<KeyspaceName>` que representa el keyspace opcional.
/// * `name` - Un `KeyspaceName` que representa el nombre de la tabla.
#[derive(Debug)]
pub struct TableName {
    /// Un `bool` que indica si la tabla existe.
    /// if_exists::= 'IF' 'EXISTS'
    pub if_exists: bool,

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

        let mut if_exists = false;
        let mut keyspace = None;
        if lista.len() > 1 {
            if check_words(lista, "IF EXISTS") {
                if_exists = true;
            }

            if lista.len() > 3 {
                keyspace = if lista[1] != "SET"
                    && lista[1] != "("
                    && lista[0] != "\'"
                    && lista[3] != "("
                    && lista[1] != "ADD"
                    && lista[1] != "DROP"
                    && lista[1] != "WITH"
                    && lista[1] != "RENAME"
                {
                    KeyspaceName::check_kind_of_name(lista)?
                } else {
                    None
                };
            }
        }

        let name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "No se proporciono un nombre de keyspace valido".to_string(),
                ))
            }
        };

        Ok(Some(TableName {
            if_exists,
            keyspace,
            name,
        }))
    }
}
