/// '"' (any character where " can appear if doubled)+ '"'

#[derive(Debug, PartialEq)]
pub struct QuotedIdentifier {
    text: String,
}

impl QuotedIdentifier {
    /// TODO: Desc básica
    pub fn new(text: String) -> Self {
        QuotedIdentifier { text }
    }

    /// TODO: Desc básica
    pub fn get_name(&self) -> &str {
        &self.text
    }

    /// TODO: Desc básica
    pub fn check_quoted_identifier(first_quotes: &str, word: &str, second_quotes: &str) -> bool {
        if (first_quotes != "\"" && first_quotes != "\'")
            || (second_quotes != "\"" && second_quotes != "\'")
        {
            return false;
        }

        if !word.chars().all(|c| c.is_ascii_alphanumeric() || c != '_') {
            return false;
        }
        true
    }
}
