use crate::{
    cassandra::errors::error::Error,
    parser::{
        data_types::term::Term, statements::ddl_statement::ddl_statement_parser::check_words,
    },
};

pub struct ListLiteral {
    values: Vec<Term>,
}

impl ListLiteral {
    pub fn check_list_literal(lista: &mut Vec<String>) -> Result<Option<Self>, Error> {
        let mut values: Vec<Term> = Vec::new();
        if check_words(lista, "[") {
            while check_words(lista, ",") || !check_words(lista, "]") {
                let term = match Term::is_term(lista)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::SyntaxError(
                            "Sintaxis de lista incorrecta".to_string(),
                        ))
                    }
                };
                values.push(term);
            }
        }
        Ok(Some(ListLiteral { values }))
    }
}
