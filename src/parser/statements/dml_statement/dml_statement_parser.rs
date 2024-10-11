use crate::{cassandra::errors::error::Error, parser::{data_types::{constant::Constant, cql_type::CQLType, identifier::Identifier, native_types::NativeType, quoted_identifier::QuotedIdentifier, term::Term, unquoted_identifier::UnquotedIdentifier}, group_by::GroupBy, order_by::OrderBy, select::Select, selector::Selector, r#where::Where }};

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
            let res = select_clause(lista)?;
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

pub fn select_clause(lista: &mut Vec<String>) -> Result<Option<Vec<Selector>>, Error> {
    if lista[0] != "FROM"{
        let mut vec: Vec<Selector> = Vec::new();
        if let Some(sel) = selector(lista)?{
            vec.push(sel);
        }
        if lista[0] == ","{
            lista.remove(0);
            if let Some(mut clasules) = select_clause(lista)?{
                vec.append(&mut clasules);
            };
        }
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

pub fn selector(lista: &mut Vec<String>) -> Result<Option<Selector>, Error> {

    if let Some(column) = is_column_name(lista)?{
        return Ok(Some(Selector::ColumnName(column)));
    }
    if let Some(term) = is_term(lista)?{
        return Ok(Some(Selector::Term(term)));
    }
    // if let Some(cast) = is_cast(lista)?{
    //     return Ok(Some(cast));
    // }

    Ok(None)
}

// identifier
pub fn is_column_name(lista: &mut Vec<String>) -> Result<Option<Identifier>, Error>{
    if QuotedIdentifier::check_quoted_identifier(&lista[0], &lista[1], &lista[2]){
        lista.remove(0);
        let string = lista.remove(0);
        lista.remove(0);
        return Ok(Some(Identifier::QuotedIdentifier(QuotedIdentifier::new(string))));
    } else if UnquotedIdentifier::check_unquoted_identifier(&lista[0]){
        let string = lista.remove(0);
        return Ok(Some(Identifier::UnquotedIdentifier(UnquotedIdentifier::new(string))));
    }
    Ok(None)
}


pub fn is_term(lista: &mut Vec<String>) -> Result<Option<Term>, Error>{
    if Constant::check_string(&lista[0], &lista[2]){
        lista.remove(0);
        let string = Constant::String(lista.remove(0));
        lista.remove(0);
        return Ok(Some(Term::Constant(string)));
    } else if Constant::check_integer(&lista[0]){
        let integer_string: String = lista.remove(0);
        let int = Constant::new_integer(integer_string)?;
        return Ok(Some(Term::Constant(int)));
    } else if Constant::check_float(&lista[0]){
        let float_string = lista.remove(0);
        let float = Constant::new_float(float_string)?;
        return Ok(Some(Term::Constant(float)));
    } else if Constant::check_boolean(&lista[0]){
        let bool = lista.remove(0);
        let bool = Constant::new_boolean(bool)?;
        return Ok(Some(Term::Constant(bool)))
    } else if Constant::check_uuid(&lista[0]){
        let uuid = lista.remove(0);
        let uuid = Constant::new_uuid(uuid)?;
        return Ok(Some(Term::Constant(uuid)))
    } else if Constant::check_hex(&lista[0]){
        let hex = Constant::new_hex(lista.remove(0))?;
        return Ok(Some(Term::Constant(hex)))
    } else if Constant::check_blob(&lista[0]){
        let blob = Constant::new_blob(lista.remove(0))?;
        return Ok(Some(Term::Constant(blob)))
    }

    Ok(None)
}

// pub fn is_cast(lista: &mut Vec<String>) -> Result<Option<Term>, Error>{


//     Ok(None)
// }

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

    if lista[index] == "BEGIN" {
        lista.remove(index);
        if lista[index] == "UNLOGGED" {
            // Lógica para el Unlogged Batch -> Aplicación parcial del batch
            lista.remove(index);
            lista.remove(index);
        } else if lista[index] == "COUNTER" {
            // Lógica para el Counter Batch -> Aplicación para contadores
            lista.remove(index);
            lista.remove(index);
        } else {
            // Lógica para el Logged Batch -> Aplicación total del batch
            lista.remove(index);
        }       
    }
    
    let mut query = None;    
    if lista[index] == "INSERT" {
        query = insert_statement(lista)?;
    } else if lista[index] == "UPDATE" {
        query = update_statement(lista)?;
    } else if lista[index] == "DELETE" {
        query = delete_statement(lista)?;
    }
    
    Ok(query)
}