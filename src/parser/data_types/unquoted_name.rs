use crate::cassandra::errors::error::Error;

#[derive(Default)]

pub struct UnquotedName {
    name: String,
}

impl UnquotedName {
    pub fn new(word: String) -> Result<Self, Error> {
        // verificaciones
        Ok(UnquotedName { name: word })
    }
}
