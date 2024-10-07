use crate::cassandra::errors::error::Error;

pub enum TriggerStatement {
    CreateTriggerStatement,
    DropTriggerStatement,
}

pub fn trigger_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<TriggerStatement>, Error> {
    if let Some(_x) = create_trigger_statement(lista, index)? {
        return Ok(Some(TriggerStatement::CreateTriggerStatement));
    } else if let Some(_x) = drop_trigger_statement(lista, index)? {
        return Ok(Some(TriggerStatement::DropTriggerStatement));
    }
    Ok(None)
}

pub fn create_trigger_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<TriggerStatement>, Error> {
    Ok(None)
}

pub fn drop_trigger_statement(
    lista: &mut Vec<String>,
    index: i32,
) -> Result<Option<TriggerStatement>, Error> {
    Ok(None)
}
