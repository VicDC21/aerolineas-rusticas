use {
    crate::protocol::{aliases::results::Result, errors::error::Error},
    serde::{Deserialize, Serialize},
    std::str::FromStr,
};

/// Representa la dirección de ordenación en una cláusula ORDER BY.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProtocolOrdering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
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
