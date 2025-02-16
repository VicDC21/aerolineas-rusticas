use crate::statements::dml_statement::{
    main_statements::select::{
        group_by::GroupBy, limit::Limit, order_by::OrderBy, per_partition_limit::PerPartitionLimit,
    },
    r#where::where_parser::Where,
};

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
