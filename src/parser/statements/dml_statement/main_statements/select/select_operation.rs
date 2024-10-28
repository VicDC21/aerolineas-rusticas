use super::{kind_of_columns::KindOfColumns, options::SelectOptions};
use crate::parser::table_name::TableName;

/// Representa una declaración SELECT en el lenguaje de consulta.
#[derive(Debug)]
pub struct Select {
    /// Columnas a seleccionar.
    pub columns: KindOfColumns,
    /// Nombre de la tabla de la cual se seleccionarán los datos.
    pub from: TableName,
    /// Opciones de la declaración SELECT.
    pub options: SelectOptions,
}

impl Select {
    /// Crea una nueva sentencia SELECT.
    pub fn new(columns: KindOfColumns, from: TableName, options: SelectOptions) -> Select {
        Select {
            columns,
            from,
            options,
        }
    }

    /// Obtiene el nombre de la tabla de la cual se seleccionarán los datos.
    pub fn get_columns_names(&self) -> Vec<String> {
        self.columns.get_columns()
    }
}
