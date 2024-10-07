use crate::cassandra::errors::error::Error;

pub enum DmlStatement {
    SelectStatement,
    InsertStatement,
    UpdateStatement,
    DeleteStatement,
    BatchStatement,
}

pub fn dml_statement(_lista: &mut [String], _index: i32) -> Result<Option<DmlStatement>, Error> {
    if let Some(_x) = use_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::SelectStatement));
    } else if let Some(_x) = create_keyspace_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::InsertStatement));
    } else if let Some(_x) = alter_keyspace_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::UpdateStatement));
    } else if let Some(_x) = drop_keyspace_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::DeleteStatement));
    } else if let Some(_x) = create_table_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::BatchStatement));
    }
    Ok(None)
}

pub fn create_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn alter_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn drop_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn create_table_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}

pub fn use_statement(_lista: &mut [String], _index: i32) -> Result<Option<DmlStatement>, Error> {
    Ok(None)
}
