use crate::cassandra::errors::error::Error;
pub struct PrimaryKey {
    pub columns: Vec<String>,
}

impl PrimaryKey {
    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        // Implementa la lógica para parsear la definición de la clave primaria
        // Esta es una implementación simplificada, ajústala según tus necesidades
        let mut columns = Vec::new();
        while !lista.is_empty() && lista[0] != ")" {
            columns.push(lista.remove(0));
        }
        Ok(PrimaryKey { columns })
    }
}
