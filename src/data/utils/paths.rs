//! Módulo para funciones auxiliares de rutas.

use std::fs::File;
use std::io::BufReader;

use crate::protocol::aliases::results::Result;
use crate::protocol::errors::error::Error;

/// Genera un reader desde una ruta.
pub fn reader_from(path: &str) -> Result<BufReader<File>> {
    match File::open(path) {
        Ok(file) => Ok(BufReader::new(file)),
        Err(_) => Err(Error::ServerError(format!(
            "No se encontró un archivo en la ruta '{}'.",
            path
        ))),
    }
}

/// Separa un &[str] con un delimitador dado, y también verifica si tiene una longitud necesaria.
pub fn get_tokens(string: &str, delimiter: char, expected_len: usize) -> Result<Vec<&str>> {
    let tokens = string.split(delimiter).collect::<Vec<&str>>();
    if tokens.len() < expected_len {
        return Err(Error::ServerError(format!(
            "La línea '{}' no parece tener suficientes elementos.",
            string
        )));
    }
    Ok(tokens)
}
