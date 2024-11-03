use crate::{
    parser::{
        data_types::{identifier::identifier::Identifier, keyspace_name::KeyspaceName},
        primary_key::PrimaryKey,
        table_name::TableName,
    },
    protocol::errors::error::Error,
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

#[derive(Debug)]
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
pub fn ddl_statement(list: &mut Vec<String>) -> Result<Option<DdlStatement>, Error> {
    if let Some(parsed_value) = use_statement(list)? {
        return Ok(Some(DdlStatement::UseStatement(parsed_value)));
    } else if let Some(parsed_value) = create_keyspace_statement(list)? {
        return Ok(Some(DdlStatement::CreateKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = alter_keyspace_statement(list)? {
        return Ok(Some(DdlStatement::AlterKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = drop_keyspace_statement(list)? {
        return Ok(Some(DdlStatement::DropKeyspaceStatement(parsed_value)));
    } else if let Some(parsed_value) = create_table_statement(list)? {
        return Ok(Some(DdlStatement::CreateTableStatement(parsed_value)));
    } else if let Some(parsed_value) = alter_table_statement(list)? {
        return Ok(Some(DdlStatement::AlterTableStatement(parsed_value)));
    } else if let Some(parsed_value) = drop_table_statement(list)? {
        return Ok(Some(DdlStatement::DropTableStatement(parsed_value)));
    } else if let Some(parsed_value) = truncate_statement(list)? {
        return Ok(Some(DdlStatement::TruncateStatement(parsed_value)));
    }
    Ok(None)
}

fn use_statement(list: &mut Vec<String>) -> Result<Option<KeyspaceName>, Error> {
    if check_words(list, "USE") || check_words(list, "use") {
        let keyspace = match KeyspaceName::check_kind_of_name(list)? {
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

fn create_keyspace_statement(list: &mut Vec<String>) -> Result<Option<CreateKeyspace>, Error> {
    if check_words(list, "CREATE KEYSPACE") {
        let if_not_exists = check_words(list, "IF NOT EXISTS");
        let name = match KeyspaceName::check_kind_of_name(list)? {
            Some(name) => name,
            None => return Err(Error::SyntaxError("No se indico la Keyspace".to_string())),
        };
        if !check_words(list, "WITH") {
            return Err(Error::SyntaxError("Falta el WITH con opciones".to_string()));
        }
        let options = options(list)?;
        return Ok(Some(CreateKeyspace::new(if_not_exists, name, options)));
    }
    Ok(None)
}

fn alter_keyspace_statement(list: &mut Vec<String>) -> Result<Option<AlterKeyspace>, Error> {
    if check_words(list, "ALTER KEYSPACE") {
        let mut if_exists = false;
        if check_words(list, "IF EXISTS") {
            if_exists = true;
        }
        let name = match KeyspaceName::check_kind_of_name(list)? {
            Some(name) => name,
            None => return Err(Error::SyntaxError("No se indicó la Keyspace".to_string())),
        };
        if !check_words(list, "WITH") {
            return Err(Error::SyntaxError("Falta el WITH con opciones".to_string()));
        }
        let options = options(list)?;
        return Ok(Some(AlterKeyspace::new(if_exists, name, options)));
    }
    Ok(None)
}

fn drop_keyspace_statement(list: &mut Vec<String>) -> Result<Option<DropKeyspace>, Error> {
    if check_words(list, "DROP KEYSPACE") {
        let mut if_exists = false;
        if check_words(list, "IF EXISTS") {
            if_exists = true;
        }

        let name = match KeyspaceName::check_kind_of_name(list)? {
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

fn create_table_statement(list: &mut Vec<String>) -> Result<Option<CreateTable>, Error> {
    if check_words(list, "CREATE TABLE") {
        let mut if_not_exists = false;
        if check_words(list, "IF NOT EXISTS") {
            if_not_exists = true;
        }

        let name = match TableName::check_kind_of_name(list)? {
            Some(name) => name,
            None => {
                return Err(Error::SyntaxError(
                    "No se indicó el nombre de la tabla".to_string(),
                ))
            }
        };

        if !check_words(list, "(") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de apertura".to_string(),
            ));
        }

        let columns = parse_column_definitions(list)?;
        let primary_key = parse_primary_key(list)?;

        if !check_words(list, ")") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de cierre".to_string(),
            ));
        }

        let mut compact_storage = false;
        let mut clustering_order = None;

        if check_words(list, "WITH") {
            let options = parse_table_options(list)?;
            for option in options {
                if let Options::Constant(term) = option {
                    match term.get_value().as_str() {
                        "COMPACT STORAGE" => compact_storage = true,
                        "CLUSTERING ORDER BY" => {
                            clustering_order = Some(parse_clustering_order(&list.join(" "))?);
                        }
                        _ => {}
                    }
                } else if let Options::Identifier(term) = option {
                    match term.get_name() {
                        "COMPACT STORAGE" => compact_storage = true,
                        "CLUSTERING ORDER BY" => {
                            clustering_order = Some(parse_clustering_order(&list.join(" "))?);
                        }
                        _ => {}
                    }
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

fn alter_table_statement(list: &mut Vec<String>) -> Result<Option<AlterTable>, Error> {
    if check_words(list, "ALTER TABLE") {
        let name = match TableName::check_kind_of_name(list)? {
            Some(name) => name,
            None => {
                return Err(Error::SyntaxError(
                    "No se indicó el nombre de la tabla".to_string(),
                ))
            }
        };

        let instruction = parse_alter_table_instruction(list)?;
        return Ok(Some(AlterTable::new(name, instruction)));
    }
    Ok(None)
}

fn drop_table_statement(list: &mut Vec<String>) -> Result<Option<DropTable>, Error> {
    if check_words(list, "DROP TABLE") {
        let table_name = match TableName::check_kind_of_name(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "Falta el nombre de la tabla".to_string(),
                ))
            }
        };
        return Ok(Some(DropTable::new(table_name)));
    }
    Ok(None)
}

fn truncate_statement(list: &mut Vec<String>) -> Result<Option<Truncate>, Error> {
    if check_words(list, "TRUNCATE") {
        check_words(list, "TABLE");
        let table_name = match TableName::check_kind_of_name(list)? {
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

fn options(list: &mut Vec<String>) -> Result<Vec<Options>, Error> {
    let mut options: Vec<Options> = Vec::new();
    match is_an_option(list)? {
        Some(value) => options.push(value),
        None => return Err(Error::SyntaxError("".to_string())),
    };

    if list.is_empty() {
        return Ok(options);
    }

    while check_words(list, "AND") {
        match is_an_option(list)? {
            Some(value) => options.push(value),
            None => return Err(Error::SyntaxError("".to_string())),
        };
    }
    Ok(options)
}

fn is_an_option(list: &mut Vec<String>) -> Result<Option<Options>, Error> {
    let value = match Identifier::check_identifier(list)? {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "Debe haber un valor luego del WITH".to_string(),
            ))
        }
    };
    let option_word: &str = value.get_name();
    if option_word != "replication" && option_word != "durable_writes" {
        return Err(Error::SyntaxError("OPTION no permitida".to_string()));
    }
    if !check_words(list, "=") {
        return Err(Error::SyntaxError(
            "Falto el '=' de las opciones".to_string(),
        ));
    }
    let options = Options::check_options(list)?;
    Ok(Some(options))
}

/// Verifica si las siguientes palabras en la lista coinciden con la cadena dada y las elimina si es así.
///
/// # Argumentos
///
/// * `list` - Una referencia mutable a un vector de cadenas que representa la lista de palabras.
/// * `palabra` - Una porción de cadena que representa las palabras a verificar.
///
/// # Retornos
///
/// * `true` si las palabras coinciden y fueron eliminadas, `false` de lo contrario.
pub fn check_words(list: &mut Vec<String>, palabra: &str) -> bool {
    let palabras: Vec<&str> = palabra.split_whitespace().collect();
    if palabras.len() > list.len() {
        return false;
    };

    for (index, &word) in palabras.iter().enumerate() {
        if list[index] != word {
            return false;
        }
    }
    list.drain(..palabras.len());
    true
}

fn parse_table_options(list: &mut Vec<String>) -> Result<Vec<Options>, Error> {
    let mut options = Vec::new();
    loop {
        let option = Options::check_options(list)?;
        options.push(option);
        if !check_words(list, "AND") {
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

fn parse_column_definitions(list: &mut Vec<String>) -> Result<Vec<ColumnDefinition>, Error> {
    let mut columns = Vec::new();
    loop {
        check_words(list, "(");
        let column = ColumnDefinition::parse(list)?;
        columns.push(column);
        if !check_words(list, ",") {
            break;
        }
        if list.len() >= 2 && list[0] == "PRIMARY" && list[1] == "KEY" {
            break;
        }
    }
    Ok(columns)
}

fn parse_primary_key(list: &mut Vec<String>) -> Result<Option<PrimaryKey>, Error> {
    if check_words(list, "PRIMARY KEY") {
        if !check_words(list, "(") {
            return Err(Error::SyntaxError(
                "Falta el paréntesis de apertura en PRIMARY KEY".to_string(),
            ));
        }
        let partition_key_and_clustering_columns = PrimaryKey::parse(list)?;
        Ok(Some(partition_key_and_clustering_columns))
    } else {
        Ok(None)
    }
}

fn parse_alter_table_instruction(list: &mut Vec<String>) -> Result<AlterTableInstruction, Error> {
    if check_words(list, "ADD") {
        let if_not_exists = check_words(list, "IF NOT EXISTS");
        let columns = parse_column_definitions(list)?;
        Ok(AlterTableInstruction::AddColumns(if_not_exists, columns))
    } else if check_words(list, "DROP") {
        let if_exists = check_words(list, "IF EXISTS");
        let columns = parse_column_names(list)?;
        Ok(AlterTableInstruction::DropColumns(if_exists, columns))
    } else if check_words(list, "WITH") {
        let if_exists = check_words(list, "IF EXISTS");
        let options = options(list)?;
        Ok(AlterTableInstruction::WithOptions(if_exists, options))
    } else if check_words(list, "RENAME") {
        let if_exists = check_words(list, "IF EXISTS");
        let renames = parse_column_renames(list)?;
        Ok(AlterTableInstruction::RenameColumns(if_exists, renames))
    } else {
        Err(Error::SyntaxError(
            "Instrucción ALTER TABLE no válida".to_string(),
        ))
    }
}

fn parse_column_renames(list: &mut Vec<String>) -> Result<Vec<(String, String)>, Error> {
    let mut renames = Vec::new();

    loop {
        let old_name = if list.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba el nombre antiguo de la columna".to_string(),
            ));
        } else {
            list.remove(0)
        };

        if !check_words(list, "TO") {
            return Err(Error::SyntaxError(
                "Se esperaba la palabra clave 'TO'".to_string(),
            ));
        }

        if list.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba el nuevo nombre de la columna".to_string(),
            ));
        }

        let new_name = list.remove(0);

        renames.push((old_name, new_name));

        if !check_words(list, "AND") {
            break;
        }
    }

    Ok(renames)
}

fn parse_column_names(list: &mut Vec<String>) -> Result<Vec<String>, Error> {
    let mut columns = Vec::new();
    loop {
        if list.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba un nombre de columna".to_string(),
            ));
        }
        columns.push(list.remove(0));

        if !check_words(list, ",") {
            break;
        }
    }
    Ok(columns)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::data_types::cql_type::cql_type::CQLType;
    use crate::{
        parser::data_types::{
            cql_type::native_types::NativeType::{Int, Text},
            identifier::{
                quoted_identifier::QuotedIdentifier, unquoted_identifier::UnquotedIdentifier,
            },
            unquoted_name::UnquotedName,
        },
        tokenizer::tokenizer::tokenize_query,
    };

    // USE STATEMENT TESTS:
    #[test]
    fn test_01_basic_use_statement() -> Result<(), Error> {
        let query = "USE my_keyspace";
        let mut tokens = tokenize_query(query);

        let result = use_statement(&mut tokens)?;
        assert!(result.is_some());

        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_02_use_statement_with_quoted_keyspace() -> Result<(), Error> {
        let query = "USE \"My Keyspace\"";
        let mut tokens = tokenize_query(query);

        let result = use_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace,
            KeyspaceName::QuotedName(UnquotedName::new("My Keyspace".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_03_use_statement_case_sensitivity() -> Result<(), Error> {
        let query = "use MY_KEYSPACE";
        let mut tokens = tokenize_query(query);

        let result = use_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace,
            KeyspaceName::UnquotedName(UnquotedName::new("MY_KEYSPACE".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_04_invalid_use_statement() -> Result<(), Error> {
        let query = "USE";
        let mut tokens = tokenize_query(query);

        let result = use_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // CREATE KEYSPACE TESTS:
    #[test]
    fn test_01_basic_create_keyspace_statement() -> Result<(), Error> {
        let query = "CREATE KEYSPACE my_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens)?;
        assert!(result.is_some());

        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace.keyspace_name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(!keyspace.if_not_exist);
        assert!(!keyspace.options.is_empty());
        Ok(())
    }

    #[test]
    fn test_02_create_keyspace_with_if_not_exists() -> Result<(), Error> {
        let query = "CREATE KEYSPACE IF NOT EXISTS my_keyspace WITH replication = {'class': 'NetworkTopologyStrategy', 'dc1': 3, 'dc2': 2}";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.keyspace_name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(keyspace.if_not_exist);
        Ok(())
    }

    #[test]
    fn test_03_create_keyspace_with_multiple_options() -> Result<(), Error> {
        let query = "CREATE KEYSPACE my_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1} AND durable_writes = false";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.keyspace_name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(!keyspace.if_not_exist);
        Ok(())
    }

    #[test]
    fn test_04_create_keyspace_with_quoted_name() -> Result<(), Error> {
        let query = "CREATE KEYSPACE \"My Keyspace\" WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.keyspace_name,
            KeyspaceName::QuotedName(UnquotedName::new("My Keyspace".to_string())?)
        );
        assert!(!keyspace.if_not_exist);
        Ok(())
    }

    #[test]
    fn test_05_invalid_create_keyspace_statement() -> Result<(), Error> {
        let query = "CREATE KEYSPACE";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_06_create_keyspace_missing_with_clause() -> Result<(), Error> {
        let query = "CREATE KEYSPACE my_keyspace";
        let mut tokens = tokenize_query(query);

        let result = create_keyspace_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // ALTER KEYSPACE TESTS:
    #[test]
    fn test_01_basic_alter_keyspace_statement() -> Result<(), Error> {
        let query = "ALTER KEYSPACE my_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens)?;
        assert!(result.is_some());

        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(!keyspace.if_exists);
        assert!(!keyspace.options.is_empty());
        Ok(())
    }

    #[test]
    fn test_02_alter_keyspace_with_if_exists() -> Result<(), Error> {
        let query = "ALTER KEYSPACE IF EXISTS my_keyspace WITH replication = {'class': 'NetworkTopologyStrategy', 'dc1': 3, 'dc2': 2}";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(keyspace.if_exists);
        Ok(())
    }

    #[test]
    fn test_03_alter_keyspace_with_multiple_options() -> Result<(), Error> {
        let query = "ALTER KEYSPACE my_keyspace WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1} AND durable_writes = false";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(!keyspace.if_exists);
        assert_eq!(keyspace.options.len(), 2);
        Ok(())
    }

    #[test]
    fn test_04_alter_keyspace_with_quoted_name() -> Result<(), Error> {
        let query = "ALTER KEYSPACE \"My Keyspace\" WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 3}";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.name,
            KeyspaceName::QuotedName(UnquotedName::new("My Keyspace".to_string())?)
        );
        assert!(!keyspace.if_exists);
        Ok(())
    }

    #[test]
    fn test_05_invalid_alter_keyspace_statement() -> Result<(), Error> {
        let query = "ALTER KEYSPACE";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_06_alter_keyspace_missing_with_clause() -> Result<(), Error> {
        let query = "ALTER KEYSPACE my_keyspace";
        let mut tokens = tokenize_query(query);

        let result = alter_keyspace_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // DROP KEYSPACE TESTS:
    #[test]
    fn test_01_basic_drop_keyspace_statement() -> Result<(), Error> {
        let query = "DROP KEYSPACE my_keyspace";
        let mut tokens = tokenize_query(query);

        let result = drop_keyspace_statement(&mut tokens)?;
        assert!(result.is_some());

        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            keyspace.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(!keyspace.if_exists);
        Ok(())
    }

    #[test]
    fn test_02_drop_keyspace_with_if_exists() -> Result<(), Error> {
        let query = "DROP KEYSPACE IF EXISTS my_keyspace";
        let mut tokens = tokenize_query(query);

        let result = drop_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_keyspace".to_string())?)
        );
        assert!(keyspace.if_exists);
        Ok(())
    }

    #[test]
    fn test_03_drop_keyspace_with_quoted_name() -> Result<(), Error> {
        let query = "DROP KEYSPACE \"My Keyspace\"";
        let mut tokens = tokenize_query(query);

        let result = drop_keyspace_statement(&mut tokens)?;
        let keyspace = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            keyspace.name,
            KeyspaceName::QuotedName(UnquotedName::new("My Keyspace".to_string())?)
        );
        assert!(!keyspace.if_exists);
        Ok(())
    }

    #[test]
    fn test_04_invalid_drop_keyspace_statement() -> Result<(), Error> {
        let query = "DROP KEYSPACE";
        let mut tokens = tokenize_query(query);

        let result = drop_keyspace_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // CREATE TABLE TESTS:
    #[test]
    fn test_01_basic_create_table_statement() -> Result<(), Error> {
        let query =
            "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT, age INT) WITH 'COMPACT STORAGE'";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        assert!(result.is_some());

        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert!(!table.if_not_exists);
        assert_eq!(table.columns.len(), 3);
        assert_eq!(
            table
                .primary_key
                .as_ref()
                .map_or(1, |pk| pk.partition_key.len()),
            1
        );
        assert!(table.compact_storage);
        assert!(table.clustering_order.is_none());
        Ok(())
    }

    #[test]
    fn test_02_create_table_with_if_not_exists() -> Result<(), Error> {
        let query = "CREATE TABLE IF NOT EXISTS users (id UUID PRIMARY KEY, name TEXT, age INT) WITH 'COMPACT STORAGE'";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert!(table.if_not_exists);
        assert!(table.compact_storage);
        Ok(())
    }

    #[test]
    fn test_03_create_table_with_primary_key_and_clustering_column() -> Result<(), Error> {
        let query = "CREATE TABLE posts (
            user_id UUID,
            post_id TIMEUUID,
            content TEXT,
            PRIMARY KEY (user_id, post_id)
        ) WITH 'CLUSTERING ORDER BY' (post_id DESC)";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(table.columns.len(), 3);
        assert_eq!(
            table
                .primary_key
                .as_ref()
                .map_or(0, |pk| pk.partition_key.len()),
            1
        );
        assert_eq!(
            table
                .primary_key
                .as_ref()
                .map_or(0, |pk| pk.clustering_columns.len()),
            1
        );
        assert!(table.clustering_order.is_some());
        Ok(())
    }

    #[test]
    fn test_04_create_table_with_compound_primary_key_and_clustering_column() -> Result<(), Error> {
        let query = "CREATE TABLE posts (
            user_id UUID,
            post_id TIMEUUID,
            content TEXT,
            PRIMARY KEY ((user_id, post_id), content)
        ) WITH 'CLUSTERING ORDER BY' (post_id DESC)";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(table.columns.len(), 3);
        assert_eq!(
            table
                .primary_key
                .as_ref()
                .map_or(0, |pk| pk.partition_key.len()),
            2
        );
        assert_eq!(
            table
                .primary_key
                .as_ref()
                .map_or(0, |pk| pk.clustering_columns.len()),
            1
        );
        assert!(table.clustering_order.is_some());
        Ok(())
    }

    #[test]
    fn test_05_create_table_without_compact_storage() -> Result<(), Error> {
        let query = "CREATE TABLE events (
            id UUID PRIMARY KEY,
            data TEXT
        ) WITH read_repair_chance = 1.0";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(table.columns.len(), 2);
        assert!(!table.compact_storage);
        assert!(table.clustering_order.is_none());
        Ok(())
    }

    #[test]
    fn test_06_create_table_with_quoted_names() -> Result<(), Error> {
        let query = "CREATE TABLE \"My Table\" (
            \"User ID\" UUID PRIMARY KEY,
            \"Full Name\" TEXT
        ) WITH 'COMPACT STORAGE'";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            table.name.name,
            KeyspaceName::QuotedName(UnquotedName::new("My Table".to_string())?)
        );
        assert_eq!(table.columns.len(), 2);
        assert_eq!(
            table.columns[0].column_name,
            Identifier::QuotedIdentifier(QuotedIdentifier::new("User ID".to_string()))
        );
        assert_eq!(
            table.columns[1].column_name,
            Identifier::QuotedIdentifier(QuotedIdentifier::new("Full Name".to_string()))
        );
        assert!(table.compact_storage);
        Ok(())
    }

    #[test]
    fn test_07_invalid_create_table_statement() -> Result<(), Error> {
        let query = "CREATE TABLE";
        let mut tokens = tokenize_query(query);

        let result = create_table_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_07_create_table_missing_with_clause() -> Result<(), Error> {
        let query = "CREATE TABLE users (id UUID PRIMARY KEY, name TEXT)";
        let mut tokens = tokenize_query(query);
        assert!(create_table_statement(&mut tokens).is_ok());
        Ok(())
    }

    // ALTER TABLE TESTS:

    #[test]
    fn test_01_basic_alter_table_statement() -> Result<(), Error> {
        let query = "ALTER TABLE users ADD new_column TEXT";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens)?;
        assert!(result.is_some());

        let alter_table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            alter_table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(
            alter_table.instruction,
            AlterTableInstruction::AddColumns(
                false,
                vec![ColumnDefinition::new(
                    Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                        "new_column".to_string()
                    )),
                    CQLType::NativeType(Text),
                    false,
                    false
                )]
            )
        );
        Ok(())
    }

    #[test]
    fn test_02_alter_table_add_columns_with_if_not_exists() -> Result<(), Error> {
        let query = "ALTER TABLE users ADD IF NOT EXISTS (new_column1 TEXT, new_column2 INT)";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens)?;
        let alter_table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            alter_table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(
            alter_table.instruction,
            AlterTableInstruction::AddColumns(
                true,
                vec![
                    ColumnDefinition::new(
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                            "new_column1".to_string()
                        )),
                        CQLType::NativeType(Text),
                        false,
                        false
                    ),
                    ColumnDefinition::new(
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                            "new_column2".to_string()
                        )),
                        CQLType::NativeType(Int),
                        false,
                        false
                    )
                ]
            )
        );
        Ok(())
    }

    #[test]
    fn test_03_alter_table_drop_columns_with_if_exists() -> Result<(), Error> {
        let query = "ALTER TABLE users DROP IF EXISTS old_column1, old_column2";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens)?;
        let alter_table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            alter_table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(
            alter_table.instruction,
            AlterTableInstruction::DropColumns(
                true,
                vec!["old_column1".to_string(), "old_column2".to_string()]
            )
        );
        Ok(())
    }

    #[test]
    fn test_04_alter_table_rename_columns() -> Result<(), Error> {
        let query =
            "ALTER TABLE users RENAME old_column1 TO new_column1 AND old_column2 TO new_column2";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens)?;
        let alter_table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            alter_table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(
            alter_table.instruction,
            AlterTableInstruction::RenameColumns(
                false,
                vec!(
                    ("old_column1".to_string(), "new_column1".to_string()),
                    ("old_column2".to_string(), "new_column2".to_string())
                )
            )
        );
        Ok(())
    }

    #[test]
    fn test_05_alter_table_with_if_exists() -> Result<(), Error> {
        let query = "ALTER TABLE IF EXISTS users RENAME old_column TO new_column";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens)?;
        let alter_table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            alter_table.name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert!(alter_table.name.if_exists);
        Ok(())
    }

    #[test]
    fn test_06_invalid_alter_table_statement() -> Result<(), Error> {
        let query = "ALTER TABLE";
        let mut tokens = tokenize_query(query);

        let result = alter_table_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // DROP TABLE TESTS:
    #[test]
    fn test_01_basic_drop_table_statement() -> Result<(), Error> {
        let query = "DROP TABLE my_table";
        let mut tokens = tokenize_query(query);

        let result = drop_table_statement(&mut tokens)?;
        assert!(result.is_some());

        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            table.table_name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_table".to_string())?)
        );
        assert!(!table.table_name.if_exists);
        Ok(())
    }

    #[test]
    fn test_02_drop_table_with_if_exists() -> Result<(), Error> {
        let query = "DROP TABLE IF EXISTS my_table";
        let mut tokens = tokenize_query(query);

        let result = drop_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            table.table_name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("my_table".to_string())?)
        );
        assert!(table.table_name.if_exists);
        Ok(())
    }

    #[test]
    fn test_03_drop_table_with_quoted_name() -> Result<(), Error> {
        let query = "DROP TABLE \"My Table\"";
        let mut tokens = tokenize_query(query);

        let result = drop_table_statement(&mut tokens)?;
        let table = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            table.table_name.name,
            KeyspaceName::QuotedName(UnquotedName::new("My Table".to_string())?)
        );
        assert!(!table.table_name.if_exists);
        Ok(())
    }

    #[test]
    fn test_04_invalid_drop_table_statement() -> Result<(), Error> {
        let query = "DROP TABLE";
        let mut tokens = tokenize_query(query);

        let result = drop_table_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_01_basic_truncate_statement() -> Result<(), Error> {
        let query = "TRUNCATE table_name";
        let mut tokens = tokenize_query(query);

        let result = truncate_statement(&mut tokens)?;
        assert!(result.is_some());

        let truncate = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            truncate.table_name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("table_name".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_02_truncate_statement_with_table_keyword() -> Result<(), Error> {
        let query = "TRUNCATE TABLE table_name";
        let mut tokens = tokenize_query(query);

        let result = truncate_statement(&mut tokens)?;
        let truncate = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            truncate.table_name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("table_name".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_03_truncate_statement_with_quoted_table_name() -> Result<(), Error> {
        let query = "TRUNCATE \"My Table\"";
        let mut tokens = tokenize_query(query);

        let result = truncate_statement(&mut tokens)?;
        let truncate = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            truncate.table_name.name,
            KeyspaceName::QuotedName(UnquotedName::new("My Table".to_string())?)
        );
        Ok(())
    }

    #[test]
    fn test_04_invalid_truncate_statement() -> Result<(), Error> {
        let query = "TRUNCATE";
        let mut tokens = tokenize_query(query);

        let result = truncate_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // EMPTY INPUT TESTS:
    #[test]
    fn test_01_keyspace_empty_input() -> Result<(), Error> {
        let mut tokens = vec![];
        let use_statement = use_statement(&mut tokens)?;
        let create_keyspace = create_keyspace_statement(&mut tokens)?;
        let alter_keyspace = alter_keyspace_statement(&mut tokens)?;
        let drop_keyspace = drop_keyspace_statement(&mut tokens)?;
        assert!(
            use_statement.is_none()
                && create_keyspace.is_none()
                && alter_keyspace.is_none()
                && drop_keyspace.is_none()
        );
        Ok(())
    }

    #[test]
    fn test_02_table_empty_input() -> Result<(), Error> {
        let mut tokens = vec![];
        let create_table = create_table_statement(&mut tokens)?;
        let alter_table = alter_table_statement(&mut tokens)?;
        let drop_table = drop_table_statement(&mut tokens)?;
        let truncate = truncate_statement(&mut tokens)?;
        assert!(
            create_table.is_none()
                && alter_table.is_none()
                && drop_table.is_none()
                && truncate.is_none()
        );
        Ok(())
    }
}
