//! Módulo para _traits_ de las estructuras de datos.

/// Nombres apropiados para ser mostrados por pantalla, y no para ser
/// usado como ID internamente en el código.
pub trait PrettyShow {
    /// Una variante del nombre, apto para ser impreso en interfaces de usuario.
    fn pretty_name(&self) -> &str;
}
