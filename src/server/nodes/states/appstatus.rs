//! Módulo para el estado de un nodo.

/// El estado actual de un nodo.
pub enum AppStatus {
    /// El nodo funciona normalmente.
    Normal,

    /// El nodo se está conectando.
    Bootstrap,

    /// El nodo esta siendo dado de baja.
    Left,

    /// El nodo esta siendo dado de baja porque no se puede acceder a él.
    Remove,
}
