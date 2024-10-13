use crate::{
    cassandra::errors::error::Error,
    parser::{
        column_definition::ColumnDefinition,
        data_types::{
            identifier::Identifier, keyspace_name::KeyspaceName, option::Options,
            unquoted_name::UnquotedName,
        },
        primary_key::PrimaryKey,
        table_name::TableName,
    },
};

use super::{
    alter_keyspace::AlterKeyspace, alter_table::AlterTable, alter_table::AlterTableInstruction,
    create_keyspace::CreateKeyspace, create_table::CreateTable, drop_keyspace::DropKeyspace,
    drop_table::DropTable, truncate::Truncate,
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
    if let Some(ddl_statement) = use_statement(lista.to_vec())? {
        return Ok(Some(ddl_statement));
        // return Ok(Some(DdlStatement::UseStatement(KeyspaceName)));
    } else if let Some(ddl_statement) = create_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::CreateKeyspaceStatement(ddl_statement)));
    } else if let Some(ddl_statement) = alter_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::AlterKeyspaceStatement(ddl_statement)));
    } else if let Some(ddl_statement) = drop_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::DropKeyspaceStatement(ddl_statement)));
    } else if let Some(ddl_statement) = create_table_statement(lista)? {
        return Ok(Some(DdlStatement::CreateTableStatement(ddl_statement)));
    } else if let Some(ddl_statement) = alter_table_statement(lista)? {
        return Ok(Some(DdlStatement::AlterTableStatement(ddl_statement)));
    } else if let Some(_ddl_statement) = drop_table_statement(lista)? {
        return Ok(Some(DdlStatement::DropTableStatement(DropTable {})));
    } else if let Some(_ddl_statement) = truncate_statement(lista)? {
        return Ok(Some(DdlStatement::TruncateStatement(Truncate {})));
    }
    Ok(None)
}

pub fn use_statement(lista: Vec<String>) -> Result<Option<DdlStatement>, Error> {
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

pub fn alter_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<AlterKeyspace>, Error> {
    if check_words(lista, "ALTER KEYSPACE") {
        let mut if_exists = false;
        if check_words(lista, "IF EXISTS") {
            if_exists = true;
        }
        let name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(name) => name,
            None => return Err(Error::SyntaxError("No se indicó la Keyspace".to_string())),
        };
        if !check_words(lista, "WITH") {
            return Err(Error::SyntaxError("Falta el WITH con opciones".to_string()));
        }
        let options = options(lista)?;
        return Ok(Some(AlterKeyspace::new(if_exists, name, options)));
    }
    Ok(None)
}

pub fn drop_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<DropKeyspace>, Error> {
    if check_words(lista, "DROP KEYSPACE") {
        let mut if_exists = false;
        if check_words(lista, "IF EXISTS") {
            if_exists = true;
        }

        let name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(name) => name,
            None => {
                return Err(Error::SyntaxError(
                    "No se indicó el nombre del keyspace".to_string(),
                ))
            }
        };

        return Ok(Some(DropKeyspace::new(if_exists, name)));
    }
    Ok(None)
}

pub fn create_table_statement(lista: &mut Vec<String>) -> Result<Option<CreateTable>, Error> {
    if check_words(lista, "CREATE TABLE") {
        let mut if_not_exists = false;
        if check_words(lista, "IF NOT EXISTS") {
            if_not_exists = true;
        }

        let name = match TableName::check_kind_of_name(lista)? {
            Some(name) => name,
            None => {
                return Err(Error::SyntaxError(
                    "No se indicó el nombre de la tabla".to_string(),
                ))
            }
        };

        if !check_words(lista, "(") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de apertura".to_string(),
            ));
        }

        let columns = parse_column_definitions(lista)?;
        let primary_key = parse_primary_key(lista)?;

        if !check_words(lista, ")") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de cierre".to_string(),
            ));
        }

        let options = if check_words(lista, "WITH") {
            Some(options(lista)?)
        } else {
            None
        };

        return Ok(Some(CreateTable::new(
            if_not_exists,
            name,
            columns,
            primary_key,
            options,
        )));
    }
    Ok(None)
}

fn parse_column_definitions(lista: &mut Vec<String>) -> Result<Vec<ColumnDefinition>, Error> {
    let mut columns = Vec::new();
    loop {
        let column = ColumnDefinition::parse(lista)?;
        columns.push(column);

        if !check_words(lista, ",") {
            break;
        }

        if lista.len() >= 2 && lista[0] == "PRIMARY" && lista[1] == "KEY" {
            break;
        }
    }
    Ok(columns)
}

fn parse_primary_key(lista: &mut Vec<String>) -> Result<Option<PrimaryKey>, Error> {
    if check_words(lista, "PRIMARY KEY") {
        if !check_words(lista, "(") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de apertura en PRIMARY KEY".to_string(),
            ));
        }

        let primary_key = PrimaryKey::parse(lista)?;

        if !check_words(lista, ")") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de cierre en PRIMARY KEY".to_string(),
            ));
        }

        Ok(Some(primary_key))
    } else {
        Ok(None)
    }
}

pub fn alter_table_statement(lista: &mut Vec<String>) -> Result<Option<AlterTable>, Error> {
    if check_words(lista, "ALTER TABLE") {
        let name = match TableName::check_kind_of_name(lista)? {
            Some(name) => name,
            None => {
                return Err(Error::SyntaxError(
                    "No se indicó el nombre de la tabla".to_string(),
                ))
            }
        };

        let instruction = parse_alter_table_instruction(lista)?;

        return Ok(Some(AlterTable::new(name, instruction)));
    }
    Ok(None)
}

fn parse_alter_table_instruction(lista: &mut Vec<String>) -> Result<AlterTableInstruction, Error> {
    if check_words(lista, "ADD") {
        let columns = parse_column_definitions(lista)?;
        Ok(AlterTableInstruction::AddColumns(columns))
    } else if check_words(lista, "DROP") {
        let columns = parse_column_names(lista)?;
        Ok(AlterTableInstruction::DropColumns(columns))
    } else if check_words(lista, "WITH") {
        let options = options(lista)?;
        Ok(AlterTableInstruction::WithOptions(options))
    } else {
        Err(Error::SyntaxError(
            "Instrucción ALTER TABLE no válida".to_string(),
        ))
    }
}

fn parse_column_names(lista: &mut Vec<String>) -> Result<Vec<String>, Error> {
    let mut columns = Vec::new();
    loop {
        if lista.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba un nombre de columna".to_string(),
            ));
        }
        columns.push(lista.remove(0));

        if !check_words(lista, ",") {
            break;
        }
    }
    Ok(columns)
}

pub fn drop_table_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}

pub fn truncate_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    Ok(None)
}
