/// Representa la dirección de ordenación en una cláusula ORDER BY.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolOrdering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
}

impl ProtocolOrdering {
    /// Obtiene la dirección de ordenación de un slice de String.
    pub fn ordering_from_str(order: &str) -> Option<Self> {
        match order.to_ascii_lowercase().as_str() {
            "asc" => Some(Self::Asc),
            "desc" => Some(Self::Desc),
            _ => None,
        }
    }
}
