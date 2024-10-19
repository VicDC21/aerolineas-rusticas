use crate::parser::{
    assignment::Assignment,
    statements::dml_statement::{if_condition::IfCondition, r#where::r#where_parser::Where},
    table_name::TableName,
};
/// Representa una sentencia CQL UPDATE.
#[derive(Debug)]
pub struct Update {
    /// Nombre de la tabla a actualizar.
    pub table_name: TableName,
    /// Lista de asignaciones de valores a actualizar.
    pub set_parameter: Vec<Assignment>,
    /// Condición de actualización.
    pub the_where: Option<Where>,
    /// Condición de existencia.
    pub if_exists: IfCondition,
}

impl Update {
    /// Crea una nueva sentencia UPDATE.
    pub fn new(
        table_name: TableName,
        set_parameter: Vec<Assignment>,
        the_where: Option<Where>,
        if_exists: IfCondition,
    ) -> Update {
        Update {
            table_name,
            set_parameter,
            the_where,
            if_exists,
        }
    }
}
