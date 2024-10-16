use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// '"' unquoted_name '"'
pub struct QuotedName {
    /// TODO: Desc básica
    name: String,
}

impl QuotedName {
    /// TODO: Desc básica
    pub fn new(word: String) -> Result<Self, Error> {
        // verificaciones
        Ok(QuotedName { name: word })
    }
}
