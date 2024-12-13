use crate::protocol::aliases::results::Result;

/// '"' unquoted_name '"'
pub struct QuotedName {
    /// Nombre con comillas.
    name: String,
}

impl QuotedName {
    /// Devuelve el nombre con comillas.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl QuotedName {
    /// Crea un nuevo nombre con comillas.
    pub fn new(word: String) -> Result<Self> {
        Ok(QuotedName { name: word })
    }
}
