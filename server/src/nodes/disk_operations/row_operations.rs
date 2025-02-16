//! Módulo que detalla las operaciones de filas

use {
    crate::nodes::disk_operations::disk_handler::DiskHandler,
    parser::statements::dml_statement::{
        if_condition::{Condition, IfCondition},
        r#where::where_parser::Where,
    },
    protocol::aliases::results::Result,
};

/// Estructura para manejar operaciones comunes sobre filas
pub struct RowOperations;

impl RowOperations {
    /// Verifica si una fila cumple con las condiciones dadas.
    pub fn verify_row_conditions(
        rows: &[Vec<String>],
        conditions: &[Condition],
        columns: &[String],
    ) -> Result<bool> {
        DiskHandler::verify_conditions(rows, conditions, columns)
    }

    /// Verifica si una fila deberia ser procesada, en base a si cumple con las condiciones dadas.
    pub fn should_process_row(
        row: &[String],
        if_condition: &IfCondition,
        columns: &[String],
        where_clause: Option<&Where>,
    ) -> Result<bool> {
        let passes_where = match where_clause {
            Some(the_where) => the_where.filter(row, columns)?,
            None => true,
        };

        if !passes_where {
            return Ok(false);
        }

        let passes_conditions = match if_condition {
            IfCondition::Conditions(conditions) => {
                Self::verify_row_conditions(&[row.to_vec()], conditions, columns)?
            }
            IfCondition::Exists => true,
            _ => true,
        };

        Ok(passes_conditions)
    }
}
