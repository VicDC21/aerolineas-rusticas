//! Módulo para estructura global de encabezados.

use std::convert::TryFrom;

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::headers::{
    flags::Flag, length::Length, opcode::Opcode, stream::Stream, version::Version,
};
use crate::protocol::traits::{Byteable, Maskable};

/// Estructura que engloba a todos los encabezados de cualquier mensaje en el protocolo.
pub struct Headers {
    /// La [versión](Version) del mensaje.
    pub version: Version,

    /// Las diferentes flags del mensaje.
    pub flags: Vec<Flag>,

    /// El ID único de este mensaje _(o `-1` si es un evento de servidor)_.
    pub stream: Stream,

    /// El tipo de operación del mensaje. Influye en la estructura del contenido.
    pub opcode: Opcode,

    /// La longitud del **contenido** del mensaje en su totalidad.
    pub length: Length,
}

impl Headers {
    /// Crea una nueva instancia de encabezados.
    pub fn new(
        version: Version,
        flags: Vec<Flag>,
        stream: Stream,
        opcode: Opcode,
        length: Length,
    ) -> Self {
        Self {
            version,
            flags,
            stream,
            opcode,
            length,
        }
    }
}

impl Byteable for Headers {
    fn as_bytes(&self) -> Vec<Byte> {
        let mut bytes_vec = Vec::<Byte>::new();
        bytes_vec.extend(self.version.as_bytes());

        let mut borrowed_flags = Vec::<&Flag>::new();
        for flag in &self.flags {
            borrowed_flags.push(flag);
        }
        bytes_vec.push(Flag::accumulate(&borrowed_flags[..]));

        bytes_vec.extend(self.stream.as_bytes());
        bytes_vec.extend(self.opcode.as_bytes());
        bytes_vec.extend(self.length.as_bytes());

        bytes_vec
    }
}

impl TryFrom<&[Byte]> for Headers {
    type Error = Error;
    fn try_from(bytes: &[Byte]) -> Result<Self, Self::Error> {
        if bytes.len() < 9 {
            return Err(Error::Invalid(
                "Se necesitan al menos 9 bytes para formar los encabezados.".to_string(),
            ));
        }

        let version = Version::try_from(bytes[0])?;
        let flags = Flag::decompose(&bytes[1]);
        let stream = Stream::try_from(bytes[2..=3].to_vec())?;
        let opcode = Opcode::try_from(bytes[4])?;
        let length = Length::try_from(bytes[5..=8].to_vec())?;
        Ok(Self::new(version, flags, stream, opcode, length))
    }
}
