use crate::parser::{
    statements::dml_statement::{if_condition::IfCondition, r#where::r#where_parser::Where},
    table_name::TableName,
};

/// Representa una declaración DELETE en el analizador.
#[derive(Debug)]
pub struct Delete {
    /// Columnas a eliminar.
    pub cols: Vec<String>,
    /// Nombre de la tabla de la cual se eliminarán los datos.
    pub from: TableName,
    /// Condición de eliminación.
    pub the_where: Option<Where>,
    /// Condición de eliminación.
    pub if_condition: IfCondition,
}

impl Delete {
    /// Crea una nueva sentencia DELETE.
    pub fn new(
        cols: Vec<String>,
        from: TableName,
        the_where: Option<Where>,
        if_condition: IfCondition,
    ) -> Delete {
        Delete {
            cols,
            from,
            the_where,
            if_condition,
        }
    }
}
