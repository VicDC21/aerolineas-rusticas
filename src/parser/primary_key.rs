use crate::cassandra::errors::error::Error;
pub struct PrimaryKey {
    pub columns: Vec<String>,
}

impl PrimaryKey {
    pub fn parse(lista: &mut Vec<String>) -> Result<Self, Error> {
        let mut columns = Vec::new();
        while !lista.is_empty() && lista[0] != ")" {
            columns.push(lista.remove(0));
        }
        Ok(PrimaryKey { columns })
    }
}
