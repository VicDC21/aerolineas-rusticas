use crate::{
    parser::statements::ddl_statement::ddl_statement_parser::check_words,
    protocol::aliases::results::Result,
};

/// Verifica si la lista dada es una sentencia de STARTUP. Si lo es, retorna Some(()), si no, retorna None.
pub fn startup_statement(lista: &mut Vec<String>) -> Result<Option<()>> {
    if check_words(lista, "STARTUP") {
        return Ok(Some(()));
    }
    Ok(None)
}
