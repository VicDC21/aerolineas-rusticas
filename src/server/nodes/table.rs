//! Módulo que detalla una tabla.

use std::{fmt, str::FromStr};

use crate::parser::statements::dml_statement::main_statements::select::ordering::ProtocolOrdering;
use crate::protocol::{aliases::results::Result, errors::error::Error};

use super::{column_config::ColumnConfig, column_data_type::ColumnDataType};
/// Representa una tabla en CQL.
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

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = format!("{}-{}-", self.name, self.keyspace);

        for column in &self.columns {
            res.push_str(&column.to_string());
            res.push('_');
        }

        res.pop();
        res.push('-');

        for key in &self.partition_key {
            res.push_str(key);
            res.push('_');
        }

        res.pop();
        res.push('-');

        if let Some(clustering_key_and_order) = &self.clustering_key_and_order {
            for (key, order) in clustering_key_and_order {
                res.push_str(key);
                res.push_str(&order.to_string());
                res.push('_');
            }
            res.pop();
        }

        write!(f, "{}", res)
    }
}

impl FromStr for Table {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 4 {
            return Err(Error::ServerError(
                "No se pudo parsear la tabla".to_string(),
            ));
        }

        let name: String = parts[0].to_string();
        let keyspace: String = parts[1].to_string();

        let columns: Vec<ColumnConfig> = parts[2]
            .split('_')
            .map(|column| column.parse())
            .collect::<Result<Vec<ColumnConfig>>>()?;

        let partition_key: Vec<String> = parts[3].split('_').map(|key| key.to_string()).collect();

        let clustering_key_and_order: Option<Vec<(String, ProtocolOrdering)>> = if parts.len() == 5
        {
            let clustering_key_and_order = parts[4]
                .split('_')
                .map(|key_and_order| {
                    let (key, order_str) = key_and_order.split_at(key_and_order.len() - 1);
                    let key = key.to_string();
                    let order = order_str.parse()?;
                    Ok((key, order))
                })
                .collect::<Result<Vec<(String, ProtocolOrdering)>>>()?;

            Some(clustering_key_and_order)
        } else {
            None
        };

        Ok(Table::new(
            name,
            keyspace,
            columns,
            partition_key,
            clustering_key_and_order,
        ))
    }
}
