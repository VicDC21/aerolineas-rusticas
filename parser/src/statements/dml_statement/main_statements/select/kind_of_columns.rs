use crate::statements::dml_statement::main_statements::select::selector::Selector;

#[derive(Default, Debug, PartialEq)]

/// Representa el tipo de columnas a seleccionar.
pub enum KindOfColumns {
    /// Columnas específicas.
    SelectClause(Vec<Selector>),
    #[default]
    /// Todas las columnas.
    All,
}

impl KindOfColumns {
    /// Obtiene las columnas a seleccionar.
    pub fn get_columns(&self) -> Vec<String> {
        match self {
            KindOfColumns::SelectClause(columns) => {
                columns.iter().map(|column| column.get_name()).collect()
            }
            KindOfColumns::All => vec!["*".to_string()],
        }
    }
}
