//! Módulo para manejo del almacenamiento en disco.

use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    messages::responses::result::col_type::ColType,
    traits::Byteable,
    utils::encode_string_to_bytes,
};
use crate::server::nodes::{
    graph::NODES_PATH,
    keyspace_metadata::{keyspace::Keyspace, replication_strategy::ReplicationStrategy},
    node::NodeId,
    table_metadata::table::Table,
};
use crate::{
    parser::{
        assignment::Assignment,
        data_types::{constant::Constant, term::Term},
        primary_key::PrimaryKey,
        statements::{
            ddl_statement::{
                create_keyspace::CreateKeyspace, create_table::CreateTable, option::Options,
            },
            dml_statement::{
                if_condition::{Condition, IfCondition},
                main_statements::{
                    delete::Delete,
                    insert::Insert,
                    select::{
                        order_by::OrderBy, ordering::ProtocolOrdering, select_operation::Select,
                    },
                    update::Update,
                },
                r#where::operator::Operator,
            },
        },
    },
    protocol::aliases::types::Int,
};

use std::{
    fs::{create_dir, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
    str::FromStr,
};

use super::{
    row_operations::RowOperations, table_operations::TableOperations, table_path::TablePath,
};

/// La ruta para el almacenamiento de los nodos.
pub const STORAGE_PATH: &str = "storage";
/// El nombre individual del directorio de un nodo.
pub const STORAGE_NODE_PATH: &str = "storage_node";

/// Encargado de hacer todas las operaciones sobre archivos en disco.
pub struct DiskHandler;

impl DiskHandler {
    /// Crea una carpeta de almacenamiento para el nodo.
    /// Devuelve la ruta a dicho almacenamiento.
    pub fn new_node_storage(id: NodeId) -> Result<String> {
        Self::create_directory(STORAGE_PATH)?;
        let storage_addr: String = Self::get_node_storage(id);
        Self::create_directory(&storage_addr)?;
        Ok(storage_addr)
    }

    /// Obtiene la ruta de almacenamiento de un nodo dado su ID.
    pub fn get_node_storage(id: NodeId) -> String {
        format!("{}/{}_{}", STORAGE_PATH, STORAGE_NODE_PATH, id)
    }

    /// Almacena los metadatos de un nodo en el archivo de metadatos de los nodos `nodes.csv`.
    pub fn store_node_metadata(id: NodeId, metadata: &[Byte]) -> Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .read(true)
            .open(NODES_PATH)
            .map_err(|e| {
                Error::ServerError(format!("Error abriendo el archivo de metadata: {}", e))
            })?;
        let mut reader = BufReader::new(&file);

        let mut id_exists = false;
        let mut buf = Vec::new();
        let mut pos = 0;
        // Leo línea por línea y verifico si el ID del nodo ya existe
        while let Some(Ok(line)) = reader.by_ref().lines().next() {
            if line.starts_with(&id.to_string()) {
                id_exists = true;
                break;
            }
            pos += line.len() + 1; // Actualizo la posicion a sobreescribir si existe el ID
            buf.push(line);
        }

        // Abro el archivo nuevamente para escribir
        let mut writer = OpenOptions::new()
            .write(true)
            .open(NODES_PATH)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        if id_exists {
            // Si el ID ya existia, sobrescribo la linea
            writer
                .seek(SeekFrom::Start(pos as u64))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(metadata)
                .map_err(|e| Error::ServerError(e.to_string()))?;
        } else {
            // Si no existia el ID, escribo al final del archivo
            writer
                .seek(SeekFrom::End(0))
                .map_err(|e| Error::ServerError(e.to_string()))?;
            writer
                .write_all(metadata)
                .map_err(|e| Error::ServerError(e.to_string()))?;
        }

        Ok(())
    }

    /// Crea un nuevo keyspace en el caso que corresponda.
    pub fn create_keyspace(
        statement: &CreateKeyspace,
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
        match Self::get_keyspace_replication(statement.get_options()) {
            Ok(Some(replication)) => {
                Ok(Some(Keyspace::new(keyspace_name.to_string(), replication)))
            }
            Ok(None) => Err(Error::ServerError(
                "La estrategia de replicación es obligatoria".to_string(),
            )),
            Err(e) => Err(e),
        }
    }

    /// Elimina un keyspace en el caso que corresponda.
    pub fn drop_keyspace(keyspace_name: &str, storage_addr: &str) -> Result<()> {
        let keyspace_addr = format!("{}/{}", storage_addr, keyspace_name);
        let path_folder = Path::new(&keyspace_addr);

        if path_folder.exists() && path_folder.is_dir() {
            std::fs::remove_dir_all(path_folder).map_err(|e| {
                Error::ServerError(format!(
                    "Error al eliminar el keyspace {}: {}",
                    keyspace_name, e
                ))
            })?;
            Ok(())
        } else {
            Err(Error::ServerError(format!(
                "El directorio del keyspace {} no existe",
                keyspace_name
            )))
        }
    }

    /// Crea una nueva tabla en el caso que corresponda.
    pub fn create_table(
        statement: &CreateTable,
        storage_addr: &str,
        default_keyspace: &str,
        node_number: Byte,
    ) -> Result<Option<Table>> {
        let (keyspace_name, table_name) =
            Self::validate_and_get_keyspace_table_names(statement, default_keyspace, storage_addr)?;
        let columns = statement.get_columns()?;
        let columns_names = columns
            .iter()
            .map(|c| c.get_name())
            .collect::<Vec<String>>();

        Self::create_table_csv_file(
            storage_addr,
            &keyspace_name,
            &table_name,
            &columns_names,
            node_number,
        )?;

        let primary_key = Self::validate_and_get_primary_key(statement)?;
        let clustering_keys_and_order = Self::get_clustering_keys_and_order(statement)?;

        Ok(Some(Table::new(
            table_name,
            keyspace_name,
            columns,
            primary_key.partition_key,
            clustering_keys_and_order,
        )))
    }

    /// Repara las filas de la tabla con las filas pasadas por parámetro.
    ///
    /// **PRECAUCIÓN**: Esta función trunca todo el contenido previo de la tabla y este es irrecuperable luego de su uso, por lo que se debe
    /// tener cuidado al utilizarla.
    pub fn repair_rows(
        storage_addr: &str,
        table_name: &str,
        keyspace_name: &str,
        default_keyspace: &str,
        node_number: Byte,
        repaired_rows: &[Vec<String>],
    ) -> Result<()> {
        let path = TablePath::new(
            storage_addr,
            Some(keyspace_name.to_string()),
            table_name,
            default_keyspace,
            node_number,
        );

        let table_ops = TableOperations::new(path)?;
        table_ops.write_rows(repaired_rows)?;

        Ok(())
    }

    /// Inserta una nueva fila en una tabla en el caso que corresponda.
    pub fn do_insert(
        statement: &Insert,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
        timestamp: i64,
        node_number: Byte,
    ) -> Result<Vec<String>> {
        let path = TablePath::new(
            storage_addr,
            statement.table.get_keyspace(),
            &statement.table.get_name(),
            default_keyspace,
            node_number,
        );
        let table_ops = TableOperations::new(path)?;
        table_ops.validate_columns(&statement.get_columns_names())?;
        let mut rows = table_ops.read_rows(false)?;
        let values = statement.get_values();
        let new_row = Self::generate_row_values(statement, &table_ops, &values, timestamp);
        if !rows.contains(&new_row) {
            rows.push(new_row.clone());
            Self::order_and_save_rows(&table_ops, &mut rows, table)?;
            return Ok(new_row);
        }
        Ok(Vec::new())
    }

    /// Devuelve las filas de la tabla como un string
    pub fn get_rows_with_timestamp_as_string(
        storage_addr: &str,
        default_keyspace: &str,
        statement: &Select,
        node_number: Byte,
    ) -> Result<String> {
        let path = TablePath::new(
            storage_addr,
            statement.from.get_keyspace(),
            &statement.from.get_name(),
            default_keyspace,
            node_number,
        );
        let table_ops = TableOperations::new(path)?;
        let query_cols = vec!["*".to_string()];
        let mut rows = table_ops.read_rows(false)?;
        if let Some(the_where) = &statement.options.the_where {
            rows.retain(|row| the_where.filter(row, &table_ops.columns).unwrap_or(false));
        }

        if let Some(order) = &statement.options.order_by {
            order.order(&mut rows, &table_ops.columns);
        }
        let result_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| Self::generate_row_to_select(&row, &table_ops.columns, &query_cols, false))
            .collect();

        let rows_as_string = result_rows
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<String>>()
            .join("\n");
        Ok(rows_as_string)
    }

    /// Selecciona filas en una tabla en el caso que corresponda.
    pub fn do_select(
        statement: &Select,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
        node_number: Byte,
    ) -> Result<Vec<Byte>> {
        let path = TablePath::new(
            storage_addr,
            statement.from.get_keyspace(),
            &statement.from.get_name(),
            default_keyspace,
            node_number,
        );

        let mut table_ops = TableOperations::new(path)?;
        table_ops.remove_row_timestamp_column();
        let query_cols = statement.columns.get_columns();

        if query_cols.len() != 1 && query_cols[0] != "*" {
            table_ops.validate_columns(&query_cols)?;
        }

        let mut result = Vec::new();
        // if query_cols.len() == 1 && query_cols[0] == "*" {
        //     result.push(table_ops.columns.clone());
        // } else {
        //     result.push(query_cols.clone());
        // }

        let mut rows = table_ops.read_rows(true)?;
        if let Some(the_where) = &statement.options.the_where {
            rows.retain(|row| the_where.filter(row, &table_ops.columns).unwrap_or(false));
        }

        if let Some(order) = &statement.options.order_by {
            order.order(&mut rows, &table_ops.columns);
        }

        let result_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| Self::generate_row_to_select(&row, &table_ops.columns, &query_cols, true))
            .collect();

        result.extend(result_rows);

        Ok(Self::serialize_select_result(
            result,
            &query_cols,
            &table_ops.columns,
            table,
        ))
    }

    /// Actualiza filas en una tabla en el caso que corresponda.
    pub fn do_update(
        statement: &Update,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
        node_number: Byte,
    ) -> Result<Vec<String>> {
        let path = TablePath::new(
            storage_addr,
            statement.table_name.get_keyspace(),
            &statement.table_name.get_name(),
            default_keyspace,
            node_number,
        );
        let table_ops = TableOperations::new(path)?;
        Self::validate_update_columns(&table_ops, &statement.set_parameter)?;
        let mut rows = table_ops.read_rows(true)?;

        if matches!(statement.if_condition, IfCondition::Exists) && rows.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_rows = Vec::new();
        let mut should_write = false;

        if matches!(statement.if_condition, IfCondition::Conditions(_))
            && statement.the_where.is_none()
        {
            if let IfCondition::Conditions(conditions) = &statement.if_condition {
                let all_conditions_met =
                    RowOperations::verify_row_conditions(&rows, conditions, &table_ops.columns)?;
                if !all_conditions_met {
                    return Ok(Vec::new());
                }
                should_write = true;
            }
        }

        let mut modified_rows = Vec::new();
        for row in rows.iter_mut() {
            if RowOperations::should_process_row(
                row,
                &statement.if_condition,
                &table_ops.columns,
                statement.the_where.as_ref(),
            )? {
                for assignment in &statement.set_parameter {
                    Self::update_row_value(row, assignment, &table_ops.columns)?;
                }
                updated_rows.push(row.clone());
                should_write = true;
            }
            modified_rows.push(row.clone());
        }

        if should_write {
            Self::order_and_save_rows(&table_ops, &mut rows, table)?;
        }

        Ok(updated_rows.iter().map(|row| row.join(",")).collect())
    }

    /// Elimina filas en una tabla en el caso que corresponda.
    pub fn do_delete(
        statement: &Delete,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
        node_number: Byte,
    ) -> Result<Vec<String>> {
        let path = TablePath::new(
            storage_addr,
            statement.from.get_keyspace(),
            &statement.from.get_name(),
            default_keyspace,
            node_number,
        );

        let table_ops = TableOperations::new(path)?;
        let rows = table_ops.read_rows(true)?;

        if matches!(statement.if_condition, IfCondition::Exists) && rows.is_empty() {
            return Ok(Vec::new());
        }

        let result = if statement.cols.is_empty() {
            Self::process_full_row_delete(statement, &rows, &table_ops)?
        } else {
            Self::process_partial_row_delete(statement, &rows, &table_ops)?
        };

        let (modified_rows, deleted_data) = result;

        if !deleted_data.is_empty() || modified_rows.is_empty() {
            let mut rows_to_write = modified_rows.clone();
            Self::order_and_save_rows(&table_ops, &mut rows_to_write, table)?;
        }

        Ok(deleted_data)
    }

    fn generate_row_values(
        statement: &Insert,
        table_ops: &TableOperations,
        values: &[String],
        timestamp: i64,
    ) -> Vec<String> {
        let table_columns: Vec<&str> = table_ops.columns.iter().map(|s| s.as_str()).collect();

        Self::generate_row_to_insert(
            values,
            &statement.get_columns_names(),
            &table_columns,
            timestamp,
        )
        .trim()
        .split(',')
        .map(|s| s.to_string())
        .collect()
    }

    fn order_and_save_rows(
        table_ops: &TableOperations,
        rows: &mut [Vec<String>],
        table: &Table,
    ) -> Result<()> {
        let order_by = Self::get_table_ordering(table);
        order_by.order(rows, &table_ops.columns);
        table_ops.write_rows(rows)
    }

    fn process_full_row_delete(
        statement: &Delete,
        rows: &[Vec<String>],
        table_ops: &TableOperations,
    ) -> Result<(Vec<Vec<String>>, Vec<String>)> {
        let mut modified_rows = Vec::new();
        let mut deleted_data = Vec::new();

        if statement.the_where.is_none() {
            if let IfCondition::Conditions(conditions) = &statement.if_condition {
                let all_conditions_met =
                    RowOperations::verify_row_conditions(rows, conditions, &table_ops.columns)?;
                if !all_conditions_met {
                    return Ok((rows.to_vec(), Vec::new()));
                }
                return Ok((Vec::new(), rows.iter().map(|row| row.join(",")).collect()));
            }
        }

        for row in rows {
            if RowOperations::should_process_row(
                row,
                &statement.if_condition,
                &table_ops.columns,
                statement.the_where.as_ref(),
            )? {
                deleted_data.push(row.join(","));
            } else {
                modified_rows.push(row.to_vec());
            }
        }

        Ok((modified_rows, deleted_data))
    }

    fn process_partial_row_delete(
        statement: &Delete,
        rows: &[Vec<String>],
        table_ops: &TableOperations,
    ) -> Result<(Vec<Vec<String>>, Vec<String>)> {
        let mut modified_rows = Vec::new();
        let mut deleted_data = Vec::new();

        let columns_to_modify: Vec<usize> = statement
            .cols
            .iter()
            .filter_map(|col_name| table_ops.columns.iter().position(|col| col == col_name))
            .collect();

        for row in rows {
            if RowOperations::should_process_row(
                row,
                &statement.if_condition,
                &table_ops.columns,
                statement.the_where.as_ref(),
            )? {
                let mut new_row = row.to_vec();
                let deleted_values: Vec<String> = columns_to_modify
                    .iter()
                    .filter_map(|&idx| row.get(idx))
                    .cloned()
                    .collect();

                deleted_data.push(deleted_values.join(","));

                for &col_idx in &columns_to_modify {
                    if col_idx < new_row.len() {
                        new_row[col_idx] = String::new();
                    }
                }
                modified_rows.push(new_row);
            } else {
                modified_rows.push(row.to_vec());
            }
        }

        Ok((modified_rows, deleted_data))
    }

    fn verify_row_conditions(
        row: &[String],
        conditions: &[Condition],
        columns: &[String],
    ) -> Result<bool> {
        for condition in conditions {
            let col_idx = columns
                .iter()
                .position(|col| col == condition.first_column.get_name())
                .ok_or_else(|| Error::ServerError("Columna no encontrada".to_string()))?;

            let row_value = row.get(col_idx).ok_or_else(|| {
                Error::ServerError("Índice de columna fuera de rango".to_string())
            })?;

            let condition_value = match &condition.second_column {
                Term::Constant(Constant::String(s)) => s.to_string(),
                Term::Constant(Constant::Integer(i)) => i.to_string(),
                Term::Constant(Constant::Double(f)) => f.to_string(),
                Term::Constant(Constant::Boolean(b)) => b.to_string(),
                Term::Constant(Constant::Uuid(u)) => u.to_string(),
                Term::Constant(Constant::Blob(b)) => b.to_string(),
                Term::Constant(Constant::NULL) => "NULL".to_string(),
            };

            let matches = match condition.operator {
                Operator::Equal => row_value == &condition_value,
                Operator::Distinct => row_value != &condition_value,
                Operator::Mayor => row_value > &condition_value,
                Operator::MayorEqual => row_value >= &condition_value,
                Operator::Minor => row_value < &condition_value,
                Operator::MinorEqual => row_value <= &condition_value,
                Operator::In => condition_value.split(',').any(|v| v == row_value),
                Operator::Contains => row_value.contains(&condition_value),
                Operator::ContainsKey => row_value.contains(&condition_value),
            };

            if !matches {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Verifica si se cumplen las condiciones de las filas de una tabla.
    pub fn verify_conditions(
        rows: &[Vec<String>],
        conditions: &[Condition],
        columns: &[String],
    ) -> Result<bool> {
        for row in rows {
            if Self::verify_row_conditions(row, conditions, columns)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn create_directory(path: &str) -> Result<()> {
        let path_folder = Path::new(path);
        if !path_folder.exists() && !path_folder.is_dir() {
            create_dir(path_folder).map_err(|e| {
                Error::ServerError(format!(
                    "No se pudo crear la carpeta de almacenamiento {}: {}",
                    path, e
                ))
            })?;
        }
        Ok(())
    }

    /// Obtiene la estrategia de replicación de un keyspace.
    pub fn get_keyspace_replication(options: &[Options]) -> Result<Option<ReplicationStrategy>> {
        let mut i = 0;
        while i < options.len() {
            match &options[i] {
                Options::MapLiteral(map_literal) => {
                    let values = map_literal.get_values().as_slice();
                    let (term1, term2) = &values[0];
                    if term1.get_value() == "class" && term2.get_value() == "SimpleStrategy" {
                        return Self::get_single_strategy_replication(values);
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

    fn get_single_strategy_replication(
        values: &[(Term, Term)],
    ) -> Result<Option<ReplicationStrategy>> {
        let (term3, term4) = &values[1];
        if term3.get_value() == "replication_factor" {
            let replicas = match term4.get_value().parse::<u32>() {
                Ok(replicas) => replicas,
                Err(_) => {
                    return Err(Error::Invalid(
                        "El valor de 'replication_factor' debe ser un número".to_string(),
                    ));
                }
            };
            Ok(Some(ReplicationStrategy::SimpleStrategy(replicas)))
        } else {
            Err(Error::Invalid(
                "Falto el campo replication_factor".to_string(),
            ))
        }
    }

    fn validate_and_get_keyspace_table_names(
        statement: &CreateTable,
        default_keyspace: &str,
        storage_addr: &str,
    ) -> Result<(String, String)> {
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

        Ok((keyspace_name, table_name))
    }

    fn create_table_csv_file(
        storage_addr: &str,
        keyspace_name: &str,
        table_name: &str,
        columns_names: &[String],
        node_number: Byte,
    ) -> Result<()> {
        let table_addr = format!(
            "{}/{}/{}_replica_node_{}.csv",
            storage_addr, keyspace_name, table_name, node_number
        );
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
            .write_all((",row_timestamp\n").as_bytes())
            .map_err(|e| Error::ServerError(e.to_string()))?;
        Ok(())
    }

    fn validate_and_get_primary_key(statement: &CreateTable) -> Result<PrimaryKey> {
        match &statement.primary_key {
            Some(primary_key) => Ok(primary_key.clone()),
            None => Err(Error::SyntaxError(
                "La clave primaria es obligatoria".to_string(),
            )),
        }
    }

    fn get_clustering_keys_and_order(
        statement: &CreateTable,
    ) -> Result<Option<Vec<(String, ProtocolOrdering)>>> {
        if statement
            .primary_key
            .as_ref()
            .map_or(true, |pk| pk.clustering_columns.is_empty())
        {
            return Ok(None);
        }

        let mut clustering_keys_and_order = statement
            .primary_key
            .as_ref()
            .map(|pk| {
                pk.clustering_columns
                    .iter()
                    .map(|key| (key.clone(), ProtocolOrdering::Asc))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        if let Some(clustering_order) = &statement.clustering_order {
            Self::update_clustering_order(&mut clustering_keys_and_order, clustering_order)?;
        }

        Ok(Some(clustering_keys_and_order))
    }

    fn update_clustering_order(
        clustering_keys_and_order: &mut [(String, ProtocolOrdering)],
        clustering_order: &[(String, String)],
    ) -> Result<()> {
        for (key, order) in clustering_order {
            if let Some(j) = clustering_keys_and_order.iter().position(|(k, _)| k == key) {
                let order = match ProtocolOrdering::from_str(order) {
                    Ok(order) => order,
                    Err(_) => {
                        return Err(Error::Invalid(format!(
                            "La dirección de ordenación {} no es válida",
                            order
                        )))
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
        Ok(())
    }

    fn get_table_ordering(table: &Table) -> OrderBy {
        let partition_key = table.get_partition_key();
        let mut order_criteria = vec![];

        for key in partition_key {
            order_criteria.push((key.to_string(), ProtocolOrdering::Asc));
        }

        if let Some(clustering_key_and_order) = &table.clustering_key_and_order {
            order_criteria.extend(clustering_key_and_order.clone());
        }

        OrderBy::new_from_vec(order_criteria)
    }

    fn generate_row_to_insert(
        values: &[String],
        query_cols: &[String],
        table_cols: &[&str],
        timestamp: i64,
    ) -> String {
        let mut values_to_insert: Vec<&str> = vec![""; table_cols.len() - 1]; // - 1 porque hay que ignorar la columna del timestamp, se agrega luego

        for i in 0..query_cols.len() {
            if let Some(j) = table_cols.iter().position(|c| *c == query_cols[i]) {
                values_to_insert[j] = values[i].as_str();
            }
        }
        let timestamp_reference = &timestamp.to_string();
        values_to_insert.push(timestamp_reference);

        values_to_insert.join(",") + "\n"
    }

    fn serialize_select_result(
        result: Vec<Vec<String>>,
        query_cols: &[String],
        table_cols: &[String],
        table: &Table,
    ) -> Vec<u8> {
        let mut res: Vec<u8> = vec![0x0, 0x0, 0x0, 0x2];
        let mut metadata: Vec<u8> = Vec::new();
        let flags: Int = 0;
        metadata.append(&mut flags.to_be_bytes().to_vec());

        let selected_cols = if query_cols[0] == "*" {
            table_cols
        } else {
            query_cols
        };

        metadata.append(&mut (selected_cols.len() as Int).to_be_bytes().to_vec());

        let cols_name_and_type = table.get_columns_name_and_data_type();
        for col_name in selected_cols {
            if let Some((_, data_type)) =
                cols_name_and_type.iter().find(|(name, _)| name == col_name)
            {
                let col_type = ColType::from(data_type.clone());
                metadata.append(&mut encode_string_to_bytes(col_name));
                metadata.append(&mut col_type.as_bytes());
            }
        }

        let rows_count = result.len() as Int;
        metadata.append(&mut rows_count.to_be_bytes().to_vec());

        let mut rows_content: Vec<Byte> = Vec::new();
        for row in result {
            for value in row {
                let value_length = value.len() as i32;
                rows_content.append(&mut value_length.to_be_bytes().to_vec());
                rows_content.append(&mut value.as_bytes().to_vec());
            }
        }

        res.append(&mut metadata);
        res.append(&mut rows_content);
        res
    }

    fn generate_row_to_select(
        table_row: &[String],
        table_cols: &[String],
        query_cols: &[String],
        without_timestamp: bool,
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
            if !without_timestamp {
                new_row.push(table_row.last().unwrap_or(&"0".to_string()).to_string())
            }
        }
        new_row
    }

    fn validate_update_columns(
        table_ops: &TableOperations,
        assignments: &[Assignment],
    ) -> Result<()> {
        for assignment in assignments {
            match assignment {
                Assignment::ColumnNameTerm(col, _) => {
                    table_ops.validate_columns(&[col.get_name().to_string()])?;
                }
                Assignment::ColumnNameColTerm(target_col, source_col, _) => {
                    table_ops.validate_columns(&[
                        target_col.get_name().to_string(),
                        source_col.get_name().to_string(),
                    ])?;
                }
                Assignment::ColumnNameListCol(target_col, _, source_col) => {
                    table_ops.validate_columns(&[
                        target_col.get_name().to_string(),
                        source_col.get_name().to_string(),
                    ])?;
                }
            }
        }
        Ok(())
    }

    fn update_row_value(
        row: &mut [String],
        assignment: &Assignment,
        columns: &[String],
    ) -> Result<()> {
        match assignment {
            Assignment::ColumnNameTerm(col, term) => {
                if let Some(col_index) = columns
                    .iter()
                    .position(|c| c == &col.get_name().to_string())
                {
                    row[col_index] = term.get_value().to_string();
                }
            }
            Assignment::ColumnNameColTerm(target_col, _, term) => {
                if let Some(col_index) = columns
                    .iter()
                    .position(|c| c == &target_col.get_name().to_string())
                {
                    row[col_index] = term.get_value().to_string();
                }
            }
            Assignment::ColumnNameListCol(target_col, _, source_col) => {
                if let Some(col_index) = columns
                    .iter()
                    .position(|c| c == &target_col.get_name().to_string())
                {
                    row[col_index] = source_col.get_name().to_string();
                }
            }
        }
        Ok(())
    }

    /// Borra todas las filas existentes, excepto la de los nombres de las columnas y inserta todas las filas pasadas por parametro
    pub fn actualize_all_rows(_rows: &str) {
        todo!() // ESTA SERIA LA FUNCION, HAY QUE VER LOS PARAMETROS QUE DEBERIA RECIBIR
    }
}
