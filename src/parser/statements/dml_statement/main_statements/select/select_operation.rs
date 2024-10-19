use super::{order_by::OrderBy, selector::Selector};
use crate::parser::{
    data_types::keyspace_name::KeyspaceName,
    statements::dml_statement::{
        main_statements::select::{
            group_by::GroupBy, limit::Limit, per_partition_limit::PerPartitionLimit,
        },
        r#where::r#where_parser::Where,
    },
};

/// Representa una declaración SELECT en el lenguaje de consulta.
#[derive(Debug)]
pub struct Select {
    /// Columnas a seleccionar.
    pub columns: KindOfColumns,
    /// Nombre de la tabla de la cual se seleccionarán los datos.
    pub from: KeyspaceName,
    /// Opciones de la declaración SELECT.
    pub options: SelectOptions,
}

/// Opciones para la declaración SELECT.
#[derive(Debug)]
pub struct SelectOptions {
    /// Condición de selección.
    pub the_where: Option<Where>,
    /// Agrupación de datos.
    pub group_by: Option<GroupBy>,
    /// Ordenamiento de datos.
    pub order_by: Option<OrderBy>,
    /// Límite de datos por partición.
    pub per_partition_limit: Option<PerPartitionLimit>,
    /// Límite de datos.
    pub limit: Option<Limit>,
    /// Indica si se permite el filtrado de datos.
    pub allow_filtering: Option<bool>,
}

impl Select {
    /// Crea una nueva sentencia SELECT.
    pub fn new(columns: KindOfColumns, from: KeyspaceName, options: SelectOptions) -> Select {
        Select {
            columns,
            from,
            options,
        }
    }
}

#[derive(Default)]
/// Representa el tipo de columnas a seleccionar.
#[derive(Debug, PartialEq)]
pub enum KindOfColumns {
    /// Columnas específicas.
    SelectClause(Vec<Selector>),
    #[default]
    /// Todas las columnas.
    All,
}
