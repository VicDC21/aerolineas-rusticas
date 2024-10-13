use crate::cassandra::errors::error::Error;

#[derive(Default)]

pub struct UnquotedName {
    /// re('[a-zA-Z_0-9]\{1, 48}')
    name: String,
}

impl UnquotedName {
    pub fn new(word: String) -> Result<Self, Error> {
        // verificaciones
        Ok(UnquotedName { name: word })
    }


    pub fn is_unquoted_name(word: &str)-> bool{
        let length = word.chars().count();
        if length < 1 && length > 48 {
            return false;
        }
        word.chars().all(|c| c.is_ascii_alphanumeric())
    }

}
