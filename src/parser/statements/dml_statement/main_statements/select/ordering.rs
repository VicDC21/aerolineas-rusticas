/// Representa la dirección de ordenación en una cláusula ORDER BY.
pub enum Ordering {
    /// Orden ascendente.
    Asc,
    /// Orden descendente.
    Desc,
}

impl Ordering {
    /// Obtiene la dirección de ordenación de un slice de String.
    pub fn ordering_from_str(order: &str) -> Option<Ordering> {
        match order.to_ascii_lowercase().as_str() {
            "asc" => Some(Ordering::Asc),
            "desc" => Some(Ordering::Desc),
            _ => None,
        }
    }
}
