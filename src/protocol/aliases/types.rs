//! Módulo para aliases de tipos de datos explicados en el protocolo de Cassandra.

use std::{collections::HashMap, net::IpAddr};

// Definiciones de notaciones

/// Un entero compuesto por 2 bytes *con signo*.
pub type Short = i16;

/// Un entero de 4 bytes *con signo*.
pub type Int = i32;

/// Un entero de 8 bytes *con signo*.
pub type Long = i64;

/// Un entero compuesto por un solo byte *sin signo*.
pub type Byte = u8;

/// Un entero de 2 bytes *sin signo*.
pub type UShort = u16;

/// Un entero de 4 bytes *sin signo*.
pub type Uint = u32;

/// Un entero de 8 bytes *sin signo*.
pub type Ulong = u64;

/// Un entero de 16 bytes que emula un UUID (asumimos no tiene signo).
pub type Uuid = u128;

/// Un número de punto flotante IEEE 754 (Binary32) de precisión simple.
pub type Float = f32;

/// Un número de punto flotante IEEE 754 (Binary64) de precisión doble.
pub type Double = f64;

// Abreviaciones auxiliares

/// Un mapa de endpoints con códigos de errores.
pub type ReasonMap = HashMap<IpAddr, UShort>;

/// Un mapa de valores posibles para las opciones de un mensaje de tipo [STARTUP](crate::protocol::headers::opcode::Opcode::Startup).
pub type SupportedMultiMap = HashMap<String, Vec<String>>;
