use crate::client::cql_frame::flags::table_flag::TableFlag;

/// Flags específicas para la creación/alteración de tablas
#[derive(Clone, Copy)]
pub struct TableFlags(u8);

impl TableFlags {
    /// Crea un nuevo conjunto de flags vacío
    pub fn new() -> Self {
        Self(0)
    }

    /// Añade una flag al conjunto
    pub fn add(&mut self, flag: TableFlag) {
        self.0 |= flag as u8;
    }

    /// Obtiene el valor raw de las flags
    pub fn bits(&self) -> u8 {
        self.0
    }
}

impl Default for TableFlags {
    fn default() -> Self {
        Self::new()
    }
}
