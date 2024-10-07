use crate::{cassandra::errors::error::Error, parser::data_types::{keyspace_name::KeyspaceName, quoted_name::QuotedName, unquoted_name::UnquotedName}};

pub enum DdlStatement {
    UseStatement(KeyspaceName),
    CreateKeyspaceStatement,
    AlterKeyspaceStatement,
    DropKeyspaceStatement,
    CreateTableStatement,
    AlterTableStatement,
    DropTableStatement,
    TruncateStatement,
}

pub fn ddl_statement(_lista: &mut [String], _index: i32) -> Result<Option<DdlStatement>, Error> {
    if let Some(_x) = use_statement(_lista, _index)? {
        return Ok(Some(_x))
        // return Ok(Some(DdlStatement::UseStatement(KeyspaceName)));
    } else if let Some(_x) = create_keyspace_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::CreateKeyspaceStatement));
    } else if let Some(_x) = alter_keyspace_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::AlterKeyspaceStatement));
    } else if let Some(_x) = drop_keyspace_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::DropKeyspaceStatement));
    } else if let Some(_x) = create_table_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::CreateTableStatement));
    } else if let Some(_x) = alter_table_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::AlterTableStatement));
    } else if let Some(_x) = drop_table_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::DropTableStatement));
    } else if let Some(_x) = truncate_statement(_lista, _index)? {
        return Ok(Some(DdlStatement::TruncateStatement));
    }
    Ok(None)
}

pub fn use_statement(_lista: &mut [String], _index: i32) -> Result<Option<DdlStatement>, Error> {
    if _lista[0] == "USE"{
        if _lista[1] == "\""{
            let keyspace = DdlStatement::UseStatement(KeyspaceName::QuotedName(UnquotedName::new(_lista[2].clone())?));
            return Ok(Some(keyspace))
        } else {
            let keyspace = DdlStatement::UseStatement(KeyspaceName::UnquotedName(UnquotedName::new(_lista[1].clone())?));
            return Ok(Some(keyspace))
        }
    }
    Ok(None)
}

pub fn create_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn alter_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn drop_keyspace_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn create_table_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn alter_table_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn drop_table_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn truncate_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}
