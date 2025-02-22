use crate::data_types::identifier::{
    quoted_identifier::QuotedIdentifier, unquoted_identifier::UnquotedIdentifier,
};
use protocol::aliases::results::Result;

/// column_name::= identifier
///
/// identifier::= unquoted_identifier | quoted_identifier

#[derive(Debug, PartialEq, Clone)]
pub enum Identifier {
    /// Identificador sin comillas.
    UnquotedIdentifier(UnquotedIdentifier),
    /// Identificador con comillas.
    QuotedIdentifier(QuotedIdentifier),
}

impl Identifier {
    /// Crea un nuevo identificador.
    pub fn new(string: String) -> Self {
        if UnquotedIdentifier::check_unquoted_identifier(&string) {
            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(string))
        } else {
            Identifier::QuotedIdentifier(QuotedIdentifier::new(string))
        }
    }

    /// Obtiene el nombre del identificador.
    pub fn get_name(&self) -> &str {
        match self {
            Identifier::QuotedIdentifier(id) => id.get_name(),
            Identifier::UnquotedIdentifier(id) => id.get_name(),
        }
    }

    /// Verifica si la lista de tokens es un identificador. Si lo es, lo retorna.
    /// Si no lo es, retorna None.
    pub fn check_identifier(lista: &mut Vec<String>) -> Result<Option<Identifier>> {
        if UnquotedIdentifier::check_unquoted_identifier(&lista[0]) {
            let string = lista.remove(0);
            return Ok(Some(Identifier::UnquotedIdentifier(
                UnquotedIdentifier::new(string),
            )));
        } else if QuotedIdentifier::check_quoted_identifier(&lista[0], &lista[1], &lista[2]) {
            lista.remove(0);
            let string = lista.remove(0);
            lista.remove(0);
            return Ok(Some(Identifier::QuotedIdentifier(QuotedIdentifier::new(
                string,
            ))));
        }
        Ok(None)
    }
}
