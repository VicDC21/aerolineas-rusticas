use crate::cassandra::errors::error::Error;

pub enum DmlStatement {
    SelectStatement,
    InsertStatement,
    UpdateStatement,
    DeleteStatement,
    BatchStatement,
}

pub fn dml_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<DmlStatement>, Error> {
    if let Some(_x) = use_statement(lista, index)? {
        return Ok(Some(DmlStatement::SelectStatement));
    } else if let Some(_x) = create_keyspace_statement(lista, index)? {
        return Ok(Some(DmlStatement::InsertStatement));
    } else if let Some(_x) = alter_keyspace_statement(lista, index)? {
        return Ok(Some(DmlStatement::UpdateStatement));
    } else if let Some(_x) = drop_keyspace_statement(lista, index)? {
        return Ok(Some(DmlStatement::DeleteStatement));
    } else if let Some(_x) = create_table_statement(lista, index)? {
        return Ok(Some(DmlStatement::BatchStatement));
    }
    Ok(None)
}

pub fn create_keyspace_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn alter_keyspace_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn drop_keyspace_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn create_table_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn use_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}
