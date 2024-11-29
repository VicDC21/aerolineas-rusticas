use crate::protocol::errors::error::Error;

use super::ddl_statement::ddl_statement_parser::check_words;


/// TODO
pub fn startup_statement(lista: &mut Vec<String>) -> Result<Option<()>, Error>{
    if check_words(lista, "STARTUP"){
        return Ok(Some(()))
    }
    Ok(None)
}