//! Módulo para manejo del almacenamiento en disco.

use std::{
    fs::{create_dir, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

use super::{keyspace::Keyspace, replication_strategy::ReplicationStrategy, table::Table};
use crate::{
    parser::{
        assignment::Assignment,
        data_types::term::Term,
        primary_key::PrimaryKey,
        statements::{
            ddl_statement::{
                create_keyspace::CreateKeyspace, create_table::CreateTable, option::Options,
            },
            dml_statement::main_statements::{
                insert::Insert,
                select::{order_by::OrderBy, ordering::ProtocolOrdering, select_operation::Select},
                update::Update,
            },
        },
    },
    protocol::{
        aliases::{results::Result, types::Byte},
        errors::error::Error,
        messages::responses::result::col_type::ColType,
        traits::Byteable,
        utils::encode_string_to_bytes,
    },
    server::nodes::node::NodeId,
};

// Estructura común para manejar paths
struct TablePath {
    storage_addr: String,
    keyspace: String,
    table_name: String,
}

impl TablePath {
    fn new(
        storage_addr: &str,
        keyspace: Option<String>,
        table_name: &str,
        default_keyspace: &str,
    ) -> Self {
        let keyspace = keyspace.unwrap_or_else(|| default_keyspace.to_string());
        Self {
            storage_addr: storage_addr.to_string(),
            keyspace,
            table_name: table_name.to_string(),
        }
    }

    fn full_path(&self) -> String {
        format!(
            "{}/{}/{}.csv",
            self.storage_addr, self.keyspace, self.table_name
        )
    }
}

// Estructura para manejar operaciones comunes sobre tablas
struct TableOperations {
    path: TablePath,
    columns: Vec<String>,
}

impl TableOperations {
    fn new(path: TablePath) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .open(path.full_path())
            .map_err(|e| Error::ServerError(e.to_string()))?;

        let mut reader = BufReader::new(&file);
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|e| Error::ServerError(e.to_string()))?;

        if header.trim().is_empty() {
            return Err(Error::ServerError(format!(
                "No se pudo leer la tabla con ruta {}",
                path.full_path()
            )));
        }

        let columns = header.trim().split(',').map(|s| s.to_string()).collect();

        Ok(Self { path, columns })
    }

    fn validate_columns(&self, columns: &[String]) -> Result<()> {
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

    fn read_rows(&self) -> Result<Vec<Vec<String>>> {
        let file = OpenOptions::new()
            .read(true)
            .open(self.path.full_path())
            .map_err(|e| Error::ServerError(e.to_string()))?;

        let reader = BufReader::new(file);
        let mut rows = Vec::new();

        for line in reader.lines().skip(1) {
            let line = line.map_err(|e| Error::ServerError(e.to_string()))?;
            if !line.trim().is_empty() {
                rows.push(line.trim().split(',').map(|s| s.to_string()).collect());
            }
        }

        Ok(rows)
    }

    fn write_rows(&self, rows: &[Vec<String>]) -> Result<()> {
        let mut content = self.columns.join(",");
        content.push('\n');

        for row in rows {
            content.push_str(&row.join(","));
            content.push('\n');
        }

        std::fs::write(self.path.full_path(), content)
            .map_err(|e| Error::ServerError(e.to_string()))
    }
}

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
                    let values = map_literal.get_values().as_slice();
                    let (term1, term2) = &values[0];
                    if term1.get_value() == "class" && term2.get_value() == "SimpleStrategy" {
                        return DiskHandler::get_single_strategy_replication(values);
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

    /// Crea una nueva tabla en el caso que corresponda.
    pub fn create_table(
        statement: CreateTable,
        storage_addr: &str,
        default_keyspace: &str,
    ) -> Result<Option<Table>> {
        let (keyspace_name, table_name) = Self::validate_and_get_keyspace_table_names(
            &statement,
            default_keyspace,
            storage_addr,
        )?;
        let columns = statement.get_columns()?;
        let columns_names = columns
            .iter()
            .map(|c| c.get_name())
            .collect::<Vec<String>>();

        Self::create_table_csv_file(storage_addr, &keyspace_name, &table_name, &columns_names)?;

        let primary_key = Self::validate_and_get_primary_key(&statement)?;
        let clustering_keys_and_order = Self::get_clustering_keys_and_order(&statement)?;

        Ok(Some(Table::new(
            table_name,
            keyspace_name,
            columns,
            primary_key.partition_key,
            clustering_keys_and_order,
        )))
    }

    /// Valida y obtiene los nombres de keyspace y tabla
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

    /// Crea el archivo CSV para la tabla
    fn create_table_csv_file(
        storage_addr: &str,
        keyspace_name: &str,
        table_name: &str,
        columns_names: &[String],
    ) -> Result<()> {
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

        Ok(())
    }

    /// Valida y obtiene la clave primaria
    fn validate_and_get_primary_key(statement: &CreateTable) -> Result<PrimaryKey> {
        match &statement.primary_key {
            Some(primary_key) => Ok(primary_key.clone()),
            None => Err(Error::SyntaxError(
                "La clave primaria es obligatoria".to_string(),
            )),
        }
    }

    /// Obtiene las columnas de clustering y su orden
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

    /// Actualiza el orden de las columnas de clustering
    fn update_clustering_order(
        clustering_keys_and_order: &mut [(String, ProtocolOrdering)],
        clustering_order: &[(String, String)],
    ) -> Result<()> {
        for (key, order) in clustering_order {
            if let Some(j) = clustering_keys_and_order.iter().position(|(k, _)| k == key) {
                let order = match ProtocolOrdering::ordering_from_str(order) {
                    Some(order) => order,
                    None => {
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

    /// Inserta una nueva fila en una tabla en el caso que corresponda.
    pub fn do_insert(
        statement: &Insert,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
    ) -> Result<Vec<String>> {
        let path = TablePath::new(
            storage_addr,
            statement.table.get_keyspace(),
            &statement.table.get_name(),
            default_keyspace,
        );

        let table_ops = TableOperations::new(path)?;
        table_ops.validate_columns(&statement.get_columns_names())?;
        let mut rows = table_ops.read_rows()?;
        let values = statement.get_values();
        let mut id_exists = false;
        let mut id_position = None;

        for (i, row) in rows.iter().enumerate() {
            if row[0] == values[0] {
                id_exists = true;
                id_position = Some(i);
                break;
            }
        }

        if id_exists && statement.if_not_exists {
            return Ok(Vec::new());
        }

        let new_row = Self::generate_row_to_insert(
            &values,
            &statement.get_columns_names(),
            &table_ops
                .columns
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>(),
        );

        let new_row_vec: Vec<String> = new_row.trim().split(',').map(|s| s.to_string()).collect();

        if let Some(pos) = id_position {
            rows[pos] = new_row_vec.clone();
        } else {
            rows.push(new_row_vec.clone());
        }

        let order_by = Self::get_table_ordering(table);
        order_by.order(&mut rows, &table_ops.columns);
        table_ops.write_rows(&rows)?;
        Ok(new_row_vec)
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

    /// Selecciona filas en una tabla en el caso que corresponda.
    pub fn do_select(
        statement: &Select,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
    ) -> Result<Vec<Byte>> {
        let path = TablePath::new(
            storage_addr,
            statement.from.get_keyspace(),
            &statement.from.get_name(),
            default_keyspace,
        );

        let table_ops = TableOperations::new(path)?;
        let query_cols = statement.columns.get_columns();

        if query_cols.len() != 1 && query_cols[0] != "*" {
            table_ops.validate_columns(&query_cols)?;
        }

        let mut result = Vec::new();
        if query_cols.len() == 1 && query_cols[0] == "*" {
            result.push(table_ops.columns.clone());
        } else {
            result.push(query_cols.clone());
        }

        let mut rows = table_ops.read_rows()?;

        if let Some(the_where) = &statement.options.the_where {
            rows.retain(|row| the_where.filter(row, &table_ops.columns).unwrap_or(false));
        }

        if let Some(order) = &statement.options.order_by {
            order.order(&mut rows, &table_ops.columns);
        }

        let result_rows: Vec<Vec<String>> = rows
            .into_iter()
            .map(|row| Self::generate_row_to_select(&row, &table_ops.columns, &query_cols))
            .collect();

        result.extend(result_rows);

        Ok(Self::serialize_select_result(
            result,
            &query_cols,
            &table_ops.columns,
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
        let mut res: Vec<u8> = vec![0x0, 0x0, 0x0, 0x2];
        let mut metadata: Vec<u8> = Vec::new();
        let flags: i32 = 0;
        metadata.append(&mut flags.to_be_bytes().to_vec());

        if query_cols[0] == "*" {
            metadata.append(&mut table_cols.len().to_be_bytes().to_vec())
        } else {
            metadata.append(&mut query_cols.len().to_be_bytes().to_vec())
        }

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
        res.append(&mut metadata);
        res.append(&mut rows_count.to_be_bytes().to_vec());
        res.append(&mut rows_content);

        res
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

    /// Valida que las columnas de una asignación existan en la tabla
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

    /// Actualiza un valor de fila según el tipo de asignación
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

    /// Actualiza filas en una tabla en el caso que corresponda.
    pub fn do_update(
        statement: &Update,
        storage_addr: &str,
        table: &Table,
        default_keyspace: &str,
    ) -> Result<Vec<String>> {
        let path = TablePath::new(
            storage_addr,
            statement.table_name.get_keyspace(),
            &statement.table_name.get_name(),
            default_keyspace,
        );

        let table_ops = TableOperations::new(path)?;
        Self::validate_update_columns(&table_ops, &statement.set_parameter)?;
        let mut rows = table_ops.read_rows()?;
        let mut updated_rows = Vec::new();

        for row in rows.iter_mut() {
            let should_update = match &statement.the_where {
                Some(the_where) => the_where.filter(row, &table_ops.columns)?,
                None => true,
            };
            if should_update {
                for assignment in &statement.set_parameter {
                    Self::update_row_value(row, assignment, &table_ops.columns)?;
                }
                updated_rows.push(row.clone());
            }
        }

        if !updated_rows.is_empty() {
            let order_by = Self::get_table_ordering(table);
            order_by.order(&mut rows, &table_ops.columns);
            table_ops.write_rows(&rows)?;
        }

        Ok(updated_rows.iter().map(|row| row.join(",")).collect())
    }
}
