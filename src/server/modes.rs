//! Módulo para modos de conección al servidor.

/// Indica el modo de conexión al instanciar el servidor.
#[derive(Clone, Debug)]
pub enum ConnectionMode {
    /// Modo de prueba para testear conexión.
    Echo,

    /// El modo general para parsear _queries_ de CQL.
    Parsing,
}
