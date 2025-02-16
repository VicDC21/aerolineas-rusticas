use crate::data_types::unquoted_name::UnquotedName;
use protocol::{aliases::results::Result, errors::error::Error};

/// keyspace_name::= name
/// name::= unquoted_name | quoted_name

#[derive(Debug, PartialEq, Clone)]
pub enum KeyspaceName {
    /// Nombre de keyspace sin comillas.
    UnquotedName(UnquotedName),
    /// Nombre de keyspace con comillas.
    QuotedName(UnquotedName),
}

impl Default for KeyspaceName {
    fn default() -> Self {
        KeyspaceName::UnquotedName(UnquotedName::default())
    }
}

impl KeyspaceName {
    /// Verifica si la lista de tokens es un nombre de keyspace. Si lo es, lo retorna.
    /// Si no lo es, retorna None, o Error en caso de no cumplir con el protocolo.
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>> {
        if lista.is_empty() {
            return Err(Error::SyntaxError("Faltan argumentos".to_string()));
        }
        if lista.len() > 2
            && (lista[0] == "\"" || lista[0] == "\'")
            && (lista[2] == "\"" || lista[2] == "\'")
        {
            lista.remove(0);
            if !UnquotedName::is_unquoted_name(&lista[0]) {
                return Err(Error::SyntaxError(
                    "Palabra con comillas no cumple el protocolo".to_string(),
                ));
            };
            let keyspace_name = KeyspaceName::QuotedName(UnquotedName::new(lista.remove(0))?);
            lista.remove(0);
            return Ok(Some(keyspace_name));
        };
        if UnquotedName::is_unquoted_name(&lista[0]) {
            let keyspace_name = KeyspaceName::UnquotedName(UnquotedName::new(lista.remove(0))?);
            return Ok(Some(keyspace_name));
        };
        Ok(None)
    }

    /// Devuelve el nombre del keyspace como un String.
    pub fn get_name(&self) -> &str {
        match self {
            KeyspaceName::UnquotedName(name) | KeyspaceName::QuotedName(name) => name.get_name(),
        }
    }
}
