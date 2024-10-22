use crate::protocol::errors::error::Error;

use super::quoted_identifier::QuotedIdentifier;
use super::unquoted_identifier::UnquotedIdentifier;

/// column_name::= identifier
/// identifier::= unquoted_identifier | quoted_identifier

#[derive(Debug, PartialEq)]
pub enum Identifier {
    /// TODO: Desc básica
    UnquotedIdentifier(UnquotedIdentifier),
    /// TODO: Desc básica
    QuotedIdentifier(QuotedIdentifier),
}

impl Identifier {
    /// TODO: Desc básica
    pub fn get_name(&self) -> &str {
        match self {
            Identifier::QuotedIdentifier(id) => id.get_name(),
            Identifier::UnquotedIdentifier(id) => id.get_name(),
        }
    }

    /// TODO: Desc básica
    pub fn check_identifier(lista: &mut Vec<String>) -> Result<Option<Identifier>, Error> {
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
