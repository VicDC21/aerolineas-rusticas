use crate::cassandra::errors::error::Error;

pub enum UdfStatement {
    CreateFunctionStatement,
    DropFunctionStatement,
    CreateAggregateStatement,
    DropAggregateStatement,
}

pub fn udf_statement(_lista: &mut [String], _index: i32) -> Result<Option<UdfStatement>, Error> {
    if let Some(_x) = create_function_statement(_lista, _index)? {
        return Ok(Some(UdfStatement::CreateFunctionStatement));
    } else if let Some(_x) = drop_function_statement(_lista, _index)? {
        return Ok(Some(UdfStatement::DropFunctionStatement));
    } else if let Some(_x) = create_aggregate_statement(_lista, _index)? {
        return Ok(Some(UdfStatement::CreateAggregateStatement));
    } else if let Some(_x) = drop_aggregate_statement(_lista, _index)? {
        return Ok(Some(UdfStatement::DropAggregateStatement));
    }
    Ok(None)
}

pub fn create_function_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn drop_function_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn create_aggregate_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}

pub fn drop_aggregate_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<UdfStatement>, Error> {
    Ok(None)
}
