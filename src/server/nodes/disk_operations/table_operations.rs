//! Módulo que detalla las operaciones de tablas

use {
    crate::{
        protocol::{aliases::results::Result, errors::error::Error},
        server::nodes::disk_operations::table_path::TablePath,
    },
    std::{
        fs::OpenOptions,
        io::{BufRead, BufReader},
    },
};

/// Estructura para manejar operaciones comunes sobre tablas
pub struct TableOperations {
    /// Ruta de la tabla
    pub path: TablePath,
    /// Columnas de la tabla
    pub columns: Vec<String>,
}

impl TableOperations {
    /// Crea una nueva instancia de `TableOperations`.
    pub fn new(path: TablePath) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path.full_path())
            .map_err(|_| {
                Error::ServerError(format!(
                    "No se pudo abrir {} para lectura",
                    path.full_path()
                ))
            })?;

        let mut reader = BufReader::new(&file);
        let mut header = String::new();
        reader.read_line(&mut header).map_err(|_| {
            Error::ServerError(format!(
                "No se pudo abrir {} para escritura",
                path.full_path()
            ))
        })?;

        if header.trim().is_empty() {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                path.full_path()
            )));
        }

        let columns: Vec<String> = header.trim().split(',').map(|s| s.to_string()).collect();

        Ok(Self { path, columns })
    }

    /// Valida que las columnas existan en la tabla.
    pub fn validate_columns(&self, columns: &[String]) -> Result<()> {
        for col in columns {
            if !self.columns.contains(col) {
                return Err(Error::ServerError(format!(
                    "La tabla con ruta {} no contiene la columna {}",
                    self.path.full_path(),
                    col
                )));
            }
        }
        Ok(())
    }

    /// Saca la columna del timestamp de las filas
    pub fn remove_row_timestamp_column(&mut self) {
        self.columns.pop();
    }

    /// Lee las filas de la tabla, sin contar la columna extra del timestamp.
    pub fn read_rows(&self, without_timestamp: bool) -> Result<Vec<Vec<String>>> {
        let file = OpenOptions::new()
            .read(true)
            .open(self.path.full_path())
            .map_err(|_| {
                Error::ServerError(format!(
                    "No se pudo abrir {} para lectura",
                    self.path.full_path()
                ))
            })?;

        let reader = BufReader::new(file);
        let mut rows = Vec::new();

        for line in reader.lines().skip(1) {
            let line = line.map_err(|e| Error::ServerError(e.to_string()))?;
            if !line.trim().is_empty() {
                let mut line_separated: Vec<String> =
                    line.trim().split(',').map(|s| s.to_string()).collect();
                if without_timestamp {
                    line_separated.pop(); // saco la columna del timestamp
                }
                rows.push(line_separated);
            }
        }

        Ok(rows)
    }

    /// Escribe las filas en la tabla.
    pub fn write_rows(&self, rows: &[Vec<String>]) -> Result<()> {
        let mut content = self.columns.join(",");
        content.push('\n');

        for row in rows {
            content.push_str(&row.join(","));
            content.push('\n');
        }

        std::fs::write(self.path.full_path(), content).map_err(|_| {
            Error::ServerError(format!("No se pudo escribir en {}", self.path.full_path()))
        })
    }
}
