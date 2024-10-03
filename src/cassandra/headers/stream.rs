//! Módulo para un header de stream.

/// Cada frame tiene un stream id para hacer coincidir el IDs entre las requests y responses.
pub struct Stream {
    /// El ID del stream.
    id: i16,
}

impl Stream {
    /// Crea un nuevo header de Stream.
    pub fn new(id: i16) -> Self {
        Self { id }
    }

    /// Transforma el ID en una secuencia de dos bytes.
    pub fn as_bytes(&self) -> [u8; 2] {
        self.id.to_be_bytes()
    }
}
