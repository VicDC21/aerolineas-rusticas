//! Módulo que detalla una tabla.

use {
    crate::nodes::table_metadata::{
        column_config::ColumnConfig,
        column_data_type::ColumnDataType,
    },
    parser::statements::dml_statement::main_statements::select::ordering::ProtocolOrdering,
    protocol::{aliases::results::Result, errors::error::Error},
    serde::{Deserialize, Serialize},
};

/// Representa una tabla en CQL.
#[derive(Serialize, Deserialize)]
pub struct Table {
    /// Nombre de la tabla.
    pub name: String,
    /// Nombre del keyspace al que pertenece la tabla.
    pub keyspace: String,
    /// Columnas de la tabla.
    pub columns: Vec<ColumnConfig>,
    /// Clave primaria de la tabla.
    pub partition_key: Vec<String>,
    /// Clave de clustering de la tabla y orden de agrupamiento de las columnas.
    pub clustering_key_and_order: Option<Vec<(String, ProtocolOrdering)>>,
}

impl Table {
    /// Crea una nueva tabla.
    pub fn new(
        name: String,
        keyspace: String,
        columns: Vec<ColumnConfig>,
        partition_key: Vec<String>,
        clustering_key_and_order: Option<Vec<(String, ProtocolOrdering)>>,
    ) -> Self {
        Table {
            name,
            keyspace,
            columns,
            partition_key,
            clustering_key_and_order,
        }
    }

    /// Obtiene el nombre de la tabla.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Obtiene el nombre del keyspace al que pertenece la tabla.
    pub fn get_keyspace(&self) -> &str {
        &self.keyspace
    }

    /// Obtiene la partition key de la tabla.
    pub fn get_partition_key(&self) -> Vec<String> {
        self.partition_key.clone()
    }

    /// Obtiene los nombres de las columnas de la tabla.
    pub fn get_columns_names(&self) -> Vec<String> {
        self.columns
            .iter()
            .map(|column| column.get_name())
            .collect()
    }

    /// Obtiene la posicion de la partition key en relacion a las demas columnas. Aclaracion, asumo que solo hay un partition key, no un vector.
    pub fn get_position_of_partition_key(&self) -> Result<usize> {
        for (index, column) in self.columns.iter().enumerate() {
            if column.get_name() == self.partition_key[0] {
                return Ok(index);
            }
        }
        Err(Error::ServerError(
            "Las columnas no tienen el partition key".to_string(),
        ))
    }

    /// Obtiene la posicion de todos los valores de la primary key en relacion a las demas columnas.
    /// Empieza por el partition key y sigue segun como se haya declarado la tabla
    pub fn get_position_of_primary_key(&self) -> Result<Vec<usize>> {
        let mut primary_key_values: Vec<String> = Vec::new();
        // let mut positions: Vec<usize> = Vec::new();
        let mut columns: Vec<String> = Vec::new();
        for column_config in &self.columns {
            columns.push(column_config.get_name());
        }
        primary_key_values.push(self.partition_key[0].clone());
        if let Some(clustering_columns) = &self.clustering_key_and_order {
            for clus_column in clustering_columns {
                primary_key_values.push(clus_column.0.clone());
            }
        }
        let res: Vec<usize> = primary_key_values
            .iter()
            .filter_map(|primary_key_value| columns.iter().position(|s| s == primary_key_value))
            .collect();
        Ok(res)
    }

    /// Obtiene los tipos de datos de las columnas de la tabla.
    pub fn get_columns_data_type(&self) -> Vec<ColumnDataType> {
        self.columns
            .iter()
            .map(|column| column.get_data_type())
            .collect()
    }

    /// Obtiene los nombres y tipos de datos de las columnas de la tabla.
    pub fn get_columns_name_and_data_type(&self) -> Vec<(String, ColumnDataType)> {
        self.columns
            .iter()
            .map(|column| (column.get_name(), column.get_data_type()))
            .collect()
    }
}
