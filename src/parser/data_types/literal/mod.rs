//! Paquete que contiene los módulos que representan los literales de los tipos de datos.

/// Módulo que contiene el literal de una colección.
pub mod collection_literal;
/// Módulo que contiene el literal de una lista.
pub mod list_literal;
#[allow(clippy::module_inception)]
/// Módulo que contiene los literales.
pub mod literal;
/// Módulo que contiene el literal de un mapa.
pub mod map_literal;
/// Módulo que contiene el literal de un conjunto.
pub mod set_literal;
/// Módulo que contiene el literal de una tupla.
pub mod tuple_literal;
/// Módulo que contiene el literal de un vector.
pub mod vector_literal;
