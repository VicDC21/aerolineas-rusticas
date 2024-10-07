use crate::cassandra::errors::error::Error;

pub enum SecondaryIndexStatement {
    CreateIndexStatement,
    DropIndexStatement,
}

pub fn secondary_index_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<SecondaryIndexStatement>, Error> {
    if let Some(_x) = create_index_statement(_lista, _index)? {
        return Ok(Some(SecondaryIndexStatement::CreateIndexStatement));
    } else if let Some(_x) = drop_index_statement(_lista, _index)? {
        return Ok(Some(SecondaryIndexStatement::DropIndexStatement));
    }
    Ok(None)
}

pub fn create_index_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<SecondaryIndexStatement>, Error> {
    Ok(None)
}

pub fn drop_index_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<SecondaryIndexStatement>, Error> {
    Ok(None)
}
