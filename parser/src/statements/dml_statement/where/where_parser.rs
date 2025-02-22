use crate::statements::dml_statement::r#where::expression::Expression;
use protocol::aliases::results::Result;

/// Representa una cláusula WHERE en una declaración CQL.
/// La cláusula WHERE se utiliza para filtrar filas de una tabla.
#[derive(Debug)]
pub struct Where {
    /// Expresión que se evaluará para cada fila de la tabla.
    pub expression: Option<Box<Expression>>,
}

impl Where {
    /// Constructor de la cláusula WHERE.
    pub fn new(expression: Option<Box<Expression>>) -> Self {
        Where { expression }
    }

    /// Evalúa la expresión de la cláusula WHERE.
    pub fn filter(&self, line_to_review: &[String], general_columns: &[String]) -> Result<bool> {
        match &self.expression {
            Some(value) => value.evaluate(line_to_review, general_columns),
            None => Ok(true),
        }
    }
}
