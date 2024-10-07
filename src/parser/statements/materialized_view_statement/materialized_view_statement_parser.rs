use crate::cassandra::errors::error::Error;

pub enum MaterializedViewStatement {
    CreateMaterializedViewStatement,
    DropMaterializedViewStatement,
}

pub fn materialized_view_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<MaterializedViewStatement>, Error> {
    if let Some(_x) = create_materialized_view_statement(_lista, _index)? {
        return Ok(Some(
            MaterializedViewStatement::CreateMaterializedViewStatement,
        ));
    } else if let Some(_x) = drop_materialized_view_statement(_lista, _index)? {
        return Ok(Some(
            MaterializedViewStatement::DropMaterializedViewStatement,
        ));
    }
    Ok(None)
}

pub fn create_materialized_view_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<MaterializedViewStatement>, Error> {
    Ok(None)
}

pub fn drop_materialized_view_statement(
    _lista: &mut [String],
    _index: i32,
) -> Result<Option<MaterializedViewStatement>, Error> {
    Ok(None)
}
