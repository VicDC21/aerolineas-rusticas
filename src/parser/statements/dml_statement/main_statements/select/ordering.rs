use crate::protocol::{aliases::results::Result, errors::error::Error};
use std::{fmt, str::FromStr};

/// Representa la dirección de ordenación en una cláusula ORDER BY.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolOrdering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
}

impl fmt::Display for ProtocolOrdering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProtocolOrdering::Asc => write!(f, "Asc"),
            ProtocolOrdering::Desc => write!(f, "Desc"),
        }
    }
}

impl FromStr for ProtocolOrdering {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "Asc" => Ok(ProtocolOrdering::Asc),
            "Desc" => Ok(ProtocolOrdering::Desc),
            _ => Err(Error::ServerError(
                "No se pudo parsear la dirección de ordenación".to_string(),
            )),
        }
    }
}
