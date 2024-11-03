use crate::parser::data_types::keyspace_name::KeyspaceName;

use super::option::Options;

/// alter_keyspace_statement::= ALTER KEYSPACE [ IF EXISTS ] keyspace_name
/// WITH options

#[derive(Debug, Clone)]
pub struct AlterKeyspace {
    /// Indica si se debe verificar la existencia del keyspace.
    pub if_exists: bool,
    /// Nombre del keyspace a alterar.
    pub name: KeyspaceName,
    /// Opciones del keyspace.
    pub options: Vec<Options>,
}

impl AlterKeyspace {
    /// Crea una nueva instancia de `AlterKeyspace`.
    pub fn new(if_exists: bool, name: KeyspaceName, options: Vec<Options>) -> Self {
        AlterKeyspace {
            if_exists,
            name,
            options,
        }
    }

    /// Devuelve las opciones del keyspace.
    pub fn get_options(&self) -> &Vec<Options> {
        &self.options
    }
}
