//! Módulo para manejo del almacenamiento en disco.

use std::{
    fs::{create_dir, File, OpenOptions},
    io::{BufRead, BufReader, Read, Seek, SeekFrom, Write},
    path::Path,
};

use crate::parser::statements::dml_statement::main_statements::{
    insert::Insert,
    select::{order_by::OrderBy, select_operation::Select},
};
use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::node::NodeId;

/// Encargado de hacer todas las operaciones sobre archivos en disco.
pub struct DiskHandler;

impl DiskHandler {
    /// Crea una carpeta de almacenamiento para el nodo.
    /// Devuelve la ruta a dicho almacenamiento.
    pub fn new_node_storage(id: NodeId) -> String {
        let path_folder = Path::new("storage");
        if !path_folder.exists() && !path_folder.is_dir() {
            create_dir(path_folder).expect("No se pudo crear la carpeta de almacenamiento");
        }
        let storage_addr: String = format!("storage/storage_node_{}", id);
        let path_folder = Path::new(&storage_addr);
        if !path_folder.exists() && !path_folder.is_dir() {
            create_dir(path_folder)
                .expect("No se pudo crear la carpeta de almacenamiento del nodo");
        }
        storage_addr
    }

    /// Inserta una nueva fila en una tabla en el caso que corresponda.
    pub fn do_insert(statement: Insert, storage_addr: &str) -> Result<()> {
        let keyspace = statement.table_name.get_keyspace();
        let name = statement.table_name.get_name();
        let table_addr = match keyspace {
            Some(keyspace) => format!("{}/{}/{}.csv", storage_addr, keyspace, name),
            None => format!("{}/{}.csv", storage_addr, name),
        };

        let file = OpenOptions::new()
            .read(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let mut reader = BufReader::new(&file);

        let mut line = String::new();
        let read_bytes = reader
            .read_line(&mut line)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        if read_bytes == 0 {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                &table_addr
            )));
        }
        line = line.trim().to_string();

        let query_cols = statement.get_columns_names();
        let table_cols: Vec<&str> = line.split(",").collect();
        for col in &query_cols {
            if !table_cols.contains(&col.as_str()) {
                return Err(Error::ServerError(format!(
                    "La tabla con ruta {} no contiene la columna {}",
                    &table_addr, col
                )));
            }
        }

        let values = statement.get_values();
        let mut id_exists = false;
        let mut buffer = Vec::new();
        let mut position = 0;
        // Leo línea por línea y verifico si el ID de la fila ya existe
        while let Some(Ok(line)) = reader.by_ref().lines().next() {
            if line.starts_with(&values[0]) {
                id_exists = true;
                break;
            }
            position += line.len() + 1; // Actualizo la posicion a sobreescribir si existe el ID
            buffer.push(line);
        }
        // Si el ID existe y no se debe sobreescribir la línea, no hago nada
        if id_exists && statement.if_not_exists {
            return Ok(());
        }

        // Abro el archivo nuevamente para escribir
        let mut writer = OpenOptions::new()
            .write(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        let new_row = Self::generate_row_to_insert(&values, &query_cols, &table_cols);
        if id_exists {
            // Si el ID ya existia, sobrescribo la línea
            writer
                .seek(SeekFrom::Start(position as u64))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(new_row.as_bytes())
                .map_err(|e| Error::ServerError(e.to_string()))?;
        } else {
            // Si no existia el ID, escribo al final del archivo
            writer
                .seek(SeekFrom::End(0))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(new_row.as_bytes())
                .map_err(|e| Error::ServerError(e.to_string()))?;
        }

        Ok(())
    }

    fn generate_row_to_insert(
        values: &[String],
        query_cols: &[String],
        table_cols: &[&str],
    ) -> String {
        let mut values_to_insert: Vec<&str> = vec![""; table_cols.len()];

        for i in 0..query_cols.len() {
            if let Some(j) = table_cols.iter().position(|c| *c == query_cols[i]) {
                values_to_insert[j] = values[i].as_str();
            }
        }

        values_to_insert.join(",") + "\n"
    }

    /// Selecciona filas en una tabla en el caso que corresponda.
    pub fn do_select(statement: Select, storage_addr: &str) -> Result<Vec<u8>> {
        let keyspace = statement.from.get_keyspace();
        let name = statement.from.get_name();
        let table_addr = match keyspace {
            Some(keyspace) => format!("{}/{}/{}.csv", storage_addr, keyspace, name),
            None => format!("{}/{}.csv", storage_addr, name),
        };

        let file = OpenOptions::new()
            .read(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let mut reader = BufReader::new(&file);

        let mut line = String::new();
        let read_bytes = reader
            .read_line(&mut line)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        if read_bytes == 0 {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                &table_addr
            )));
        }
        line = line.trim().to_string();

        let table_cols: Vec<String> = line.split(",").map(|s| s.to_string()).collect();
        let query_cols = statement.columns.get_columns();

        if query_cols.len() != 1 && query_cols[0] != "*" {
            for col in &query_cols {
                if !table_cols.contains(col) {
                    return Err(Error::ServerError(format!(
                        "La tabla con ruta {} no contiene la columna {}",
                        &table_addr, col
                    )));
                }
            }
        }

        let mut result: Vec<Vec<String>> = Vec::new();
        if query_cols.len() == 1 && query_cols[0] == "*" {
            result.push(table_cols.clone());
        } else {
            result.push(query_cols.clone());
        }

        let result_rows = if let Some(order) = &statement.options.order_by {
            Self::do_select_and_order(&statement, &mut reader, &table_cols, order)?
        } else {
            Self::do_select_and_not_order(&statement, &mut reader, &table_cols)?
        };
        result.extend(result_rows);

        Ok(Self::serialize_result(result, &query_cols, &table_cols))
    }

    fn serialize_result(
        result: Vec<Vec<String>>,
        query_cols: &[String],
        table_cols: &[String],
    ) -> Vec<u8> {
        let mut metadata: Vec<u8> = Vec::new();

        // <flags>, por ahora la mascara tiene seteados todos los valores en 0
        let flags: i32 = 0;
        metadata.append(&mut flags.to_be_bytes().to_vec());

        // <columns_count>
        if query_cols[0] == "*" {
            metadata.append(&mut table_cols.len().to_be_bytes().to_vec())
        } else {
            metadata.append(&mut query_cols.len().to_be_bytes().to_vec())
        }

        // Si activamos flags entonces aca iria
        // [<paging_state>][<new_metadata_id>][<global_table_spec>?<col_spec_1>...<col_spec_n>]
        let rows_count = result.len() as i32;
        let mut rows_content: Vec<u8> = result
            .into_iter()
            .flat_map(|subvec| subvec.into_iter().flat_map(|s| s.into_bytes()))
            .collect();

        let mut res = Vec::new();
        res.append(&mut metadata);
        res.append(&mut rows_count.to_be_bytes().to_vec());
        res.append(&mut rows_content);

        res
    }

    fn do_select_and_order(
        statement: &Select,
        reader: &mut BufReader<&File>,
        table_cols: &[String],
        order_by: &OrderBy,
    ) -> Result<Vec<Vec<String>>> {
        let mut rows = Vec::new();

        for line in reader.lines() {
            let table_line = line.map_err(|e| Error::ServerError(e.to_string()))?;
            let table_row: Vec<String> = table_line
                .trim()
                .split(",")
                .map(|s| s.to_string())
                .collect();

            if let Some(the_where) = &statement.options.the_where {
                if the_where.filter(&table_row, table_cols)? {
                    rows.push(table_row);
                }
            } else {
                rows.push(table_row);
            }
        }

        order_by.order(&mut rows, table_cols);

        let query_cols = statement.columns.get_columns();
        let mut result_rows = Vec::new();
        for row in rows {
            result_rows.push(Self::generate_row_to_select(&row, table_cols, &query_cols));
        }

        Ok(result_rows)
    }

    fn do_select_and_not_order(
        statement: &Select,
        reader: &mut BufReader<&File>,
        table_cols: &[String],
    ) -> Result<Vec<Vec<String>>> {
        let mut result_rows = Vec::new();
        let query_cols = statement.columns.get_columns();

        let mut line = String::new();
        let mut read_bytes = 1;
        while read_bytes != 0 {
            read_bytes = reader
                .read_line(&mut line)
                .map_err(|e| Error::ServerError(e.to_string()))?;
            if read_bytes == 0 {
                break;
            }

            line = line.trim().to_string();
            let table_row: Vec<String> = line.trim().split(",").map(|s| s.to_string()).collect();

            if let Some(the_where) = &statement.options.the_where {
                if the_where.filter(&table_row, table_cols)? {
                    result_rows.push(Self::generate_row_to_select(
                        &table_row,
                        table_cols,
                        &query_cols,
                    ));
                }
            } else {
                result_rows.push(Self::generate_row_to_select(
                    &table_row,
                    table_cols,
                    &query_cols,
                ));
            }

            line.clear();
        }

        Ok(result_rows)
    }

    fn generate_row_to_select(
        table_row: &[String],
        table_cols: &[String],
        query_cols: &[String],
    ) -> Vec<String> {
        let mut new_row: Vec<String> = Vec::new();
        if query_cols.len() == 1 && query_cols[0] == "*" {
            new_row = table_row.to_vec();
        } else {
            for query_col in query_cols {
                if let Some(j) = table_cols
                    .iter()
                    .position(|table_col| *table_col == *query_col)
                {
                    new_row.push(table_row[j].clone());
                }
            }
        }

        new_row
    }
}
