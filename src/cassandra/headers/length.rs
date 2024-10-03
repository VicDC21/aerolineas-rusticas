//! Módulo para el header Length.

use crate::cassandra::errors::error::Error;

/// Este header indica qué tan largo es el cuerpo del frame.
///
/// _(Actualmente está limitado a 256 MB)_
pub struct Length {
    len: u32,
}

impl Length {
    /// Crea un nuevo header de Stream.
    pub fn new(len: u32) -> Self {
        Self { len }
    }

    /// Transforma el length en una secuencia de cuatro bytes.
    pub fn as_bytes(&self) -> [u8; 4] {
        self.len.to_be_bytes()
    }
}

impl TryFrom<[u8; 4]> for Length {
    type Error = Error;
    fn try_from(integer_in_bytes: [u8; 4]) -> Result<Self, Self::Error> {
        let value = u32::from_be_bytes(integer_in_bytes);
        let bytes_lenght_limit = 0x10000000; // limite del frame de 256 MB
        match value {
            n if n <= bytes_lenght_limit => Ok(Length { len: n }),
            _ => Err(Error::Invalid(
                "El body del mensaje es muy largo (supera los 256MB)".to_string(),
            )), // Deberia ser este error?
        }
    }
}
