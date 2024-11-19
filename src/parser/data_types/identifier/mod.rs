//! Paquete de módulos que contienen los identificadores.

#[allow(clippy::module_inception)]
/// Módulo que contiene los identificadores.
pub mod identifier;
/// Módulo que contiene los identificadores con comillas.
pub mod quoted_identifier;
/// Módulo que contiene los identificadores sin comillas.
pub mod unquoted_identifier;
