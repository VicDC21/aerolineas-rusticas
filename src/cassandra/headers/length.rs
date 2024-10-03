//! Módulo para el header Length.

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
