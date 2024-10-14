use crate::{
    cassandra::errors::error::Error,
    parser::{
        data_types::{
            constant::Constant,
            identifier::{
                identifier::Identifier, quoted_identifier::QuotedIdentifier,
                unquoted_identifier::UnquotedIdentifier,
            },
            keyspace_name::KeyspaceName,
            term::Term,
        },
        statements::{
            ddl_statement::ddl_statement_parser::check_words,
            dml_statement::{
                delete::{Delete, DeleteBuilder},
                expression::expression,
                group_by::GroupBy,
                insert::Insert,
                limit::Limit,
                per_partition_limit::PerPartitionLimit,
                r#where::Where,
                relation::Relation,
                select::{KindOfColumns, Select, SelectBuilder},
                selector::Selector,
                update::{Update, UpdateBuilder},
            },
        },
    },
};

pub enum DmlStatement {
    SelectStatement(Select),
    InsertStatement(Insert),
    UpdateStatement(Update),
    DeleteStatement(Delete),
    BatchStatement,
}

pub fn dml_statement(lista: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    if let Some(dml_statement) = select_statement(lista)? {
        return Ok(Some(DmlStatement::SelectStatement(dml_statement)));
    } else if let Some(dml_statement) = insert_statement(lista)? {
        return Ok(Some(DmlStatement::InsertStatement(dml_statement)));
    } else if let Some(dml_statement) = delete_statement(lista)? {
        return Ok(Some(DmlStatement::DeleteStatement(dml_statement)));
    } else if let Some(dml_statement) = update_statement(lista)? {
        return Ok(Some(DmlStatement::UpdateStatement(dml_statement)));
    } else if let Some(_dml_statement) = batch_statement(lista)? {
        return Ok(Some(DmlStatement::BatchStatement));
    }
    Ok(None)
}

pub fn select_statement(lista: &mut Vec<String>) -> Result<Option<Select>, Error> {
    let index = 0;
    if lista[index] == "SELECT" {
        lista.remove(index);
        let mut builder = SelectBuilder::default();
        if lista[index] == "*" {
            lista.remove(index);
            builder.set_select_clause(KindOfColumns::All);
        } else {
            let res = match select_clause(lista)? {
                Some(columns) => columns,
                None => {
                    return Err(Error::SyntaxError(
                        "No se especifico ninguna columna".to_string(),
                    ))
                }
            };
            builder.set_select_clause(KindOfColumns::SelectClause(res));
        }
        from_clause(lista, &mut builder)?;
        builder.set_where(where_clause(lista)?);
        ordering_clause(lista, &mut builder)?;
        per_partition_limit_clause(lista, &mut builder)?;
        limit_clause(lista, &mut builder)?;
        allow_filtering_clause(lista.to_vec(), &mut builder);
        return Ok(Some(builder.build()));
    }
    Ok(None)
}

fn select_clause(lista: &mut Vec<String>) -> Result<Option<Vec<Selector>>, Error> {
    if lista[0] != "FROM" {
        let mut vec: Vec<Selector> = Vec::new();
        if let Some(sel) = selector(lista)? {
            vec.push(sel);
        }
        if lista[0] == "," {
            lista.remove(0);
            if let Some(mut clasules) = select_clause(lista)? {
                vec.append(&mut clasules);
            };
        }
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

pub fn from_clause(lista: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if check_words(lista, "FROM") {
        let table_name = match KeyspaceName::check_kind_of_name(lista)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Tipo de dato no admitido".to_string())),
        };
        builder.set_from(table_name);
        Ok(())
    } else {
        Err(Error::SyntaxError(
            "Falta el from en la consulta".to_string(),
        ))
    }
}

pub fn where_clause(lista: &mut Vec<String>) -> Result<Option<Where>, Error> {
    if check_words(lista, "WHERE") {
        Ok(Some(Where::new(expression(lista)?)))
    } else {
        Ok(Some(Where::new(None)))
    }
}

pub fn relation(lista: &mut Vec<String>) -> Result<Option<Relation>, Error> {
    if let Some(_value) = is_column_name(lista)? {
        lista.remove(0);
    }
    Ok(None)
}

pub fn group_by_clause(lista: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if lista[0] == "GROUP" && lista[1] == "BY" {
        lista.remove(0);
        lista.remove(0);
        let mut columns: Vec<Identifier> = Vec::new();
        match is_column_name(lista)? {
            Some(value) => columns.push(value),
            None => {
                return Err(Error::SyntaxError(
                    "Columnas de GROUP BY no encontradas".to_string(),
                ))
            }
        };
        while lista[0] == "," {
            match is_column_name(lista)? {
                Some(value) => columns.push(value),
                None => {
                    return Err(Error::SyntaxError(
                        "Columnas de GROUP BY no encontradas".to_string(),
                    ))
                }
            };
        }
        builder.set_group_by(Some(GroupBy::new(columns)));
    } else {
        builder.set_group_by(None);
    }
    Ok(())
}

pub fn ordering_clause(lista: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if check_words(lista, "ORDER BY") {
        if let Some(value) = is_column_name(lista)? {} // TERMINAR
    } else {
        builder.set_order_by(None);
    }
    Ok(())
}

pub fn per_partition_limit_clause(
    lista: &mut Vec<String>,
    builder: &mut SelectBuilder,
) -> Result<(), Error> {
    if lista[0] == "PER" && lista[1] == "PARTITION" && lista[2] == "LIMIT" {
        lista.remove(0);
        lista.remove(0);
        lista.remove(0);
        let int = lista.remove(0);
        let int = match int.parse::<i32>() {
            Ok(value) => Limit::new(value),
            Err(_e) => {
                return Err(Error::SyntaxError(
                    "El valor brindado al Per Partition Limit no es un int".to_string(),
                ))
            }
        };
        builder.set_limit(Some(int));
    } else {
        builder.set_per_partition_limit(None);
    }
    Ok(())
}
pub fn limit_clause(lista: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if lista[0] == "LIMIT" {
        lista.remove(0);
        let int = lista.remove(0);
        let int = match int.parse::<i32>() {
            Ok(value) => Limit::new(value),
            Err(_e) => {
                return Err(Error::SyntaxError(
                    "El valor brindado al Limit no es un int".to_string(),
                ))
            }
        };
        builder.set_limit(Some(int));
    } else {
        builder.set_limit(None);
    }
    Ok(())
}
pub fn allow_filtering_clause(
    lista: Vec<String>,
    builder: &mut SelectBuilder,
) -> Option<PerPartitionLimit> {
    if lista[0] == "ALLOW" && lista[1] == "FILTERING" {
        builder.set_allow_filtering(Some(true));
    } else {
        builder.set_allow_filtering(None);
    }
    None
}

pub fn selector(lista: &mut Vec<String>) -> Result<Option<Selector>, Error> {
    if let Some(column) = is_column_name(lista)? {
        return Ok(Some(Selector::ColumnName(column)));
    }
    if let Some(term) = is_term(lista)? {
        return Ok(Some(Selector::Term(term)));
    }
    Ok(None)
}

// identifier
pub fn is_column_name(lista: &mut Vec<String>) -> Result<Option<Identifier>, Error> {
    if QuotedIdentifier::check_quoted_identifier(&lista[0], &lista[1], &lista[2]) {
        lista.remove(0);
        let string = lista.remove(0);
        lista.remove(0);
        return Ok(Some(Identifier::QuotedIdentifier(QuotedIdentifier::new(
            string,
        ))));
    } else if UnquotedIdentifier::check_unquoted_identifier(&lista[0]) {
        let string = lista.remove(0);
        return Ok(Some(Identifier::UnquotedIdentifier(
            UnquotedIdentifier::new(string),
        )));
    }
    Ok(None)
}

pub fn is_term(lista: &mut Vec<String>) -> Result<Option<Term>, Error> {
    // Todo: falta corroborar que el largo de la lista sea de al menos X largo asi no rompe con remove
    if Constant::check_string(&lista[0], &lista[2]) {
        lista.remove(0);
        let string = Constant::String(lista.remove(0));
        lista.remove(0);
        return Ok(Some(Term::Constant(string)));
    } else if Constant::check_integer(&lista[0]) {
        let integer_string: String = lista.remove(0);
        let int = Constant::new_integer(integer_string)?;
        return Ok(Some(Term::Constant(int)));
    } else if Constant::check_float(&lista[0]) {
        let float_string = lista.remove(0);
        let float = Constant::new_float(float_string)?;
        return Ok(Some(Term::Constant(float)));
    } else if Constant::check_boolean(&lista[0]) {
        let bool = lista.remove(0);
        let bool = Constant::new_boolean(bool)?;
        return Ok(Some(Term::Constant(bool)));
    } else if Constant::check_uuid(&lista[0]) {
        let uuid = lista.remove(0);
        let uuid = Constant::new_uuid(uuid)?;
        return Ok(Some(Term::Constant(uuid)));
    } else if Constant::check_hex(&lista[0]) {
        let hex = Constant::new_hex(lista.remove(0))?;
        return Ok(Some(Term::Constant(hex)));
    } else if Constant::check_blob(&lista[0]) {
        let blob = Constant::new_blob(lista.remove(0))?;
        return Ok(Some(Term::Constant(blob)));
    }
    Ok(None)
}

pub fn get_clauses(lista: &mut Vec<String>) -> Result<Option<Vec<Selector>>, Error> {
    //cambiaste select_clause por esta funcion no?
    if lista[0] != "FROM" {
        let mut vec: Vec<Selector> = Vec::new();
        if let Some(sel) = selector(lista)? {
            vec.push(sel);
        }
        if lista[0] == "," {
            lista.remove(0);
            if let Some(mut clausules) = get_clauses(lista)? {
                vec.append(&mut clausules);
            };
        }
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

pub fn delete_statement(lista: &mut Vec<String>) -> Result<Option<Delete>, Error> {
    let index: usize = 0;
    if lista[index] == "DELETE" {
        let mut builder = DeleteBuilder::default();
        lista.remove(index);

        if lista[index] != "FROM" {
            let res = get_clauses(lista);
            match res {
                Ok(Some(_x)) => {}
                Ok(None) => return Err(Error::SyntaxError("Columna(s) inválida(s)".to_string())),
                Err(_x) => {
                    return Err(Error::SyntaxError(
                        "Falta el from en la consulta".to_string(),
                    ))
                }
            }
        }

        let file_name: String = lista.remove(index);

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        } else {
            return Ok(None);
        }

        if lista[index] == "WHERE" {
            lista.remove(index);
            builder.set_where(where_clause(lista)?);
            if lista[index] == "IF" {
                lista.remove(index);
                // Chequeo sintaxis de condicionales para la query
            } else {
                return Ok(None);
            }

            return Ok(Some(builder.build()));
        } else {
            return Ok(None);
        }
    }
    Ok(None)
}

pub fn insert_statement(lista: &mut Vec<String>) -> Result<Option<Insert>, Error> {
    let index = 0;
    if lista[index] == "INSERT" && lista[index + 1] == "INTO" {
        lista.remove(index);
        lista.remove(index);

        //let mut builder = InsertBuilder::default();
        // Guardar el nombre de la tabla
        let file_name: String = lista.remove(index);

        if lista[index] == "JSON" {
            // Chequeo si la sintaxis JSON es válida
        } else {
            // Chequeo si la sintaxis de las columnas es válida (o crear si no existe alguna)
        }

        if lista[index] == "IF" && lista[index + 1] == "NOT" && lista[index + 2] == "EXISTS" {
            // Chequeo de la sintaxis de IF NOT EXISTS
        }

        if lista[index] == "VALUES" {
            lista.remove(index);
            // Chequeo/match de valores con columnas
        }

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        }
        //return Ok(Some(builder.build()));
    }
    Ok(None)
}

pub fn update_statement(lista: &mut Vec<String>) -> Result<Option<Update>, Error> {
    let index = 0;
    if lista[0] == "UPDATE" {
        let mut builder = UpdateBuilder::default();
        // Guardar el nombre de la tabla
        let file_name: String = lista.remove(index);

        if lista[index] == "USING" {
            lista.remove(index);
            // Chequeo de la sintaxis de USING
        }

        if lista[index] == "SET" {
            lista.remove(index);
            // Chequeo de la sintaxis de SET
        } else {
            return Ok(None);
        }

        if lista[index] == "WHERE" {
            lista.remove(index);
            builder.set_where(where_clause(lista)?);
            if lista[index] == "IF" {
                lista.remove(index);
                // Chequeo sintaxis de condicionales para la query
            }
        } else {
            return Ok(None);
        }
        return Ok(Some(builder.build()));
    }

    Ok(None)
}

pub fn batch_statement(lista: &mut Vec<String>) -> Result<Option<Vec<DmlStatement>>, Error> {
    let index = 0;
    let query: Vec<DmlStatement> = Vec::new();

    if lista[index] == "BEGIN" {
        lista.remove(index);
        if lista[index] == "UNLOGGED" {
            // Lógica para el Unlogged Batch -> Aplicación parcial del batch
            lista.remove(index);
        } else if lista[index] == "COUNTER" {
            // Lógica para el Counter Batch -> Aplicación para contadores
            lista.remove(index);
        } else {
            // Lógica para el Logged Batch -> Aplicación total del batch
        }
    } else {
        return Ok(Some(query));
    }

    lista.remove(index);

    if lista[index] != "BATCH" {
        return Ok(Some(query));
    }

    let mut query: Vec<DmlStatement> = Vec::new();
    while lista[index] != "APPLY" && lista[index + 1] != "BATCH" {
        if lista.is_empty() {
            break;
        }
        if lista[index] == "INSERT" {
            if let Some(insert_stmt) = insert_statement(lista)? {
                query.push(DmlStatement::InsertStatement(insert_stmt));
            }
        } else if lista[index] == "UPDATE" {
            if let Some(update_stmt) = update_statement(lista)? {
                query.push(DmlStatement::UpdateStatement(update_stmt));
            }
        } else if lista[index] == "DELETE" {
            if let Some(delete_stmt) = delete_statement(lista)? {
                query.push(DmlStatement::DeleteStatement(delete_stmt));
            }
        }
        lista.remove(index);
    }
    Ok(Some(query))
}
