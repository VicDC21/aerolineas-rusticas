use crate::protocol::errors::error::Error;

use super::unquoted_name::UnquotedName;

/// keyspace_name::= name
/// name::= unquoted_name | quoted_name
pub enum KeyspaceName {
    UnquotedName(UnquotedName),
    QuotedName(UnquotedName),
}

impl Default for KeyspaceName {
    fn default() -> Self {
        KeyspaceName::UnquotedName(UnquotedName::default())
    }
}

impl KeyspaceName {
    pub fn check_kind_of_name(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        if lista.is_empty() {
            return Err(Error::SyntaxError("Faltan argumentos".to_string()));
        }
        if lista.len() > 2 && lista[0] == "\"" && lista[2] == "\"" {
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
}
