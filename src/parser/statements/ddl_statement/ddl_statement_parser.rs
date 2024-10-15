use crate::{
    cassandra::errors::error::Error,
    parser::{
        data_types::{identifier::identifier::Identifier, keyspace_name::KeyspaceName},
        primary_key::PrimaryKey,
        table_name::TableName,
    },
};

use super::{
    alter_keyspace::AlterKeyspace,
    alter_table::{AlterTable, AlterTableInstruction},
    column_definition::ColumnDefinition,
    create_keyspace::CreateKeyspace,
    create_table::CreateTable,
    drop_keyspace::DropKeyspace,
    drop_table::DropTable,
    option::Options,
    truncate::Truncate,
};

/// ddl_statement::= use_statement
///         | create_keyspace_statement
///         | alter_keyspace_statement
///         | drop_keyspace_statement
///         | create_table_statement
///         | alter_table_statement
///         | drop_table_statement
///         | truncate_statement
pub enum DdlStatement {
    /// use_statement::= USE keyspace_name
    UseStatement(KeyspaceName),

    /// create_keyspace_statement::= CREATE KEYSPACE [ IF NOT EXISTS ] keyspace_name
    /// WITH options
    CreateKeyspaceStatement(CreateKeyspace),

    /// alter_keyspace_statement::= ALTER KEYSPACE [ IF EXISTS ] keyspace_name
    /// WITH options
    AlterKeyspaceStatement(AlterKeyspace),

    /// drop_keyspace_statement::= DROP KEYSPACE [ IF EXISTS ] keyspace_name
    DropKeyspaceStatement(DropKeyspace),

    /// create_table_statement::= CREATE TABLE [ IF NOT EXISTS ] table_name '('
    /// column_definition  ( ',' column_definition )*
    /// [ ',' PRIMARY KEY '(' primary_key ')' ]
    ///  ')' [ WITH table_options ]
    CreateTableStatement(CreateTable),

    /// alter_table_statement::= ALTER TABLE [ IF EXISTS ] table_name alter_table_instruction
    AlterTableStatement(AlterTable),

    /// drop_table_statement::= DROP TABLE [ IF EXISTS ] table_name
    DropTableStatement(DropTable),

    /// truncate_statement::= TRUNCATE [ TABLE ] table_name
    TruncateStatement(Truncate),
}

/// Crea el enum `DdlStatement` con el tipo de struct de acuerdo a la sintaxis dada, si la entrada proporcionada no satisface
/// los requerimientos de los tipos de datos, entonces devuelve None.
pub fn ddl_statement(lista: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    if let Some(parsed_value) = use_statement(lista)? {
        return Ok(Some(DdlStatement::UseStatement(parsed_value)));
    } else if let Some(parsed_value) = create_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::CreateKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = alter_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::AlterKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = drop_keyspace_statement(lista)? {
        return Ok(Some(DdlStatement::DropKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = create_table_statement(lista)? {
        return Ok(Some(DdlStatement::CreateTableStatement(parsed_value)));
    } else if let Some(parsed_value) = alter_table_statement(lista)? {
        return Ok(Some(DdlStatement::AlterTableStatement(parsed_value)));
    } else if let Some(parsed_value) = drop_table_statement(lista)? {
        return Ok(Some(DdlStatement::DropTableStatement(parsed_value)));
    } else if let Some(parsed_value) = truncate_statement(lista)? {
        return Ok(Some(DdlStatement::TruncateStatement(parsed_value)));
    }
    Ok(None)
}

fn use_statement(lista: &mut Vec<String>) -> Result<Option<KeyspaceName>, Error> {
    if check_words(lista, "USE") {
        let keyspace = match KeyspaceName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "Se esperaba el nombre de una keyspace valida".to_string(),
                ))
            }
        };
        return Ok(Some(keyspace));
    }
    Ok(None)
}

fn create_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<CreateKeyspace>, Error> {
    if check_words(lista, "CREATE KEYSPACE") {
        let if_not_exists = check_words(lista, "IF NOT EXISTS");
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

fn alter_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<AlterKeyspace>, Error> {
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

fn drop_keyspace_statement(lista: &mut Vec<String>) -> Result<Option<DropKeyspace>, Error> {
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

fn create_table_statement(lista: &mut Vec<String>) -> Result<Option<CreateTable>, Error> {
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

        if !check_words(lista, "WITH") {
            return Err(Error::SyntaxError("Falta la condición WITH".to_string()));
        }
        let options = parse_table_options(lista)?;
        let mut compact_storage = false;
        let mut clustering_order = None;

        for option in options {
            if let Options::Identifier(id) = option {
                match id.get_name() {
                    "COMPACT STORAGE" => compact_storage = true,
                    "CLUSTERING ORDER" => {
                        clustering_order = Some(parse_clustering_order(&lista.join(" "))?);
                    }
                    _ => {}
                }
            }
        }

        return Ok(Some(CreateTable::new(
            if_not_exists,
            name,
            columns,
            primary_key,
            compact_storage,
            clustering_order,
        )));
    }
    Ok(None)
}

fn alter_table_statement(lista: &mut Vec<String>) -> Result<Option<AlterTable>, Error> {
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

fn drop_table_statement(lista: &mut Vec<String>) -> Result<Option<DropTable>, Error> {
    if check_words(lista, "DROP TABLE") {
        let exist = check_words(lista, "IF EXISTS");
        let table_name = match TableName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "Falta el nombre de la tabla".to_string(),
                ))
            }
        };
        return Ok(Some(DropTable::new(exist, table_name)));
    }
    Ok(None)
}

fn truncate_statement(lista: &mut Vec<String>) -> Result<Option<Truncate>, Error> {
    if check_words(lista, "TRUNCATE") {
        check_words(lista, "TABLE");
        let table_name = match TableName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "Falta el nombre de la tabla".to_string(),
                ))
            }
        };
        return Ok(Some(Truncate::new(table_name)));
    }
    Ok(None)
}

fn options(lista: &mut Vec<String>) -> Result<Vec<Options>, Error> {
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

fn is_an_option(lista: &mut Vec<String>) -> Result<Option<Options>, Error> {
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

fn parse_table_options(lista: &mut Vec<String>) -> Result<Vec<Options>, Error> {
    let mut options = Vec::new();
    loop {
        let option = Options::check_options(lista)?;
        options.push(option);
        if !check_words(lista, "AND") {
            break;
        }
    }
    Ok(options)
}

fn parse_clustering_order(order: &str) -> Result<Vec<(String, String)>, Error> {
    let order = order.trim_matches(|c| c == '(' || c == ')');
    let parts: Vec<&str> = order.split(',').collect();
    let mut result = Vec::new();
    for part in parts {
        let mut column_order = part.split_whitespace();
        let column = column_order.next().ok_or_else(|| {
            Error::SyntaxError("Expected column name in clustering order".to_string())
        })?;
        let order = column_order.next().ok_or_else(|| {
            Error::SyntaxError("Expected order (ASC or DESC) in clustering order".to_string())
        })?;
        result.push((column.to_string(), order.to_string()));
    }
    Ok(result)
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

fn parse_alter_table_instruction(lista: &mut Vec<String>) -> Result<AlterTableInstruction, Error> {
    if check_words(lista, "ADD") {
        let if_not_exists = check_words(lista, "IF NOT EXISTS");
        let columns = parse_column_definitions(lista)?;
        Ok(AlterTableInstruction::AddColumns(if_not_exists, columns))
    } else if check_words(lista, "DROP") {
        let if_exists = check_words(lista, "IF EXISTS");
        let columns = parse_column_names(lista)?;
        Ok(AlterTableInstruction::DropColumns(if_exists, columns))
    } else if check_words(lista, "WITH") {
        let if_exists = check_words(lista, "IF EXISTS");
        let options = options(lista)?;
        Ok(AlterTableInstruction::WithOptions(if_exists, options))
    } else if check_words(lista, "RENAME") {
        let if_exists = check_words(lista, "IF EXISTS");
        let renames = parse_column_renames(lista)?;
        Ok(AlterTableInstruction::RenameColumns(if_exists, renames))
    } else {
        Err(Error::SyntaxError(
            "Instrucción ALTER TABLE no válida".to_string(),
        ))
    }
}

fn parse_column_renames(lista: &mut Vec<String>) -> Result<Vec<(String, String)>, Error> {
    let mut renames = Vec::new();

    loop {
        let old_name = if lista.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba el nombre antiguo de la columna".to_string(),
            ));
        } else {
            lista.remove(0)
        };

        if !check_words(lista, "TO") {
            return Err(Error::SyntaxError(
                "Se esperaba la palabra clave 'TO'".to_string(),
            ));
        }

        if lista.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba el nuevo nombre de la columna".to_string(),
            ));
        }

        let new_name = lista.remove(0);

        renames.push((old_name, new_name));

        if !check_words(lista, "AND") {
            break;
        }
    }

    Ok(renames)
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
