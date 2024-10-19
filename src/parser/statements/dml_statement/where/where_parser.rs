use super::expression::Expression;

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

    // pub fn add_condition(&mut self, relation: Relation) {
    //     self.relations.push(relation);
    // }
}
