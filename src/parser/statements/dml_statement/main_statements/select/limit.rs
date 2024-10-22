/// Una estructura que representa un límite en una consulta SQL.
#[derive(Debug)]
pub struct Limit {
    /// Límite de datos.
    pub limit: i32, // bind _marker
}

impl Limit {
    /// Crea una nueva instancia de `Limit`.
    pub fn new(limit: i32) -> Self {
        Limit { limit }
    }
}
