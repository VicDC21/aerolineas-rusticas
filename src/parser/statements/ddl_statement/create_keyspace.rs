use crate::parser::{
    data_types::keyspace_name::KeyspaceName, statements::ddl_statement::option::Options,
};

/// Representa una declaración CQL `CREATE KEYSPACE`.
#[derive(Debug)]
pub struct CreateKeyspace {
    /// Indica si la declaración contiene la cláusula `IF NOT EXISTS`.
    pub if_not_exist: bool,
    /// Nombre del keyspace a crear.
    pub keyspace_name: KeyspaceName,
    /// Opciones del keyspace.
    pub options: Vec<Options>,
}

impl CreateKeyspace {
    /// Crea una nueva instancia de `CreateKeyspace`.
    pub fn new(if_not_exist: bool, keyspace_name: KeyspaceName, options: Vec<Options>) -> Self {
        CreateKeyspace {
            if_not_exist,
            keyspace_name,
            options,
        }
    }

    /// Devuelve las opciones del keyspace.
    pub fn get_options(&self) -> &Vec<Options> {
        &self.options
    }
}
