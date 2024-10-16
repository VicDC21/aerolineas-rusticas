use crate::protocol::errors::error::Error;

/// Representa la clave primaria de una tabla.
/// primary_key ::= PRIMARY KEY '(' column_name (',' column_name)* ')'

pub struct PrimaryKey {
    /// Un vector de nombres de columnas que componen la clave primaria.
    /// columns ::= column_name (',' column_name)*
    pub columns: Vec<String>,
}

impl PrimaryKey {
    /// Analiza una lista de nombres de columnas en un `PrimaryKey`.
    ///
    /// # Argumentos
    ///
    /// * `lista` - Una referencia mutable a un vector de cadenas que representan nombres de columnas.
    ///
    /// # Retornos
    ///
    /// * `Result<Self, Error>` - Un resultado que contiene el `PrimaryKey` si tiene éxito, o un `Error` si el análisis falla.
    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let mut columns = Vec::new();
        while !lista.is_empty() && lista[0] != ")" {
            columns.push(lista.remove(0));
        }
        Ok(PrimaryKey { columns })
    }
}
