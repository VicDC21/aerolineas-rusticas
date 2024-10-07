use crate::cassandra::errors::error::Error;
use crate::parser::statements::{
    ddl_statement::ddl_statement_parser::ddl_statement,
    dml_statement::dml_statement_parser::dml_statement,
    materialized_view_statement::materialized_view_statement_parser::materialized_view_statement,
    role_or_permission_statement::role_or_permission_statement_parser::role_or_permission_statement,
    secondary_index_statement::secondary_index_statement_parser::secondary_index_statement,
    statement::Statement, trigger_statement::trigger_statement_parser::trigger_statement,
    udf_statement::udf_statement_parser::udf_statement,
    udt_statement::udt_statement_parser::udt_statement,
};

pub fn make_parse(lista: &mut [String]) -> Result<Statement, Error> {
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

fn cql_statement(lista: &mut [String]) -> Result<Option<Statement>, Error> {
    let index = 0;

    if let Some(statement) = ddl_statement(lista, index)? {
        return Ok(Some(Statement::DdlStatement(statement)));
    } else if let Some(statement) = dml_statement(lista, index)? {
        return Ok(Some(Statement::DmlStatement(statement)));
    } else if let Some(statement) = secondary_index_statement(lista, index)? {
        return Ok(Some(Statement::SecondaryIndexStatement(statement)));
    } else if let Some(statement) = materialized_view_statement(lista, index)? {
        return Ok(Some(Statement::MaterializedViewStatement(statement)));
    } else if let Some(statement) = role_or_permission_statement(lista, index)? {
        return Ok(Some(Statement::RoleOrPermissionStatement(statement)));
    } else if let Some(statement) = udf_statement(lista, index)? {
        return Ok(Some(Statement::UdfStatement(statement)));
    } else if let Some(statement) = udt_statement(lista, index)? {
        return Ok(Some(Statement::UdtStatement(statement)));
    } else if let Some(statement) = trigger_statement(lista, index)? {
        return Ok(Some(Statement::TriggerStatement(statement)));
    }
    Ok(None)
}
