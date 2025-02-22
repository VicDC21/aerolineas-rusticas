use crate::cql_frame::query_flags::QueryFlags;
use protocol::{
    aliases::{
        results::Result,
        types::{Byte, Int, Long, ShortInt},
    },
    errors::error::Error,
    notations::consistency::Consistency,
    traits::Byteable,
    utils::{encode_long_string_to_bytes, parse_bytes_to_long_string},
};

/// Body para queries individuales
pub struct QueryBody {
    query: String,
    consistency: Consistency,
    flags: Vec<QueryFlags>,
    values: Option<Vec<Vec<Byte>>>,
    page_size: Option<Int>,
    paging_state: Option<Vec<Byte>>,
    serial_consistency: Option<Consistency>,
    timestamp: Option<Long>,
}

impl QueryBody {
    /// Crea un nuevo body para queries
    pub fn new(query: String, consistency: Consistency) -> Self {
        Self {
            query,
            consistency,
            flags: Vec::new(),
            values: None,
            page_size: None,
            paging_state: None,
            serial_consistency: None,
            timestamp: None,
        }
    }

    /// Devuelve la query del body
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Devuelve el _Consistency Level_ del body
    pub fn get_consistency_level(&self) -> &Consistency {
        &self.consistency
    }
}

impl Byteable for QueryBody {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = Vec::new();

        // Query string
        let query_bytes = encode_long_string_to_bytes(&self.query);
        // bytes.extend((query_bytes.len() as Int).to_be_bytes());
        bytes.extend(query_bytes);

        // Consistency
        bytes.extend(self.consistency.as_bytes());

        // Flags
        let flags_byte = self.flags.iter().fold(0u8, |acc, flag| acc | *flag as Byte);
        bytes.push(flags_byte);

        // Optional values based on flags
        for flag in &self.flags {
            match flag {
                QueryFlags::Values => {
                    if let Some(values) = &self.values {
                        bytes.extend((values.len() as ShortInt).to_be_bytes());
                        for value in values {
                            bytes.extend((value.len() as Int).to_be_bytes());
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
                        bytes.extend((state.len() as Int).to_be_bytes());
                        bytes.extend(state);
                    }
                }
                QueryFlags::WithSerialConsistency => {
                    if let Some(consistency) = &self.serial_consistency {
                        bytes.extend((*consistency as Byte).to_be_bytes());
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

impl TryFrom<&[Byte]> for QueryBody {
    type Error = Error;

    fn try_from(bytes: &[Byte]) -> Result<Self> {
        let mut query_lenght: usize = 0;
        let query = parse_bytes_to_long_string(bytes, &mut query_lenght)?;
        let consistency = Consistency::try_from(&bytes[query_lenght..(query_lenght + 2)])?;
        Ok(QueryBody {
            query,
            consistency,
            flags: Vec::new(),
            values: None,
            page_size: None,
            paging_state: None,
            serial_consistency: None,
            timestamp: None,
        })
    }
}
