/// '"' (any character where " can appear if doubled)+ '"'

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
    pub fn check_quoted_identifier(
        first_quotes: &String,
        word: &str,
        second_quotes: &String,
    ) -> bool {
        if first_quotes != "\"" || second_quotes != "\"" {
            return false;
        }
        if !word.chars().all(char::is_alphabetic) {
            return false;
        }
        true
    }
}
