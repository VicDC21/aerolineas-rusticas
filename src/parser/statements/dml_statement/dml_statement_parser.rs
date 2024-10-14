use super::if_condition::{Condition, IfCondition, Value};
use crate::{
    cassandra::errors::error::Error,
    parser::{
        assignment::Assignment, data_types::{
            constant::Constant, identifier::{
                identifier::Identifier, quoted_identifier::QuotedIdentifier,
                unquoted_identifier::UnquotedIdentifier,
            }, keyspace_name::KeyspaceName, literal::tuple_literal::TupleLiteral, term::Term
        }, statements::{
            ddl_statement::ddl_statement_parser::check_words,
            dml_statement::{
                delete::{Delete, DeleteBuilder}, expression::expression, group_by::GroupBy, insert::Insert, limit::Limit, per_partition_limit::PerPartitionLimit, relation::Relation, select::{KindOfColumns, Select, SelectBuilder}, 
                selector::Selector, update::Update, r#where::Where,
            },
        }, table_name::TableName
    },
};

#[derive(Default)]
enum BatchType {
    #[default]
    Logged,
    Unlogged,
    Counter,
}

struct Batch {
    batch_type: BatchType,
    queries : Vec<DmlStatement>,
}

impl Batch {
    fn new(
        batch_type: BatchType,
        queries : Vec<DmlStatement>,
    ) -> Batch {
        Batch {
            batch_type,
            queries,
        }
    }
}

#[derive(Default)]
struct BatchBuilder {
    batch_type: BatchType,
    queries : Vec<DmlStatement>,
}

impl BatchBuilder {
    fn set_batch_clause(&mut self, batch_type: BatchType) {
        self.batch_type = batch_type;
    }

    fn set_queries(&mut self, queries : Vec<DmlStatement>) {
        self.queries = queries;
    }

    fn build(self) -> Batch {
        Batch::new(
            self.batch_type,
            self.queries,
        )
    }
}

pub enum DmlStatement {
    SelectStatement(Select),
    InsertStatement(Insert),
    UpdateStatement(Update),
    DeleteStatement(Delete),
    BatchStatement(Batch),
}

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
    let index = 0;
    if list[index] == "SELECT" {
        list.remove(index);
        let mut builder = SelectBuilder::default();
        if list[index] == "*" {
            list.remove(index);
            builder.set_select_clause(KindOfColumns::All);
        } else {
            let res = match select_clause(list)? {
                Some(columns) => columns,
                None => {
                    return Err(Error::SyntaxError(
                        "No se especifico ninguna columna".to_string(),
                    ))
                }
            };
            builder.set_select_clause(KindOfColumns::SelectClause(res));
        }
        builder.set_from(from_clause(list)?);
        builder.set_where(where_clause(list)?);
        ordering_clause(list, &mut builder)?;
        per_partition_limit_clause(list, &mut builder)?;
        limit_clause(list, &mut builder)?;
        allow_filtering_clause(list, &mut builder);
        return Ok(Some(builder.build()));
    }
    Ok(None)
}

fn select_clause(list: &mut Vec<String>) -> Result<Option<Vec<Selector>>, Error> {
    if list[0] != "FROM" {
        let mut vec: Vec<Selector> = Vec::new();
        if let Some(sel) = selector(list)? {
            vec.push(sel);
        }
        if list[0] == "," {
            list.remove(0);
            if let Some(mut clasules) = select_clause(list)? {
                vec.append(&mut clasules);
            };
        }
        Ok(Some(vec))
    } else {
        Ok(None)
    }
}

fn from_clause(list: &mut Vec<String>) -> Result<KeyspaceName, Error> {
    if check_words(list, "FROM") {
        let table_name = match KeyspaceName::check_kind_of_name(list)? {
            Some(value) => value,
            None => return Err(Error::SyntaxError("Tipo de dato no admitido".to_string())),
        };
        Ok(table_name)
    } else {
        Err(Error::SyntaxError(
            "Falta el from en la consulta".to_string(),
        ))
    }
}

fn where_clause(list: &mut Vec<String>) -> Result<Option<Where>, Error> {
    if check_words(list, "WHERE") {
        Ok(Some(Where::new(expression(list)?)))
    } else {
        Ok(Some(Where::new(None)))
    }
}

fn group_by_clause(list: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if check_words(list, "GROUP BY"){
        let mut columns: Vec<Identifier> = Vec::new();
        match Identifier::check_identifier(list)? {
            Some(value) => columns.push(value),
            None => {
                return Err(Error::SyntaxError(
                    "Columnas de GROUP BY no encontradas".to_string(),
                ))
            }
        };
        while list[0] == "," {
            match Identifier::check_identifier(list)? {
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

fn ordering_clause(list: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if check_words(list, "ORDER BY") {
        if let Some(value) = Identifier::check_identifier(list)? {} // TERMINAR
    } else {
        builder.set_order_by(None);
    }
    Ok(())
}

fn per_partition_limit_clause(
    list: &mut Vec<String>,
    builder: &mut SelectBuilder,
) -> Result<(), Error> {
    if  check_words(list, "PER PARTITION LIMIT"){
        list.remove(0);
        list.remove(0);
        list.remove(0);
        let int = list.remove(0);
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
fn limit_clause(list: &mut Vec<String>, builder: &mut SelectBuilder) -> Result<(), Error> {
    if check_words(list, "LIMIT"){
        list.remove(0);
        let int = list.remove(0);
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
fn allow_filtering_clause(
    list: &mut Vec<String>,
    builder: &mut SelectBuilder,
) -> Option<PerPartitionLimit> {
    if check_words(list, "LIMIT"){
        builder.set_allow_filtering(Some(true));
    } else {
        builder.set_allow_filtering(None);
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

fn delete_statement(list: &mut Vec<String>) -> Result<Option<Delete>, Error> {
    if !check_words(list, "DELETE") {
        return Ok(None);
    }

    let mut builder = DeleteBuilder::default();
    
    let mut cols = Vec::new();
    while !check_words(list, "FROM") {
        if list.is_empty() {
            return Err(Error::SyntaxError("Se esperaba FROM después de las columnas".to_string()));
        }
        cols.push(list.remove(0));
    }
    if !cols.is_empty() {
        builder.set_cols(cols);
    }

    builder.set_from(from_clause(list)?);

    if !check_words(list, "WHERE") {
        return Err(Error::SyntaxError("Falta la cláusula WHERE".to_string()));
    }
    builder.set_where(where_clause(list)?);

    if check_words(list, "IF") {
        if check_words(list, "EXISTS") {
            builder.set_if_condition(Some(IfCondition::Exists));
        } else {
            let mut conditions = Vec::new();
            loop {
                let condition = parse_condition(list)?;
                conditions.push(condition);
                if !check_words(list, "AND") {
                    break;
                }
            }
            builder.set_if_condition(Some(IfCondition::Conditions(conditions)));
        }
    }

    Ok(Some(builder.build()))
}

fn parse_condition(list: &mut Vec<String>) -> Result<Condition, Error> {
    if list.len() < 3 {
        return Err(Error::SyntaxError("Condición IF incompleta".to_string()));
    }

    let column = list.remove(0);
    let operator = list.remove(0);
    let value = parse_value(list)?;

    match operator.as_str() {
        "=" => Ok(Condition::Equals(column, value)),
        "!=" => Ok(Condition::NotEquals(column, value)),
        ">" => Ok(Condition::GreaterThan(column, value)),
        ">=" => Ok(Condition::GreaterThanOrEqual(column, value)),
        "<" => Ok(Condition::LessThan(column, value)),
        "<=" => Ok(Condition::LessThanOrEqual(column, value)),
        "IN" => {
            if let Value::List(list) = value {
                Ok(Condition::In(column, list))
            } else {
                Err(Error::SyntaxError("Se esperaba una list para el operador IN".to_string()))
            }
        },
        _ => Err(Error::SyntaxError(format!("Operador desconocido: {}", operator))),
    }
}

fn parse_value(list: &mut Vec<String>) -> Result<Value, Error> {
    if list.is_empty() {
        return Err(Error::SyntaxError("Valor esperado".to_string()));
    }

    let first = list.remove(0);
    if first == "(" {
        let mut values = Vec::new();
        while let Some(next) = list.first() {
            if next == ")" {
                list.remove(0);
                return Ok(Value::List(values));
            }
            values.push(parse_single_value(list.remove(0))?);
            if list.first() == Some(&",".to_string()) {
                list.remove(0);
            }
        }
        Err(Error::SyntaxError("List no cerrada".to_string()))
    } else {
        parse_single_value(first)
    }
}

fn parse_single_value(value: String) -> Result<Value, Error> {
    if value.starts_with('\'') && value.ends_with('\'') {
        Ok(Value::String(value[1..value.len()-1].to_string()))
    } else if let Ok(num) = value.parse::<i64>() {
        Ok(Value::Integer(num))
    } else if let Ok(num) = value.parse::<f64>() {
        Ok(Value::Float(num))
    } else if value == "true" || value == "false" {
        Ok(Value::Boolean(value == "true"))
    } else {
        Ok(Value::Identifier(value))
    }
}

fn insert_statement(list: &mut Vec<String>) -> Result<Option<Insert>, Error> {
    if check_words(list, "INSERT INTO"){
        let table_name: TableName = match TableName::check_kind_of_name(list)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError("El nombre de la tabla no es sintacticamente valido".to_string()))
        };
        let names = check_insert_names(list)?;

        if !check_words(list, "VALUES") {
            return Err(Error::SyntaxError("Falto VALUES".to_string()));
        }
        let values = match TupleLiteral::check_tuple_literal(list)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError("No se encontro ninguna tupla".to_string()))
        };
        let if_not_exists = check_words(list, "IF NOT EXISTS");
        return Ok(Some(Insert::new(table_name, names, values, if_not_exists)))
    }
    Ok(None)
}

fn check_insert_names(list: &mut Vec<String>) -> Result<Vec<Identifier>, Error> {
    if !check_words(list, "("){
        return  Err(Error::SyntaxError("Falto (".to_string()));
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
    while check_words(list, ","){
        match Identifier::check_identifier(list)? {
            Some(value) => names.push(value),
            None => {
                return Err(Error::SyntaxError(
                    "Columnas de INSERT no encontradas".to_string(),
                ))
            }
        };
    }
    if !check_words(list, ")"){
        return  Err(Error::SyntaxError("Falta el cierre ')'".to_string()));
    }
    Ok(names)
}

fn update_statement(list: &mut Vec<String>) -> Result<Option<Update>, Error> {
    if check_words(list, "UPDATE"){
        // Guardar el nombre de la tabla
        let table_name: TableName = match TableName::check_kind_of_name(list)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError("El nombre de la tabla no es sintacticamente valido".to_string()))
        };
        let set = set_clause(list)?;
        let r#where = where_clause(list)?;
        return Ok(Some(Update::new(table_name, set, r#where)));
    }

    Ok(None)
}

fn set_clause(list: &mut Vec<String>) -> Result<Vec<Assignment>, Error>{
    if !check_words(list, "SET"){
        return Err(Error::SyntaxError("No se encontro el SET".to_string()));
    }
    let mut assignments: Vec<Assignment> = Vec::new();
    let mut assignment = match Assignment::check_kind_of_assignment(list)?{
        Some(value) => value,
        None => return Err(Error::SyntaxError("No se indico ninguna columna en el SET".to_string()))
    };
    assignments.push(assignment);
    while check_words(list, ","){
        assignment = match Assignment::check_kind_of_assignment(list)?{
            Some(value) => value,
            None => return Err(Error::SyntaxError("No se indico ninguna columna en el SET".to_string()))
        }; 
        assignments.push(assignment);
    }
    Ok(assignments)

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
        builder.set_batch_clause(BatchType::Logged);
    } else {
        return Err(Error::SyntaxError("Falta BEGIN en la consulta".to_string()));
    }

    let mut queries: Vec<DmlStatement> = Vec::new();
    while list[0] != "APPLY" && list[1] != "BATCH" {
        if list.is_empty() {
            break;
        }
        if check_words(list, "INSERT") {
            if let Some(insert_stmt) = insert_statement(list)? {
                queries.push(DmlStatement::InsertStatement(insert_stmt));
            }
        } else if check_words(list, "UPDATE") {
            if let Some(update_stmt) = update_statement(list)? {
                queries.push(DmlStatement::UpdateStatement(update_stmt));
            }
        } else if check_words(list, "DELETE") {
            if let Some(delete_stmt) = delete_statement(list)? {
                queries.push(DmlStatement::DeleteStatement(delete_stmt));
            }
        }
        list.remove(0);
    }
    if queries.is_empty() {
        return Err(Error::SyntaxError("No se encontraron consultas en el batch".to_string()));
    }
    builder.set_queries(queries);
    Ok(Some(builder.build()))
}
