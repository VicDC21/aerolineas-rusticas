use crate::cassandra::errors::error::Error;

pub enum UdfStatement {
    CreateFunctionStatement,
    DropFunctionStatement,
    CreateAggregateStatement,
    DropAggregateStatement,
}

pub fn udf_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<UdfStatement>, Error> {
    if let Some(_x) = create_function_statement(lista, index)? {
        return Ok(Some(UdfStatement::CreateFunctionStatement));
    } else if let Some(_x) = drop_function_statement(lista, index)? {
        return Ok(Some(UdfStatement::DropFunctionStatement));
    } else if let Some(_x) = create_aggregate_statement(lista, index)? {
        return Ok(Some(UdfStatement::CreateAggregateStatement));
    } else if let Some(_x) = drop_aggregate_statement(lista, index)? {
        return Ok(Some(UdfStatement::DropAggregateStatement));
    }
    Ok(None)
}

pub fn create_function_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn drop_function_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn create_aggregate_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn drop_aggregate_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}
