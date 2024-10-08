use crate::cassandra::errors::error::Error;

pub enum UdtStatement {
    CreateTypeStatement,
    AlterTypeStatement,
    DropTypeStatement,
}

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

pub fn create_type_statement(lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}

pub fn alter_type_statement(lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}

pub fn drop_type_statement(lista: &mut [String]) -> Result<Option<UdtStatement>, Error> {
    Ok(None)
}
