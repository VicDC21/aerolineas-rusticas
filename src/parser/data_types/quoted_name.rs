use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// '"' unquoted_name '"'
pub struct QuotedName {
    /// Nombre con comillas.
    name: String,
}

impl QuotedName {
    /// Crea un nuevo nombre con comillas.
    pub fn new(word: String) -> Result<Self, Error> {
        // TODO: Verificaciones
        Ok(QuotedName { name: word })
    }
}
