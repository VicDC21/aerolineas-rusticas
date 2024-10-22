use crate::{
    parser::{
        assignment::Assignment,
        data_types::{
            identifier::identifier::Identifier, literal::tuple_literal::TupleLiteral, term::Term,
        },
        statements::{
            ddl_statement::ddl_statement_parser::check_words,
            dml_statement::{
                if_condition::{Condition, IfCondition},
                main_statements::{
                    batch::{Batch, BatchBuilder, BatchType},
                    delete::Delete,
                    insert::Insert,
                    select::{
                        group_by::GroupBy, kind_of_columns::KindOfColumns, limit::Limit,
                        options::SelectOptions, order_by::OrderBy, ordering::Ordering,
                        per_partition_limit::PerPartitionLimit, select_operation::Select,
                        selector::Selector,
                    },
                    update::Update,
                },
                r#where::{expression::expression, r#where_parser::Where},
            },
        },
        table_name::TableName,
    },
    protocol::errors::error::Error,
};

use super::r#where::operator::Operator;

/// dml_statement::= select_statement
/// | insert_statement
/// | update_statement
/// | delete_statement
/// | batch_statement
#[derive(Debug)]
pub enum DmlStatement {
    /// select_statement::= SELECT [ JSON | DISTINCT ] ( select_clause | '*' )
    /// FROM `table_name`
    /// [ WHERE `where_clause` ]
    /// [ GROUP BY `group_by_clause` ]
    /// [ ORDER BY `ordering_clause` ]
    /// [ PER PARTITION LIMIT (`integer` | `bind_marker`) ]
    /// [ LIMIT (`integer` | `bind_marker`) ]
    /// [ ALLOW FILTERING ]
    SelectStatement(Select), // EL bind_marker AUN NO ESTA IMPLEMENTADO, NO SABIA MUY BIEN PARA QUE SERVIA
    // Tampoco eñ JSON / DISTINCT
    /// insert_statement::= INSERT INTO table_name names_values
    /// [ IF NOT EXISTS ]
    InsertStatement(Insert),

    /// update_statement ::= UPDATE table_name
    ///                      SET assignment( ',' assignment )*
    ///                      WHERE where_clause
    ///                      [ IF ( EXISTS | condition ( AND condition)*) ]
    UpdateStatement(Update),

    /// delete_statement::= DELETE [ simple_selection ( ',' simple_selection ) ]
    ///     FROM table_name
    ///     WHERE where_clause
    ///     [ IF ( EXISTS | condition ( AND condition)*) ]
    DeleteStatement(Delete),

    /// batch_statement ::= BEGIN [ UNLOGGED | COUNTER ] BATCH
    ///                     [ USING update_parameter( AND update_parameter)* ]
    ///                     modification_statement ( ';' modification_statement )*
    ///                     APPLY BATCH
    /// modification_statement ::= insert_statement | update_statement | delete_statement
    BatchStatement(Batch),
}

/// Crea el enum `DmlStatement` con el tipo de struct de acuerdo a la sintaxis dada, si la entrada proporcionada no satisface
/// los requerimientos de los tipos de datos, entonces devuelve None.
pub fn dml_statement(list: &mut Vec<String>) -> Result<Option<DmlStatement>, Error> {
    if let Some(dml_statement) = select_statement(list)? {
        return Ok(Some(DmlStatement::SelectStatement(dml_statement)));
    } else if let Some(dml_statement) = insert_statement(list)? {
        return Ok(Some(DmlStatement::InsertStatement(dml_statement)));
    } else if let Some(dml_statement) = delete_statement(list)? {
        return Ok(Some(DmlStatement::DeleteStatement(dml_statement)));
    } else if let Some(dml_statement) = update_statement(list)? {
        return Ok(Some(DmlStatement::UpdateStatement(dml_statement)));
    } else if let Some(dml_statement) = batch_statement(list)? {
        return Ok(Some(DmlStatement::BatchStatement(dml_statement)));
    }
    Ok(None)
}

fn select_statement(list: &mut Vec<String>) -> Result<Option<Select>, Error> {
    if check_words(list, "SELECT") {
        let select_columns = kind_of_columns(list)?;
        let from = from_clause(list)?;
        let the_where = where_clause(list)?;
        let group_by = group_by_clause(list)?;
        let order_by = ordering_clause(list)?;
        let per_partition_limit = per_partition_limit_clause(list)?;
        let limit = limit_clause(list)?;
        let allow_filtering = allow_filtering_clause(list);

        let options = SelectOptions {
            the_where,
            group_by,
            order_by,
            per_partition_limit,
            limit,
            allow_filtering,
        };

        return Ok(Some(Select::new(select_columns, from, options)));
    }
    Ok(None)
}

fn insert_statement(list: &mut Vec<String>) -> Result<Option<Insert>, Error> {
    if check_words(list, "INSERT INTO") {
        let table_name: TableName = match TableName::check_kind_of_name(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "El nombre de la tabla no es sintacticamente valido".to_string(),
                ))
            }
        };
        let names = check_insert_names(list)?;
        if !check_words(list, "VALUES") {
            return Err(Error::SyntaxError("Falto VALUES".to_string()));
        }
        let values = match TupleLiteral::check_tuple_literal(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "No se encontro ninguna tupla".to_string(),
                ))
            }
        };

        if values.size() != names.len() {
            return Err(Error::SyntaxError(
                "El numero de columnas y valores no coincide".to_string(),
            ));
        }

        let if_not_exists = check_words(list, "IF NOT EXISTS");
        return Ok(Some(Insert::new(table_name, names, values, if_not_exists)));
    }
    Ok(None)
}

fn update_statement(list: &mut Vec<String>) -> Result<Option<Update>, Error> {
    if check_words(list, "UPDATE") {
        let table_name: TableName = match TableName::check_kind_of_name(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "El nombre de la tabla no es sintacticamente valido".to_string(),
                ))
            }
        };
        let set = set_clause(list)?;
        if list.first() != Some(&"WHERE".to_string()) {
            return Err(Error::SyntaxError("Falta la cláusula WHERE".to_string()));
        }
        let r#where = where_clause(list)?;
        let if_condition = check_if_condition(list)?;
        return Ok(Some(Update::new(table_name, set, r#where, if_condition)));
    }

    Ok(None)
}

fn delete_statement(list: &mut Vec<String>) -> Result<Option<Delete>, Error> {
    if !check_words(list, "DELETE") {
        return Ok(None);
    }

    let mut cols = Vec::new();
    while list.first() != Some(&"FROM".to_string()) {
        if list.is_empty() {
            return Err(Error::SyntaxError(
                "Se esperaba FROM después de las columnas".to_string(),
            ));
        }
        cols.push(list.remove(0));
        check_words(list, ",");
    }

    let from = from_clause(list)?;

    if list.first() != Some(&"WHERE".to_string()) {
        return Err(Error::SyntaxError("Falta la cláusula WHERE".to_string()));
    }
    let r#where = where_clause(list)?;
    let if_condition = check_if_condition(list)?;
    Ok(Some(Delete::new(cols, from, r#where, if_condition)))
}

fn batch_statement(list: &mut Vec<String>) -> Result<Option<Batch>, Error> {
    let mut builder = BatchBuilder::default();
    if check_words(list, "BEGIN") {
        if check_words(list, "UNLOGGED") {
            builder.set_batch_clause(BatchType::Unlogged);
        } else if check_words(list, "COUNTER") {
            builder.set_batch_clause(BatchType::Counter);
        } else if !check_words(list, "BATCH") {
            return Err(Error::SyntaxError("Falta BATCH en la consulta".to_string()));
        }
    } else {
        return Ok(None);
    }

    let mut queries: Vec<DmlStatement> = Vec::new();
    while list[0] != "APPLY" && list[1] != "BATCH" {
        if list.is_empty() {
            break;
        }
        if list[0] == "INSERT" {
            if let Some(insert_stmt) = insert_statement(list)? {
                queries.push(DmlStatement::InsertStatement(insert_stmt));
            }
        } else if list[0] == "UPDATE" {
            if let Some(update_stmt) = update_statement(list)? {
                queries.push(DmlStatement::UpdateStatement(update_stmt));
            }
        } else if list[0] == "DELETE" {
            if let Some(delete_stmt) = delete_statement(list)? {
                queries.push(DmlStatement::DeleteStatement(delete_stmt));
            }
        } else if list[0] == "SELECT" {
            return Err(Error::SyntaxError(
                "No se puede hacer una consulta SELECT usando BATCH".to_string(),
            ));
        }
        list.remove(0);
    }
    if queries.is_empty() {
        return Err(Error::SyntaxError(
            "No se encontraron consultas en el batch".to_string(),
        ));
    }
    builder.set_queries(queries);
    Ok(Some(builder.build()))
}

fn select_clause(list: &mut Vec<String>) -> Result<Option<Vec<Selector>>, Error> {
    if list[0] != "FROM" {
        let mut vec: Vec<Selector> = Vec::new();
        if let Some(sel) = selector(list)? {
            vec.push(sel);
        }
        if check_words(list, ",") {
            if let Some(mut clasules) = select_clause(list)? {
                vec.append(&mut clasules);
            };
        }
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

fn kind_of_columns(list: &mut Vec<String>) -> Result<KindOfColumns, Error> {
    if check_words(list, "*") {
        Ok(KindOfColumns::All)
    } else {
        let res = match select_clause(list)? {
            Some(columns) => columns,
            None => {
                return Err(Error::SyntaxError(
                    "No se especifico ninguna columna".to_string(),
                ))
            }
        };
        Ok(KindOfColumns::SelectClause(res))
    }
}

fn from_clause(list: &mut Vec<String>) -> Result<TableName, Error> {
    if check_words(list, "FROM") {
        let table_name = match TableName::check_kind_of_name(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "El nombre de la tabla no es sintacticamente valido".to_string(),
                ))
            }
        };
        Ok(table_name)
    } else {
        Err(Error::SyntaxError(
            "Falta el FROM en la consulta".to_string(),
        ))
    }
}

fn where_clause(list: &mut Vec<String>) -> Result<Option<Where>, Error> {
    if check_words(list, "WHERE") {
        return Ok(Some(r#Where::new(expression(list)?)));
    }
    Ok(None)
}

fn group_by_clause(list: &mut Vec<String>) -> Result<Option<GroupBy>, Error> {
    if check_words(list, "GROUP BY") {
        let mut columns: Vec<Identifier> = Vec::new();
        loop {
            match Identifier::check_identifier(list)? {
                Some(value) => columns.push(value),
                None => {
                    return Err(Error::SyntaxError(
                        "Columnas de GROUP BY no encontradas".to_string(),
                    ))
                }
            };
            if !check_words(list, ",") {
                break;
            }
        }
        return Ok(Some(GroupBy::new(columns)));
    }
    Ok(None)
}

fn ordering_clause(list: &mut Vec<String>) -> Result<Option<OrderBy>, Error> {
    if check_words(list, "ORDER BY") {
        let mut columns: Vec<(Identifier, Option<Ordering>)> = Vec::new();
        loop {
            let value = match Identifier::check_identifier(list)? {
                Some(value) => value,
                None => {
                    return Err(Error::SyntaxError(
                        "En ORDER BY se esperaba una columna".to_string(),
                    ))
                }
            };
            if check_words(list, "ASC") {
                columns.push((value, Some(Ordering::Asc)));
            } else if check_words(list, "DESC") {
                columns.push((value, Some(Ordering::Desc)));
            } else {
                columns.push((value, None));
            }
            if !check_words(list, ",") {
                break;
            }
        }

        return Ok(Some(OrderBy::new(columns)));
    }
    Ok(None)
}

fn per_partition_limit_clause(list: &mut Vec<String>) -> Result<Option<PerPartitionLimit>, Error> {
    if check_words(list, "PER PARTITION LIMIT") {
        let int = list.remove(0);
        let int = match int.parse::<i32>() {
            Ok(value) => PerPartitionLimit::new(value),
            Err(_e) => {
                return Err(Error::SyntaxError(
                    "El valor brindado al Per Partition Limit no es un numero".to_string(),
                ))
            }
        };
        return Ok(Some(int));
    }
    Ok(None)
}

fn limit_clause(list: &mut Vec<String>) -> Result<Option<Limit>, Error> {
    if check_words(list, "LIMIT") {
        let int = list.remove(0);
        let int = match int.parse::<i32>() {
            Ok(value) => Limit::new(value),
            Err(_e) => {
                return Err(Error::SyntaxError(
                    "El valor brindado al Limit no es un int".to_string(),
                ))
            }
        };
        return Ok(Some(int));
    }
    Ok(None)
}

fn allow_filtering_clause(list: &mut Vec<String>) -> Option<bool> {
    if check_words(list, "ALLOW FILTERING") {
        return Some(true);
    }
    None
}

fn selector(list: &mut Vec<String>) -> Result<Option<Selector>, Error> {
    if let Some(column) = Identifier::check_identifier(list)? {
        return Ok(Some(Selector::ColumnName(column)));
    }
    if let Some(term) = Term::is_term(list)? {
        return Ok(Some(Selector::Term(term)));
    }
    Ok(None)
}

fn parse_condition(list: &mut Vec<String>) -> Result<Condition, Error> {
    if list.len() < 3 {
        return Err(Error::SyntaxError("Condición IF incompleta".to_string()));
    }

    let column = match Identifier::check_identifier(list)? {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "La condicion IF no tiene una sintaxis adecuada".to_string(),
            ))
        }
    };
    let operator = match Operator::is_operator(&list.remove(0)) {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "La condicion IF no tiene una sintaxis adecuada".to_string(),
            ))
        }
    };
    let term = match Term::is_term(list)? {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "La condicion IF no tiene una sintaxis adecuada".to_string(),
            ))
        }
    };

    Ok(Condition::new(column, operator, term))
}

fn check_insert_names(list: &mut Vec<String>) -> Result<Vec<Identifier>, Error> {
    if !check_words(list, "(") {
        return Err(Error::SyntaxError("Falto (".to_string()));
    }
    let mut names: Vec<Identifier> = Vec::new();
    match Identifier::check_identifier(list)? {
        Some(value) => names.push(value),
        None => {
            return Err(Error::SyntaxError(
                "Columnas de INSERT no encontradas".to_string(),
            ))
        }
    };
    while check_words(list, ",") {
        match Identifier::check_identifier(list)? {
            Some(value) => names.push(value),
            None => {
                return Err(Error::SyntaxError(
                    "Columnas de INSERT no encontradas".to_string(),
                ))
            }
        };
    }
    if !check_words(list, ")") {
        return Err(Error::SyntaxError("Falta el cierre ')'".to_string()));
    }
    Ok(names)
}

fn set_clause(list: &mut Vec<String>) -> Result<Vec<Assignment>, Error> {
    if !check_words(list, "SET") {
        return Err(Error::SyntaxError("No se encontro el SET".to_string()));
    }
    let mut assignments: Vec<Assignment> = Vec::new();
    let mut assignment = match Assignment::check_kind_of_assignment(list)? {
        Some(value) => value,
        None => {
            return Err(Error::SyntaxError(
                "No se indico ninguna columna en el SET".to_string(),
            ))
        }
    };
    assignments.push(assignment);
    while check_words(list, ",") {
        assignment = match Assignment::check_kind_of_assignment(list)? {
            Some(value) => value,
            None => {
                return Err(Error::SyntaxError(
                    "No se indico ninguna columna en el SET".to_string(),
                ))
            }
        };
        assignments.push(assignment);
    }
    Ok(assignments)
}

fn check_if_condition(list: &mut Vec<String>) -> Result<IfCondition, Error> {
    if check_words(list, "IF") {
        if check_words(list, "EXISTS") {
            return Ok(IfCondition::Exists);
        } else {
            let mut conditions = Vec::new();
            loop {
                let condition = parse_condition(list)?;
                conditions.push(condition);
                if !check_words(list, "AND") {
                    break;
                }
            }
            return Ok(IfCondition::Conditions(conditions));
        }
    }
    Ok(IfCondition::None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parser::{
            data_types::{
                identifier::{
                    quoted_identifier::QuotedIdentifier, unquoted_identifier::UnquotedIdentifier,
                },
                unquoted_name::UnquotedName,
            },
            statements::dml_statement::r#where::expression::Expression,
        },
        tokenizer::tokenizer::tokenize_query,
    };

    // SELECT TESTS:
    #[test]
    fn test_01_basic_select_all() -> Result<(), Error> {
        let query = "SELECT * FROM users";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(select.columns, KindOfColumns::All);
        assert_eq!(
            select.from,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert!(select.options.the_where.is_none());
        Ok(())
    }

    #[test]
    fn test_02_select_specific_columns() -> Result<(), Error> {
        let query = "SELECT id, name FROM users";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        if let KindOfColumns::SelectClause(columns) = select.columns {
            assert_eq!(columns.len(), 2);
            assert!(matches!(columns[0], Selector::ColumnName(_)));
            assert!(matches!(columns[1], Selector::ColumnName(_)));
        } else {
            return Err(Error::SyntaxError("Expected SelectClause".into()));
        }
        Ok(())
    }

    #[test]
    fn test_03_simple_where_clause() -> Result<(), Error> {
        let query = "SELECT * FROM users WHERE id = 5";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(select.options.the_where.is_some());
        if let Some(where_clause) = select.options.the_where {
            if let Some(expr) = where_clause.expression {
                if let Expression::Relation(relation) = *expr {
                    assert_eq!(
                        relation.first_column,
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new("id".to_string()))
                    );
                    assert_eq!(relation.operator, Operator::Equal);
                    assert_eq!(
                        relation.second_column,
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new("5".to_string()))
                    );
                } else {
                    return Err(Error::SyntaxError("Expected Relation expression".into()));
                }
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_04_select_with_group_by() -> Result<(), Error> {
        let query = "SELECT country FROM users GROUP BY country";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert!(select.options.group_by.is_some());
        if let Some(group_by) = select.options.group_by {
            assert_eq!(group_by.columns.len(), 1);
            assert!(matches!(
                group_by.columns[0],
                Identifier::UnquotedIdentifier(_) | Identifier::QuotedIdentifier(_)
            ));
        } else {
            return Err(Error::SyntaxError("Expected Some GroupBy".into()));
        }
        Ok(())
    }

    #[test]
    fn test_05_select_with_order_by() -> Result<(), Error> {
        let query = "SELECT * FROM users ORDER BY age DESC";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert!(select.options.order_by.is_some());
        if let Some(order_by) = select.options.order_by {
            assert_eq!(order_by.columns.len(), 1);
            assert!(matches!(
                order_by.columns[0].0,
                Identifier::UnquotedIdentifier(_) | Identifier::QuotedIdentifier(_)
            ));
            assert_eq!(order_by.columns[0].1, Some(Ordering::Desc));
        } else {
            return Err(Error::SyntaxError("Expected Some OrderBy".into()));
        }
        Ok(())
    }

    #[test]
    fn test_06_select_with_limit() -> Result<(), Error> {
        let query = "SELECT * FROM users LIMIT 10";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert!(select.options.limit.is_some());
        if let Some(limit) = select.options.limit {
            assert_eq!(limit.limit, 10);
        } else {
            return Err(Error::SyntaxError("Expected Some Limit".into()));
        }
        Ok(())
    }

    #[test]
    fn test_07_select_with_allow_filtering() -> Result<(), Error> {
        let query = "SELECT * FROM users ALLOW FILTERING";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(select.options.allow_filtering, Some(true));
        Ok(())
    }

    #[test]
    fn test_08_invalid_select() -> Result<(), Error> {
        let query = "SELECT FROM users";
        let mut tokens = tokenize_query(query);

        assert!(select_statement(&mut tokens).is_err());
        Ok(())
    }

    /// WHERE TESTS:
    #[test]
    fn test_01_where_clause_with_and() -> Result<(), Error> {
        let query = "SELECT * FROM users WHERE age > 18 AND country = 'USA'";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(select.options.the_where.is_some());
        if let Some(where_clause) = select.options.the_where {
            if let Some(expr) = where_clause.expression {
                if let Expression::And(and_expr) = *expr {
                    if let Expression::Relation(relation1) = *and_expr.first_relation {
                        assert_eq!(
                            relation1.first_column,
                            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                "age".to_string()
                            ))
                        );
                        assert_eq!(relation1.operator, Operator::Mayor);
                        assert_eq!(
                            relation1.second_column,
                            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                "18".to_string()
                            ))
                        );
                    } else {
                        return Err(Error::SyntaxError(
                            "Expected Relation expression for first part of AND".into(),
                        ));
                    }

                    if let Expression::Relation(relation2) = *and_expr.second_relation {
                        assert_eq!(
                            relation2.first_column,
                            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                "country".to_string()
                            ))
                        );
                        assert_eq!(relation2.operator, Operator::Equal);
                        assert_eq!(
                            relation2.second_column,
                            Identifier::QuotedIdentifier(QuotedIdentifier::new("USA".to_string()))
                        );
                    } else {
                        return Err(Error::SyntaxError(
                            "Expected Relation expression for second part of AND".into(),
                        ));
                    }
                } else {
                    return Err(Error::SyntaxError("Expected AND expression".into()));
                }
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_02_where_clause_with_multiple_and() -> Result<(), Error> {
        let query =
            "SELECT * FROM products WHERE category = 'electronics' AND price < 1000 AND stock > 0";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(select.options.the_where.is_some());
        if let Some(where_clause) = select.options.the_where {
            if let Some(expr) = where_clause.expression {
                if let Expression::And(and_expr1) = *expr {
                    if let Expression::And(and_expr2) = *and_expr1.first_relation {
                        if let Expression::Relation(relation1) = *and_expr2.first_relation {
                            assert_eq!(
                                relation1.first_column,
                                Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                    "category".to_string()
                                ))
                            );
                            assert_eq!(relation1.operator, Operator::Equal);
                            assert_eq!(
                                relation1.second_column,
                                Identifier::QuotedIdentifier(QuotedIdentifier::new(
                                    "electronics".to_string()
                                ))
                            );
                        } else {
                            return Err(Error::SyntaxError(
                                "Expected Relation expression for first part".into(),
                            ));
                        }
                        if let Expression::Relation(relation2) = *and_expr2.second_relation {
                            assert_eq!(
                                relation2.first_column,
                                Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                    "price".to_string()
                                ))
                            );
                            assert_eq!(relation2.operator, Operator::Minor);
                            assert_eq!(
                                relation2.second_column,
                                Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                    "1000".to_string()
                                ))
                            );
                        } else {
                            return Err(Error::SyntaxError(
                                "Expected Relation expression for second part".into(),
                            ));
                        }
                    } else {
                        return Err(Error::SyntaxError("Expected nested AND expression".into()));
                    }
                    if let Expression::Relation(relation3) = *and_expr1.second_relation {
                        assert_eq!(
                            relation3.first_column,
                            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                "stock".to_string()
                            ))
                        );
                        assert_eq!(relation3.operator, Operator::Mayor);
                        assert_eq!(
                            relation3.second_column,
                            Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                                "0".to_string()
                            ))
                        );
                    } else {
                        return Err(Error::SyntaxError(
                            "Expected Relation expression for third part".into(),
                        ));
                    }
                } else {
                    return Err(Error::SyntaxError(
                        "Expected top-level AND expression".into(),
                    ));
                }
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_03_where_clause_with_contains() -> Result<(), Error> {
        let query = "SELECT * FROM users WHERE hobbies CONTAINS reading";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(select.options.the_where.is_some());
        if let Some(where_clause) = select.options.the_where {
            if let Some(expr) = where_clause.expression {
                if let Expression::Relation(relation) = *expr {
                    assert_eq!(
                        relation.first_column,
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                            "hobbies".to_string()
                        ))
                    );
                    assert_eq!(relation.operator, Operator::Contains);
                    assert_eq!(
                        relation.second_column,
                        Identifier::UnquotedIdentifier(UnquotedIdentifier::new(
                            "reading".to_string()
                        ))
                    );
                } else {
                    return Err(Error::SyntaxError("Expected Relation expression".into()));
                }
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_04_invalid_where_clause() -> Result<(), Error> {
        let query = "SELECT * FROM users WHERE age >";
        let mut tokens = tokenize_query(query);

        let result = select_statement(&mut tokens)?;
        let select = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert!(select.options.the_where.unwrap().expression.is_none());
        Ok(())
    }

    // INSERT TESTS:
    #[test]
    fn test_01_basic_insert() -> Result<(), Error> {
        let query =
            "INSERT INTO users (id, name, email) VALUES (1, 'John Doe', 'john@example.com')";
        let mut tokens = tokenize_query(query);

        let result = insert_statement(&mut tokens)?;
        assert!(result.is_some());

        let insert = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            insert.table.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(insert.names.len(), 3);
        assert_eq!(insert.values.items.len(), 3);
        assert!(!insert.if_not_exists);
        Ok(())
    }

    #[test]
    fn test_02_insert_with_if_not_exists() -> Result<(), Error> {
        let query = "INSERT INTO users (id, name) VALUES (2, 'Jane Doe') IF NOT EXISTS";
        let mut tokens = tokenize_query(query);

        let result = insert_statement(&mut tokens)?;
        let insert = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(insert.if_not_exists);
        Ok(())
    }

    #[test]
    fn test_03_insert_with_quoted_identifiers() -> Result<(), Error> {
        let query = "INSERT INTO \'users\' (\'ID\', \'Name\') VALUES (3, 'Bob Smith')";
        let mut tokens = tokenize_query(query);

        let result = insert_statement(&mut tokens)?;
        let insert = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(
            insert.table.name,
            KeyspaceName::QuotedName(UnquotedName::new("users".to_string())?)
        );

        assert_eq!(insert.names.len(), 2);
        assert_eq!(
            insert.names[0],
            Identifier::QuotedIdentifier(QuotedIdentifier::new("ID".to_string()))
        );
        assert_eq!(
            insert.names[1],
            Identifier::QuotedIdentifier(QuotedIdentifier::new("Name".to_string()))
        );
        Ok(())
    }

    #[test]
    fn test_04_insert_without_column_names() -> Result<(), Error> {
        let query = "INSERT INTO logs VALUES (1001, 'Error', '2023-05-01 10:30:00')";
        let mut tokens = tokenize_query(query);
        assert!(insert_statement(&mut tokens).is_err());
        Ok(())
    }

    // UPDATE TESTS:
    #[test]
    fn test_01_basic_update() -> Result<(), Error> {
        let query = "UPDATE users SET name = 'John' WHERE id = 1";
        let mut tokens = tokenize_query(query);

        let result = update_statement(&mut tokens)?;
        assert!(result.is_some());

        let update = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            update.table_name.name,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert_eq!(update.set_parameter.len(), 1);
        assert!(update.the_where.is_some());
        Ok(())
    }

    #[test]
    fn test_02_update_multiple_columns() -> Result<(), Error> {
        let query = "UPDATE users SET name = 'John', age = 30 WHERE id = 1";
        let mut tokens = tokenize_query(query);

        let result = update_statement(&mut tokens)?;
        let update = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(update.set_parameter.len(), 2);
        Ok(())
    }

    #[test]
    fn test_03_update_with_complex_where() -> Result<(), Error> {
        let query = "UPDATE users SET status = 'active' WHERE age > 18 AND country = 'USA'";
        let mut tokens = tokenize_query(query);

        let result = update_statement(&mut tokens)?;
        let update = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(update.the_where.is_some());
        if let Some(where_clause) = update.the_where {
            if let Some(expr) = where_clause.expression {
                assert!(matches!(*expr, Expression::And(_)));
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_04_update_with_if_condition() -> Result<(), Error> {
        let query =
            "UPDATE users SET email = 'new@email.com' WHERE id = 1 IF email = 'old@email.com'";
        let mut tokens = tokenize_query(query);

        let result = update_statement(&mut tokens)?;
        let update = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(matches!(update.if_condition, IfCondition::Conditions(_)));
        Ok(())
    }

    #[test]
    fn test_05_update_with_if_exists() -> Result<(), Error> {
        let query = "UPDATE users SET last_login = 'toTimestamp(now())' WHERE id = 1 IF EXISTS";
        let mut tokens = tokenize_query(query);

        let result = update_statement(&mut tokens)?;
        let update = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(update.if_condition, IfCondition::Exists);
        Ok(())
    }

    #[test]
    fn test_06_invalid_update() -> Result<(), Error> {
        let query = "UPDATE users SET name = 'John'";
        let mut tokens = tokenize_query(query);
        assert!(update_statement(&mut tokens).is_err());
        Ok(())
    }

    // DELETE TESTS:
    #[test]
    fn test_01_basic_delete() -> Result<(), Error> {
        let query = "DELETE FROM users WHERE id = 1";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        assert!(result.is_some());

        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(
            delete.from,
            KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
        );
        assert!(delete.cols.is_empty());
        assert!(delete.the_where.is_some());
        Ok(())
    }

    #[test]
    fn test_02_delete_specific_columns() -> Result<(), Error> {
        let query = "DELETE name, email FROM users WHERE id = 1";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(delete.cols.len(), 2);
        assert_eq!(delete.cols[0], "name");
        assert_eq!(delete.cols[1], "email");
        Ok(())
    }

    #[test]
    fn test_03_delete_with_complex_where() -> Result<(), Error> {
        let query = "DELETE FROM users WHERE age > 18 AND country = 'USA'";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(delete.the_where.is_some());
        if let Some(where_clause) = delete.the_where {
            if let Some(expr) = where_clause.expression {
                assert!(matches!(*expr, Expression::And(_)));
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_04_delete_with_if_condition() -> Result<(), Error> {
        let query = "DELETE FROM users WHERE id = 1 IF email = 'old@email.com'";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(matches!(delete.if_condition, IfCondition::Conditions(_)));
        Ok(())
    }

    #[test]
    fn test_05_delete_with_if_exists() -> Result<(), Error> {
        let query = "DELETE FROM users WHERE id = 1 IF EXISTS";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(delete.if_condition, IfCondition::Exists);
        Ok(())
    }

    #[test]
    fn test_06_delete_with_contains() -> Result<(), Error> {
        let query = "DELETE FROM users WHERE hobbies CONTAINS 'reading'";
        let mut tokens = tokenize_query(query);

        let result = delete_statement(&mut tokens)?;
        let delete = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert!(delete.the_where.is_some());
        if let Some(where_clause) = delete.the_where {
            if let Some(expr) = where_clause.expression {
                if let Expression::Relation(relation) = *expr {
                    assert_eq!(relation.operator, Operator::Contains);
                } else {
                    return Err(Error::SyntaxError("Expected Relation expression".into()));
                }
            } else {
                return Err(Error::SyntaxError("Expected Some expression".into()));
            }
        }
        Ok(())
    }

    #[test]
    fn test_07_invalid_delete() -> Result<(), Error> {
        let query = "DELETE FROM users";
        let mut tokens = tokenize_query(query);
        assert!(delete_statement(&mut tokens).is_err());
        Ok(())
    }

    // BATCH TESTS:
    #[test]
    fn test_01_basic_batch() -> Result<(), Error> {
        let query = "BEGIN BATCH
                     INSERT INTO users (id, name) VALUES (1, 'John');
                     UPDATE users SET email = 'john@example.com' WHERE id = 1;
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens)?;
        assert!(result.is_some());

        let batch = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;
        assert_eq!(batch.batch_type, BatchType::Logged);
        assert_eq!(batch.queries.len(), 2);
        assert!(matches!(batch.queries[0], DmlStatement::InsertStatement(_)));
        assert!(matches!(batch.queries[1], DmlStatement::UpdateStatement(_)));
        Ok(())
    }

    #[test]
    fn test_02_unlogged_batch() -> Result<(), Error> {
        let query = "BEGIN UNLOGGED BATCH
                     INSERT INTO users (id, name) VALUES (2, 'Jane');
                     DELETE FROM users WHERE id = 1;
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens)?;
        let batch = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(batch.batch_type, BatchType::Unlogged);
        assert_eq!(batch.queries.len(), 2);
        assert!(matches!(batch.queries[0], DmlStatement::InsertStatement(_)));
        assert!(matches!(batch.queries[1], DmlStatement::DeleteStatement(_)));
        Ok(())
    }

    #[test]
    fn test_03_counter_batch() -> Result<(), Error> {
        let query = "BEGIN COUNTER BATCH
                     UPDATE counters SET views = views + 1 WHERE id = 'page1';
                     UPDATE counters SET views = views + 1 WHERE id = 'page2';
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens)?;
        let batch = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(batch.batch_type, BatchType::Counter);
        assert_eq!(batch.queries.len(), 2);
        assert!(matches!(batch.queries[0], DmlStatement::UpdateStatement(_)));
        assert!(matches!(batch.queries[1], DmlStatement::UpdateStatement(_)));
        Ok(())
    }

    #[test]
    fn test_04_batch_with_multiple_tables() -> Result<(), Error> {
        let query = "BEGIN BATCH
                     INSERT INTO users (id, name) VALUES (3, 'Alice');
                     INSERT INTO logs (user_id, action) VALUES (3, 'created');
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens)?;
        let batch = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(batch.queries.len(), 2);
        if let DmlStatement::InsertStatement(insert1) = &batch.queries[0] {
            assert_eq!(
                insert1.table.name,
                KeyspaceName::UnquotedName(UnquotedName::new("users".to_string())?)
            );
        } else {
            return Err(Error::SyntaxError("Expected InsertStatement".into()));
        }
        if let DmlStatement::InsertStatement(insert2) = &batch.queries[1] {
            assert_eq!(
                insert2.table.name,
                KeyspaceName::UnquotedName(UnquotedName::new("logs".to_string())?)
            );
        } else {
            return Err(Error::SyntaxError("Expected InsertStatement".into()));
        }
        Ok(())
    }

    #[test]
    fn test_05_batch_with_if_conditions() -> Result<(), Error> {
        let query = "BEGIN BATCH
                     INSERT INTO users (id, name) VALUES (4, 'Bob') IF NOT EXISTS;
                     UPDATE users SET email = 'bob@example.com' WHERE id = 4 IF name = 'Bob';
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens)?;
        let batch = result.ok_or(Error::SyntaxError("Expected Some, got None".into()))?;

        assert_eq!(batch.queries.len(), 2);
        if let DmlStatement::InsertStatement(insert) = &batch.queries[0] {
            assert!(insert.if_not_exists);
        } else {
            return Err(Error::SyntaxError("Expected InsertStatement".into()));
        }
        if let DmlStatement::UpdateStatement(update) = &batch.queries[1] {
            assert!(matches!(update.if_condition, IfCondition::Conditions(_)));
        } else {
            return Err(Error::SyntaxError("Expected UpdateStatement".into()));
        }
        Ok(())
    }

    #[test]
    fn test_06_empty_batch() -> Result<(), Error> {
        let query = "BEGIN BATCH APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_07_invalid_batch() -> Result<(), Error> {
        let query = "BEGIN BATCH
                     INSERT INTO users (id, name) VALUES (5, 'Charlie');
                     SELECT * FROM users;
                     APPLY BATCH";
        let mut tokens = tokenize_query(query);

        let result = batch_statement(&mut tokens);
        assert!(result.is_err());
        Ok(())
    }

    // EMPTY INPUT TEST:
    #[test]
    fn test_01_empty_input() -> Result<(), Error> {
        let mut tokens = vec![];
        let select = select_statement(&mut tokens)?;
        let update = update_statement(&mut tokens)?;
        let insert = insert_statement(&mut tokens)?;
        let delete = delete_statement(&mut tokens)?;
        let batch = batch_statement(&mut tokens)?;

        assert!(
            select.is_none()
                && update.is_none()
                && insert.is_none()
                && delete.is_none()
                && batch.is_none()
        );
        Ok(())
    }
}
