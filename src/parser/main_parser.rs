use crate::{
    parser::statements::{
        ddl_statement::ddl_statement_parser::ddl_statement,
        dml_statement::dml_statement_parser::dml_statement, login_user_statement::login_statement,
        startup_statement::startup_statement, statement::Statement,
    },
    protocol::{aliases::results::Result, errors::error::Error},
};

/// Analiza una lista de declaraciones CQL y devuelve un `Statement` o un `Error`.
///
/// # Argumentos
///
/// * `lista` - Una referencia mutable a un vector de cadenas que representan declaraciones CQL.
///
/// # Retornos
///
/// * `Result<Statement, Error>` - Un resultado que contiene un `Statement` o un `Error`.
pub fn make_parse(lista: &mut Vec<String>) -> Result<Statement> {
    let statement = match cql_statement(lista)? {
        Some(value) => value,
        None => {
            return Err(Error::ConfigError(
                "Valor no coincide entre los esperados.".to_string(),
            ))
        }
    };

    Ok(statement)
}

fn cql_statement(lista: &mut Vec<String>) -> Result<Option<Statement>> {
    if let Some(statement) = ddl_statement(lista)? {
        return Ok(Some(Statement::DdlStatement(statement)));
    } else if let Some(statement) = dml_statement(lista)? {
        return Ok(Some(Statement::DmlStatement(statement)));
    } else if let Some(statement) = login_statement(lista)? {
        return Ok(Some(Statement::LoginUser(statement)));
    } else if (startup_statement(lista)?).is_some() {
        return Ok(Some(Statement::Startup));
    }
    Ok(None)
}
