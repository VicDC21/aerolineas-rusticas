//! Paquete del cliente.

/// Módulo que contiene la funcionalidad para el cliente.
pub mod cli;

/// Módulo que contiene los frames del protocolo CQL.
pub mod cql_frame;

/// Módulo que contiene los datos de las columnas.
pub mod col_data;

/// Módulo que contiene los resultados de las consultas.
pub mod protocol_result;

pub mod conn_holder;
