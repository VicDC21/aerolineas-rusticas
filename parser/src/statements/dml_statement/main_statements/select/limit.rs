use protocol::aliases::types::Int;

#[derive(Debug)]
/// Una estructura que representa un límite en una consulta SQL.
pub struct Limit {
    /// Límite de datos.
    pub limit: Int, // bind _marker
}

impl Limit {
    /// Crea una nueva instancia de `Limit`.
    pub fn new(limit: Int) -> Self {
        Limit { limit }
    }
}
