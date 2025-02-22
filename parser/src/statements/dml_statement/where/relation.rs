use crate::{
    data_types::constant::Constant, data_types::identifier::identifier_mod::Identifier,
    data_types::term::Term, statements::dml_statement::r#where::operator::Operator,
};
use protocol::{
    aliases::{
        results::Result,
        types::{Double, Int},
    },
    errors::error::Error,
};

/// Representa una relación en una cláusula WHERE con dos columnas y un operador.
#[derive(Debug)]
pub struct Relation {
    /// Identificador de la primera columna.
    /// La primera columna es la columna de la izquierda en la relación.
    pub column: Identifier,
    /// Operador de la relación.
    /// El operador se utiliza para comparar las dos columnas.
    pub operator: Operator,
    /// Término con el que se comparará la primera columna.
    pub term_to_compare: Term,
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
        let column = self.column.get_name();
        let index = match general_columns.iter().position(|word| word == column) {
            Some(position) => position,
            None => {
                return Err(Error::Invalid(
                    "La columna solicitada no existe".to_string(),
                ))
            }
        };

        if index >= line_to_review.len() {
            return Err(Error::Invalid(
                "Índice de columna fuera de rango".to_string(),
            ));
        }

        let column_value = &line_to_review[index];
        let column_term = self.parse_csv_value_to_term(column_value)?;

        self.compare_terms(&column_term, &self.term_to_compare)
    }

    fn parse_csv_value_to_term(&self, value: &str) -> Result<Term> {
        if let Ok(int_val) = value.parse::<Int>() {
            return Ok(Term::Constant(Constant::Integer(int_val)));
        }

        if let Ok(double_val) = value.parse::<Double>() {
            return Ok(Term::Constant(Constant::Double(double_val)));
        }

        match value.to_uppercase().as_str() {
            "TRUE" => return Ok(Term::Constant(Constant::Boolean(true))),
            "FALSE" => return Ok(Term::Constant(Constant::Boolean(false))),
            _ => {}
        }

        if value.to_uppercase() == "NULL" {
            return Ok(Term::Constant(Constant::NULL));
        }

        let cleaned_value = value.trim_matches('"').trim_matches('\'');
        Ok(Term::Constant(Constant::String(cleaned_value.to_string())))
    }

    fn compare_terms(&self, column_term: &Term, compare_term: &Term) -> Result<bool> {
        match &self.operator {
            Operator::Equal => Ok(column_term == compare_term),
            Operator::Distinct => Ok(column_term != compare_term),
            Operator::Minor => match column_term.partial_cmp(compare_term) {
                Some(ordering) => Ok(ordering == std::cmp::Ordering::Less),
                None => Err(Error::Invalid(
                    "Tipos incompatibles para comparación".to_string(),
                )),
            },
            Operator::Mayor => match column_term.partial_cmp(compare_term) {
                Some(ordering) => Ok(ordering == std::cmp::Ordering::Greater),
                None => Err(Error::Invalid(
                    "Tipos incompatibles para comparación".to_string(),
                )),
            },
            Operator::MinorEqual => match column_term.partial_cmp(compare_term) {
                Some(ordering) => Ok(ordering != std::cmp::Ordering::Greater),
                None => Err(Error::Invalid(
                    "Tipos incompatibles para comparación".to_string(),
                )),
            },
            Operator::MayorEqual => match column_term.partial_cmp(compare_term) {
                Some(ordering) => Ok(ordering != std::cmp::Ordering::Less),
                None => Err(Error::Invalid(
                    "Tipos incompatibles para comparación".to_string(),
                )),
            },
            Operator::In | Operator::Contains | Operator::ContainsKey => {
                Err(Error::Invalid("Operador no implementado".to_string()))
            }
        }
    }
}
