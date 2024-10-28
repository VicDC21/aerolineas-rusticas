use crate::protocol::utils::encode_string_to_bytes;
use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    headers::msg_headers::Headers,
    notations::consistency::Consistency,
    traits::Byteable,
};

use crate::protocol::headers::{
    flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version,
};

use super::cli::QueryFlags;

/// Representa un frame del protocolo CQL, tanto para requests como responses
pub struct Frame {
    headers: Headers,
    body: Vec<Byte>,
}

impl Frame {
    /// Crea un nuevo frame
    pub fn new(headers: Headers, body: Vec<Byte>) -> Self {
        Self { headers, body }
    }

    /// Crea un frame para una query
    pub fn query(stream_id: i16, query: String) -> Self {
        
        let body = query.as_bytes().to_vec();
        let headers = Headers::new(
            Version::RequestV5,
            vec![Flag::Default],
            Stream::new(stream_id),
            Opcode::Query,
            Length::new(body.len() as u32),
        );
        Self::new(headers, body)
    }

    /// Crea un frame para un DDL statement
    pub fn ddl(stream_id: i16, statement: String) -> Self {
        let body = statement.as_bytes().to_vec();
        let headers = Headers::new(
            Version::RequestV5,
            vec![Flag::Default],
            Stream::new(stream_id),
            Opcode::Query, // DDL usa el mismo opcode que las queries normales
            Length::new(body.len() as u32),
        );
        Self::new(headers, body)
    }

    /* Crea un frame para un batch
    pub fn batch(stream_id: i16, queries: Vec<DmlStatement>) -> Self {
        let body = BatchBody::new(queries).as_bytes();
        let headers = Headers::new(
            Version::RequestV5,
            vec![Flag::Default],
            Stream::new(stream_id),
            Opcode::Batch,
            Length::new(body.len() as u32),
        );
        Self::new(headers, body)
    }*/

    /// Parsea bytes a un Frame
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.len() < 9 {
            return Err(Error::ProtocolError("Header incompleto".to_string()));
        }

        let version = Version::try_from(bytes[0])?;
        let flags = Flag::try_from(bytes[1])?;
        let stream = Stream::try_from(bytes[2..4].to_vec())?;
        let opcode = Opcode::try_from(bytes[4])?;
        let length = Length::try_from(bytes[5..9].to_vec())?;

        let headers = Headers::new(version, vec![flags], stream, opcode, length);
        let body = bytes[9..].to_vec();

        Ok(Self::new(headers, body))
    }
}

impl Byteable for Frame {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = self.headers.as_bytes();
        bytes.extend(&self.body);
        bytes
    }
}

/// Body para queries individuales
pub struct QueryBody {
    query: String,
    consistency: Consistency,
    flags: Vec<QueryFlags>,
    values: Option<Vec<Vec<u8>>>,
    page_size: Option<i32>,
    paging_state: Option<Vec<u8>>,
    serial_consistency: Option<Consistency>,
    timestamp: Option<i64>,
}

impl QueryBody {
    /// Crea un nuevo body para queries
    pub fn new(query: String) -> Self {
        Self {
            query,
            consistency: Consistency::One,
            flags: Vec::new(),
            values: None,
            page_size: None,
            paging_state: None,
            serial_consistency: None,
            timestamp: None,
        }
    }
}

impl Byteable for QueryBody {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Query string
        let query_bytes = encode_string_to_bytes(&self.query);
        bytes.extend((query_bytes.len() as i32).to_be_bytes());
        bytes.extend(query_bytes);

        // Consistency
        bytes.extend((self.consistency as u16).to_be_bytes());

        // Flags
        let flags_byte = self.flags.iter().fold(0u8, |acc, flag| acc | *flag as u8);
        bytes.push(flags_byte);

        // Optional values based on flags
        for flag in &self.flags {
            match flag {
                QueryFlags::Values => {
                    if let Some(values) = &self.values {
                        bytes.extend((values.len() as i16).to_be_bytes());
                        for value in values {
                            bytes.extend((value.len() as i32).to_be_bytes());
                            bytes.extend(value);
                        }
                    }
                }
                QueryFlags::PageSize => {
                    if let Some(size) = self.page_size {
                        bytes.extend(size.to_be_bytes());
                    }
                }
                QueryFlags::WithPagingState => {
                    if let Some(state) = &self.paging_state {
                        bytes.extend((state.len() as i32).to_be_bytes());
                        bytes.extend(state);
                    }
                }
                QueryFlags::WithSerialConsistency => {
                    if let Some(consistency) = &self.serial_consistency {
                        bytes.extend((*consistency as u16).to_be_bytes());
                    }
                }
                QueryFlags::WithDefaultTimestamp => {
                    if let Some(ts) = self.timestamp {
                        bytes.extend(ts.to_be_bytes());
                    }
                }
                _ => {}
            }
        }
        bytes
    }
}

/// Body específico para DDL statements
pub struct DdlBody {
    statement: String,
    consistency: Consistency,
    keyspace_flags: Option<KeyspaceFlags>,
    table_flags: Option<TableFlags>,
}

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

/// Flags individuales para keyspaces
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum KeyspaceFlag {
    /// Flag para estrategia de replicación
    ReplicationStrategy = 0x01,
    /// Flag para factor de replicación
    ReplicationFactor = 0x02,
    /// Flag para durable writes
    DurableWrites = 0x04,
}

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

/// Flags individuales para tablas
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum TableFlag {
    /// Flag para ordenamiento
    ClusteringOrder = 0x01,
    /// Flags para compresión
    Compression = 0x02,
    /// Flags para caching
    Caching = 0x04,
    /// Flags para compaction
    Compaction = 0x08,
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
    fn as_bytes(&self) -> Vec<u8> {
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

/* Body para batch queries
/pub struct BatchBody {
    queries: Vec<DmlStatement>,
    consistency: Consistency,
    flags: Vec<QueryFlags>,
    timestamp: Option<i64>,
}

impl BatchBody {
    /// Crea un nuevo body para batch queries
    pub fn new(queries: Vec<DmlStatement>) -> Self {
        Self {
            queries,
            consistency: Consistency::One,
            flags: Vec::new(),
            timestamp: None,
        }
    }
}

impl Byteable for BatchBody {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Batch type (0 = LOGGED)
        bytes.push(0);

        // Number of queries
        bytes.extend((self.queries.len() as i16).to_be_bytes());

        // Queries
        for query in &self.queries {
            // Kind (0 = QUERY)
            bytes.push(0);

            let query_bytes = encode_string_to_bytes(&query.to_string());
            bytes.extend((query_bytes.len() as i32).to_be_bytes());
            bytes.extend(query_bytes);

            // No values for now (0)
            bytes.extend([0, 0]);
        }

        // Consistency
        bytes.extend((self.consistency as u16).to_be_bytes());

        // Flags
        let flags_byte = self.flags.iter().fold(0u8, |acc, flag| acc | *flag as u8);
        bytes.push(flags_byte);

        // Timestamp if present
        if let Some(ts) = self.timestamp {
            bytes.extend(ts.to_be_bytes());
        }

        bytes
    }
}*/
