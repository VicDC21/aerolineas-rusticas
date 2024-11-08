use crate::protocol::{
    aliases::types::Byte, notations::consistency::Consistency, traits::Byteable,
    utils::encode_string_to_bytes,
};

use super::flags::{
    keyspace_flag::KeyspaceFlag, keyspace_flags::KeyspaceFlags, table_flag::TableFlag,
    table_flags::TableFlags,
};

/// Body específico para DDL statements
pub struct DdlBody {
    statement: String,
    consistency: Consistency,
    keyspace_flags: Option<KeyspaceFlags>,
    table_flags: Option<TableFlags>,
}

impl DdlBody {
    /// Crea un nuevo body para DDL statements
    pub fn new(statement: String) -> Self {
        Self {
            statement,
            consistency: Consistency::All, // DDL statements típicamente usan consistencia ALL
            keyspace_flags: None,
            table_flags: None,
        }
    }

    /// Configura flags para keyspaces de manera segura
    pub fn with_keyspace_flags(mut self, flags: &[KeyspaceFlag]) -> Self {
        let mut keyspace_flags = KeyspaceFlags::new();
        for &flag in flags {
            keyspace_flags.add(flag);
        }
        self.keyspace_flags = Some(keyspace_flags);
        self
    }

    /// Configura flags para tablas de manera segura
    pub fn with_table_flags(mut self, flags: &[TableFlag]) -> Self {
        let mut table_flags = TableFlags::new();
        for &flag in flags {
            table_flags.add(flag);
        }
        self.table_flags = Some(table_flags);
        self
    }
}

impl Byteable for DdlBody {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = Vec::new();

        // Statement string
        let statement_bytes = encode_string_to_bytes(&self.statement);
        bytes.extend((statement_bytes.len() as i32).to_be_bytes());
        bytes.extend(statement_bytes);

        // Consistency
        bytes.extend((self.consistency as u16).to_be_bytes());

        // Flags base (0 para DDL statements básicos)
        bytes.push(0);

        // Si hay flags de keyspace, las agregamos usando el valor raw seguro
        if let Some(keyspace_flags) = &self.keyspace_flags {
            bytes.push(keyspace_flags.bits());
        }

        // Si hay flags de tabla, las agregamos usando el valor raw seguro
        if let Some(table_flags) = &self.table_flags {
            bytes.push(table_flags.bits());
        }

        bytes
    }
}
