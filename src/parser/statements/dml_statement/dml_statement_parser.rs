use crate::{cassandra::errors::error::Error, parser::{group_by::GroupBy, order_by::OrderBy, select::Select, r#where::Where }};

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
        lista.remove(0);
        // Podria hacer un builder que cree los campos poco a poco?
        if lista[index] == "*" {
            // Select{

            // };
        } else {
            let res = select_clause(lista);
        }
        if lista[index] != "FROM"{
            return Err(Error::SyntaxError("Falta el from en la consulta".to_string()))
        }



        if lista[index] == "WHERE" {
            let res = where_clause(lista);
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

pub fn select_clause(lista: &mut Vec<String>) -> Option<Vec<String>> {
    if lista[0] != "FROM"{
        let mut vec: Vec<String> = Vec::new();
        if let Some(mut sel) = selector(lista){
            vec.push(sel);
        }
        if lista[0] == ","{
            lista.remove(0);
            if let Some(mut clasules) = select_clause(lista){
                vec.append(&mut clasules);
            };
        }
        Some(vec)
    } else {
        None
    }
}

pub fn selector(lista: &mut Vec<String>) -> Option<String> {
    if lista[0] == "column_name"{

    } else if lista[0] == "term"{

    } else if lista[0] == "CAST" && lista[1] == "("{
        selector(lista);
        if lista[0] != "AS"{
            // Error
        }
        cql_type(lista);



    }


    None
}

pub fn cql_type(lista: &mut Vec<String>){

}


pub fn where_clause(lista: &mut Vec<String>) -> Option<Where> {
    None
}

pub fn relation(lista: &mut Vec<String>) -> Option<Where> {
    None
}

pub fn operator(lista: &mut Vec<String>) -> Option<Where> {
    None
}

pub fn group_by_clause(lista: &mut Vec<String>) -> Option<GroupBy> {
    None
}

pub fn ordering_clause(lista: &mut Vec<String>) -> Option<OrderBy> {
    None
}

pub fn insert_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    let index = 0;
    if lista[index] == "INSERT" && lista[index + 1] == "INTO" {
        lista.remove(index);
        lista.remove(index);

        if lista[index] == "file_name" {
            lista.remove(index);
            // Chequeo si es un archivo válido
            if lista[index] == "JSON" {
                // Chequeo si la sintaxis JSON es válida
            } else {
                // Chequeo si la sintaxis de las columnas es válida (o crear si no existe alguna)
            }
        } else {
            return Ok(None);
        }

        if lista[index] == "IF" {
            lista.remove(index);
            // Chequeo de la sintaxis de IF NOT EXISTS
        } 

        if lista[index] == "VALUES" {
            lista.remove(index);
            // Chequeo/match de valores con columnas 
        } else {
            return Ok(None);
        }

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        } else {
            return Ok(None);
        }

    }
    Ok(None)
}

pub fn delete_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    let index: usize = 0;
    if lista[index] == "DELETE" {
        lista.remove(index);

        if lista[index] == "col_name" {
            lista.remove(index);
            // Chequeo de columnas específicas
        } 

        if lista[index] == "FROM" {
            lista.remove(index);
            if lista[index] == "file_name" {
                lista.remove(index);
                // Chequeo si es un archivo válido
            } else {
                return Ok(None);
            }        
        } else {
            return Ok(None);
        }

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        } else {
            return Ok(None);
        }

        if lista[index] == "WHERE" {
            lista.remove(index);
            let res = where_clause(lista);
            if lista[index] == "IF" {
                lista.remove(index);
                // Chequeo sintaxis de condicionales para la query
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None); 
        }
    }
    Ok(None)
}

pub fn update_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    let index = 0;
    if lista[0] == "UPDATE" {
        lista.remove(index);
        if lista[index] == "file_name" {
            lista.remove(index);
            // Chequeo si es un archivo válido
        } else {
            return Ok(None);
        }

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        } else {
            return Ok(None);
        }

        if lista[index] == "SET" {
            lista.remove(index);
            // Chequeo de la sintaxis de SET
        } else {
            return Ok(None);
        }

        if lista[index] == "WHERE" {
            lista.remove(index);
            let res = where_clause(lista);
            if lista[index] == "IF" {
                lista.remove(index);
                // Chequeo sintaxis de condicionales para la query
            } else {
                return Ok(None);
            }
        } else {
            return Ok(None); 
        }
    }

    Ok(None)
}

pub fn batch_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    let index = 0;
    if lista[0] == "BEGIN" {
        lista.remove(index);
        if lista[index] == "UNLOGGED" {
            lista.remove(index);
            // Lógica para el Unlogged Batch -> Aplicación parcial del batch
        } else if lista[index] == "COUNTER" {
            lista.remove(index);
            // Lógica para el Counter Batch -> Aplicación parcial del batch
        } else {
            lista.remove(index);
            // Lógica para el Logged Batch -> Por defecto aplicación total o no aplicación
            if lista[index] == "INSERT" {
                insert_statement(lista)?;
            } else if lista[index] == "UPDATE" {
                update_statement(lista)?;
            } else if lista[index] == "DELETE" {
                delete_statement(lista)?;
            } else if lista[index] == "SELECT" {
                select_statement(lista)?;
            } else {
                return Ok(None);   
            }
        }
    }

    Ok(None)
}
