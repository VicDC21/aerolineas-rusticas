use protocol::aliases::types::{Double, Int, Long};

/// Representa el tipo de dato y el dato en cuestión que se almacena en una columna de una tabla.
#[derive(Clone, Debug)]
pub enum ColData {
    /// Representa un dato de tipo String.
    String(String),
    /// Representa un dato de tipo Timestamp.
    Timestamp(Long),
    /// Representa un dato de tipo Double.
    Double(Double),
    /// Representa un dato de tipo Int.
    Int(Int),
}
