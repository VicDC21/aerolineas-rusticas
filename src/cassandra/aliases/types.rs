//! Módulo para aliases de tipos de datos explicados en el protocolo de Cassandra.

use crate::cassandra::errors::error::Error;
use std::result;

/// Resultado que envuelve un error personalizado del protocolo.
pub type Result<T> = result::Result<T, Error>;

// Definiciones de notaciones

/// Un entero de 4 bytes *con signo*.
pub type Int = i32;

/// Un entero de 8 bytes *con signo*.
pub type Long = i64;

/// Un entero compuesto por un solo byte *sin signo*.
pub type Byte = u8;

/// Un entero de 2 bytes *sin signo*.
pub type Short = u16;

/// Un entero de 16 bytes que emula un UUID (asumimos no tiene signo).
pub type Uuid = u128;
