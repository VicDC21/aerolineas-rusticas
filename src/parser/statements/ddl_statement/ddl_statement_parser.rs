use crate::{
    cassandra::errors::error::Error,
    parser::data_types::{
        identifier::Identifier, keyspace_name::KeyspaceName, option::Options,
        unquoted_name::UnquotedName,
    },
};

use super::{
    alter_keyspace::AlterKeyspace, alter_table::AlterTable, create_keyspace::CreateKeyspace,
    create_table::CreateTable, drop_keyspace::DropKeyspace, drop_table::DropTable,
    truncate::Truncate,
};

pub enum DdlStatement {
    UseStatement(KeyspaceName),
    CreateKeyspaceStatement(CreateKeyspace),
    AlterKeyspaceStatement(AlterKeyspace),
    DropKeyspaceStatement(DropKeyspace),
    CreateTableStatement(CreateTable),
    AlterTableStatement(AlterTable),
    DropTableStatement(DropTable),
    TruncateStatement(Truncate),
}

pub fn ddl_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    if let Some(_x) = use_statement(lista)? {
        return Ok(Some(_x));
        // return Ok(Some(DdlStatement::UseStatement(KeyspaceName)));
    } else if let Some(_x) = create_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::CreateKeyspaceStatement(_x)));
    } else if let Some(_x) = alter_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::AlterKeyspaceStatement(AlterKeyspace {})));
    } else if let Some(_x) = drop_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::DropKeyspaceStatement(DropKeyspace {})));
    } else if let Some(_x) = create_table_statement(lista)? {
        return Ok(Some(DdlStatement::CreateTableStatement(CreateTable {})));
    } else if let Some(_x) = alter_table_statement(lista)? {
        return Ok(Some(DdlStatement::AlterTableStatement(AlterTable {})));
    } else if let Some(_x) = drop_table_statement(lista)? {
        return Ok(Some(DdlStatement::DropTableStatement(DropTable {})));
    } else if let Some(_x) = truncate_statement(lista)? {
        return Ok(Some(DdlStatement::TruncateStatement(Truncate {})));
    }
    Ok(None)
}

pub fn use_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    if lista[0] == "USE" {
        if lista[1] == "\"" {
            let keyspace = DdlStatement::UseStatement(KeyspaceName::QuotedName(UnquotedName::new(
                lista[2].clone(),
            )?));
            return Ok(Some(keyspace));
        } else {
            let keyspace = DdlStatement::UseStatement(KeyspaceName::UnquotedName(
                UnquotedName::new(lista[1].clone())?,
            ));
            return Ok(Some(keyspace));
        }
    }
    Ok(None)
}

pub fn create_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<CreateKeyspace>, Error> {
    if check_words(lista, "CREATE KEYSPACE") {
        let mut if_not_exists = false;
        if check_words(lista, "IF NOT EXISTS") {
            if_not_exists = true;
        }
        let name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(name) => name,
            None => return Err(Error::SyntaxError("No se indico la Keyspace".to_string())),
        };
        if !check_words(lista, "WITH") {
            return Err(Error::SyntaxError("Falta el WITH con opciones".to_string()));
        }
        let options = options(lista)?;
        return Ok(Some(CreateKeyspace::new(if_not_exists, name, options)));
    }
    Ok(None)
}

pub fn options(lista: &mut Vec<String>) -> Result<Vec<Options>, Error> {
    let mut options: Vec<Options> = Vec::new();
    match is_an_option(lista)? {
        Some(value) => options.push(value),
        None => return Err(Error::SyntaxError("".to_string())),
    };
    while lista[0] == "AND" {
        match is_an_option(lista)? {
            Some(value) => options.push(value),
            None => return Err(Error::SyntaxError("".to_string())),
        };
    }
    Ok(options)
}

pub fn is_an_option(lista: &mut Vec<String>) -> Result<Option<Options>, Error> {
    let value = match Identifier::check_identifier(lista)? {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "Debe haber un valor luego del WITH".to_string(),
            ))
        }
    };
    let option_word: &str = value.get_name();
    if option_word != "replication" || option_word != "durable_writes" {
        return Err(Error::SyntaxError("OPTION no permitida".to_string()));
    }
    if !check_words(lista, "=") {
        return Err(Error::SyntaxError(
            "Falto el '=' de las opciones".to_string(),
        ));
    }
    let options = Options::check_options(lista)?;
    Ok(Some(options))
}

pub fn check_words(lista: &mut Vec<String>, palabra: &str) -> bool {
    let palabras: Vec<&str> = palabra.split_whitespace().collect();
    if palabras.len() > lista.len() {
        return false;
    };

    for (index, &word) in palabras.iter().enumerate() {
        if lista[index] != word {
            return false;
        }
    }
    lista.drain(..palabras.len());
    true
}

pub fn alter_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn drop_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn create_table_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn alter_table_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn drop_table_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn truncate_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}
