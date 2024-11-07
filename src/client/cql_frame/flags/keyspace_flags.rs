use super::keyspace_flag::KeyspaceFlag;

/// Flags específicas para la creación/alteración de keyspaces
#[derive(Clone, Copy)]
pub struct KeyspaceFlags(u8);

impl KeyspaceFlags {
    /// Crea un nuevo conjunto de flags vacío
    pub fn new() -> Self {
        Self(0)
    }

    /// Añade una flag al conjunto
    pub fn add(&mut self, flag: KeyspaceFlag) {
        self.0 |= flag as u8;
    }

    /// Obtiene el valor raw de las flags
    pub fn bits(&self) -> u8 {
        self.0
    }
}

impl Default for KeyspaceFlags {
    fn default() -> Self {
        Self::new()
    }
}
