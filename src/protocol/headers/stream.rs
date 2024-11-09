//! Módulo para un header de stream.

use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::protocol::aliases::types::Byte;
use crate::protocol::errors::error::Error;
use crate::protocol::traits::Byteable;

/// Cada frame tiene un stream id para hacer coincidir el IDs entre las requests y responses.
#[derive(Eq, Clone, Hash, PartialEq)]
pub struct Stream {
    /// El ID del stream.
    id: i16,
}

impl Stream {
    /// Crea un nuevo header de Stream.
    pub fn new(id: i16) -> Self {
        Self { id }
    }
}

impl Byteable for Stream {
    fn as_bytes(&self) -> Vec<Byte> {
        self.id.to_be_bytes().to_vec()
    }
}

impl TryFrom<Vec<Byte>> for Stream {
    type Error = Error;
    fn try_from(short: Vec<Byte>) -> Result<Self, Self::Error> {
        let bytes_array: [Byte; 2] = match short.try_into() {
            Ok(bytes_array) => bytes_array,
            Err(_) => {
                return Err(Error::ConfigError(
                    "No se pudo castear el vector de bytes en un array en Stream".to_string(),
                ))
            }
        };
        let value = i16::from_be_bytes(bytes_array);
        Ok(Stream::new(value))
    }
}

impl Display for Stream {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "{}", self.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::aliases::types::Byte;
    use crate::protocol::errors::error::Error;
    use crate::protocol::headers::stream::Stream;
    use crate::protocol::traits::Byteable;

    #[test]
    fn test_1_serializar() {
        for i in 0..1000 {
            let ind = i as i16; // por las dudas casteamos

            let stream = Stream::new(ind);
            let id_bytes = stream.as_bytes();

            // El Short es un entero de 2 bytes
            assert_eq!(id_bytes.len(), 2);
            assert_eq!(id_bytes, ind.to_be_bytes());
        }
    }

    #[test]
    fn test_2_deserializar() {
        for i in 0..1000 {
            let ind = i as i16;

            let stream_res = Stream::try_from(ind.to_be_bytes().to_vec());
            assert!(stream_res.is_ok());
            if let Ok(stream) = stream_res {
                assert_eq!(stream.id, ind);
            }
        }
    }

    #[test]
    fn test_3_bytes_de_longitud_incorrecta() {
        let muy_corto: Vec<Byte> = vec![0x1];
        let muy_largo: Vec<Byte> = vec![0x0, 0x10, 0x1];

        let corto_res = Stream::try_from(muy_corto);
        let largo_res = Stream::try_from(muy_largo);

        assert!(corto_res.is_err());
        if let Err(err) = corto_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }

        assert!(largo_res.is_err());
        if let Err(err) = largo_res {
            assert!(matches!(err, Error::ConfigError(_)));
        }
    }
}
