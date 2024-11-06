//! Módulo que detallas la ruta de una tabla

/// Estructura común para manejar paths
pub struct TablePath {
    /// Dirección del storage
    pub storage_addr: String,
    /// Keyspace de la tabla
    pub keyspace: String,
    /// Nombre de la tabla
    pub table_name: String,
}

impl TablePath {
    /// Crea una nueva instancia de `TablePath`.
    pub fn new(
        storage_addr: &str,
        keyspace: Option<String>,
        table_name: &str,
        default_keyspace: &str,
    ) -> Self {
        let keyspace = keyspace.unwrap_or_else(|| default_keyspace.to_string());
        Self {
            storage_addr: storage_addr.to_string(),
            keyspace,
            table_name: table_name.to_string(),
        }
    }

    /// Devuelve el path completo de la tabla.
    pub fn full_path(&self) -> String {
        format!(
            "{}/{}/{}.csv",
            self.storage_addr, self.keyspace, self.table_name
        )
    }
}
