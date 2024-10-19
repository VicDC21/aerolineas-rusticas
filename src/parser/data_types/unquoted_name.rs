use crate::protocol::errors::error::Error;

#[allow(dead_code)]
/// re('[a-zA-Z_0-9]\{1, 48}')
#[derive(Default, Debug, PartialEq)]
pub struct UnquotedName {
    /// TODO: Desc básica
    name: String,
}

impl UnquotedName {
    /// TODO: Desc básica
    pub fn new(word: String) -> Result<Self, Error> {
        Ok(UnquotedName { name: word })
    }

    /// TODO: Desc básica
    pub fn is_unquoted_name(word: &str) -> bool {
        let length = word.chars().count();
        if !(1..=48).contains(&length) {
            return false;
        }
        word.chars().all(|c| c.is_ascii_alphanumeric())
    }
}
