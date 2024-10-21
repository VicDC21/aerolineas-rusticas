use crate::parser::{
    data_types::identifier::identifier::Identifier, data_types::term::Term,
    statements::dml_statement::r#where::operator::Operator,
};
use crate::protocol::{aliases::results::Result, errors::error::Error};
use crate::tokenizer::tokenizer::tokenize_query;

/// Representa una relación en una cláusula WHERE con dos columnas y un operador.
pub struct Relation {
    /// Identificador de la primera columna.
    /// La primera columna es la columna de la izquierda en la relación.
    column: Identifier,
    /// Operador de la relación.
    /// El operador se utiliza para comparar las dos columnas.
    operator: Operator,
    /// Término con el que se comparará la primera columna.
    term_to_compare: Term,
}

impl Relation {
    /// Constructor de la relación.
    pub fn new(column: Identifier, operator: Operator, term_to_compare: Term) -> Self {
        Relation {
            column,
            operator,
            term_to_compare,
        }
    }

    /// Evalúa la relación entre la columna de la tabla y el término dados.
    pub fn evaluate(&self, line_to_review: &[String], general_columns: &[String]) -> Result<bool> {
        match self.operator {
            Operator::In => todo!(),
            Operator::Contains => todo!(),
            Operator::ContainsKey => todo!(),
            _ => self.make_comparison(line_to_review, general_columns),
        }
    }

    fn make_comparison(
        &self,
        line_to_review: &[String],
        general_columns: &[String],
    ) -> Result<bool> {
        let column = self.column.get_name();
        let index = match general_columns.iter().position(|word| word == column) {
            Some(position) => position,
            None => {
                return Err(Error::Invalid(
                    "La columna solicitada no existe".to_string(),
                ))
            }
        };

        let column_term = match Term::is_term(&mut tokenize_query(&line_to_review[index]))? {
            Some(value) => value,
            None => {
                return Err(Error::Invalid(
                    "La columna es un tipo de dato no valido".to_string(),
                ))
            }
        };

        // if self.first_column.is_a_string() || self.second_column.is_a_string() {
        //     return Err(Error::SyntaxError(
        //         "For comparisons whose operator is not '=' the parameters must be numbers"
        //             .to_string(),
        //     ));
        // }
        // let num1_comparator: i32 =
        //     self.parse_to_a_number(line_to_review, general_columns, &self.first_column)?;
        // let num2_comparator: i32 =
        //     self.parse_to_a_number(line_to_review, general_columns, &self.second_column)?;

        match self.operator {
            Operator::Minor => Ok(column_term < self.term_to_compare),
            Operator::Mayor => Ok(column_term > self.term_to_compare),
            Operator::MayorEqual => Ok(column_term >= self.term_to_compare),
            Operator::MinorEqual => Ok(column_term <= self.term_to_compare),
            Operator::Equal => Ok(column_term == self.term_to_compare),
            Operator::Distinct => Ok(column_term != self.term_to_compare),
            Operator::In => todo!(),
            Operator::Contains => todo!(),
            Operator::ContainsKey => todo!(),
        }
    }
}
