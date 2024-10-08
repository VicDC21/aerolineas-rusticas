use crate::cassandra::errors::error::Error;
use crate::parser::statements::{
    ddl_statement::ddl_statement_parser::ddl_statement,
    dml_statement::dml_statement_parser::dml_statement,
    role_or_permission_statement::role_or_permission_statement_parser::role_or_permission_statement,
    udt_statement::udt_statement_parser::udt_statement,
    statement::Statement
};

pub fn make_parse(lista: &mut Vec<String>) -> Result<Statement, Error> {
    let tree = match cql_statement(lista)? {
        Some(value) => value,
        None => {
            return Err(Error::ConfigError(
                "Valor no coincide entre los esperados.".to_string(),
            ))
        }
    };

    Ok(tree)
}

fn cql_statement(lista: &mut Vec<String>) -> Result<Option<Statement>, Error> {
    let index: usize = 0;

    if let Some(statement) = ddl_statement(lista, index)? {
        return Ok(Some(Statement::DdlStatement(statement)));
    } else if let Some(statement) = dml_statement(lista, index)? {
        return Ok(Some(Statement::DmlStatement(statement)));
    } else if let Some(statement) = role_or_permission_statement(lista, index)? {
        return Ok(Some(Statement::RoleOrPermissionStatement(statement)));
    } else if let Some(statement) = udt_statement(lista, index)? {
        return Ok(Some(Statement::UdtStatement(statement)));
    }
    Ok(None)
}
