use std::cmp::Ordering;

use crate::parser::data_types::identifier::identifier::Identifier;

/// ordering_clause::= column_name [ ASC | DESC ] ( ',' column_name [ ASC | DESC ] )*
#[derive(Debug)]
pub struct OrderBy {
    /// Lista de columnas y dirección de ordenación.
    pub order_columns: Vec<(Identifier, Option<ProtocolOrdering>)>,
}

impl OrderBy {
    /// Crea una nueva cláusula ORDER BY.
    pub fn new(order_columns: Vec<(Identifier, Option<ProtocolOrdering>)>) -> Self {
        OrderBy { order_columns }
    }

    /// Ordena las filas de acuerdo a las columnas y direcciones de ordenación.
    pub fn order(&self, rows: &mut [Vec<String>], general_columns: &[String]) {
        rows.sort_by(|row_a, row_b| {
            for (column, order) in &self.order_columns {
                let mut result = Ordering::Equal;
                if let Some(index) = general_columns.iter().position(|x| x == column.get_name()) {
                    result = Self::cmp_values_with_parse(&row_a[index], &row_b[index]);
                }

                if result == Ordering::Equal {
                    continue;
                }

                match order {
                    Some(ProtocolOrdering::Asc) | None => {
                        return result;
                    }
                    Some(ProtocolOrdering::Desc) => {
                        return result.reverse();
                    }
                }
            }
            Ordering::Equal
        });
    }

    fn cmp_values_with_parse(value_a: &str, value_b: &str) -> Ordering {
        match (value_a.parse::<f64>(), value_b.parse::<f64>()) {
            (Ok(a), Ok(b)) => a.partial_cmp(&b).unwrap_or(Ordering::Equal),
            _ => value_a.cmp(value_b),
        }
    }
}
/// Representa la dirección de ordenación en una cláusula ORDER BY.
#[derive(Debug, PartialEq)]
pub enum ProtocolOrdering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
}
