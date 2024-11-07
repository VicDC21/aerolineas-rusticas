use crate::protocol::{
    aliases::{results::Result, types::Byte},
    errors::error::Error,
    headers::msg_headers::Headers,
    traits::Byteable,
};

use crate::protocol::headers::{
    flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version,
};

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
