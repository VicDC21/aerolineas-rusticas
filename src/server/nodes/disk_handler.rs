//! Módulo para manejo del almacenamiento en disco.

use std::{
    fs::{create_dir, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
};

use super::{
    column_config::ColumnConfig, keyspace::Keyspace, replication_strategy::ReplicationStrategy,
    table::Table,
};
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
};
use crate::server::nodes::node::NodeId;
use crate::{
    parser::statements::{
        ddl_statement::{
            create_keyspace::CreateKeyspace, create_table::CreateTable, option::Options,
        },
        dml_statement::main_statements::{
            insert::Insert,
            select::{order_by::OrderBy, ordering::ProtocolOrdering, select_operation::Select},
        },
    },
    protocol::{
        messages::responses::result::col_type::ColType, traits::Byteable,
        utils::encode_string_to_bytes,
    },
};

/// Encargado de hacer todas las operaciones sobre archivos en disco.
pub struct DiskHandler;

impl DiskHandler {
    /// Crea una carpeta de almacenamiento para el nodo.
    /// Devuelve la ruta a dicho almacenamiento.
    pub fn new_node_storage(id: NodeId) -> String {
        let main_path = "storage";
        DiskHandler::create_directory(main_path);
        let storage_addr: String = format!("{}/storage_node_{}", main_path, id);
        DiskHandler::create_directory(&storage_addr);
        storage_addr
    }

    fn create_directory(path: &str) {
        let path_folder = Path::new(path);
        if !path_folder.exists() && !path_folder.is_dir() {
            let err_msg = format!("No se pudo crear la carpeta de almacenamiento {}", path);
            create_dir(path_folder).expect(&err_msg);
        }
    }

    /// Crea un nuevo keyspace en el caso que corresponda.
    pub fn create_keyspace(
        statement: CreateKeyspace,
        storage_addr: &str,
    ) -> Result<Option<Keyspace>> {
        let keyspace_name = statement.keyspace_name.get_name();
        let keyspace_addr = format!("{}/{}", storage_addr, keyspace_name);
        let path_folder = Path::new(&keyspace_addr);
        if path_folder.exists() && path_folder.is_dir() {
            if statement.if_not_exist {
                return Ok(None);
            } else {
                return Err(Error::ServerError(format!(
                    "El keyspace {} ya existe",
                    keyspace_name
                )));
            }
        } else {
            create_dir(path_folder).map_err(|e| Error::ServerError(e.to_string()))?;
        }
        match Self::get_keyspace_replication(statement.options) {
            Ok(Some(replication)) => {
                Ok(Some(Keyspace::new(keyspace_name.to_string(), replication)))
            }
            Ok(None) => Err(Error::ServerError(
                "La estrategia de replicación es obligatoria".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Obtiene la estrategia de replicación de un keyspace.
    fn get_keyspace_replication(options: Vec<Options>) -> Result<Option<ReplicationStrategy>> {
        let mut i = 0;
        while i < options.len() {
            match &options[i] {
                Options::MapLiteral(map_literal) => {
                    let values = map_literal.get_values();
                    let (term1, term2) = &values[0];
                    if term1.get_value() == "class" && term2.get_value() == "SimpleStrategy" {
                        let (term3, term4) = &values[1];
                        if term3.get_value() == "replication_factor" {
                            let replicas = match term4.get_value().parse::<u32>() {
                                Ok(replicas) => replicas,
                                Err(_) => {
                                    return Err(Error::Invalid(
                                        "El valor de 'replication_factor' debe ser un número"
                                            .to_string(),
                                    ));
                                }
                            };
                            return Ok(Some(ReplicationStrategy::SimpleStrategy(replicas)));
                        } else {
                            return Err(Error::Invalid(
                                "Falto el campo replication_factor".to_string(),
                            ));
                        }
                    } else if term1.get_value() == "class"
                        && term2.get_value() == "NetworkTopologyStrategy"
                    {
                        // Aca estaria el caso de NetworkTopologyStrategy
                        todo!()
                    }
                }
                _ => break,
            }
            i += 1;
        }
        Ok(None)
    }

    /// Crea una nueva tabla en el caso que corresponda.
    pub fn create_table(
        statement: CreateTable,
        storage_addr: &str,
        default_keyspace: &str,
    ) -> Result<Option<Table>> {
        let table_name = statement.get_name();
        let keyspace_name = match statement.get_keyspace() {
            Some(keyspace) => keyspace,
            None => default_keyspace.to_string(),
        };
        let keyspace_addr = format!("{}/{}", storage_addr, keyspace_name);
        let path_folder = Path::new(&keyspace_addr);
        if !path_folder.exists() && !path_folder.is_dir() {
            return Err(Error::ServerError(format!(
                "El keyspace {} no existe",
                keyspace_name
            )));
        }
        let columns: Vec<ColumnConfig> = statement.get_columns()?;
        let columns_names = columns
            .iter()
            .map(|c| c.get_name())
            .collect::<Vec<String>>();
        let table_addr = format!("{}/{}/{}.csv", storage_addr, keyspace_name, table_name);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let mut writer = BufWriter::new(&file);
        writer
            .write_all(columns_names.join(",").as_bytes())
            .map_err(|e| Error::ServerError(e.to_string()))?;
        writer
            .write_all("\n".as_bytes())
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let primary_key = match statement.primary_key {
            Some(primary_key) => primary_key,
            None => {
                return Err(Error::SyntaxError(
                    "La clave primaria es obligatoria".to_string(),
                ))
            }
        };
        let partition_key = primary_key.partition_key;
        if primary_key.clustering_columns.is_empty() {
            return Ok(Some(Table::new(
                table_name,
                keyspace_name,
                columns,
                partition_key,
                None,
            )));
        }

        let clustering_keys = primary_key.clustering_columns;
        let mut clustering_keys_and_order: Vec<(String, ProtocolOrdering)> = Vec::new();
        for key in clustering_keys {
            clustering_keys_and_order.push((key, ProtocolOrdering::Asc));
        }

        if let Some(clustering_order) = &statement.clustering_order {
            for (key, order) in clustering_order {
                if let Some(j) = clustering_keys_and_order.iter().position(|(k, _)| k == key) {
                    let order = match ProtocolOrdering::ordering_from_str(order) {
                        Some(order) => order,
                        None => {
                            return Err(Error::Invalid(format!(
                                "La dirección de ordenación {} no es válida",
                                order
                            )));
                        }
                    };
                    clustering_keys_and_order[j] = (key.to_string(), order);
                } else {
                    return Err(Error::Invalid(format!(
                        "La columna {} no es parte de la clave de clustering",
                        key
                    )));
                }
            }
        }

        Ok(Some(Table::new(
            table_name,
            keyspace_name,
            columns,
            partition_key,
            Some(clustering_keys_and_order),
        )))
    }

    /// Inserta una nueva fila en una tabla en el caso que corresponda.
    pub fn do_insert(
        statement: &Insert,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
    ) -> Result<Vec<String>> {
        let keyspace = statement.table.get_keyspace();
        let name = statement.table.get_name();
        let table_addr = match keyspace {
            Some(keyspace) => format!("{}/{}/{}.csv", storage_addr, keyspace, name),
            None => format!("{}/{}/{}.csv", storage_addr, default_keyspace, name),
        };

        let mut content = String::new();
        {
            let file = OpenOptions::new()
                .read(true)
                .open(&table_addr)
                .map_err(|e| Error::ServerError(e.to_string()))?;
            let mut reader = BufReader::new(&file);
            reader
                .read_to_string(&mut content)
                .map_err(|e| Error::ServerError(e.to_string()))?;
        }

        let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        if lines.is_empty() {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                &table_addr
            )));
        }

        let header = lines[0].clone();
        let table_cols: Vec<&str> = header.split(",").collect();
        let query_cols = statement.get_columns_names();
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
        let mut id_position = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.starts_with(&values[0]) {
                id_exists = true;
                id_position = Some(i);
                break;
            }
        }

        if id_exists && statement.if_not_exists {
            return Ok(Vec::new());
        }

        let new_row = Self::generate_row_to_insert(&values, &query_cols, &table_cols);
        if let Some(pos) = id_position {
            lines[pos] = new_row.clone();
        } else {
            lines.push(new_row.clone());
        }

        let mut writer = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                writer
                    .write_all(b"\n")
                    .map_err(|e| Error::ServerError(e.to_string()))?;
            }
            writer
                .write_all(line.as_bytes())
                .map_err(|e| Error::ServerError(e.to_string()))?;
        }

        if let Some(order_by) = &table.clustering_key_and_order {
            Self::order_table_data(
                &table_addr,
                &query_cols,
                &OrderBy::new_from_vec(order_by.to_vec()),
            )?;
        }

        Ok(new_row.trim().split(",").map(|s| s.to_string()).collect())
    }

    /// Genera una fila para insertar en una tabla, en base a las columnas dadas en la query y de la tabla.
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

    /// Ordena una tabla según el criterio de ordenación dado.
    fn order_table_data(table_addr: &str, table_cols: &[String], order_by: &OrderBy) -> Result<()> {
        let file = OpenOptions::new()
            .read(true)
            .open(table_addr)
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

        let mut rows: Vec<Vec<String>> = Vec::new();
        for line in reader.lines() {
            let table_line = line.map_err(|e| Error::ServerError(e.to_string()))?;
            let table_row: Vec<String> = table_line
                .trim()
                .split(",")
                .map(|s| s.to_string())
                .collect();
            rows.push(table_row);
        }

        order_by.order(&mut rows, table_cols);

        let mut ordered_table_data = Vec::new();
        ordered_table_data.push(table_cols.join(","));
        for row in rows {
            ordered_table_data.push(row.join(","));
        }
        let final_table_data = ordered_table_data.join("\n");

        let ordered_table = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(table_addr)
            .map_err(|e| Error::ServerError(e.to_string()))?;
        let mut writer = BufWriter::new(&ordered_table);

        writer
            .write_all(final_table_data.as_bytes())
            .map_err(|e| Error::ServerError(e.to_string()))?;

        Ok(())
    }

    /// Selecciona filas en una tabla en el caso que corresponda.
    pub fn do_select(statement: &Select, storage_addr: &str, table: &Table) -> Result<Vec<Byte>> {
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
            Self::do_select_and_order(statement, &mut reader, &table_cols, order)?
        } else {
            Self::do_select_and_not_order(statement, &mut reader, &table_cols)?
        };
        result.extend(result_rows);

        Ok(Self::serialize_select_result(
            result,
            &query_cols,
            &table_cols,
            table,
        ))
    }

    /// Serializa en bytes el resultado de una consulta SELECT.
    fn serialize_select_result(
        result: Vec<Vec<String>>,
        query_cols: &[String],
        table_cols: &[String],
        table: &Table,
    ) -> Vec<u8> {
        //kind of Result
        let mut res: Vec<u8> = vec![0x0, 0x0, 0x0, 0x2];
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

        let cols_name_and_type = table.get_columns_name_and_data_type();
        for (col_name, data_type) in cols_name_and_type {
            let col_type = ColType::from(data_type);
            metadata.append(&mut encode_string_to_bytes(&col_name));
            metadata.append(&mut col_type.as_bytes())
        }

        let rows_count = result.len() as i32;

        let mut rows_content: Vec<u8> = Vec::new();
        for row in result {
            for value in row {
                let value_lenght = value.len() as i32;
                rows_content.append(&mut value_lenght.to_be_bytes().to_vec());
                rows_content.append(&mut value.as_bytes().to_vec());
            }
        }
        // let mut rows_content: Vec<u8> = result
        //     .into_iter()
        //     .flat_map(|subvec| subvec.into_iter().flat_map(|s| s.into_bytes()))
        //     .collect();

        res.append(&mut metadata);
        res.append(&mut rows_count.to_be_bytes().to_vec());
        res.append(&mut rows_content);

        res
    }

    /// Realiza una consulta SELECT y ordena las filas según el criterio de ordenación dado.
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

    /// Realiza una consulta SELECT sin ordenar las filas.
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

    /// Genera una fila para seleccionar en una tabla, en base a las columnas dadas en la query y de la tabla.
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
        new_row.push("\n".to_string());
        new_row
    }
}
