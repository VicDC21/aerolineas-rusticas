use crate::cassandra::errors::error::Error;
use super::statements::{ddl_statement::ddl_statement, statement::Statement};


pub fn make_parse(lista: &mut Vec<String>) -> Result<CqlStatement, Error> {
    let tree: CqlStatement = match cql_statement(lista)? {
        Some(value) => value,
        None => return Err(Error::ConfigError("dsa".to_string())),
    };
    Ok(tree)
}

fn cql_statement(lista: &mut Vec<String>) -> Result<Option<Statement>, Error> {
    statement(lista)
}

fn statement(lista: &mut Vec<String>) -> Result<Option<Statement>, Error> {
    let index = 0;

    if ddl_statement(&mut lista, index)?.is_some(){

    } else if dml_statement(&mut lista, index){

    } else if secondary_index_statement(&mut lista, index){
        
    } else if materialized_view_statement(&mut lista, index){
        
    } else if role_or_permission_statement(&mut lista, index){
        
    }
    Ok(None)
}

pub fn ddl_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<ddl_statement>, Error>{
    // if use_statement(lista) | create_keyspace_statement(lista) | alter_keyspace_statement(lista){
    //     return Ok(None)
    // };

    Ok(Some(ddl_statement::alter_keyspace_statement))

}

pub fn use_statement() -> Result<Option<ddl_statement>, Error>{


    Ok(None)
}


pub fn dml_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<DmlStatement>, Error>{
    // if !is_use_statement(lista) | !is_create_keyspace_statement(lista) | is_alter_keyspace_statement(lista){
    //     return None
    // };

    // Ok(Some(DmlStatement::))
    Ok(None)
}


pub fn secondary_index_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<SecondaryIndexStatement>, Error>{
    // if use_statement(lista) | create_keyspace_statement(lista) | alter_keyspace_statement(lista){
    //     return Ok(None)
    // };

    // Ok(Some(SecondaryIndexStatement::))
    Ok(None)
}

pub fn materialized_view_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<MaterializedViewStatement>, Error>{
    // if use_statement(lista) | create_keyspace_statement(lista) | alter_keyspace_statement(lista){
    //     return Ok(None)
    // };


    // Ok(Some(MaterializedViewStatement::))
    Ok(None)
}

pub fn role_or_permission_statement(lista: &mut Vec<String>, index: i32) -> Result<Option<RoleOrPermissionStatement>, Error>{
    // if use_statement(lista) | create_keyspace_statement(lista) | alter_keyspace_statement(lista){
    //     return Ok(None)
    // };


    // Ok(Some(RoleOrPermissionStatement::))
    Ok(None)
}
