pub enum Statement{
    ddl_statement,
    dml_statement,
    secondary_index_statement,
    materialized_view_statement,
    role_or_permission_statement,
    udf_statement,
    udt_statement,
    trigger_statement
}