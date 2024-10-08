use crate::{cassandra::errors::error::Error, parser::r#where::Where, parser::select::Select};

pub enum DmlStatement {
    SelectStatement,
    InsertStatement,
    UpdateStatement,
    DeleteStatement,
    BatchStatement,
}

pub fn dml_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    if let Some(_x) = select_statement(lista)? {
        return Ok(Some(DmlStatement::SelectStatement));
    } else if let Some(_x) = insert_statement(lista)? {
        return Ok(Some(DmlStatement::InsertStatement));
    } else if let Some(_x) = delete_statement(lista)? {
        return Ok(Some(DmlStatement::UpdateStatement));
    } else if let Some(_x) = update_statement(lista)? {
        return Ok(Some(DmlStatement::DeleteStatement));
    } else if let Some(_x) = batch_statement(lista)? {
        return Ok(Some(DmlStatement::BatchStatement));
    }
    Ok(None)
}

pub fn select_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    let index = 0;
    if lista[index] == "SELECT" {
        // Podria hacer un builder que cree los campos poco a poco?
        if lista[index] == "*" {
            // Select{

            // };
        } else {
            return Err(Error::SyntaxError("Falta la tabla".to_string()));
        }
        if lista[index] == "WHERE" {
            let res = select_clause(lista);
            // aca completar el builder poco a poco
        } else {
            return Ok(None); // al builder pasarle esto
        }
        if lista[index] == "GROUP" && lista[index + 1] == "BY" {
        } else {
            return Ok(None);
        }
        if lista[index] == "ORDER" && lista[index + 1] == "BY" {}
        if lista[index] == "PER" && lista[index + 1] == "PARTITION" && lista[index + 2] == "LIMIT" {
        }
        if lista[index] == "LIMIT" {}
        if lista[index] == "ALLOW" && lista[index + 1] == "FILTERING" {}
    }

    Ok(None)
}

pub fn select_clause(lista: &mut Vec<String>) -> Option<Where> {
    None
}

pub fn insert_statement(lista: &mut [String]) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "INSERT" {}

    Ok(None)
}

pub fn delete_statement(lista: &mut [String]) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "DELETE" {}
    Ok(None)
}

pub fn update_statement(lista: &mut [String]) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "UPDATE" {}
    Ok(None)
}

pub fn batch_statement(lista: &mut [String]) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "BATCH" {}

    Ok(None)
}
