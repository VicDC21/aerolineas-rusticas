use crate::protocol::errors::error::Error;

/// Representa diferentes tipos de declaraciones de UDT (Tipo Definido por el Usuario) personalizado.
/// Las declaraciones UDT se utilizan para definir un nuevo tipo de datos personalizado.
/// Este tipo de datos personalizado se puede utilizar en la definición de columnas de una tabla.
pub enum UdtStatement {
    /// Representa una declaración CREATE TYPE.
    /// Dicha declaración se utiliza para crear un nuevo tipo de datos personalizado.
    /// CREATE TYPE statement ::= CREATE TYPE keyspace_name '.' type_name
    CreateTypeStatement,
    /// Representa una declaración ALTER TYPE.
    /// Dicha declaración se utiliza para modificar un tipo de datos personalizado existente.
    /// ALTER TYPE statement ::= ALTER TYPE keyspace_name '.' type_name
    AlterTypeStatement,
    /// Representa una declaración DROP TYPE.
    /// Dicha declaración se utiliza para eliminar un tipo de datos personalizado existente.
    /// DROP TYPE statement ::= DROP TYPE keyspace_name '.' type_name
    DropTypeStatement,
}

/// Parsea una declaración de UDT (Tipo Definido por el Usuario) personalizado.
pub fn udt_statement(lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    if let Some(_x) = create_type_statement(lista)? {
        return Ok(Some(UdtStatement::CreateTypeStatement));
    } else if let Some(_x) = alter_type_statement(lista)? {
        return Ok(Some(UdtStatement::AlterTypeStatement));
    } else if let Some(_x) = drop_type_statement(lista)? {
        return Ok(Some(UdtStatement::DropTypeStatement));
    }
    Ok(None)
}

/// Parsea una declaración CREATE TYPE.
pub fn create_type_statement(_lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}

/// Parsea una declaración ALTER TYPE.
pub fn alter_type_statement(_lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}

/// Parsea una declaración DROP TYPE.
pub fn drop_type_statement(_lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}
