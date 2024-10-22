//! Módulo para funciones auxiliares de rutas.

use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::data::utils::strings::unify_quotes_tokens;
use crate::protocol::aliases::results::Result;
use crate::protocol::errors::error::Error;

/// Genera un reader desde una ruta.
///
/// Se puede elegir si se intenta saltarse la primera línea.
pub fn reader_from(path: &str, skip_header: bool) -> Result<BufReader<File>> {
    match File::open(path) {
        Ok(file) => {
            let mut bufreader = BufReader::new(file);
            if skip_header {
                if let Err(err) = bufreader.read_line(&mut String::new()) {
                    println!("No se pudo saltar la primera línea:\n\n{}", err);
                }
            }
            Ok(bufreader)
        }
        Err(_) => Err(Error::ServerError(format!(
            "No se encontró un archivo en la ruta '{}'.",
            path
        ))),
    }
}

/// Separa un &[str] con un delimitador dado, y también verifica si tiene una longitud necesaria.
pub fn get_tokens(string: &str, delimiter: char, expected_len: usize) -> Result<Vec<String>> {
    let tokens = string.split(delimiter).collect::<Vec<&str>>();
    if tokens.len() < expected_len {
        return Err(Error::ServerError(format!(
            "La línea '{}' no parece tener suficientes elementos.",
            string
        )));
    }
    unify_quotes_tokens(tokens)
}
