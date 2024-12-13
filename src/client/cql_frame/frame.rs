use crate::{
    client::cql_frame::query_body::QueryBody,
    protocol::{
        aliases::{
            results::Result,
            types::{Byte, Short, Uint},
        },
        errors::error::Error,
        headers::{
            flags::Flag, length::Length, msg_headers::Headers, opcode::Opcode, stream::Stream,
            version::Version,
        },
        notations::consistency::Consistency,
        traits::Byteable,
    },
};

/// Representa un frame del protocolo CQL, tanto para requests como responses
pub struct Frame {
    headers: Headers,
    body: Vec<Byte>,
}

impl Frame {
    /// Crea un nuevo frame dada la query y el _Consistency Level_.
    pub fn new(stream_id: Short, query: &str, consistency: Consistency) -> Self {
        let body = QueryBody::new(query.to_string(), consistency).as_bytes();
        let headers = Headers::new(
            Version::RequestV5,
            vec![Flag::Default],
            Stream::new(stream_id),
            Opcode::Query,
            Length::new(body.len() as Uint),
        );

        Self { headers, body }
    }
}

impl Byteable for Frame {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes = self.headers.as_bytes();
        bytes.extend(&self.body);
        bytes
    }
}

impl TryFrom<&[Byte]> for Frame {
    type Error = Error;

    fn try_from(bytes: &[Byte]) -> Result<Self> {
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

        Ok(Self { headers, body })
    }
}
