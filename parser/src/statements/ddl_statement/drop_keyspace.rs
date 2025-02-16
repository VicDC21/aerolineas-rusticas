use crate::data_types::keyspace_name::KeyspaceName;

/// Representa una sentencia CQL DROP KEYSPACE.
#[derive(Debug)]
pub struct DropKeyspace {
    /// Indica si se debe verificar la existencia de la tabla.
    pub if_exists: bool,
    /// Nombre del keyspace a eliminar.
    pub name: KeyspaceName,
}

impl DropKeyspace {
    /// Crea una nueva instancia de `DropKeyspace`.
    pub fn new(if_exists: bool, name: KeyspaceName) -> Self {
        DropKeyspace { if_exists, name }
    }
}
