use crate::cassandra::errors::error::Error;

/// '"' unquoted_name '"'
pub struct QuotedName {
    name: String,
}

impl QuotedName {
    pub fn new(word: String) -> Result<Self, Error> {
        // verificaciones
        Ok(QuotedName { name: word })
    }
}
