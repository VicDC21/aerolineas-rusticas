/// re('[a-zA-Z][link:[a-zA-Z0-9]]*')

pub struct UnquotedIdentifier {
    text: String,
}

impl UnquotedIdentifier {
    /// TODO: Desc básica
    pub fn new(text: String) -> Self {
        UnquotedIdentifier { text }
    }
    /// TODO: Desc básica
    pub fn get_name(&self) -> &str {
        &self.text
    }
    /// TODO: Desc básica
    pub fn check_unquoted_identifier(first: &str) -> bool {
        if !first.chars().all(char::is_alphabetic) {
            return false;
        }
        true
    }
}
