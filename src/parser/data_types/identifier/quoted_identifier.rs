/// '"' (any character where " can appear if doubled)+ '"'
#[derive(Debug, PartialEq, Clone)]
pub struct QuotedIdentifier {
    /// Nombre con comillas.
    text: String,
}

impl QuotedIdentifier {
    /// Crea un nuevo identificador con comillas.
    pub fn new(text: String) -> Self {
        QuotedIdentifier { text }
    }

    /// Obtiene el nombre del identificador.
    pub fn get_name(&self) -> &str {
        &self.text
    }

    /// Verifica si los quotes y la palabra constituyen un identificador con comillas.
    /// Si lo es, returna true. En caso contrario, retorna false.
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
