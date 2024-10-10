//! Módulo para alises de resultados.

use crate::protocol::errors::error::Error;
use std::result;

/// Resultado que envuelve un error personalizado del protocolo.
pub type Result<T> = result::Result<T, Error>;
