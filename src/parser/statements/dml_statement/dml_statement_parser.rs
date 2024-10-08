use std::ops::Index;

use crate::{cassandra::errors::error::Error, parser::select::Select, parser::r#where::Where};

pub enum DmlStatement {
    SelectStatement,
    InsertStatement,
    UpdateStatement,
    DeleteStatement,
    BatchStatement,
}

pub fn dml_statement(_lista: &mut Vec<String>, _index: usize) -> Result<Option<DmlStatement>, Error> {
    if let Some(_x) = select_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::SelectStatement));
    } else if let Some(_x) = insert_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::InsertStatement));
    } else if let Some(_x) = delete_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::UpdateStatement));
    } else if let Some(_x) = update_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::DeleteStatement));
    } else if let Some(_x) = batch_statement(_lista, _index)? {
        return Ok(Some(DmlStatement::BatchStatement));
    }
    Ok(None)
}

pub fn select_statement(lista: &mut Vec<String>, mut index: usize) -> Result<Option<DmlStatement>, Error> {
    if lista[index] == "SELECT"{
        index += 1;
        // Podria hacer un builder que cree los campos poco a poco?
        if lista[index] == "*"{
            // Select{

            // };
        } else {
            return Err(Error::SyntaxError("Falta la tabla".to_string()))
        }
        if lista[index] == "WHERE"{
            let res = select_clause(lista, index);
            // aca completar el builder poco a poco
        } else {
            return Ok(None) // al builder pasarle esto
        }
        if lista[index] == "GROUP" && lista[index + 1] == "BY"{

        } else {
            return Ok(None)
        }
        if lista[index] == "ORDER" && lista[index + 1] == "BY"{
            
        }
        if lista[index] == "PER" && lista[index + 1] == "PARTITION" && lista[index + 2] == "LIMIT"{
            
        }
        if lista[index] == "LIMIT"{
            
        }
        if lista[index] == "ALLOW" && lista[index + 1] == "FILTERING"{
            
        }
    }


    Ok(None)
}

pub fn select_clause(lista: &mut Vec<String>, mut index: usize) -> Option<Where>{


    None

}




pub fn insert_statement(
    lista: &mut [String],
    _index: usize,
) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "INSERT"{
        
    }

    Ok(None)
}

pub fn delete_statement(
    lista: &mut [String],
    _index: usize,
) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "DELETE"{
        
    }
    Ok(None)
}

pub fn update_statement(
    lista: &mut [String],
    _index: usize,
) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "UPDATE"{
        
    }
    Ok(None)
}

pub fn batch_statement(
    lista: &mut [String],
    _index: usize,
) -> Result<Option<DmlStatement>, Error> {
    if lista[0] == "BATCH"{
        
    }




    Ok(None)
}