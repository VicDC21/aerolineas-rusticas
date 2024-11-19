/// re('[a-zA-Z][link:[a-zA-Z0-9]]*')
#[derive(Debug, PartialEq, Clone)]
pub struct UnquotedIdentifier {
    /// Nombre sin comillas.
    text: String,
}

impl UnquotedIdentifier {
    /// Crea un nuevo identificador sin comillas.
    pub fn new(text: String) -> Self {
        UnquotedIdentifier { text }
    }

    /// Obtiene el nombre del identificador.
    pub fn get_name(&self) -> &str {
        &self.text
    }

    /// Verifica si el string recibido constituye un identificador sin comillas.
    /// Si lo es, returna true. En caso contrario, retorna false.
    pub fn check_unquoted_identifier(first: &str) -> bool {
        first.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}
