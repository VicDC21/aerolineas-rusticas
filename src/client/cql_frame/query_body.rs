use crate::protocol::{
    aliases::results::Result,
    errors::error::Error,
    notations::consistency::Consistency,
    traits::Byteable,
    utils::{encode_long_string_to_bytes, parse_bytes_to_long_string},
};

use super::flags::query_flags::QueryFlags;

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
}

impl Byteable for QueryBody {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Query string
        let query_bytes = encode_long_string_to_bytes(&self.query);
        // bytes.extend((query_bytes.len() as i32).to_be_bytes());
        bytes.extend(query_bytes);

        // Consistency
        bytes.extend(self.consistency.as_bytes());

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

impl TryFrom<Vec<u8>> for QueryBody {
    type Error = Error;

    fn try_from(bytes: Vec<u8>) -> Result<Self> {
        let mut query_lenght: usize = 0;
        let query = parse_bytes_to_long_string(&bytes, &mut query_lenght)?;
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
