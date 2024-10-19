/// re('[a-zA-Z][link:[a-zA-Z0-9]]*')

#[derive(Debug, PartialEq)]
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
        first.chars().all(|c| c.is_ascii_alphanumeric())
    }
}
