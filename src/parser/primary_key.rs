use crate::protocol::errors::error::Error;

use super::statements::ddl_statement::ddl_statement_parser::check_words;

/// Representa la clave primaria de una tabla.
/// primary_key ::= PRIMARY KEY '(' column_name (',' column_name)* ')'

#[derive(Debug)]
pub struct PrimaryKey {
    /// Un vector de nombres de columnas que componen la clave primaria.
    /// columns ::= column_name (',' column_name)*
    pub partition_key: Vec<String>,

    /// Un vector de nombres de columnas que componen la clave de agrupamiento.
    /// clustering_columns ::= column_name (',' column_name)*
    pub clustering_columns: Vec<String>,
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
        let mut partition_key = Vec::new();
        let mut clustering_columns = Vec::new();

        if check_words(lista, "(") {
            while !lista.is_empty() && !check_words(lista, ")") {
                partition_key.push(lista.remove(0));
                check_words(lista, ",");
            }
        } else {
            partition_key.push(lista.remove(0));
        }

        if check_words(lista, ",") {
            while !lista.is_empty() && !check_words(lista, ")") {
                clustering_columns.push(lista.remove(0));
                check_words(lista, ",");
            }
        }

        Ok(PrimaryKey {
            partition_key,
            clustering_columns,
        })
    }
}
