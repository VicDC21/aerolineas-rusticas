use crate::{
    parser::{
        data_types::keyspace_name::KeyspaceName,
        statements::ddl_statement::ddl_statement_parser::check_words,
    },
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Representa un nombre de tabla en Cassandra, que puede incluir un keyspace opcional.
/// table_name::= [keyspace_name '.' ] name
/// # Campos
///
/// * `keyspace` - Un `Option<KeyspaceName>` que representa el keyspace opcional.
/// * `name` - Un `KeyspaceName` que representa el nombre de la tabla.
#[derive(Debug, PartialEq)]
pub struct TableName {
    /// Un `bool` que indica si la tabla existe.
    /// if_exists::= 'IF' 'EXISTS'
    pub if_exists: bool,

    /// Un `Option<KeyspaceName>` que representa el keyspace opcional.
    /// keyspace_name::= identifier
    keyspace: Option<KeyspaceName>,

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
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>> {
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
                    && lista[1] != "WHERE"
                    && lista[1] != "VALUES"
                    && lista[1] != "GROUP"
                    && lista[1] != "ORDER"
                    && lista[1] != "IF"
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

    /// Devuelve el keyspace de la tabla como un `Option<String>`.
    pub fn get_keyspace(&self) -> Option<String> {
        self.keyspace
            .as_ref()
            .map(|keyspace| keyspace.get_name().to_string())
    }

    /// Devuelve el nombre de la tabla como un `String`.
    pub fn get_name(&self) -> String {
        self.name.get_name().to_string()
    }
}
