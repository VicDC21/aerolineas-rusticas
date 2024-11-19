//! Paquete de módulos que contienen los tipos de datos de CQL.

/// Módulo que contiene los tipos de colección.
pub mod collection_type;
#[allow(clippy::module_inception)]
/// Módulo que contiene los tipos de _CQL_.
pub mod cql_type;
/// Módulo que contiene los tipos de datos nativos.
pub mod native_types;
/// Módulo que contiene los tipos de tupla.
pub mod tuple_type;
