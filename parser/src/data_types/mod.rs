//! Paquete de módulos que contienen los tipos de datos que se pueden utilizar en la gramática de CQL.

/// Módulo que contiene el tipo de dato `Constant`.
pub mod constant;
/// Módulo que contiene los tipos de _CQL_.
pub mod cql_type;
/// Módulo que contiene los identificadores.
pub mod identifier;
/// Módulo que contiene el tipo de dato `KeyspaceName`.
pub mod keyspace_name;
/// Módulo que contiene los literales.
pub mod literal;
/// Módulo que contiene los nombres con comillas.
pub mod quoted_name;
/// Módulo que contiene los términos.
pub mod term;
/// Módulo que contiene los nombres sin comillas.
pub mod unquoted_name;
