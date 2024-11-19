use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// re('[a-zA-Z_0-9]\{1, 48}')
#[derive(Default, Debug, PartialEq, Clone)]
pub struct UnquotedName {
    /// Nombre sin comillas.
    name: String,
}

impl UnquotedName {
    /// Crea un nuevo nombre sin comillas.
    pub fn new(word: String) -> Result<Self, Error> {
        Ok(UnquotedName { name: word })
    }

    /// Verifica si la palabra recibida es un nombre sin comillas.
    /// Si lo es retorna true, de lo contrario retorna false.
    pub fn is_unquoted_name(word: &str) -> bool {
        let length = word.chars().count();
        if !(1..=48).contains(&length) {
            return false;
        }
        word.chars()
            .all(|c| c.to_ascii_lowercase().is_ascii_alphanumeric() || c == '_' || c == ' ')
    }

    /// Devuelve el nombre como un String.
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
