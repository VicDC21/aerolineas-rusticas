use crate::cassandra::errors::error::Error;

pub struct UnquotedName {
    name: String,
}

impl UnquotedName {
    pub fn new(word: String) -> Result<Self, Error> {
        // verificaciones
        Ok(UnquotedName { name: word })
    }
}
