use crate::cassandra::errors::error::Error;
use crate::parser::statements::ddl_statement::ddl_statement::ddl_statement;
use crate::parser::statements::dml_statement::dml_statement::dml_statement;
use crate::parser::statements::materialized_view_statement::materialized_view_statement::materialized_view_statement;
use crate::parser::statements::role_or_permission_statement::role_or_permission_statement::role_or_permission_statement;
use crate::parser::statements::secondary_index_statement::secondary_index_statement::secondary_index_statement;
use crate::parser::statements::statement::Statement;
use crate::parser::statements::trigger_statement::trigger_statement::trigger_statement;
use crate::parser::statements::udf_statement::udf_statement::udf_statement;
use crate::parser::statements::udt_statement::udt_statement::udt_statement;

pub fn make_parse(lista: &mut Vec<String>) -> Result<Statement, Error> {
    let tree: Statement = match cql_statement(lista)? {
        Some(value) => value,
        None => return Err(Error::ConfigError("dsa".to_string())),
    };
    Ok(tree)
}

fn cql_statement(lista: &mut Vec<String>) -> Result<Option<Statement>, Error> {
    statement(lista)
}

fn statement(lista: &mut Vec<String>) -> Result<Option<Statement>, Error> {
    let index = 0;

    if let Some(x) = ddl_statement(lista, index)? {
        return Ok(Some(Statement::DdlStatement(x)));
    } else if let Some(x) = dml_statement(lista, index)? {
        return Ok(Some(Statement::DmlStatement(x)));
    } else if let Some(x) = secondary_index_statement(lista, index)? {
        return Ok(Some(Statement::SecondaryIndexStatement(x)));
    } else if let Some(x) = materialized_view_statement(lista, index)? {
        return Ok(Some(Statement::MaterializedViewStatement(x)));
    } else if let Some(x) = role_or_permission_statement(lista, index)? {
        return Ok(Some(Statement::RoleOrPermissionStatement(x)));
    } else if let Some(x) = udf_statement(lista, index)? {
        return Ok(Some(Statement::UdfStatement(x)));
    } else if let Some(x) = udt_statement(lista, index)? {
        return Ok(Some(Statement::UdtStatement(x)));
    } else if let Some(x) = trigger_statement(lista, index)? {
        return Ok(Some(Statement::TriggerStatement(x)));
    }
    Ok(None)
}
