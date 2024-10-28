/// Representa el tipo de dato y el dato en cuestión que se almacena en una columna de una tabla.
pub enum ColData {
    /// Representa un dato de tipo String.
    String(String),
    /// Representa un dato de tipo Timestamp.
    Timestamp(i64),
    /// Representa un dato de tipo Double.
    Double(f64),
    /// Representa un dato de tipo Int.
    Int(i32),
}
