//! Módulo para alises de resultados.

use {crate::protocol::errors::error::Error, std::result};

/// Resultado que envuelve un error personalizado del protocolo.
pub type Result<T> = result::Result<T, Error>;
